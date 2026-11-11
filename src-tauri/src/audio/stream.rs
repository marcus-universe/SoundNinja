//! Progressive decode: preroll, play, fill ahead of the playhead.
//!
//! The WASAPI/cpal callback only ever copies from published `Arc` chunks.
//! A single pump thread decodes and (if needed) converts to the output format
//! with the same converter the PCM cache uses. After the first ~300 ms the
//! rest is paced so one core stays near realtime.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock, Weak};
use std::thread;
use std::time::{Duration, Instant};

use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};

use super::decode::SampleSource;
use super::peaks::PeakAcc;
use super::resample::FormatConverter;

const CHUNK_FRAMES: usize = 8192;
const PREROLL_SECS: f64 = 0.30;
const PACE_AHEAD_SECS: f64 = 6.0;
const PREROLL_TIMEOUT: Duration = Duration::from_millis(2000);
/// Source frames decoded per batch. Cancel and pacing are checked once per
/// batch; with the 20 ms pace sleep this holds a paced pump near realtime.
const PUMP_BATCH_FRAMES: usize = 1024;
/// Most chunk slots reserved up front (about 45 min of 48 kHz audio). The
/// length comes from a file header, which a damaged file can inflate.
const MAX_RESERVED_CHUNKS: usize = 16_384;

// ---------------------------------------------------------------------------
// Shared buffer
// ---------------------------------------------------------------------------

struct Shared {
    chunks: RwLock<Vec<Arc<[f32]>>>,
    samples_ready: AtomicUsize,
    consumed: AtomicUsize,
    /// The pump thread has stopped publishing (end of file, error or cancel).
    done: AtomicBool,
    /// The whole file was decoded: `duration_secs` is exact and the published
    /// samples are the complete clip. Set before `done`.
    complete: AtomicBool,
    cancel: AtomicBool,
    duration_bits: AtomicU64,
    sample_rate: SampleRate,
    channels: ChannelCount,
    path: String,
    /// Waveform peaks of everything published so far. Only the pump thread and
    /// waveform requests take this lock; the audio callback never does.
    peaks: Mutex<PeakAcc>,
}

impl Shared {
    fn duration_secs(&self) -> f64 {
        f64::from_bits(self.duration_bits.load(Ordering::Relaxed))
    }

    /// Raise the duration estimate. Never lowers it: while the pump is still
    /// filling, a shrinking timeline would make the playhead jump.
    fn set_duration(&self, secs: f64) {
        if !secs.is_finite() || secs <= 0.0 {
            return;
        }
        if secs > self.duration_secs() {
            self.duration_bits.store(secs.to_bits(), Ordering::Relaxed);
        }
    }

    /// Replace the estimate with the decoded length once the file is fully
    /// decoded. Header durations are often wrong (VBR MP3, truncated files),
    /// in either direction, so this may also lower the value.
    fn finish_duration(&self, exact_secs: f64) {
        if exact_secs.is_finite() && exact_secs > 0.0 {
            self.duration_bits
                .store(exact_secs.to_bits(), Ordering::Relaxed);
        }
    }

    fn samples_per_sec(&self) -> f64 {
        self.sample_rate.get() as f64 * self.channels.get() as f64
    }

    fn ready_secs(&self) -> f64 {
        self.samples_ready.load(Ordering::Acquire) as f64 / self.samples_per_sec()
    }
}

static ACTIVE: OnceLock<Mutex<HashMap<String, Weak<Shared>>>> = OnceLock::new();

fn active_map() -> &'static Mutex<HashMap<String, Weak<Shared>>> {
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn register(shared: &Arc<Shared>) {
    if let Ok(mut map) = active_map().lock() {
        map.retain(|_, w| w.strong_count() > 0);
        map.insert(shared.path.clone(), Arc::downgrade(shared));
    }
}

fn unregister(path: &str) {
    if let Ok(mut map) = active_map().lock() {
        map.remove(path);
    }
}

fn active_pump(path: &str) -> Option<Arc<Shared>> {
    active_map().lock().ok()?.get(path)?.upgrade()
}

/// True while a pump for this path is still decoding.
pub fn is_decoding(path: &str) -> bool {
    active_pump(path).is_some_and(|s| !s.done.load(Ordering::Acquire))
}

/// Waveform state of a live pump.
pub(crate) struct PumpWaveform {
    pub peaks: PeakAcc,
    /// The pump is still filling; `peaks` covers only the start of the clip.
    pub decoding: bool,
    pub duration_secs: f64,
    pub sample_rate: u32,
}

/// Peaks of the audio a pump has published for `path`. `None` when no pump is
/// registered, or when it stopped without decoding the whole file (its peaks
/// would pass a truncated clip off as the full waveform).
///
/// The peak lock is held only to copy the accumulator, never across a fold.
pub(crate) fn pump_waveform(path: &str) -> Option<PumpWaveform> {
    let shared = active_pump(path)?;
    let decoding = !shared.done.load(Ordering::Acquire);
    if !decoding && !shared.complete.load(Ordering::Acquire) {
        return None;
    }
    let peaks = shared.peaks.lock().ok()?.clone();
    Some(PumpWaveform {
        peaks,
        decoding,
        duration_secs: shared.duration_secs(),
        sample_rate: shared.sample_rate.get(),
    })
}

// ---------------------------------------------------------------------------
// Public handle
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct PumpHandle {
    shared: Arc<Shared>,
}

impl PumpHandle {
    pub fn source(&self) -> GrowingSource {
        GrowingSource {
            shared: Arc::clone(&self.shared),
            pos: 0,
            cached_idx: usize::MAX,
            cached: None,
        }
    }

    pub fn cancel(&self) {
        self.shared.cancel.store(true, Ordering::Release);
    }

    pub fn is_done(&self) -> bool {
        self.shared.done.load(Ordering::Acquire)
    }

    /// The whole file was decoded and `duration_secs` is its exact length.
    pub fn is_complete(&self) -> bool {
        self.shared.complete.load(Ordering::Acquire)
    }

    pub fn duration_secs(&self) -> f64 {
        self.shared.duration_secs()
    }

    pub fn ready_secs(&self) -> f64 {
        self.shared.ready_secs()
    }

    pub fn path(&self) -> &str {
        &self.shared.path
    }

    /// Output format the pump decodes to (the PCM cache key of its result).
    pub fn format(&self) -> (SampleRate, ChannelCount) {
        (self.shared.sample_rate, self.shared.channels)
    }
}

/// Start the pump and block only until preroll (or EOF for tiny files).
pub fn start(
    path: String,
    dst_rate: SampleRate,
    dst_channels: ChannelCount,
) -> Result<PumpHandle, String> {
    let shared = Arc::new(Shared {
        chunks: RwLock::new(Vec::new()),
        samples_ready: AtomicUsize::new(0),
        consumed: AtomicUsize::new(0),
        done: AtomicBool::new(false),
        complete: AtomicBool::new(false),
        cancel: AtomicBool::new(false),
        duration_bits: AtomicU64::new(0),
        sample_rate: dst_rate,
        channels: dst_channels,
        path,
        peaks: Mutex::new(PeakAcc::new()),
    });
    if let Some(secs) = super::cache::cached_duration(&shared.path) {
        shared.set_duration(secs);
    }
    register(&shared);

    let pump_shared = Arc::clone(&shared);
    thread::Builder::new()
        .name("sn-pcm-pump".into())
        .spawn(move || pump_loop(&pump_shared))
        .map_err(|e| e.to_string())?;

    wait_preroll(&shared)?;
    // Fallback only: metadata/hint already frozen on Shared when the container
    // reported a length. Do not let preroll overwrite that timeline.
    if shared.duration_secs() <= 0.0 {
        let floor = shared.ready_secs();
        if floor > 0.0 {
            shared.set_duration(floor);
        }
    }
    Ok(PumpHandle { shared })
}

fn wait_preroll(shared: &Shared) -> Result<(), String> {
    let channels = shared.channels.get() as usize;
    let need = ((PREROLL_SECS * shared.samples_per_sec()) as usize).max(channels * 64);
    let t0 = Instant::now();
    loop {
        if shared.cancel.load(Ordering::Relaxed) {
            return Err("Decode cancelled".into());
        }
        let ready = shared.samples_ready.load(Ordering::Acquire);
        if ready >= need || shared.done.load(Ordering::Acquire) {
            if ready == 0 {
                return Err("Decoded zero samples".into());
            }
            return Ok(());
        }
        if t0.elapsed() > PREROLL_TIMEOUT {
            if ready == 0 {
                return Err("Preroll timed out".into());
            }
            return Ok(());
        }
        thread::sleep(Duration::from_millis(4));
    }
}

// ---------------------------------------------------------------------------
// Growing source (audio callback)
// ---------------------------------------------------------------------------

pub struct GrowingSource {
    shared: Arc<Shared>,
    pos: usize,
    cached_idx: usize,
    cached: Option<Arc<[f32]>>,
}

impl GrowingSource {
    fn chunk_len(&self) -> usize {
        CHUNK_FRAMES * self.shared.channels.get() as usize
    }

    fn sample_at(&mut self, pos: usize) -> f32 {
        let len = self.chunk_len();
        let idx = pos / len;
        let off = pos % len;
        if self.cached_idx != idx {
            self.cached = self
                .shared
                .chunks
                .read()
                .ok()
                .and_then(|g| g.get(idx).cloned());
            self.cached_idx = idx;
        }
        self.cached.as_ref().map(|c| c[off]).unwrap_or(0.0)
    }
}

impl Iterator for GrowingSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let ready = self.shared.samples_ready.load(Ordering::Acquire);
        if self.pos < ready {
            let s = self.sample_at(self.pos);
            self.pos += 1;
            if self.pos.is_multiple_of(64) {
                self.shared.consumed.store(self.pos, Ordering::Relaxed);
            }
            return Some(s);
        }
        if self.shared.done.load(Ordering::Acquire) {
            self.shared.consumed.store(self.pos, Ordering::Relaxed);
            return None;
        }
        // Underrun: keep the clock moving. Preroll + pace make this rare.
        self.pos += 1;
        Some(0.0)
    }
}

impl Source for GrowingSource {
    fn current_span_len(&self) -> Option<usize> {
        let ready = self.shared.samples_ready.load(Ordering::Acquire);
        if self.pos >= ready && self.shared.done.load(Ordering::Acquire) {
            Some(0)
        } else {
            None
        }
    }

    fn channels(&self) -> ChannelCount {
        self.shared.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.shared.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        let secs = self.shared.duration_secs();
        if secs > 0.0 {
            Some(Duration::from_secs_f64(secs))
        } else {
            None
        }
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let channels = self.shared.channels.get() as usize;
        let target = (pos.as_secs_f64() * self.shared.samples_per_sec()) as usize;
        let ready = self.shared.samples_ready.load(Ordering::Acquire);
        self.pos = target.min(ready.saturating_sub(channels));
        self.cached_idx = usize::MAX;
        self.cached = None;
        self.shared.consumed.store(self.pos, Ordering::Relaxed);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Pump
// ---------------------------------------------------------------------------

fn pump_loop(shared: &Shared) {
    let complete = match run_pump(shared) {
        Ok(complete) => complete,
        Err(e) => {
            if !shared.cancel.load(Ordering::Relaxed) {
                eprintln!("[pcm-pump] '{}': {e}", shared.path);
            }
            false
        }
    };
    shared.complete.store(complete, Ordering::Release);
    shared.done.store(true, Ordering::Release);
    if complete {
        // Keep a finished pump visible so the waveform can use its peaks
        // without a second file decode. Dead Weaks are dropped on next register.
        promote(shared, super::pcm::max_entry_bytes());
    } else {
        // Cancelled or failed: what was published is not the whole clip.
        unregister(&shared.path);
    }
}

/// Decode the file into published chunks. `Ok(true)` when the whole file was
/// decoded, `Ok(false)` when the pump was cancelled first.
fn run_pump(shared: &Shared) -> Result<bool, String> {
    let mut opened = SampleSource::open(&shared.path)?;
    let hint = opened.duration_hint_secs;
    if hint > 0.0 {
        shared.set_duration(hint);
        super::cache::store_duration(&shared.path, hint);
    }

    let src_ch = opened.channels.max(1);
    let mut converter = FormatConverter::new(
        opened.sample_rate,
        src_ch,
        shared.sample_rate.get(),
        shared.channels.get(),
    )?;
    let mut publisher = ChunkPublisher::new(shared, hint);
    publisher.reserve_chunks(hint);

    let batch_samples = PUMP_BATCH_FRAMES * src_ch as usize;
    let mut batch: Vec<f32> = Vec::with_capacity(batch_samples);
    let mut converted: Vec<f32> = Vec::new();
    loop {
        if shared.cancel.load(Ordering::Relaxed) {
            return Ok(false);
        }
        batch.clear();
        if opened.source.read(&mut batch, batch_samples) == 0 {
            break;
        }
        if converter.is_passthrough() {
            publisher.push_slice(&batch);
        } else {
            converted.clear();
            converter.push(&batch, &mut converted)?;
            publisher.push_slice(&converted);
        }
        publisher.maybe_pace();
    }
    converted.clear();
    converter.finish(&mut converted)?;
    publisher.push_slice(&converted);
    publisher.flush();

    let exact = shared.ready_secs();
    if exact > 0.0 {
        shared.finish_duration(exact);
        super::cache::store_duration(&shared.path, exact);
    }
    Ok(true)
}

struct ChunkPublisher<'a> {
    shared: &'a Shared,
    /// Samples of the chunk being filled. Reused across chunks.
    tail: Vec<f32>,
    chunk_samples: usize,
    hint: f64,
}

impl<'a> ChunkPublisher<'a> {
    fn new(shared: &'a Shared, hint: f64) -> Self {
        let chunk_samples = CHUNK_FRAMES * shared.channels.get() as usize;
        Self {
            shared,
            tail: Vec::with_capacity(chunk_samples),
            chunk_samples,
            hint,
        }
    }

    /// Size the chunk list for the expected length up front, so publishing a
    /// chunk does not reallocate the list while holding the write lock the
    /// audio callback's reads wait on.
    fn reserve_chunks(&self, duration_secs: f64) {
        if duration_secs <= 0.0 {
            return;
        }
        let samples = duration_secs * self.shared.samples_per_sec();
        let chunks = (samples / self.chunk_samples as f64).ceil() as usize + 1;
        if let Ok(mut g) = self.shared.chunks.write() {
            g.reserve(chunks.min(MAX_RESERVED_CHUNKS));
        }
    }

    fn push_slice(&mut self, samples: &[f32]) {
        let mut rest = samples;
        while !rest.is_empty() {
            let room = self.chunk_samples - self.tail.len();
            let (head, tail) = rest.split_at(room.min(rest.len()));
            self.tail.extend_from_slice(head);
            rest = tail;
            if self.tail.len() >= self.chunk_samples {
                self.publish();
            }
        }
    }

    fn publish(&mut self) {
        if self.tail.is_empty() {
            return;
        }
        // Peaks first, while the samples are still a plain slice. Waveform
        // requests then never need to walk the published chunks.
        if let Ok(mut peaks) = self.shared.peaks.lock() {
            peaks.push(&self.tail, self.shared.channels.get());
        }
        let chunk: Arc<[f32]> = Arc::from(self.tail.as_slice());
        self.tail.clear();
        let add = chunk.len();
        if let Ok(mut g) = self.shared.chunks.write() {
            g.push(chunk);
        }
        self.shared
            .samples_ready
            .fetch_add(add, Ordering::Release);
        // Metadata hint is the timeline. Only grow duration from the buffer
        // when the container did not report a length.
        if self.hint <= 0.0 {
            self.shared.set_duration(self.shared.ready_secs());
        }
    }

    fn flush(&mut self) {
        self.publish();
    }

    fn maybe_pace(&self) {
        let ready = self.shared.ready_secs();
        if ready < PACE_AHEAD_SECS {
            return;
        }
        let consumed =
            self.shared.consumed.load(Ordering::Relaxed) as f64 / self.shared.samples_per_sec();
        if ready - consumed > PACE_AHEAD_SECS {
            thread::sleep(Duration::from_millis(20));
        }
    }
}

/// Store a fully decoded pump in the PCM cache. Returns whether it was stored.
///
/// A result above `max_entry_bytes` would be rejected by the cache anyway, so
/// it is not even flattened: that copy is the size of the whole decoded file.
fn promote(shared: &Shared, max_entry_bytes: usize) -> bool {
    if shared.cancel.load(Ordering::Relaxed) {
        return false;
    }
    let total = shared.samples_ready.load(Ordering::Acquire);
    if total == 0 || total.saturating_mul(std::mem::size_of::<f32>()) > max_entry_bytes {
        return false;
    }
    // Copy the chunk handles (cheap `Arc` bumps) and release the lock before
    // the flatten, so the audio callback is never held up behind it.
    let chunks: Vec<Arc<[f32]>> = match shared.chunks.read() {
        Ok(g) => g.clone(),
        Err(_) => return false,
    };
    let mut samples = Vec::with_capacity(total);
    for chunk in &chunks {
        samples.extend_from_slice(chunk);
    }
    drop(chunks);
    // Reuse the pump's peaks only when they cover exactly these samples.
    let frames = total / shared.channels.get() as usize;
    let peaks = shared
        .peaks
        .lock()
        .ok()
        .filter(|p| p.frames() == frames)
        .map(|p| p.clone());
    super::pcm::store(&shared.path, shared.sample_rate, shared.channels, samples, peaks);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::decode::test_support::write_wav;

    fn rate(hz: u32) -> SampleRate {
        SampleRate::new(hz).unwrap()
    }

    fn channels(n: u16) -> ChannelCount {
        ChannelCount::new(n).unwrap()
    }

    /// Run a pump to the end and return everything it published.
    fn pump_to_end(path: &str, dst_rate: u32, dst_ch: u16) -> (PumpHandle, Vec<f32>) {
        let handle = start(path.to_owned(), rate(dst_rate), channels(dst_ch)).unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        while !handle.is_done() {
            assert!(Instant::now() < deadline, "pump did not finish");
            thread::sleep(Duration::from_millis(5));
        }
        let samples: Vec<f32> = handle.source().collect();
        (handle, samples)
    }

    fn tone(frame: usize, channel: u16) -> f32 {
        let phase = frame as f32 * 0.02 * (channel as f32 + 1.0);
        phase.sin() * 0.6
    }

    #[test]
    fn pump_output_equals_cache_decode_when_resampling() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("resampled.wav");
        // Longer than one published chunk, so chunk boundaries are covered.
        write_wav(&path, 44_100, 2, 30_000, tone);
        let path = path.to_string_lossy().into_owned();

        let (handle, pumped) = pump_to_end(&path, 48_000, 2);
        let cached = super::super::pcm::decode_to_format(&path, 48_000, 2).unwrap();

        assert!(handle.is_complete());
        assert_eq!(pumped.len(), cached.len());
        let worst = pumped
            .iter()
            .zip(&cached)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(worst < 1e-6, "largest difference {worst}");
        super::super::pcm::invalidate(&path);
    }

    #[test]
    fn pump_output_equals_cache_decode_when_only_channels_differ() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("mono.wav");
        write_wav(&path, 48_000, 1, 10_000, tone);
        let path = path.to_string_lossy().into_owned();

        let (_, pumped) = pump_to_end(&path, 48_000, 2);
        let cached = super::super::pcm::decode_to_format(&path, 48_000, 2).unwrap();
        assert_eq!(pumped, cached);
        super::super::pcm::invalidate(&path);
    }

    /// Compressed input goes through rodio instead of hound, and here also
    /// through the resampler and the mono → stereo up-mix.
    #[test]
    fn pump_output_equals_cache_decode_for_an_mp3() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("tone-1s.mp3")
            .to_string_lossy()
            .into_owned();

        let (handle, pumped) = pump_to_end(&path, 48_000, 2);
        let cached = super::super::pcm::decode_to_format(&path, 48_000, 2).unwrap();
        assert!(handle.is_complete());
        assert_eq!(pumped, cached);
        // About one second of audio, and the duration is the decoded length.
        let decoded_secs = pumped.len() as f64 / (48_000.0 * 2.0);
        assert!((decoded_secs - 1.0).abs() < 0.1, "decoded {decoded_secs}s");
        assert!((handle.duration_secs() - decoded_secs).abs() < 1e-9);
        super::super::pcm::invalidate(&path);
    }

    #[test]
    fn pump_replaces_the_header_duration_with_the_decoded_length() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("exact.wav");
        write_wav(&path, 8_000, 1, 4_000, tone);
        let path = path.to_string_lossy().into_owned();
        // A stale, too-long estimate, as a VBR MP3 header would give.
        super::super::cache::store_duration(&path, 9.0);

        let (handle, _) = pump_to_end(&path, 8_000, 1);
        assert!(handle.is_complete());
        assert!((handle.duration_secs() - 0.5).abs() < 1e-9);
        let cached = super::super::cache::cached_duration(&path).unwrap();
        assert!((cached - 0.5).abs() < 1e-9);
        super::super::pcm::invalidate(&path);
    }

    #[test]
    fn pump_promotes_a_finished_decode_into_the_pcm_cache_with_peaks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("promote.wav");
        write_wav(&path, 8_000, 1, 1_600, |i, _| if i < 800 { 0.8 } else { 0.1 });
        let path = path.to_string_lossy().into_owned();

        let (handle, pumped) = pump_to_end(&path, 8_000, 1);
        // The pump stores its result right after it reports done.
        let deadline = Instant::now() + Duration::from_secs(5);
        let (buffer, duration) = loop {
            if let Some(hit) = super::super::pcm::peek(&path, rate(8_000), channels(1)) {
                break hit;
            }
            assert!(Instant::now() < deadline, "pump result never reached the cache");
            thread::sleep(Duration::from_millis(5));
        };
        assert_eq!(buffer.collect::<Vec<f32>>(), pumped);
        assert!((duration - 0.2).abs() < 1e-9);

        let live = pump_waveform(&path).unwrap();
        assert!(!live.decoding);
        let buckets = live.peaks.to_buckets(2, live.peaks.frames());
        assert!(buckets[1] > 0.7 && buckets[3] < 0.2, "{buckets:?}");
        drop(handle);
        super::super::pcm::invalidate(&path);
    }

    #[test]
    fn promote_skips_a_decode_larger_than_the_per_file_limit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.wav");
        write_wav(&path, 8_000, 1, 2_000, tone);
        let path = path.to_string_lossy().into_owned();

        // Not started as a pump: `promote` is called by hand with a tiny limit.
        let shared = Shared {
            chunks: RwLock::new(vec![Arc::from(vec![0.5f32; 2_000])]),
            samples_ready: AtomicUsize::new(2_000),
            consumed: AtomicUsize::new(0),
            done: AtomicBool::new(true),
            complete: AtomicBool::new(true),
            cancel: AtomicBool::new(false),
            duration_bits: AtomicU64::new(0),
            sample_rate: rate(8_000),
            channels: channels(1),
            path: path.clone(),
            peaks: Mutex::new(PeakAcc::new()),
        };
        let limit = 1_999 * std::mem::size_of::<f32>();
        assert!(!promote(&shared, limit));
        assert!(super::super::pcm::peek(&path, rate(8_000), channels(1)).is_none());

        assert!(promote(&shared, limit + std::mem::size_of::<f32>()));
        assert!(super::super::pcm::peek(&path, rate(8_000), channels(1)).is_some());
        super::super::pcm::invalidate(&path);
    }

    #[test]
    fn cancelled_pump_is_not_reported_as_a_waveform_source() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cancel.wav");
        write_wav(&path, 8_000, 1, 4_000, tone);
        let path = path.to_string_lossy().into_owned();

        let shared = Arc::new(Shared {
            chunks: RwLock::new(Vec::new()),
            samples_ready: AtomicUsize::new(0),
            consumed: AtomicUsize::new(0),
            done: AtomicBool::new(false),
            complete: AtomicBool::new(false),
            cancel: AtomicBool::new(true),
            duration_bits: AtomicU64::new(0),
            sample_rate: rate(8_000),
            channels: channels(1),
            path: path.clone(),
            peaks: Mutex::new(PeakAcc::new()),
        });
        register(&shared);
        assert!(is_decoding(&path));
        pump_loop(&shared);
        assert!(!is_decoding(&path));
        assert!(pump_waveform(&path).is_none());
        assert!(super::super::pcm::peek(&path, rate(8_000), channels(1)).is_none());
    }
}
