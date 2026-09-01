//! Progressive decode: preroll, play, fill ahead of the playhead.
//!
//! The WASAPI/cpal callback only ever copies from published `Arc` chunks.
//! A single pump thread decodes and (if needed) cheaply resamples. After the
//! first ~300 ms the rest is paced so one core stays near realtime.

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock, Weak};
use std::thread;
use std::time::{Duration, Instant};

use rodio::buffer::SamplesBuffer;
use rodio::source::SeekError;
use rodio::{ChannelCount, SampleRate, Source};
use rubato::{FastFixedIn, PolynomialDegree, Resampler};

use super::pcm::int_sample_to_f32;

const CHUNK_FRAMES: usize = 8192;
const PREROLL_SECS: f64 = 0.30;
const PACE_AHEAD_SECS: f64 = 6.0;
const PREROLL_TIMEOUT: Duration = Duration::from_millis(2000);

// ---------------------------------------------------------------------------
// Shared buffer
// ---------------------------------------------------------------------------

struct Shared {
    chunks: RwLock<Vec<Arc<[f32]>>>,
    samples_ready: AtomicUsize,
    consumed: AtomicUsize,
    done: AtomicBool,
    cancel: AtomicBool,
    duration_bits: AtomicU64,
    sample_rate: u32,
    channels: u16,
    path: String,
}

impl Shared {
    fn duration_secs(&self) -> f64 {
        f64::from_bits(self.duration_bits.load(Ordering::Relaxed))
    }

    fn set_duration(&self, secs: f64) {
        if secs.is_finite() && secs > 0.0 {
            self.duration_bits.store(secs.to_bits(), Ordering::Relaxed);
        }
    }

    fn ready_secs(&self) -> f64 {
        let ch = self.channels.max(1) as f64;
        let rate = self.sample_rate.max(1) as f64;
        self.samples_ready.load(Ordering::Acquire) as f64 / (rate * ch)
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

pub fn is_pumping(path: &str) -> bool {
    active_map()
        .lock()
        .ok()
        .and_then(|m| m.get(path).and_then(|w| w.upgrade()))
        .is_some()
}

/// Fold currently published samples into waveform peaks. `None` if this path
/// has no live pump or not enough audio to be useful yet.
pub fn active_peaks(path: &str, buckets: usize) -> Option<Vec<f32>> {
    let shared = active_map().lock().ok()?.get(path)?.upgrade()?;
    let ready = shared.samples_ready.load(Ordering::Acquire);
    if ready < shared.channels.max(1) as usize * 256 {
        return None;
    }
    let chunks = shared.chunks.read().ok()?;
    Some(fold_chunks(
        &chunks,
        shared.channels,
        ready,
        buckets.max(1),
    ))
}

fn fold_chunks(chunks: &[Arc<[f32]>], channels: u16, total_samples: usize, buckets: usize) -> Vec<f32> {
    let ch = channels.max(1) as usize;
    let total_frames = (total_samples / ch).max(1);
    let mut mins = vec![0.0f32; buckets];
    let mut maxs = vec![0.0f32; buckets];
    let mut frame = 0usize;
    for chunk in chunks {
        for frame_s in chunk.chunks_exact(ch) {
            let mut peak = 0.0f32;
            for &s in frame_s {
                let a = s.abs();
                if a > peak {
                    peak = a;
                }
            }
            let b = ((frame as u64 * buckets as u64) / total_frames as u64)
                .min((buckets - 1) as u64) as usize;
            mins[b] = mins[b].min(-peak);
            maxs[b] = maxs[b].max(peak);
            frame += 1;
        }
    }
    let mut out = Vec::with_capacity(buckets * 2);
    for b in 0..buckets {
        out.push(mins[b]);
        out.push(maxs[b]);
    }
    out
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

    pub fn duration_secs(&self) -> f64 {
        self.shared.duration_secs()
    }

    pub fn ready_secs(&self) -> f64 {
        self.shared.ready_secs()
    }

    /// Flatten published chunks into a `SamplesBuffer` once the pump finished.
    pub fn snapshot(&self) -> Option<SamplesBuffer> {
        if !self.is_done() {
            return None;
        }
        let chunks = self.shared.chunks.read().ok()?;
        let mut samples = Vec::with_capacity(self.shared.samples_ready.load(Ordering::Acquire));
        for c in chunks.iter() {
            samples.extend_from_slice(c);
        }
        if samples.is_empty() {
            return None;
        }
        let rate = SampleRate::new(self.shared.sample_rate)?;
        let ch = ChannelCount::new(self.shared.channels)?;
        Some(SamplesBuffer::new(ch, rate, samples))
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
        cancel: AtomicBool::new(false),
        duration_bits: AtomicU64::new(0),
        sample_rate: dst_rate.get(),
        channels: dst_channels.get(),
        path: path.clone(),
    });
    register(&shared);

    let pump_shared = Arc::clone(&shared);
    thread::Builder::new()
        .name("sn-pcm-pump".into())
        .spawn(move || {
            pump_loop(pump_shared, dst_rate.get(), dst_channels.get());
        })
        .map_err(|e| e.to_string())?;

    wait_preroll(&shared)?;
    Ok(PumpHandle { shared })
}

fn wait_preroll(shared: &Shared) -> Result<(), String> {
    let need = ((PREROLL_SECS * shared.sample_rate as f64 * shared.channels.max(1) as f64) as usize)
        .max(shared.channels.max(1) as usize * 64);
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
        CHUNK_FRAMES * self.shared.channels.max(1) as usize
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
            if self.pos % 64 == 0 {
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
        ChannelCount::new(self.shared.channels).unwrap_or(ChannelCount::new(1).unwrap())
    }

    fn sample_rate(&self) -> SampleRate {
        SampleRate::new(self.shared.sample_rate).unwrap_or(SampleRate::new(48_000).unwrap())
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
        let ch = self.shared.channels.max(1) as f64;
        let rate = self.shared.sample_rate.max(1) as f64;
        let target = (pos.as_secs_f64() * rate * ch) as usize;
        let ready = self.shared.samples_ready.load(Ordering::Acquire);
        self.pos = target.min(ready.saturating_sub(ch as usize));
        self.cached_idx = usize::MAX;
        self.cached = None;
        self.shared.consumed.store(self.pos, Ordering::Relaxed);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Pump
// ---------------------------------------------------------------------------

fn pump_loop(shared: Arc<Shared>, dst_rate: u32, dst_ch: u16) {
    let result = run_pump(&shared, dst_rate, dst_ch);
    if let Err(e) = result {
        if !shared.cancel.load(Ordering::Relaxed) {
            eprintln!("[pcm-pump] '{}': {e}", shared.path);
        }
    }
    shared.done.store(true, Ordering::Release);
    promote_if_complete(&shared, dst_rate, dst_ch);
    unregister(&shared.path);
}

fn run_pump(shared: &Shared, dst_rate: u32, dst_ch: u16) -> Result<(), String> {
    let (mut samples, src_rate, src_ch, hint) = open_sample_iter(&shared.path)?;
    if hint > 0.0 {
        shared.set_duration(hint);
    }

    let dst_ch = dst_ch.max(1);
    let src_ch = src_ch.max(1);
    let chunk_samples = CHUNK_FRAMES * dst_ch as usize;
    let mut tail: Vec<f32> = Vec::with_capacity(chunk_samples);
    let mut publisher = ChunkPublisher {
        shared,
        tail: &mut tail,
        chunk_samples,
    };

    let same_fmt = src_rate == dst_rate && src_ch == dst_ch;
    if same_fmt {
        for s in samples.by_ref() {
            if shared.cancel.load(Ordering::Relaxed) {
                return Ok(());
            }
            publisher.push(s);
            publisher.maybe_pace();
        }
        publisher.flush();
    } else {
        pump_convert(
            shared,
            &mut samples,
            src_rate,
            src_ch,
            dst_rate,
            dst_ch,
            &mut publisher,
        )?;
    }

    let exact = shared.ready_secs();
    if exact > 0.0 {
        shared.set_duration(exact);
        super::cache::store_duration(&shared.path, exact);
    }
    Ok(())
}

struct ChunkPublisher<'a> {
    shared: &'a Shared,
    tail: &'a mut Vec<f32>,
    chunk_samples: usize,
}

impl ChunkPublisher<'_> {
    fn push(&mut self, s: f32) {
        self.tail.push(s);
        if self.tail.len() >= self.chunk_samples {
            self.publish();
        }
    }

    fn push_slice(&mut self, samples: &[f32]) {
        for &s in samples {
            self.push(s);
        }
    }

    fn publish(&mut self) {
        if self.tail.is_empty() {
            return;
        }
        let chunk: Arc<[f32]> = std::mem::take(self.tail).into();
        let add = chunk.len();
        if let Ok(mut g) = self.shared.chunks.write() {
            g.push(chunk);
        }
        self.shared
            .samples_ready
            .fetch_add(add, Ordering::Release);
        self.tail.reserve(self.chunk_samples);
    }

    fn flush(&mut self) {
        self.publish();
    }

    fn maybe_pace(&self) {
        let ready = self.shared.ready_secs();
        if ready < PACE_AHEAD_SECS {
            return;
        }
        let ch = self.shared.channels.max(1) as f64;
        let rate = self.shared.sample_rate.max(1) as f64;
        let consumed = self.shared.consumed.load(Ordering::Relaxed) as f64 / (rate * ch);
        if ready - consumed > PACE_AHEAD_SECS {
            thread::sleep(Duration::from_millis(20));
        }
    }
}

fn pump_convert(
    shared: &Shared,
    samples: &mut dyn Iterator<Item = f32>,
    src_rate: u32,
    src_ch: u16,
    dst_rate: u32,
    dst_ch: u16,
    pubr: &mut ChunkPublisher<'_>,
) -> Result<(), String> {
    let resample_ch = src_ch.min(dst_ch).max(1) as usize;
    let need_resample = src_rate != dst_rate;
    let mut resampler = if need_resample {
        Some(
            FastFixedIn::<f32>::new(
                dst_rate as f64 / src_rate as f64,
                1.0,
                PolynomialDegree::Linear,
                1024,
                resample_ch,
            )
            .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };

    loop {
        if shared.cancel.load(Ordering::Relaxed) {
            return Ok(());
        }
        let pull_frames = resampler
            .as_ref()
            .map(|r| r.input_frames_next())
            .unwrap_or(1024);
        let Some(mut planes) = pull_planar(samples, pull_frames, src_ch) else {
            break;
        };
        if dst_ch < src_ch {
            planes = remap_channels(planes, dst_ch);
        }
        if let Some(rs) = resampler.as_mut() {
            let needed = rs.input_frames_next();
            for p in &mut planes {
                if p.len() < needed {
                    p.resize(needed, 0.0);
                }
            }
            let out = rs.process(&planes, None).map_err(|e| e.to_string())?;
            planes = out;
        }
        if dst_ch > src_ch {
            planes = remap_channels(planes, dst_ch);
        }
        let interleaved = interleave(&planes);
        pubr.push_slice(&interleaved);
        pubr.maybe_pace();
    }
    pubr.flush();
    Ok(())
}

fn promote_if_complete(shared: &Shared, dst_rate: u32, dst_ch: u16) {
    if shared.cancel.load(Ordering::Relaxed) {
        return;
    }
    let Some(rate) = SampleRate::new(dst_rate) else {
        return;
    };
    let Some(ch) = ChannelCount::new(dst_ch) else {
        return;
    };
    let Ok(chunks) = shared.chunks.read() else {
        return;
    };
    let mut samples = Vec::with_capacity(shared.samples_ready.load(Ordering::Acquire));
    for c in chunks.iter() {
        samples.extend_from_slice(c);
    }
    drop(chunks);
    if samples.is_empty() {
        return;
    }
    let duration = shared.duration_secs();
    super::pcm::store(&shared.path, rate, ch, samples, duration);
}

// ---------------------------------------------------------------------------
// Decode iterators
// ---------------------------------------------------------------------------

fn open_sample_iter(
    path: &str,
) -> Result<(Box<dyn Iterator<Item = f32> + Send>, u32, u16, f64), String> {
    if path.to_lowercase().ends_with(".wav") {
        if let Ok(wav) = WavIter::open(path) {
            let rate = wav.rate;
            let ch = wav.channels;
            let dur = wav.duration_secs;
            return Ok((Box::new(wav), rate, ch, dur));
        }
    }
    let file = File::open(path).map_err(|e| format!("Failed to open '{path}': {e}"))?;
    let decoder =
        rodio::Decoder::new(BufReader::with_capacity(64 * 1024, file)).map_err(|e| e.to_string())?;
    let rate: u32 = decoder.sample_rate().into();
    let ch: u16 = decoder.channels().into();
    let dur = decoder.total_duration().map(|d| d.as_secs_f64()).unwrap_or(0.0);
    Ok((Box::new(decoder), rate, ch, dur))
}

struct WavIter {
    reader: hound::WavReader<BufReader<File>>,
    bits: u16,
    float: bool,
    rate: u32,
    channels: u16,
    duration_secs: f64,
}

impl WavIter {
    fn open(path: &str) -> Result<Self, String> {
        let reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
        let spec = reader.spec();
        let duration_secs = reader.duration() as f64 / spec.sample_rate.max(1) as f64;
        Ok(Self {
            bits: spec.bits_per_sample,
            float: spec.sample_format == hound::SampleFormat::Float,
            rate: spec.sample_rate,
            channels: spec.channels,
            duration_secs,
            reader,
        })
    }
}

impl Iterator for WavIter {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.float {
            self.reader.samples::<f32>().next().map(|r| r.unwrap_or(0.0))
        } else {
            let bits = self.bits;
            self.reader
                .samples::<i32>()
                .next()
                .map(|r| r.map(|s| int_sample_to_f32(s, bits)).unwrap_or(0.0))
        }
    }
}

fn pull_planar(
    iter: &mut dyn Iterator<Item = f32>,
    frames: usize,
    src_ch: u16,
) -> Option<Vec<Vec<f32>>> {
    let ch = src_ch.max(1) as usize;
    let mut planes = vec![Vec::with_capacity(frames); ch];
    let mut got = 0usize;
    'outer: for _ in 0..frames {
        for c in 0..ch {
            match iter.next() {
                Some(s) => planes[c].push(s),
                None => break 'outer,
            }
        }
        got += 1;
    }
    if got == 0 {
        return None;
    }
    Some(planes)
}

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
