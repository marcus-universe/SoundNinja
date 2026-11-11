//! In-memory audio edit sessions: waveform, trim, normalize, denoise, export.

use hound::{WavSpec, WavWriter};
use nnnoiseless::DenoiseState;
use rayon::prelude::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

use super::decode::SampleSource;
use super::peaks::PeakAcc;
use super::playback::{pause_all, resume_all, stop_all};

#[derive(Clone)]
pub struct EditSession {
    /// Interleaved f32 samples in [-1, 1].
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    /// Previous buffer for one-level undo (stems / destructive ops).
    pub undo: Option<(Vec<f32>, u32, u16)>,
    /// Buffer restored by redo after an undo.
    pub redo: Option<(Vec<f32>, u32, u16)>,
}

const MAX_UNDO_SAMPLES: usize = 8 * 1024 * 1024; // ~32 MiB of f32

fn push_undo(sess: &mut EditSession) {
    sess.redo = None;
    if sess.samples.len() > MAX_UNDO_SAMPLES {
        sess.undo = None;
        return;
    }
    sess.undo = Some((sess.samples.clone(), sess.sample_rate, sess.channels));
}

#[derive(Clone, Serialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_secs: f64,
}

static SESSIONS: OnceLock<Mutex<HashMap<String, EditSession>>> = OnceLock::new();

fn sessions() -> &'static Mutex<HashMap<String, EditSession>> {
    SESSIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn new_session_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("sess_{ts}_{}", fastrand())
}

fn fastrand() -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    SystemTime::now().hash(&mut h);
    (h.finish() as u32) ^ std::process::id()
}

use super::pcm::{decode_file, resample_mono};

fn with_session_mut<F, R>(session_id: &str, f: F) -> Result<R, String>
where
    F: FnOnce(&mut EditSession) -> Result<R, String>,
{
    let mut map = sessions().lock().map_err(|e| e.to_string())?;
    let sess = map
        .get_mut(session_id)
        .ok_or_else(|| format!("Unknown session '{session_id}'"))?;
    f(sess)
}

fn sec_to_frame(sec: f64, sample_rate: u32, channels: u16) -> usize {
    let frames = (sec.max(0.0) * sample_rate as f64).round() as usize;
    frames.saturating_mul(channels as usize)
}

/// Resolve an optional time range to interleaved sample indices.
/// Missing bounds mean "from start" / "to end" (full session when both absent).
fn resolve_sample_range(
    sess: &EditSession,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> (usize, usize) {
    let len = sess.samples.len();
    let start = match start_sec {
        Some(s) => sec_to_frame(s.max(0.0), sess.sample_rate, sess.channels).min(len),
        None => 0,
    };
    let end = match end_sec {
        Some(e) => sec_to_frame(e.max(0.0), sess.sample_rate, sess.channels)
            .min(len)
            .max(start),
        None => len,
    };
    (start, end)
}

fn write_wav(path: &Path, samples: &[f32], sample_rate: u32, channels: u16) -> Result<(), String> {
    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create(path, spec).map_err(|e| e.to_string())?;
    for &s in samples {
        let i = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(i).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())
}

fn temp_preview_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("recordings");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // Unique name each preview so the sound cache never serves a stale rewrite.
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Ok(dir.join(format!("preview_{ts}.wav")))
}

fn denoise_channel(mono: &[f32], sample_rate: u32) -> Result<Vec<f32>, String> {
    let at_48k = resample_mono(mono, sample_rate, 48_000)?;
    let mut scaled: Vec<f32> = at_48k.iter().map(|&s| s * i16::MAX as f32).collect();
    let frame = DenoiseState::FRAME_SIZE;
    let rem = scaled.len() % frame;
    if rem != 0 {
        scaled.resize(scaled.len() + (frame - rem), 0.0);
    }

    let mut denoise = DenoiseState::new();
    let mut out_buf = [0.0f32; DenoiseState::FRAME_SIZE];
    let mut output = Vec::with_capacity(scaled.len());
    let mut first = true;
    for chunk in scaled.chunks_exact(frame) {
        denoise.process_frame(&mut out_buf[..], chunk);
        if !first {
            output.extend_from_slice(&out_buf[..]);
        }
        first = false;
    }
    let normalized: Vec<f32> = output.iter().map(|&s| s / i16::MAX as f32).collect();
    resample_mono(&normalized, 48_000, sample_rate)
}

fn deinterleave(samples: &[f32], channels: u16) -> Vec<Vec<f32>> {
    let ch = channels.max(1) as usize;
    let mut planes = vec![Vec::with_capacity(samples.len() / ch + 1); ch];
    for (i, &s) in samples.iter().enumerate() {
        planes[i % ch].push(s);
    }
    planes
}

fn interleave(planes: &[Vec<f32>]) -> Vec<f32> {
    if planes.is_empty() {
        return Vec::new();
    }
    let len = planes.iter().map(|p| p.len()).min().unwrap_or(0);
    let mut out = Vec::with_capacity(len * planes.len());
    for i in 0..len {
        for p in planes {
            out.push(p[i]);
        }
    }
    out
}

use crate::task::run_blocking;

// ── Commands ──────────────────────────────────────────────────────────────────

/// Full-file decode; can take seconds on long recordings, so it runs on the
/// blocking pool rather than a tokio worker.
#[tauri::command]
pub async fn load_edit_session(path: String) -> Result<SessionInfo, String> {
    run_blocking(move || load_edit_session_blocking(path)).await
}

fn load_edit_session_blocking(path: String) -> Result<SessionInfo, String> {
    let (samples, sample_rate, channels) = decode_file(&path)?;
    let duration_secs = samples.len() as f64 / (sample_rate as f64 * channels.max(1) as f64);
    let session_id = new_session_id();
    let info = SessionInfo {
        session_id: session_id.clone(),
        sample_rate,
        channels,
        duration_secs,
    };
    sessions().lock().map_err(|e| e.to_string())?.insert(
        session_id,
        EditSession {
            samples,
            sample_rate,
            channels,
            undo: None,
            redo: None,
        },
    );
    Ok(info)
}

fn peaks_from_samples(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
    buckets: usize,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Vec<f32> {
    let buckets = buckets.max(1);
    let ch = channels.max(1) as usize;
    let frames = samples.len() / ch;
    if frames == 0 {
        return vec![0.0; buckets * 2];
    }
    let dur = frames as f64 / sample_rate.max(1) as f64;
    let start_s = start_sec.unwrap_or(0.0).clamp(0.0, dur);
    let end_s = end_sec.unwrap_or(dur).clamp(start_s, dur);
    let start_frame = ((start_s * sample_rate as f64).round() as usize).min(frames);
    let end_frame = ((end_s * sample_rate as f64).round() as usize).min(frames);
    if end_frame <= start_frame {
        return vec![0.0; buckets * 2];
    }
    let span = end_frame - start_frame;
    let mut peaks = Vec::with_capacity(buckets * 2);
    for b in 0..buckets {
        let start = start_frame + b * span / buckets;
        let end = (start_frame + (b + 1) * span / buckets)
            .max(start + 1)
            .min(end_frame);
        if end <= start {
            peaks.push(0.0);
            peaks.push(0.0);
            continue;
        }
        let mut min_v = 0.0f32;
        let mut max_v = 0.0f32;
        for frame in start..end {
            let mut frame_peak = 0.0f32;
            for c in 0..ch {
                let s = samples[frame * ch + c].abs();
                if s > frame_peak {
                    frame_peak = s;
                }
            }
            min_v = min_v.min(-frame_peak);
            max_v = max_v.max(frame_peak);
        }
        peaks.push(min_v);
        peaks.push(max_v);
    }
    peaks
}

/// Folds the visible range of the session; long takes make this too heavy
/// for a tokio worker.
#[tauri::command]
pub async fn get_waveform_peaks(
    session_id: String,
    buckets: usize,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<Vec<f32>, String> {
    run_blocking(move || get_waveform_peaks_blocking(session_id, buckets, start_sec, end_sec)).await
}

fn get_waveform_peaks_blocking(
    session_id: String,
    buckets: usize,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<Vec<f32>, String> {
    with_session_mut(&session_id, |sess| {
        Ok(peaks_from_samples(
            &sess.samples,
            sess.sample_rate,
            sess.channels,
            buckets,
            start_sec,
            end_sec,
        ))
    })
}

// ── File waveform (player scrubber) ───────────────────────────────────────────

/// Waveform of a sound file for the scrubber.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWaveform {
    /// Interleaved `[min, max]` pairs, `2 * buckets` long.
    pub peaks: Vec<f32>,
    /// True while a pump for this path is still decoding: `peaks` covers only
    /// the start of the timeline and the caller should ask again.
    pub pumping: bool,
    /// Best known duration, `0` if unknown.
    pub duration_secs: f64,
}

/// Upper bound for one request, so a bad argument cannot allocate gigabytes.
const MAX_WAVEFORM_BUCKETS: usize = 1 << 16;
/// Finished waveforms kept for reuse.
const WAVEFORM_CACHE_ENTRIES: usize = 64;
/// Source frames decoded per batch when a waveform needs the file itself.
const WAVEFORM_BATCH_FRAMES: usize = 4096;

/// Complete peaks of one file. Holds the accumulator rather than buckets, so
/// one entry serves every bucket count the UI asks for.
struct CachedWaveform {
    /// `(mtime, size)` of the file the peaks were computed from.
    stamp: (u64, u64),
    peaks: Arc<PeakAcc>,
    duration_secs: f64,
    /// Value of the cache clock at the last hit, for LRU eviction.
    last_used: u64,
}

#[derive(Default)]
struct WaveformCache {
    entries: HashMap<String, CachedWaveform>,
    clock: u64,
}

impl WaveformCache {
    fn get(&mut self, path: &str, stamp: (u64, u64)) -> Option<(Arc<PeakAcc>, f64)> {
        self.clock += 1;
        let clock = self.clock;
        let entry = self.entries.get_mut(path).filter(|e| e.stamp == stamp)?;
        entry.last_used = clock;
        Some((Arc::clone(&entry.peaks), entry.duration_secs))
    }

    fn put(&mut self, path: &str, stamp: (u64, u64), peaks: Arc<PeakAcc>, duration_secs: f64) {
        self.clock += 1;
        if !self.entries.contains_key(path) && self.entries.len() >= WAVEFORM_CACHE_ENTRIES {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_used)
                .map(|(p, _)| p.clone());
            if let Some(oldest) = oldest {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(
            path.to_owned(),
            CachedWaveform {
                stamp,
                peaks,
                duration_secs,
                last_used: self.clock,
            },
        );
    }
}

static WAVEFORM_CACHE: OnceLock<Mutex<WaveformCache>> = OnceLock::new();

fn waveform_cache() -> &'static Mutex<WaveformCache> {
    WAVEFORM_CACHE.get_or_init(|| Mutex::new(WaveformCache::default()))
}

/// Paths whose file is being decoded for a waveform right now. A second
/// request for the same file waits for the first instead of decoding it again
/// (the scrubber polls while a long file is still being read).
struct WaveformDecodes {
    paths: Mutex<HashSet<String>>,
    finished: Condvar,
}

static WAVEFORM_DECODES: OnceLock<WaveformDecodes> = OnceLock::new();

fn waveform_decodes() -> &'static WaveformDecodes {
    WAVEFORM_DECODES.get_or_init(|| WaveformDecodes {
        paths: Mutex::new(HashSet::new()),
        finished: Condvar::new(),
    })
}

/// Marks `path` as being decoded until dropped.
struct DecodeTurn<'a> {
    path: &'a str,
}

impl<'a> DecodeTurn<'a> {
    /// Wait until nobody else decodes `path`, then take the turn.
    fn take(path: &'a str) -> Result<Self, String> {
        let decodes = waveform_decodes();
        let mut busy = decodes.paths.lock().map_err(|e| e.to_string())?;
        while busy.contains(path) {
            busy = decodes.finished.wait(busy).map_err(|e| e.to_string())?;
        }
        busy.insert(path.to_owned());
        Ok(Self { path })
    }
}

impl Drop for DecodeTurn<'_> {
    fn drop(&mut self) {
        let decodes = waveform_decodes();
        if let Ok(mut busy) = decodes.paths.lock() {
            busy.remove(self.path);
        }
        decodes.finished.notify_all();
    }
}

/// Decode a whole file into peaks without keeping the PCM. Also yields the
/// exact duration, which container headers often get wrong.
fn decode_file_peaks(path: &str) -> Result<(PeakAcc, f64), String> {
    let mut opened = SampleSource::open(path)?;
    let channels = opened.channels.max(1);
    let batch_samples = WAVEFORM_BATCH_FRAMES * channels as usize;
    let mut batch = Vec::with_capacity(batch_samples);
    let mut peaks = PeakAcc::new();
    loop {
        batch.clear();
        if opened.source.read(&mut batch, batch_samples) == 0 {
            break;
        }
        peaks.push(&batch, channels);
    }
    let duration_secs = peaks.frames() as f64 / opened.sample_rate.max(1) as f64;
    Ok((peaks, duration_secs))
}

fn cached_waveform(path: &str, stamp: (u64, u64)) -> Option<(Arc<PeakAcc>, f64)> {
    waveform_cache().lock().ok()?.get(path, stamp)
}

fn remember_waveform(path: &str, stamp: (u64, u64), peaks: &Arc<PeakAcc>, duration_secs: f64) {
    if let Ok(mut cache) = waveform_cache().lock() {
        cache.put(path, stamp, Arc::clone(peaks), duration_secs);
    }
}

fn complete_waveform(peaks: &PeakAcc, buckets: usize, duration_secs: f64) -> FileWaveform {
    FileWaveform {
        peaks: peaks.to_buckets(buckets, peaks.frames()),
        pumping: false,
        duration_secs,
    }
}

/// Cheapest source first: finished peaks, the decoded PCM cache entry, a live
/// pump, and only then a decode of the file. Returns the waveform and the
/// name of the source that served it.
fn file_waveform_blocking(path: &str, buckets: usize) -> Result<(FileWaveform, &'static str), String> {
    let buckets = buckets.clamp(1, MAX_WAVEFORM_BUCKETS);
    let stamp = super::pcm::file_stamp(path).ok_or_else(|| format!("Cannot stat '{path}'"))?;

    if let Some((peaks, duration)) = cached_waveform(path, stamp) {
        return Ok((complete_waveform(&peaks, buckets, duration), "peak-cache"));
    }
    if let Some((peaks, duration)) = super::pcm::peek_peaks(path) {
        remember_waveform(path, stamp, &peaks, duration);
        return Ok((complete_waveform(&peaks, buckets, duration), "pcm-cache"));
    }
    if let Some(live) = super::stream::pump_waveform(path) {
        if live.decoding {
            // Map the decoded part onto the metadata timeline, so it fills the
            // left of the waveform and the tail still to come stays empty.
            let hinted = (live.duration_secs * live.sample_rate as f64).round() as usize;
            let timeline = hinted.max(live.peaks.frames()).max(1);
            let waveform = FileWaveform {
                peaks: live.peaks.to_buckets(buckets, timeline),
                pumping: true,
                duration_secs: live.duration_secs,
            };
            return Ok((waveform, "pump-live"));
        }
        let peaks = Arc::new(live.peaks);
        remember_waveform(path, stamp, &peaks, live.duration_secs);
        return Ok((complete_waveform(&peaks, buckets, live.duration_secs), "pump-done"));
    }

    let _turn = DecodeTurn::take(path)?;
    // Whoever held the turn before may have just finished this very file.
    if let Some((peaks, duration)) = cached_waveform(path, stamp) {
        return Ok((complete_waveform(&peaks, buckets, duration), "peak-cache"));
    }
    let (peaks, duration) = decode_file_peaks(path)?;
    if duration > 0.0 {
        super::cache::store_duration(path, duration);
    }
    let peaks = Arc::new(peaks);
    remember_waveform(path, stamp, &peaks, duration);
    Ok((complete_waveform(&peaks, buckets, duration), "file-decode"))
}

fn file_waveform_timed(path: &str, buckets: usize) -> Result<FileWaveform, String> {
    let started = Instant::now();
    let (waveform, source) = file_waveform_blocking(path, buckets)?;
    crate::perf::log(|| {
        format!(
            "waveform source={source} buckets={buckets} {:.2}ms '{path}'",
            crate::perf::ms_since(started)
        )
    });
    Ok(waveform)
}

/// Waveform, decode state and duration of an on-disk sound file in one call.
/// Runs on the blocking pool: the fallback decodes the whole file.
#[tauri::command]
pub async fn get_file_waveform(path: String, buckets: usize) -> Result<FileWaveform, String> {
    run_blocking(move || file_waveform_timed(&path, buckets)).await
}

/// Waveform peaks only. Kept for callers that predate [`get_file_waveform`].
#[tauri::command]
pub async fn get_file_waveform_peaks(path: String, buckets: usize) -> Result<Vec<f32>, String> {
    run_blocking(move || file_waveform_timed(&path, buckets).map(|w| w.peaks)).await
}

/// True while a progressive decode pump is still filling this path.
#[tauri::command(async)]
pub fn is_sound_pumping(path: String) -> bool {
    super::stream::is_decoding(&path)
}

// ── Session edits ─────────────────────────────────────────────────────────────
//
// Each edit copies or rewrites the session buffer (minutes of audio for a long
// take), so the commands hand off to the blocking pool instead of occupying a
// tokio worker. The `*_blocking` functions hold the logic and are what the
// tests call.

#[tauri::command]
pub async fn trim_session(session_id: String, start_sec: f64, end_sec: f64) -> Result<SessionInfo, String> {
    run_blocking(move || trim_session_blocking(session_id, start_sec, end_sec)).await
}

fn trim_session_blocking(session_id: String, start_sec: f64, end_sec: f64) -> Result<SessionInfo, String> {
    with_session_mut(&session_id, |sess| {
        let start = sec_to_frame(start_sec, sess.sample_rate, sess.channels);
        let end = sec_to_frame(end_sec, sess.sample_rate, sess.channels)
            .min(sess.samples.len())
            .max(start);
        push_undo(sess);
        sess.samples = sess.samples[start..end].to_vec();
        Ok(SessionInfo {
            session_id: session_id.clone(),
            sample_rate: sess.sample_rate,
            channels: sess.channels,
            duration_secs: sess.samples.len() as f64
                / (sess.sample_rate as f64 * sess.channels.max(1) as f64),
        })
    })
}

#[tauri::command]
pub async fn delete_range(session_id: String, start_sec: f64, end_sec: f64) -> Result<SessionInfo, String> {
    run_blocking(move || delete_range_blocking(session_id, start_sec, end_sec)).await
}

fn delete_range_blocking(session_id: String, start_sec: f64, end_sec: f64) -> Result<SessionInfo, String> {
    with_session_mut(&session_id, |sess| {
        let start = sec_to_frame(start_sec, sess.sample_rate, sess.channels);
        let end = sec_to_frame(end_sec, sess.sample_rate, sess.channels)
            .min(sess.samples.len())
            .max(start);
        push_undo(sess);
        let mut new_samples = Vec::with_capacity(sess.samples.len() - (end - start));
        new_samples.extend_from_slice(&sess.samples[..start]);
        new_samples.extend_from_slice(&sess.samples[end..]);
        sess.samples = new_samples;
        Ok(SessionInfo {
            session_id: session_id.clone(),
            sample_rate: sess.sample_rate,
            channels: sess.channels,
            duration_secs: sess.samples.len() as f64
                / (sess.sample_rate as f64 * sess.channels.max(1) as f64),
        })
    })
}

#[tauri::command]
pub async fn normalize_session(
    session_id: String,
    target_peak_db: Option<f32>,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<SessionInfo, String> {
    run_blocking(move || normalize_session_blocking(session_id, target_peak_db, start_sec, end_sec))
        .await
}

fn normalize_session_blocking(
    session_id: String,
    target_peak_db: Option<f32>,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<SessionInfo, String> {
    with_session_mut(&session_id, |sess| {
        let (start, end) = resolve_sample_range(sess, start_sec, end_sec);
        if end <= start {
            return Err("Normalize range is empty".into());
        }
        let target_db = target_peak_db.unwrap_or(-1.0);
        let target = 10f32.powf(target_db / 20.0);
        let peak = sess.samples[start..end]
            .iter()
            .map(|s| s.abs())
            .fold(0.0f32, f32::max);
        if peak > 1e-9 {
            push_undo(sess);
            let gain = target / peak;
            for s in &mut sess.samples[start..end] {
                *s = (*s * gain).clamp(-1.0, 1.0);
            }
        }
        Ok(SessionInfo {
            session_id: session_id.clone(),
            sample_rate: sess.sample_rate,
            channels: sess.channels,
            duration_secs: sess.samples.len() as f64
                / (sess.sample_rate as f64 * sess.channels.max(1) as f64),
        })
    })
}

/// RNNoise over every channel; the slowest edit operation by far.
#[tauri::command]
pub async fn denoise_session(
    session_id: String,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<SessionInfo, String> {
    run_blocking(move || denoise_session_blocking(session_id, start_sec, end_sec)).await
}

fn denoise_session_blocking(
    session_id: String,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<SessionInfo, String> {
    with_session_mut(&session_id, |sess| {
        let (start, end) = resolve_sample_range(sess, start_sec, end_sec);
        if end <= start {
            return Err("Denoise range is empty".into());
        }
        push_undo(sess);
        let slice = &sess.samples[start..end];
        let planes = deinterleave(slice, sess.channels);
        let sample_rate = sess.sample_rate;
        // One RNNoise pass per channel; they are independent, so run them in parallel.
        let mut out_planes = planes
            .par_iter()
            .map(|plane| denoise_channel(plane, sample_rate))
            .collect::<Result<Vec<_>, String>>()?;
        let min_len = out_planes.iter().map(|p| p.len()).min().unwrap_or(0);
        for p in &mut out_planes {
            p.truncate(min_len);
        }
        let denoised = interleave(&out_planes);
        if start == 0 && end == sess.samples.len() {
            sess.samples = denoised;
        } else {
            let mut new_samples =
                Vec::with_capacity(sess.samples.len() - (end - start) + denoised.len());
            new_samples.extend_from_slice(&sess.samples[..start]);
            new_samples.extend_from_slice(&denoised);
            new_samples.extend_from_slice(&sess.samples[end..]);
            sess.samples = new_samples;
        }
        Ok(SessionInfo {
            session_id: session_id.clone(),
            sample_rate: sess.sample_rate,
            channels: sess.channels,
            duration_secs: sess.samples.len() as f64
                / (sess.sample_rate as f64 * sess.channels.max(1) as f64),
        })
    })
}

#[tauri::command]
pub async fn undo_session(session_id: String) -> Result<SessionInfo, String> {
    run_blocking(move || undo_session_blocking(session_id)).await
}

fn undo_session_blocking(session_id: String) -> Result<SessionInfo, String> {
    with_session_mut(&session_id, |sess| {
        let (samples, sr, ch) = sess.undo.take().ok_or("Nothing to undo")?;
        sess.redo = Some((sess.samples.clone(), sess.sample_rate, sess.channels));
        sess.samples = samples;
        sess.sample_rate = sr;
        sess.channels = ch;
        Ok(SessionInfo {
            session_id: session_id.clone(),
            sample_rate: sess.sample_rate,
            channels: sess.channels,
            duration_secs: sess.samples.len() as f64
                / (sess.sample_rate as f64 * sess.channels.max(1) as f64),
        })
    })
}

#[tauri::command]
pub async fn redo_session(session_id: String) -> Result<SessionInfo, String> {
    run_blocking(move || redo_session_blocking(session_id)).await
}

fn redo_session_blocking(session_id: String) -> Result<SessionInfo, String> {
    with_session_mut(&session_id, |sess| {
        let (samples, sr, ch) = sess.redo.take().ok_or("Nothing to redo")?;
        sess.undo = Some((sess.samples.clone(), sess.sample_rate, sess.channels));
        sess.samples = samples;
        sess.sample_rate = sr;
        sess.channels = ch;
        Ok(SessionInfo {
            session_id: session_id.clone(),
            sample_rate: sess.sample_rate,
            channels: sess.channels,
            duration_secs: sess.samples.len() as f64
                / (sess.sample_rate as f64 * sess.channels.max(1) as f64),
        })
    })
}

#[tauri::command]
pub async fn export_session(session_id: String, out_path: String) -> Result<(), String> {
    run_blocking(move || export_session_blocking(session_id, out_path)).await
}

fn export_session_blocking(session_id: String, out_path: String) -> Result<(), String> {
    with_session_mut(&session_id, |sess| {
        write_wav(
            Path::new(&out_path),
            &sess.samples,
            sess.sample_rate,
            sess.channels,
        )
    })
}

#[derive(Clone, Serialize)]
pub struct StagedClipInfo {
    pub path: String,
    pub duration_secs: f64,
}

/// Export a full session or a time range into a temp staged WAV for the batch list.
#[tauri::command]
pub async fn stage_session_clip(
    app: AppHandle,
    session_id: String,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<StagedClipInfo, String> {
    run_blocking(move || stage_session_clip_blocking(&app, &session_id, start_sec, end_sec)).await
}

fn stage_session_clip_blocking(
    app: &AppHandle,
    session_id: &str,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<StagedClipInfo, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("recordings")
        .join("staged");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = dir.join(format!("clip_{ts}.wav"));

    with_session_mut(session_id, |sess| {
        let start = sec_to_frame(start_sec.unwrap_or(0.0).max(0.0), sess.sample_rate, sess.channels)
            .min(sess.samples.len());
        let end = match end_sec {
            Some(e) => sec_to_frame(e.max(0.0), sess.sample_rate, sess.channels)
                .min(sess.samples.len())
                .max(start),
            None => sess.samples.len(),
        };
        if end <= start {
            return Err("Clip range is empty".into());
        }
        let slice = &sess.samples[start..end];
        write_wav(&path, slice, sess.sample_rate, sess.channels)?;
        let duration_secs = slice.len() as f64
            / (sess.sample_rate as f64 * sess.channels.max(1) as f64);
        Ok(StagedClipInfo {
            path: path.to_string_lossy().to_string(),
            duration_secs,
        })
    })
}

/// Writes the preview WAV and starts it. Both the write and the preroll wait
/// in `enqueue_play` block, so the whole command runs on the blocking pool.
#[tauri::command]
pub async fn preview_session(
    app: AppHandle,
    session_id: String,
    device_name: String,
    host_name: Option<String>,
    start_sec: Option<f64>,
) -> Result<(), String> {
    run_blocking(move || {
        preview_session_blocking(&app, &session_id, device_name, host_name, start_sec)
    })
    .await
}

fn preview_session_blocking(
    app: &AppHandle,
    session_id: &str,
    device_name: String,
    host_name: Option<String>,
    start_sec: Option<f64>,
) -> Result<(), String> {
    let path = temp_preview_path(app)?;
    with_session_mut(session_id, |sess| {
        let start = sec_to_frame(start_sec.unwrap_or(0.0).max(0.0), sess.sample_rate, sess.channels)
            .min(sess.samples.len());
        write_wav(
            &path,
            &sess.samples[start..],
            sess.sample_rate,
            sess.channels,
        )
    })?;
    let path_str = path.to_string_lossy().to_string();
    super::cache::invalidate_path(&path_str);
    let _ = stop_all();
    super::playback::enqueue_play(path_str, device_name, host_name, true, 1.0, false)?;
    Ok(())
}

#[tauri::command(async)]
pub fn stop_preview() -> Result<(), String> {
    stop_all()
}

#[tauri::command(async)]
pub fn pause_preview() -> Result<(), String> {
    pause_all()
}

#[tauri::command(async)]
pub fn resume_preview() -> Result<(), String> {
    resume_all()
}

/// Used by stems module to swap session audio after separation.
pub fn replace_session_samples(
    session_id: &str,
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
) -> Result<(), String> {
    replace_session_range(session_id, None, None, samples, sample_rate, channels)
}

/// Replace a time range (or the whole session) with processed samples.
pub fn replace_session_range(
    session_id: &str,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
) -> Result<(), String> {
    with_session_mut(session_id, |sess| {
        let (start, end) = resolve_sample_range(sess, start_sec, end_sec);
        let full = start == 0 && end == sess.samples.len();
        if full {
            push_undo(sess);
            sess.samples = samples;
            sess.sample_rate = sample_rate;
            sess.channels = channels;
            return Ok(());
        }
        if end <= start {
            return Err("Replace range is empty".into());
        }
        if sample_rate != sess.sample_rate || channels != sess.channels {
            return Err("Processed selection sample rate/channel mismatch".into());
        }
        push_undo(sess);
        let mut new_samples =
            Vec::with_capacity(sess.samples.len() - (end - start) + samples.len());
        new_samples.extend_from_slice(&sess.samples[..start]);
        new_samples.extend_from_slice(&samples);
        new_samples.extend_from_slice(&sess.samples[end..]);
        sess.samples = new_samples;
        Ok(())
    })
}

/// Clone a time range (or whole session) as interleaved PCM.
pub fn clone_session_range(
    session_id: &str,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<(Vec<f32>, u32, u16), String> {
    with_session_mut(session_id, |sess| {
        let (start, end) = resolve_sample_range(sess, start_sec, end_sec);
        if end <= start {
            return Err("Export range is empty".into());
        }
        Ok((
            sess.samples[start..end].to_vec(),
            sess.sample_rate,
            sess.channels,
        ))
    })
}

pub fn export_session_to_temp(
    app: &AppHandle,
    session_id: &str,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("recordings");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{session_id}_stems_in.wav"));
    with_session_mut(session_id, |sess| {
        let (start, end) = resolve_sample_range(sess, start_sec, end_sec);
        if end <= start {
            return Err("Export range is empty".into());
        }
        write_wav(
            &path,
            &sess.samples[start..end],
            sess.sample_rate,
            sess.channels,
        )
    })?;
    Ok(path)
}

pub fn session_info(session_id: &str) -> Result<SessionInfo, String> {
    with_session_mut(session_id, |sess| {
        Ok(SessionInfo {
            session_id: session_id.to_string(),
            sample_rate: sess.sample_rate,
            channels: sess.channels,
            duration_secs: sess.samples.len() as f64
                / (sess.sample_rate as f64 * sess.channels.max(1) as f64),
        })
    })
}

/// Insert an in-memory session for unit tests (no file decode).
#[cfg(test)]
pub(crate) fn insert_test_session(
    samples: Vec<f32>,
    sample_rate: u32,
    channels: u16,
) -> Result<String, String> {
    let session_id = new_session_id();
    sessions().lock().map_err(|e| e.to_string())?.insert(
        session_id.clone(),
        EditSession {
            samples,
            sample_rate,
            channels,
            undo: None,
            redo: None,
        },
    );
    Ok(session_id)
}

#[cfg(test)]
mod tests {
    use super::{
        decode_file_peaks, delete_range_blocking as delete_range, file_waveform_blocking,
        insert_test_session, normalize_session_blocking as normalize_session, peaks_from_samples,
        redo_session_blocking as redo_session, session_info, trim_session_blocking as trim_session,
        undo_session_blocking as undo_session, WaveformCache,
    };
    use crate::audio::decode::test_support::write_wav;
    use crate::audio::peaks::PeakAcc;
    use std::sync::Arc;

    fn sine_session() -> String {
        let sr = 8_000u32;
        let samples: Vec<f32> = (0..sr).map(|i| (i as f32 / sr as f32) * 0.5).collect();
        insert_test_session(samples, sr, 1).unwrap()
    }

    #[test]
    fn trim_session_should_keep_range() {
        let id = sine_session();
        let info = trim_session(id, 0.25, 0.75).unwrap();
        assert!((info.duration_secs - 0.5).abs() < 0.02);
    }

    #[test]
    fn delete_range_should_shorten_session() {
        let id = sine_session();
        let before = session_info(&id).unwrap().duration_secs;
        let after = delete_range(id, 0.0, 0.25).unwrap().duration_secs;
        assert!(after < before);
        assert!((after - 0.75).abs() < 0.05);
    }

    #[test]
    fn normalize_session_should_raise_peak() {
        let samples = vec![0.1f32; 800];
        let id = insert_test_session(samples, 8_000, 1).unwrap();
        normalize_session(id.clone(), Some(-1.0), None, None).unwrap();
        let peak = {
            let map = super::sessions().lock().unwrap();
            let sess = map.get(&id).unwrap();
            sess.samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max)
        };
        assert!(peak > 0.8);
    }

    #[test]
    fn undo_session_should_restore_samples() {
        let id = sine_session();
        let before = session_info(&id).unwrap().duration_secs;
        trim_session(id.clone(), 0.0, 0.2).unwrap();
        undo_session(id.clone()).unwrap();
        let after = session_info(&id).unwrap().duration_secs;
        assert!((after - before).abs() < 1e-6);
    }

    #[test]
    fn redo_session_should_reapply_an_undone_edit() {
        let id = sine_session();
        trim_session(id.clone(), 0.0, 0.2).unwrap();
        undo_session(id.clone()).unwrap();
        let redone = redo_session(id).unwrap().duration_secs;
        assert!((redone - 0.2).abs() < 0.01);
    }

    #[test]
    fn decode_file_peaks_keeps_loud_then_quiet() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("loud-quiet.wav");
        write_wav(&path, 8_000, 2, 2_048, |i, _| if i < 1_024 { 0.9 } else { 0.05 });
        let (acc, duration) = decode_file_peaks(&path.to_string_lossy()).unwrap();
        assert!((duration - 0.256).abs() < 1e-9);
        let peaks = acc.to_buckets(4, acc.frames());
        assert_eq!(peaks.len(), 8);
        assert!(peaks[1] > 0.8 && peaks[3] > 0.8);
        assert!(peaks[5] < 0.1 && peaks[7] < 0.1);
    }

    #[test]
    fn file_waveform_decodes_once_then_serves_any_bucket_count_from_the_cache() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wave.wav");
        write_wav(&path, 8_000, 1, 800, |i, _| if i < 400 { 0.8 } else { 0.2 });
        let path = path.to_string_lossy().into_owned();

        let (first, source) = file_waveform_blocking(&path, 10).unwrap();
        assert_eq!(source, "file-decode");
        assert_eq!(first.peaks.len(), 20);
        assert!(!first.pumping);
        assert!((first.duration_secs - 0.1).abs() < 1e-9);
        // A short clip has no zero buckets inside the audio.
        assert!(first.peaks.chunks_exact(2).all(|p| p[1] > 0.1));

        let (second, source) = file_waveform_blocking(&path, 200).unwrap();
        assert_eq!(source, "peak-cache");
        assert_eq!(second.peaks.len(), 400);
        assert!(second.peaks[1] > 0.7 && second.peaks[399] < 0.3);
    }

    #[test]
    fn file_waveform_notices_a_rewritten_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rewritten.wav");
        write_wav(&path, 8_000, 1, 800, |_, _| 0.2);
        let path_str = path.to_string_lossy().into_owned();
        let (quiet, _) = file_waveform_blocking(&path_str, 4).unwrap();
        assert!(quiet.peaks[1] < 0.3);

        // Different size, so the stamp changes even within the same second.
        write_wav(&path, 8_000, 1, 1_600, |_, _| 0.9);
        let (loud, source) = file_waveform_blocking(&path_str, 4).unwrap();
        assert_eq!(source, "file-decode");
        assert!(loud.peaks[1] > 0.8);
        assert!((loud.duration_secs - 0.2).abs() < 1e-9);
    }

    #[test]
    fn file_waveform_fails_for_a_missing_file() {
        assert!(file_waveform_blocking("/no/such/sound.wav", 8).is_err());
    }

    #[test]
    fn waveform_cache_evicts_the_least_recently_used_entry() {
        let mut cache = WaveformCache::default();
        let peaks = Arc::new(PeakAcc::new());
        for i in 0..super::WAVEFORM_CACHE_ENTRIES {
            cache.put(&format!("p{i}"), (1, 1), Arc::clone(&peaks), 1.0);
        }
        // Touch the oldest entry, then overflow by one.
        assert!(cache.get("p0", (1, 1)).is_some());
        cache.put("extra", (1, 1), Arc::clone(&peaks), 1.0);
        assert_eq!(cache.entries.len(), super::WAVEFORM_CACHE_ENTRIES);
        assert!(cache.get("p0", (1, 1)).is_some());
        assert!(cache.get("p1", (1, 1)).is_none());
        // A changed file stamp is a miss even though the path is known.
        assert!(cache.get("extra", (2, 1)).is_none());
    }

    #[test]
    fn peaks_at_end_of_clip_stay_in_range() {
        let samples = vec![0.5f32; 8];
        let peaks = peaks_from_samples(&samples, 8, 1, 4, Some(1.0), Some(1.0));
        assert_eq!(peaks.len(), 8);
        assert!(peaks.iter().all(|p| *p == 0.0));
    }
}
