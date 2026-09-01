//! In-memory audio edit sessions: waveform, trim, normalize, denoise, export.

use hound::{WavSpec, WavWriter};
use nnnoiseless::DenoiseState;
use rayon::prelude::*;
use rodio::Source;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

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
    let end_frame = ((end_s * sample_rate as f64).round() as usize)
        .min(frames)
        .max(start_frame + 1);
    let span = end_frame - start_frame;
    let mut peaks = Vec::with_capacity(buckets * 2);
    for b in 0..buckets {
        let start = start_frame + b * span / buckets;
        let end = (start_frame + (b + 1) * span / buckets).max(start + 1);
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

#[tauri::command(async)]
pub fn get_waveform_peaks(
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

/// Cache of streamed waveform peaks: (path, mtime_secs, buckets) → min/max pairs.
static PEAKS_CACHE: OnceLock<Mutex<HashMap<(String, u64, usize), Vec<f32>>>> = OnceLock::new();

fn peaks_cache() -> &'static Mutex<HashMap<(String, u64, usize), Vec<f32>>> {
    PEAKS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn file_mtime_secs(path: &str) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Fold interleaved samples into fixed bucket min/max peaks without retaining PCM.
fn fold_peaks_from_iter<I>(mut samples: I, channels: u16, total_frames: usize, buckets: usize) -> Vec<f32>
where
    I: Iterator<Item = f32>,
{
    let buckets = buckets.max(1);
    let ch = channels.max(1) as usize;
    if total_frames == 0 {
        return vec![0.0; buckets * 2];
    }
    let mut mins = vec![0.0f32; buckets];
    let mut maxs = vec![0.0f32; buckets];
    let mut frame = 0usize;
    while frame < total_frames {
        let mut frame_peak = 0.0f32;
        for _ in 0..ch {
            let s = samples.next().unwrap_or(0.0).abs();
            if s > frame_peak {
                frame_peak = s;
            }
        }
        let b = ((frame as u64 * buckets as u64) / total_frames as u64).min((buckets - 1) as u64)
            as usize;
        mins[b] = mins[b].min(-frame_peak);
        maxs[b] = maxs[b].max(frame_peak);
        frame += 1;
    }
    let mut peaks = Vec::with_capacity(buckets * 2);
    for b in 0..buckets {
        peaks.push(mins[b]);
        peaks.push(maxs[b]);
    }
    peaks
}

fn peaks_from_wav_streaming(path: &str, buckets: usize) -> Result<Vec<f32>, String> {
    let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let channels = spec.channels;
    let total_frames = reader.duration() as usize;
    if total_frames == 0 {
        return Ok(vec![0.0; buckets.max(1) * 2]);
    }
    match spec.sample_format {
        hound::SampleFormat::Float => {
            let iter = reader.samples::<f32>().map(|r| r.unwrap_or(0.0));
            Ok(fold_peaks_from_iter(iter, channels, total_frames, buckets))
        }
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            let iter = reader
                .samples::<i32>()
                .map(|r| r.map(|s| super::pcm::int_sample_to_f32(s, bits)).unwrap_or(0.0));
            Ok(fold_peaks_from_iter(iter, channels, total_frames, buckets))
        }
    }
}

fn peaks_from_rodio_streaming(path: &str, buckets: usize) -> Result<Vec<f32>, String> {
    let open = || {
        let file = File::open(path).map_err(|e| e.to_string())?;
        rodio::Decoder::new(BufReader::with_capacity(64 * 1024, file)).map_err(|e| e.to_string())
    };

    let decoder = open()?;
    let sample_rate: u32 = decoder.sample_rate().into();
    let channels: u16 = decoder.channels().into();

    let total_frames = if let Some(dur) = decoder.total_duration() {
        (dur.as_secs_f64() * sample_rate as f64).round().max(1.0) as usize
    } else {
        // Unknown length: count frames in a streaming pass (O(1) memory), then fold.
        let ch = channels.max(1) as usize;
        let mut n = 0usize;
        for _ in decoder {
            n += 1;
        }
        let frames = (n / ch).max(1);
        let decoder = open()?;
        return Ok(fold_peaks_from_iter(decoder, channels, frames, buckets));
    };

    let decoder = open()?;
    Ok(fold_peaks_from_iter(decoder, channels, total_frames, buckets))
}

fn peaks_from_file_streaming(path: &str, buckets: usize) -> Result<Vec<f32>, String> {
    let buckets = buckets.max(1);
    if path.to_lowercase().ends_with(".wav") {
        if let Ok(peaks) = peaks_from_wav_streaming(path, buckets) {
            return Ok(peaks);
        }
    }
    peaks_from_rodio_streaming(path, buckets)
}

/// Waveform peaks for an on-disk sound file (player scrubber).
/// Streams decode into buckets (O(buckets) memory) on a blocking pool thread.
#[tauri::command]
pub async fn get_file_waveform_peaks(path: String, buckets: usize) -> Result<Vec<f32>, String> {
    let buckets = buckets.max(1);
    let mtime = file_mtime_secs(&path);
    let key = (path.clone(), mtime, buckets);
    if let Ok(cache) = peaks_cache().lock() {
        if let Some(hit) = cache.get(&key) {
            return Ok(hit.clone());
        }
    }

    let path_for_worker = path.clone();
    let peaks = tauri::async_runtime::spawn_blocking(move || {
        if super::stream::is_pumping(&path_for_worker) {
            std::thread::sleep(std::time::Duration::from_millis(350));
            if let Some(p) = super::stream::active_peaks(&path_for_worker, buckets) {
                return Ok(p);
            }
        }
        peaks_from_file_streaming(&path_for_worker, buckets)
    })
    .await
    .map_err(|e| e.to_string())??;

    if let Ok(mut cache) = peaks_cache().lock() {
        if cache.len() >= 16 {
            cache.clear();
        }
        cache.insert(key, peaks.clone());
    }
    Ok(peaks)
}

#[tauri::command(async)]
pub fn trim_session(session_id: String, start_sec: f64, end_sec: f64) -> Result<SessionInfo, String> {
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

#[tauri::command(async)]
pub fn delete_range(session_id: String, start_sec: f64, end_sec: f64) -> Result<SessionInfo, String> {
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

#[tauri::command(async)]
pub fn normalize_session(
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

#[tauri::command(async)]
pub fn undo_session(session_id: String) -> Result<SessionInfo, String> {
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

#[tauri::command(async)]
pub fn redo_session(session_id: String) -> Result<SessionInfo, String> {
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
#[tauri::command(async)]
pub fn stage_session_clip(
    app: AppHandle,
    session_id: String,
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

    with_session_mut(&session_id, |sess| {
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

#[tauri::command(async)]
pub fn preview_session(
    app: AppHandle,
    session_id: String,
    device_name: String,
    host_name: Option<String>,
    start_sec: Option<f64>,
) -> Result<(), String> {
    let path = temp_preview_path(&app)?;
    with_session_mut(&session_id, |sess| {
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
    super::playback::enqueue_play(path_str, device_name, host_name, true)?;
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
