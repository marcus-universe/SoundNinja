//! Decoded-PCM cache.
//!
//! Playback pulls ready-to-mix samples from here so the cpal callback never
//! decodes and never resamples. Buffers are stored already converted to the
//! output device's sample rate and channel layout; starting a sound is then an
//! `Arc` bump instead of a decoder setup, and seeking is an index jump.
//!
//! Files whose decoded form would exceed [`SoundCache::max_entry`] stay on the
//! streaming decoder in [`super::cache`].

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use rayon::prelude::*;
use rodio::buffer::SamplesBuffer;
use rodio::{ChannelCount, SampleRate, Source};

/// Total RAM budget for decoded PCM (512 MiB). Overridable at runtime.
const DEFAULT_MAX_TOTAL: usize = 512 * 1024 * 1024;
/// Per-file budget (128 MiB ≈ 11 min of 48 kHz stereo f32).
/// Longer files still decode off-thread for playback; they just are not kept.
const DEFAULT_MAX_ENTRY: usize = 128 * 1024 * 1024;
/// Drop buffers unused for this long unless they are currently playing.
const ENTRY_TTL: std::time::Duration = std::time::Duration::from_secs(180);

const BYTES_PER_SAMPLE: usize = std::mem::size_of::<f32>();

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
}

struct PcmCache {
    entries: HashMap<PcmKey, PcmEntry>,
    total_bytes: usize,
    max_total: usize,
    max_entry: usize,
}

impl PcmCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            total_bytes: 0,
            max_total: DEFAULT_MAX_TOTAL,
            max_entry: DEFAULT_MAX_ENTRY,
        }
    }

    /// Touch and return a ready-to-play clone. `O(1)` — no access-order list to
    /// rewrite, the LRU timestamp lives inside the entry.
    fn get(&mut self, key: &PcmKey) -> Option<(SamplesBuffer, f64)> {
        let entry = self.entries.get_mut(key)?;
        entry.last_used = Instant::now();
        Some((entry.buffer.clone(), entry.duration_secs))
    }

    fn insert(&mut self, key: PcmKey, buffer: SamplesBuffer, bytes: usize, duration_secs: f64) {
        if bytes > self.max_entry {
            return;
        }
        if let Some(old) = self.entries.remove(&key) {
            self.total_bytes = self.total_bytes.saturating_sub(old.bytes);
        }
        self.evict_until_fits(bytes);
        self.total_bytes += bytes;
        self.entries.insert(
            key,
            PcmEntry {
                buffer,
                bytes,
                duration_secs,
                last_used: Instant::now(),
            },
        );
    }

    /// Drop least-recently-used entries until `incoming` fits in the budget.
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

    fn evict_idle(&mut self, keep: &HashSet<String>) {
        let now = Instant::now();
        let stale: Vec<PcmKey> = self
            .entries
            .iter()
            .filter(|(k, e)| {
                !keep.contains(&k.path) && now.duration_since(e.last_used) > ENTRY_TTL
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

/// `true` when a file of this length is worth holding as decoded PCM.
/// `false` sends the caller to the streaming decoder instead.
pub fn fits(duration_secs: f64, rate: SampleRate, channels: ChannelCount) -> bool {
    let max_entry = cache().lock().map(|c| c.max_entry).unwrap_or(DEFAULT_MAX_ENTRY);
    estimated_bytes(duration_secs, rate, channels) <= max_entry
}

/// Cached buffer for `path` at the output format, or `None` on a miss.
pub fn peek(path: &str, rate: SampleRate, channels: ChannelCount) -> Option<(SamplesBuffer, f64)> {
    let key = make_key(path, rate, channels)?;
    cache().lock().ok()?.get(&key)
}

/// Insert an already-decoded buffer (pump promote). No-op if over budget.
pub fn store(
    path: &str,
    rate: SampleRate,
    channels: ChannelCount,
    samples: Vec<f32>,
    duration_secs: f64,
) {
    let Some(key) = make_key(path, rate, channels) else {
        return;
    };
    if samples.is_empty() {
        return;
    }
    let bytes = samples.len() * BYTES_PER_SAMPLE;
    let buffer = SamplesBuffer::new(channels, rate, samples);
    if let Ok(mut guard) = cache().lock() {
        guard.insert(key, buffer, bytes, duration_secs);
    }
}

/// Cached buffer, decoding and converting on a miss.
pub fn load(
    path: &str,
    rate: SampleRate,
    channels: ChannelCount,
) -> Result<(SamplesBuffer, f64), String> {
    if let Some(hit) = peek(path, rate, channels) {
        return Ok(hit);
    }
    let key = make_key(path, rate, channels)
        .ok_or_else(|| format!("Cannot stat '{path}'"))?;

    let (samples, src_rate, src_channels) = decode_file(path)?;
    let samples = convert(samples, src_rate, src_channels, rate.get(), channels.get());
    if samples.is_empty() {
        return Err(format!("Decoded zero samples from '{path}'"));
    }

    let bytes = samples.len() * BYTES_PER_SAMPLE;
    let buffer = SamplesBuffer::new(channels, rate, samples);
    let duration_secs = buffer
        .total_duration()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);

    let mut guard = cache().lock().map_err(|e| e.to_string())?;
    guard.insert(key.clone(), buffer.clone(), bytes, duration_secs);
    Ok((buffer, duration_secs))
}

/// Only short clips are pre-decoded. Long files would burn every core on
/// tab switch; they decode on first play via the progressive pump instead.
const PREFETCH_MAX_SECS: f64 = 30.0;
/// ~30 s of compressed audio; unknown-duration files above this stay byte-warm.
const PREFETCH_MAX_BYTES: u64 = 1_500_000;

/// Decode short clips into the PCM cache, one file at a time.
/// Long / unknown-large files only warm compressed bytes.
pub fn prefetch(paths: &[String], rate: SampleRate, channels: ChannelCount) {
    for path in paths {
        if peek(path, rate, channels).is_some() {
            continue;
        }
        let secs = super::cache::cached_duration(path).unwrap_or(0.0);
        let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        let long = secs > PREFETCH_MAX_SECS
            || (secs <= 0.0 && size > PREFETCH_MAX_BYTES)
            || !fits(if secs > 0.0 { secs } else { PREFETCH_MAX_SECS }, rate, channels);
        if long {
            if let Err(e) = super::cache::load_bytes(path) {
                eprintln!("[cache] warm failed '{path}': {e}");
            }
            continue;
        }
        if let Err(e) = load(path, rate, channels) {
            eprintln!("[pcm] prefetch failed '{path}': {e}");
        }
    }
}

pub fn invalidate(path: &str) {
    if let Ok(mut c) = cache().lock() {
        c.invalidate_path(path);
    }
}

/// Drop buffers idle past the TTL, except those currently playing.
pub fn evict_idle(keep: &[String]) {
    let set: HashSet<String> = keep.iter().cloned().collect();
    if let Ok(mut c) = cache().lock() {
        c.evict_idle(&set);
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

fn make_key(path: &str, rate: SampleRate, channels: ChannelCount) -> Option<PcmKey> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Some(PcmKey {
        path: path.to_owned(),
        mtime,
        size: meta.len(),
        sample_rate: rate.get(),
        channels: channels.get(),
    })
}

// ---------------------------------------------------------------------------
// Decode
// ---------------------------------------------------------------------------

/// Decode a file to interleaved f32 in [-1, 1] plus its native format.
/// WAV goes through hound for exact PCM; everything else through rodio.
pub fn decode_file(path: &str) -> Result<(Vec<f32>, u32, u16), String> {
    if path.to_lowercase().ends_with(".wav") {
        if let Some(decoded) = decode_wav(path) {
            return Ok(decoded);
        }
    }

    let file = File::open(path).map_err(|e| format!("Failed to open '{path}': {e}"))?;
    let decoder =
        rodio::Decoder::new(BufReader::with_capacity(64 * 1024, file)).map_err(|e| e.to_string())?;
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

// ---------------------------------------------------------------------------
// Format conversion (runs once per cached file, never in the audio callback)
// ---------------------------------------------------------------------------

/// Bring interleaved samples to the output device's rate and channel count.
/// Channel count is reduced before resampling and raised after it, so the
/// expensive stage always runs on the fewest channels.
fn convert(
    samples: Vec<f32>,
    src_rate: u32,
    src_channels: u16,
    dst_rate: u32,
    dst_channels: u16,
) -> Vec<f32> {
    let src_ch = src_channels.max(1);
    let dst_ch = dst_channels.max(1);
    if src_rate == dst_rate && src_ch == dst_ch {
        return samples;
    }

    let mut planes = deinterleave(&samples, src_ch);
    drop(samples);

    if dst_ch < src_ch {
        planes = remap_channels(planes, dst_ch);
    }
    if src_rate != dst_rate {
        planes = planes
            .par_iter()
            .map(|plane| resample_mono(plane, src_rate, dst_rate).unwrap_or_else(|_| plane.clone()))
            .collect();
    }
    if dst_ch > src_ch {
        planes = remap_channels(planes, dst_ch);
    }
    interleave(&planes)
}

/// Down-mix to mono by averaging, up-mix by cycling the existing planes.
/// Any other reduction keeps the leading channels rather than matrixing them.
fn remap_channels(planes: Vec<Vec<f32>>, dst_ch: u16) -> Vec<Vec<f32>> {
    let dst = dst_ch.max(1) as usize;
    if planes.is_empty() || planes.len() == dst {
        return planes;
    }
    if dst == 1 {
        let len = planes.iter().map(Vec::len).min().unwrap_or(0);
        let scale = 1.0 / planes.len() as f32;
        let mut mono = vec![0.0f32; len];
        for plane in &planes {
            for (out, &s) in mono.iter_mut().zip(plane.iter()) {
                *out += s * scale;
            }
        }
        return vec![mono];
    }
    (0..dst).map(|i| planes[i % planes.len()].clone()).collect()
}

fn deinterleave(samples: &[f32], channels: u16) -> Vec<Vec<f32>> {
    let ch = channels.max(1) as usize;
    if ch == 1 {
        return vec![samples.to_vec()];
    }
    let frames = samples.len() / ch;
    let mut planes = vec![Vec::with_capacity(frames); ch];
    for frame in samples.chunks_exact(ch) {
        for (plane, &s) in planes.iter_mut().zip(frame) {
            plane.push(s);
        }
    }
    planes
}

fn interleave(planes: &[Vec<f32>]) -> Vec<f32> {
    match planes {
        [] => Vec::new(),
        [mono] => mono.clone(),
        _ => {
            let frames = planes.iter().map(Vec::len).min().unwrap_or(0);
            let mut out = Vec::with_capacity(frames * planes.len());
            for i in 0..frames {
                for plane in planes {
                    out.push(plane[i]);
                }
            }
            out
        }
    }
}

/// Sinc resampler for one channel. Shared with the edit-session DSP so both
/// paths sound identical.
pub fn resample_mono(input: &[f32], from: u32, to: u32) -> Result<Vec<f32>, String> {
    use rubato::{
        Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
    };

    if from == to || input.is_empty() {
        return Ok(input.to_vec());
    }
    let params = SincInterpolationParameters {
        sinc_len: 64,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Linear,
        oversampling_factor: 256,
        window: WindowFunction::BlackmanHarris2,
    };
    let mut resampler = SincFixedIn::<f32>::new(to as f64 / from as f64, 2.0, params, 1024, 1)
        .map_err(|e| e.to_string())?;

    let chunk = resampler.input_frames_next();
    let expected = (input.len() as f64 * to as f64 / from as f64) as usize + chunk;
    let mut out = Vec::with_capacity(expected);
    let mut scratch = vec![vec![0.0f32; chunk]];
    let mut pos = 0;
    while pos + chunk <= input.len() {
        scratch[0].copy_from_slice(&input[pos..pos + chunk]);
        let waves_out = resampler
            .process(&scratch, None)
            .map_err(|e| e.to_string())?;
        out.extend_from_slice(&waves_out[0]);
        pos += chunk;
    }
    if pos < input.len() {
        let tail = &input[pos..];
        scratch[0][..tail.len()].copy_from_slice(tail);
        scratch[0][tail.len()..].fill(0.0);
        if let Ok(waves_out) = resampler.process(&scratch, None) {
            out.extend_from_slice(&waves_out[0]);
        }
    }
    Ok(out)
}
