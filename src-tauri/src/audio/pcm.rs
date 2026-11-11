//! Decoded-PCM cache.
//!
//! Playback pulls ready-to-mix samples from here so the cpal callback never
//! decodes and never resamples. Buffers are stored already converted to the
//! output device's sample rate and channel layout; starting a sound is then an
//! `Arc` bump instead of a decoder setup, and seeking is an index jump.
//!
//! Files whose decoded form would exceed [`PcmCache::max_entry`] are never
//! kept; they play through the progressive pump in [`super::stream`].

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant};

use rodio::buffer::SamplesBuffer;
use rodio::{ChannelCount, SampleRate, Source};

use super::decode::{open_decoder, SampleSource};
use super::peaks::PeakAcc;
use super::resample::FormatConverter;

pub use super::resample::resample_mono;

/// Total RAM budget for decoded PCM (512 MiB). Overridable at runtime.
const DEFAULT_MAX_TOTAL: usize = 512 * 1024 * 1024;
/// Per-file budget (128 MiB ≈ 11 min of 48 kHz stereo f32).
/// Longer files still decode off-thread for playback; they just are not kept.
const DEFAULT_MAX_ENTRY: usize = 128 * 1024 * 1024;
/// Drop buffers unused for this long unless they are playing or pinned.
const ENTRY_TTL: Duration = Duration::from_secs(180);

const BYTES_PER_SAMPLE: usize = std::mem::size_of::<f32>();
/// Source frames pulled from the decoder per conversion batch.
const DECODE_BATCH_FRAMES: usize = 4096;

// ---------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------

/// Identifies one decoded buffer. `mtime`/`size` make the entry self-invalidating
/// when a file is rewritten (preview WAVs, record-editor exports).
#[derive(Clone, PartialEq, Eq, Hash)]
struct PcmKey {
    path: String,
    mtime: u64,
    size: u64,
    sample_rate: u32,
    channels: u16,
}

struct PcmEntry {
    /// Template with `pos == 0`. Cloning shares the sample data via `Arc`.
    buffer: SamplesBuffer,
    bytes: usize,
    duration_secs: f64,
    last_used: Instant,
    /// Waveform peaks of `buffer`, so the scrubber never decodes a cached file.
    peaks: Arc<PeakAcc>,
}

struct PcmCache {
    entries: HashMap<PcmKey, PcmEntry>,
    total_bytes: usize,
    max_total: usize,
    max_entry: usize,
    /// Paths of the board that is on screen. Their buffers outlive the idle
    /// TTL, so a pad that was warmed stays instant however long it sits unused.
    pinned: HashSet<String>,
}

impl PcmCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            total_bytes: 0,
            max_total: DEFAULT_MAX_TOTAL,
            max_entry: DEFAULT_MAX_ENTRY,
            pinned: HashSet::new(),
        }
    }

    /// Touch and return a ready-to-play clone. `O(1)` — no access-order list to
    /// rewrite, the LRU timestamp lives inside the entry.
    fn get(&mut self, key: &PcmKey) -> Option<(SamplesBuffer, f64)> {
        let entry = self.entries.get_mut(key)?;
        entry.last_used = Instant::now();
        Some((entry.buffer.clone(), entry.duration_secs))
    }

    fn insert(&mut self, key: PcmKey, entry: PcmEntry) {
        if entry.bytes > self.max_entry {
            return;
        }
        if let Some(old) = self.entries.remove(&key) {
            self.total_bytes = self.total_bytes.saturating_sub(old.bytes);
        }
        self.evict_until_fits(entry.bytes);
        self.total_bytes += entry.bytes;
        self.entries.insert(key, entry);
    }

    /// Drop least-recently-used entries until `incoming` fits in the budget.
    /// Pinned entries are not exempt: the byte budget is a hard limit.
    fn evict_until_fits(&mut self, incoming: usize) {
        while self.total_bytes + incoming > self.max_total && !self.entries.is_empty() {
            let lru = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_used)
                .map(|(k, _)| k.clone());
            match lru {
                Some(k) => self.remove(&k),
                None => break,
            }
        }
    }

    fn remove(&mut self, key: &PcmKey) {
        if let Some(e) = self.entries.remove(key) {
            self.total_bytes = self.total_bytes.saturating_sub(e.bytes);
        }
    }

    fn invalidate_path(&mut self, path: &str) {
        let stale: Vec<PcmKey> = self
            .entries
            .keys()
            .filter(|k| k.path == path)
            .cloned()
            .collect();
        for k in stale {
            self.remove(&k);
        }
    }

    /// Drop entries idle for longer than the TTL as of `now`, except those in
    /// `keep` (playing) and pinned ones (the visible board).
    fn evict_idle(&mut self, keep: &[String], now: Instant) {
        let stale: Vec<PcmKey> = self
            .entries
            .iter()
            .filter(|(k, e)| {
                !self.pinned.contains(&k.path)
                    && !keep.contains(&k.path)
                    && now.saturating_duration_since(e.last_used) > ENTRY_TTL
            })
            .map(|(k, _)| k.clone())
            .collect();
        for k in stale {
            self.remove(&k);
        }
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.total_bytes = 0;
    }

    fn set_limits(&mut self, max_total: usize, max_entry: usize) {
        self.max_total = max_total;
        self.max_entry = max_entry;
        let oversized: Vec<PcmKey> = self
            .entries
            .iter()
            .filter(|(_, e)| e.bytes > max_entry)
            .map(|(k, _)| k.clone())
            .collect();
        for k in oversized {
            self.remove(&k);
        }
        self.evict_until_fits(0);
    }
}

static PCM_CACHE: OnceLock<Mutex<PcmCache>> = OnceLock::new();

fn cache() -> &'static Mutex<PcmCache> {
    PCM_CACHE.get_or_init(|| Mutex::new(PcmCache::new()))
}

// ---------------------------------------------------------------------------
// In-flight loads
// ---------------------------------------------------------------------------

type LoadResult = Result<(SamplesBuffer, f64), String>;

/// One decode in progress. Later callers for the same key wait here for the
/// owner's result instead of decoding the file a second time.
struct InFlight {
    result: Mutex<Option<LoadResult>>,
    ready: Condvar,
}

impl InFlight {
    fn wait(&self) -> LoadResult {
        let mut guard = self.result.lock().map_err(|e| e.to_string())?;
        loop {
            if let Some(result) = guard.as_ref() {
                return result.clone();
            }
            guard = self.ready.wait(guard).map_err(|e| e.to_string())?;
        }
    }

    fn publish(&self, result: LoadResult) {
        if let Ok(mut guard) = self.result.lock() {
            if guard.is_none() {
                *guard = Some(result);
            }
        }
        self.ready.notify_all();
    }
}

static IN_FLIGHT: OnceLock<Mutex<HashMap<PcmKey, Arc<InFlight>>>> = OnceLock::new();

fn in_flight() -> &'static Mutex<HashMap<PcmKey, Arc<InFlight>>> {
    IN_FLIGHT.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Ownership of an in-flight slot. Dropping it always releases the waiters,
/// also when the decode failed or panicked, so nobody waits forever.
struct LoadClaim {
    key: PcmKey,
    slot: Arc<InFlight>,
}

impl Drop for LoadClaim {
    fn drop(&mut self) {
        if let Ok(mut map) = in_flight().lock() {
            map.remove(&self.key);
        }
        self.slot.publish(Err("Decode was abandoned".into()));
    }
}

enum Claim {
    Owner(LoadClaim),
    Waiter(Arc<InFlight>),
}

fn claim(key: &PcmKey) -> Result<Claim, String> {
    let mut map = in_flight().lock().map_err(|e| e.to_string())?;
    if let Some(slot) = map.get(key) {
        return Ok(Claim::Waiter(Arc::clone(slot)));
    }
    let slot = Arc::new(InFlight {
        result: Mutex::new(None),
        ready: Condvar::new(),
    });
    map.insert(key.clone(), Arc::clone(&slot));
    Ok(Claim::Owner(LoadClaim {
        key: key.clone(),
        slot,
    }))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Decoded size a file would occupy at the given output format.
pub fn estimated_bytes(duration_secs: f64, rate: SampleRate, channels: ChannelCount) -> usize {
    if !duration_secs.is_finite() || duration_secs <= 0.0 {
        return usize::MAX;
    }
    let frames = duration_secs * rate.get() as f64;
    let samples = frames * channels.get() as f64;
    (samples * BYTES_PER_SAMPLE as f64) as usize
}

/// Largest decoded size the cache keeps for one file.
pub fn max_entry_bytes() -> usize {
    cache().lock().map(|c| c.max_entry).unwrap_or(DEFAULT_MAX_ENTRY)
}

/// `true` when a file of this length is worth holding as decoded PCM.
/// `false` leaves the file to the progressive pump.
pub fn fits(duration_secs: f64, rate: SampleRate, channels: ChannelCount) -> bool {
    estimated_bytes(duration_secs, rate, channels) <= max_entry_bytes()
}

/// Cached buffer for `path` at the output format, or `None` on a miss.
pub fn peek(path: &str, rate: SampleRate, channels: ChannelCount) -> Option<(SamplesBuffer, f64)> {
    let key = make_key(path, rate, channels)?;
    cache().lock().ok()?.get(&key)
}

/// Waveform peaks and duration of a cached buffer for `path` at any output
/// format, if the file on disk is still the one that was decoded.
pub(crate) fn peek_peaks(path: &str) -> Option<(Arc<PeakAcc>, f64)> {
    let (mtime, size) = file_stamp(path)?;
    let guard = cache().lock().ok()?;
    guard
        .entries
        .iter()
        .find(|(k, _)| k.path == path && k.mtime == mtime && k.size == size)
        .map(|(_, e)| (Arc::clone(&e.peaks), e.duration_secs))
}

fn build_entry(
    rate: SampleRate,
    channels: ChannelCount,
    samples: Vec<f32>,
    peaks: Option<PeakAcc>,
) -> PcmEntry {
    let bytes = samples.len() * BYTES_PER_SAMPLE;
    let peaks = peaks.unwrap_or_else(|| PeakAcc::from_samples(&samples, channels.get()));
    let buffer = SamplesBuffer::new(channels, rate, samples);
    let duration_secs = buffer
        .total_duration()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    PcmEntry {
        buffer,
        bytes,
        duration_secs,
        last_used: Instant::now(),
        peaks: Arc::new(peaks),
    }
}

/// Insert an already-decoded buffer (pump promote). No-op if over budget.
/// `peaks` saves a fold over the samples when the caller already has them.
pub(crate) fn store(
    path: &str,
    rate: SampleRate,
    channels: ChannelCount,
    samples: Vec<f32>,
    peaks: Option<PeakAcc>,
) {
    let Some(key) = make_key(path, rate, channels) else {
        return;
    };
    if samples.is_empty() {
        return;
    }
    let entry = build_entry(rate, channels, samples, peaks);
    if let Ok(mut guard) = cache().lock() {
        guard.insert(key, entry);
    }
}

/// Cached buffer, decoding and converting on a miss.
pub fn load(path: &str, rate: SampleRate, channels: ChannelCount) -> LoadResult {
    load_with(path, rate, channels, || {
        decode_to_format(path, rate.get(), channels.get())
    })
}

/// [`load`] with the decode step supplied by the caller. A second call for the
/// same file and format while the first is still decoding waits for that
/// result, so one file is never decoded twice at the same time.
fn load_with<F>(path: &str, rate: SampleRate, channels: ChannelCount, produce: F) -> LoadResult
where
    F: FnOnce() -> Result<Vec<f32>, String>,
{
    if let Some(hit) = peek(path, rate, channels) {
        return Ok(hit);
    }
    let key = make_key(path, rate, channels).ok_or_else(|| format!("Cannot stat '{path}'"))?;
    let owned = match claim(&key)? {
        Claim::Owner(owned) => owned,
        Claim::Waiter(slot) => return slot.wait(),
    };
    // The previous owner may have finished between the miss above and the
    // claim; its result is in the cache by now.
    if let Some(hit) = cache().lock().map_err(|e| e.to_string())?.get(&key) {
        owned.slot.publish(Ok(hit.clone()));
        return Ok(hit);
    }

    let result = produce().and_then(|samples| {
        if samples.is_empty() {
            return Err(format!("Decoded zero samples from '{path}'"));
        }
        let entry = build_entry(rate, channels, samples, None);
        let loaded = (entry.buffer.clone(), entry.duration_secs);
        cache().lock().map_err(|e| e.to_string())?.insert(key, entry);
        Ok(loaded)
    });
    owned.slot.publish(result.clone());
    result
}

/// Only short clips are pre-decoded. Long files would burn every core on
/// tab switch; they decode on first play via the progressive pump instead.
const PREFETCH_MAX_SECS: f64 = 30.0;
/// ~30 s of compressed audio; unknown-duration files above this are left alone.
const PREFETCH_MAX_BYTES: u64 = 1_500_000;

/// What [`prefetch_one`] did with a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Prefetch {
    AlreadyCached,
    Decoded,
    /// Too long to pre-decode, or being decoded by a playing pump right now.
    Skipped,
    Failed,
}

/// Decode one short clip into the PCM cache. Long and unknown-large files are
/// skipped; so is a file a pump is decoding right now, because that pump
/// stores its result here when it finishes.
pub(crate) fn prefetch_one(path: &str, rate: SampleRate, channels: ChannelCount) -> Prefetch {
    if peek(path, rate, channels).is_some() {
        return Prefetch::AlreadyCached;
    }
    if super::stream::is_decoding(path) {
        return Prefetch::Skipped;
    }
    let secs = super::cache::cached_duration(path).unwrap_or(0.0);
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let long = secs > PREFETCH_MAX_SECS
        || (secs <= 0.0 && size > PREFETCH_MAX_BYTES)
        || !fits(if secs > 0.0 { secs } else { PREFETCH_MAX_SECS }, rate, channels);
    if long {
        return Prefetch::Skipped;
    }
    match load(path, rate, channels) {
        Ok((_, duration)) => {
            if duration > 0.0 {
                // The decoded length is exact; header values often are not.
                super::cache::store_duration(path, duration);
            }
            Prefetch::Decoded
        }
        Err(e) => {
            eprintln!("[pcm] prefetch failed '{path}': {e}");
            Prefetch::Failed
        }
    }
}

/// Replace the set of paths whose buffers are exempt from idle eviction.
pub(crate) fn set_pinned(paths: &[String]) {
    if let Ok(mut c) = cache().lock() {
        c.pinned = paths.iter().cloned().collect();
    }
}

pub fn invalidate(path: &str) {
    if let Ok(mut c) = cache().lock() {
        c.invalidate_path(path);
    }
}

/// Drop buffers idle past the TTL, except those playing (`keep`) or pinned.
pub fn evict_idle(keep: &[String]) {
    if let Ok(mut c) = cache().lock() {
        c.evict_idle(keep, Instant::now());
    }
}

pub fn clear() {
    if let Ok(mut c) = cache().lock() {
        c.clear();
    }
}

pub fn set_limits(max_total: usize, max_entry: usize) {
    if let Ok(mut c) = cache().lock() {
        c.set_limits(max_total, max_entry);
    }
}

/// `(entry count, bytes held, byte budget)`.
pub fn stats() -> (usize, usize, usize) {
    match cache().lock() {
        Ok(c) => (c.entries.len(), c.total_bytes, c.max_total),
        Err(_) => (0, 0, 0),
    }
}

/// `(mtime seconds, size)` of a file, or `None` when it cannot be read.
pub(crate) fn file_stamp(path: &str) -> Option<(u64, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Some((mtime, meta.len()))
}

fn make_key(path: &str, rate: SampleRate, channels: ChannelCount) -> Option<PcmKey> {
    let (mtime, size) = file_stamp(path)?;
    Some(PcmKey {
        path: path.to_owned(),
        mtime,
        size,
        sample_rate: rate.get(),
        channels: channels.get(),
    })
}

// ---------------------------------------------------------------------------
// Decode
// ---------------------------------------------------------------------------

/// Decode a file straight into the output format. Samples go from the decoder
/// through the converter in batches, so the only full-size buffer is the result.
pub(crate) fn decode_to_format(
    path: &str,
    dst_rate: u32,
    dst_channels: u16,
) -> Result<Vec<f32>, String> {
    let mut opened = SampleSource::open(path)?;
    let src_channels = opened.channels.max(1);
    let mut converter =
        FormatConverter::new(opened.sample_rate, src_channels, dst_rate, dst_channels)?;

    // Reserve the result once. The length comes from the file header, which a
    // damaged file can inflate, so the reservation stops at what the cache
    // would keep anyway.
    let expected_frames = opened.duration_hint_secs.max(0.0) * dst_rate as f64;
    let expected_samples = (expected_frames * dst_channels.max(1) as f64) as usize;
    let mut out = Vec::with_capacity(expected_samples.min(max_entry_bytes() / BYTES_PER_SAMPLE));
    let batch_samples = DECODE_BATCH_FRAMES * src_channels as usize;
    let mut batch = Vec::with_capacity(batch_samples);
    loop {
        batch.clear();
        if opened.source.read(&mut batch, batch_samples) == 0 {
            break;
        }
        converter.push(&batch, &mut out)?;
    }
    converter.finish(&mut out)?;
    Ok(out)
}

/// Decode a file to interleaved f32 in [-1, 1] plus its native format.
/// WAV goes through hound for exact PCM; everything else through rodio.
pub fn decode_file(path: &str) -> Result<(Vec<f32>, u32, u16), String> {
    if path.to_lowercase().ends_with(".wav") {
        if let Some(decoded) = decode_wav(path) {
            return Ok(decoded);
        }
    }

    let decoder = open_decoder(path)?;
    let sample_rate: u32 = decoder.sample_rate().into();
    let channels: u16 = decoder.channels().into();
    let samples: Vec<f32> = decoder.collect();
    if samples.is_empty() {
        return Err("Decoded zero samples".into());
    }
    Ok((samples, sample_rate, channels))
}

/// Hound hands integer samples at the file's bit depth, not stretched to i32.
/// 24-bit lives in ±2^23; dividing by `i32::MAX` made those clips ~48 dB too quiet.
pub fn int_sample_to_f32(sample: i32, bits: u16) -> f32 {
    let bits = bits.clamp(1, 32);
    let max = (1i64 << (bits - 1)) as f32;
    if max == 0.0 {
        return 0.0;
    }
    (sample as f32 / max).clamp(-1.0, 1.0)
}

fn decode_wav(path: &str) -> Option<(Vec<f32>, u32, u16)> {
    let mut reader = hound::WavReader::open(path).ok()?;
    let spec = reader.spec();
    let bits = spec.bits_per_sample;
    let samples: Result<Vec<f32>, _> = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().collect(),
        hound::SampleFormat::Int => reader
            .samples::<i32>()
            .map(|r| r.map(|s| int_sample_to_f32(s, bits)))
            .collect(),
    };
    let samples = samples.ok()?;
    if samples.is_empty() {
        return None;
    }
    Some((samples, spec.sample_rate, spec.channels))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::decode::test_support::write_wav;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn rate(hz: u32) -> SampleRate {
        SampleRate::new(hz).unwrap()
    }

    fn channels(n: u16) -> ChannelCount {
        ChannelCount::new(n).unwrap()
    }

    fn key(path: &str) -> PcmKey {
        PcmKey {
            path: path.to_owned(),
            mtime: 1,
            size: 1,
            sample_rate: 48_000,
            channels: 2,
        }
    }

    fn entry(samples: usize) -> PcmEntry {
        build_entry(rate(48_000), channels(2), vec![0.25; samples], None)
    }

    #[test]
    fn int_sample_to_f32_full_scale_16bit_is_one() {
        assert!((int_sample_to_f32(i16::MAX as i32, 16) - 1.0).abs() < 0.001);
    }

    #[test]
    fn evict_idle_drops_stale_entries_but_keeps_pinned_and_playing_ones() {
        let mut cache = PcmCache::new();
        for path in ["pinned", "playing", "idle"] {
            cache.insert(key(path), entry(64));
        }
        cache.pinned = HashSet::from(["pinned".to_owned()]);
        let keep = vec!["playing".to_owned()];

        // Nothing is old enough yet.
        cache.evict_idle(&keep, Instant::now());
        assert_eq!(cache.entries.len(), 3);

        let later = Instant::now() + ENTRY_TTL + Duration::from_secs(1);
        cache.evict_idle(&keep, later);
        assert!(cache.entries.contains_key(&key("pinned")));
        assert!(cache.entries.contains_key(&key("playing")));
        assert!(!cache.entries.contains_key(&key("idle")));
        assert_eq!(cache.total_bytes, 2 * 64 * BYTES_PER_SAMPLE);
    }

    #[test]
    fn insert_evicts_pinned_entries_when_the_byte_budget_is_full() {
        let mut cache = PcmCache::new();
        cache.set_limits(100 * BYTES_PER_SAMPLE, 100 * BYTES_PER_SAMPLE);
        cache.pinned = HashSet::from(["old".to_owned()]);
        cache.insert(key("old"), entry(80));
        cache.insert(key("new"), entry(80));
        assert!(!cache.entries.contains_key(&key("old")));
        assert!(cache.entries.contains_key(&key("new")));
    }

    #[test]
    fn insert_rejects_entries_over_the_per_file_limit() {
        let mut cache = PcmCache::new();
        cache.set_limits(1024 * BYTES_PER_SAMPLE, 16 * BYTES_PER_SAMPLE);
        cache.insert(key("big"), entry(32));
        assert!(cache.entries.is_empty());
        assert_eq!(cache.total_bytes, 0);
    }

    #[test]
    fn load_decodes_once_when_two_callers_ask_for_the_same_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("once.wav");
        write_wav(&path, 8_000, 1, 800, |_, _| 0.5);
        let path = path.to_string_lossy().into_owned();
        let decodes = AtomicUsize::new(0);

        let results: Vec<LoadResult> = std::thread::scope(|scope| {
            let workers: Vec<_> = (0..2)
                .map(|_| {
                    scope.spawn(|| {
                        load_with(&path, rate(8_000), channels(1), || {
                            decodes.fetch_add(1, Ordering::SeqCst);
                            // Long enough for the other caller to arrive while
                            // this decode is still running.
                            std::thread::sleep(Duration::from_millis(150));
                            decode_to_format(&path, 8_000, 1)
                        })
                    })
                })
                .collect();
            workers.into_iter().map(|w| w.join().unwrap()).collect()
        });

        assert_eq!(decodes.load(Ordering::SeqCst), 1);
        for result in results {
            let (buffer, duration) = result.unwrap();
            assert_eq!(buffer.count(), 800);
            assert!((duration - 0.1).abs() < 1e-6);
        }
        invalidate(&path);
    }

    #[test]
    fn load_hands_a_failed_decode_to_waiters_and_lets_the_next_call_retry() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("retry.wav");
        write_wav(&path, 8_000, 1, 80, |_, _| 0.5);
        let path = path.to_string_lossy().into_owned();

        let failed = load_with(&path, rate(8_000), channels(1), || Err("boom".into()));
        assert_eq!(failed.err().as_deref(), Some("boom"));
        // The failed claim is gone: a later load decodes for real.
        let (buffer, _) = load(&path, rate(8_000), channels(1)).unwrap();
        assert_eq!(buffer.count(), 80);
        invalidate(&path);
    }

    #[test]
    fn load_keeps_peaks_next_to_the_buffer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("peaks.wav");
        write_wav(&path, 8_000, 2, 1_600, |i, _| if i < 800 { 0.8 } else { 0.1 });
        let path = path.to_string_lossy().into_owned();

        load(&path, rate(8_000), channels(2)).unwrap();
        let (peaks, duration) = peek_peaks(&path).unwrap();
        assert!((duration - 0.2).abs() < 1e-6);
        let buckets = peaks.to_buckets(2, peaks.frames());
        assert!(buckets[1] > 0.7 && buckets[3] < 0.2, "{buckets:?}");
        invalidate(&path);
        assert!(peek_peaks(&path).is_none());
    }

    #[test]
    fn decode_to_format_resamples_and_remaps_to_the_requested_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mono.wav");
        write_wav(&path, 44_100, 1, 4_410, |_, _| 0.5);
        let samples = decode_to_format(&path.to_string_lossy(), 48_000, 2).unwrap();
        // 0.1 s of mono 44.1 kHz becomes exactly 0.1 s of stereo 48 kHz.
        assert_eq!(samples.len(), 4_800 * 2);
        let mid = samples.len() / 2;
        assert!((samples[mid] - 0.5).abs() < 0.01);
        assert_eq!(samples[mid], samples[mid + 1]);
    }
}
