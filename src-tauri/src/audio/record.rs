//! Input / loopback recording via cpal.
//!
//! WASAPI loopback: call `build_input_stream` on an *output* device — cpal sets
//! `AUDCLNT_STREAMFLAGS_LOOPBACK` automatically.
//!
//! Capture callback never locks the sample buffer (live peak UI used to block it
//! after ~8s → pops/crackles). Chunks go through an mpsc channel; a collector
//! thread owns the `Vec<f32>`.

use cpal::traits::{DeviceTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use rodio::cpal;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender, SyncSender, TrySendError};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

use super::devices::find_device_by_name;

#[derive(Clone, Serialize)]
pub struct RecordStartInfo {
    pub sample_rate: u32,
    pub channels: u16,
}

struct ActiveRecording {
    stop_flag: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
    channels: u16,
    /// Stream thread (owns cpal stream + chunk sender).
    _stream_join: Option<thread::JoinHandle<()>>,
    /// Appends channel chunks into `samples`.
    _collector_join: Option<thread::JoinHandle<()>>,
}

static ACTIVE: OnceLock<Mutex<Option<ActiveRecording>>> = OnceLock::new();
static LEVEL: AtomicU32 = AtomicU32::new(0);
/// Capture gain applied while recording (0.0–2.0, default 1.0). Stored as f32 bits.
static INPUT_VOLUME: AtomicU32 = AtomicU32::new(0x3F800000);

fn active() -> &'static Mutex<Option<ActiveRecording>> {
    ACTIVE.get_or_init(|| Mutex::new(None))
}

fn current_input_volume() -> f32 {
    f32::from_bits(INPUT_VOLUME.load(Ordering::Relaxed))
}

fn write_level(peak: f32) {
    LEVEL.store(peak.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

/// Apply gain, update level meter, enqueue chunk. Never touches the sample `Mutex`.
fn enqueue_f32(tx: &SyncSender<Vec<f32>>, data: &[f32]) {
    let gain = current_input_volume();
    let mut peak = 0.0f32;
    let mut chunk = Vec::with_capacity(data.len());
    for &s in data {
        let v = (s * gain).clamp(-1.0, 1.0);
        let a = v.abs();
        if a > peak {
            peak = a;
        }
        chunk.push(v);
    }
    write_level(peak);
    match tx.try_send(chunk) {
        Ok(()) => {}
        // Bound full: block briefly so we prefer latency over silent drops.
        Err(TrySendError::Full(chunk)) => {
            let _ = tx.send(chunk);
        }
        Err(TrySendError::Disconnected(_)) => {}
    }
}

fn enqueue_i16(tx: &SyncSender<Vec<f32>>, data: &[i16]) {
    let gain = current_input_volume();
    let scale = gain / i16::MAX as f32;
    let mut peak = 0.0f32;
    let mut chunk = Vec::with_capacity(data.len());
    for &s in data {
        let v = (s as f32 * scale).clamp(-1.0, 1.0);
        let a = v.abs();
        if a > peak {
            peak = a;
        }
        chunk.push(v);
    }
    write_level(peak);
    match tx.try_send(chunk) {
        Ok(()) => {}
        Err(TrySendError::Full(chunk)) => {
            let _ = tx.send(chunk);
        }
        Err(TrySendError::Disconnected(_)) => {}
    }
}

#[tauri::command]
pub fn set_input_volume(volume: f32) {
    // Allow up to +6 dB (~2.0) for quiet mics.
    INPUT_VOLUME.store(volume.clamp(0.0, 2.0).to_bits(), Ordering::Relaxed);
}

#[tauri::command]
pub fn get_input_volume() -> f32 {
    current_input_volume()
}

fn build_stream_f32(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    tx: SyncSender<Vec<f32>>,
    stop: Arc<AtomicBool>,
) -> Result<cpal::Stream, String> {
    let err_fn = |e| eprintln!("record stream error: {e}");
    device
        .build_input_stream(
            config,
            move |data: &[f32], _| {
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                enqueue_f32(&tx, data);
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())
}

fn build_stream_i16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    tx: SyncSender<Vec<f32>>,
    stop: Arc<AtomicBool>,
) -> Result<cpal::Stream, String> {
    let err_fn = |e| eprintln!("record stream error: {e}");
    device
        .build_input_stream(
            config,
            move |data: &[i16], _| {
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                enqueue_i16(&tx, data);
            },
            err_fn,
            None,
        )
        .map_err(|e| e.to_string())
}

fn spawn_collector(
    rx: Receiver<Vec<f32>>,
    samples: Arc<Mutex<Vec<f32>>>,
    pre_reserve: usize,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if let Ok(mut g) = samples.lock() {
            g.reserve(pre_reserve);
        }
        while let Ok(chunk) = rx.recv() {
            if let Ok(mut g) = samples.lock() {
                g.extend_from_slice(&chunk);
            }
        }
    })
}

fn temp_record_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("recordings");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[tauri::command]
pub fn start_recording(
    app: AppHandle,
    device_name: String,
    host_name: Option<String>,
    loopback: bool,
) -> Result<RecordStartInfo, String> {
    let mut guard = active().lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        return Err("Already recording".into());
    }

    // Strip "(PC Audio)" suffix if the UI sent the decorated name.
    let clean_name = device_name
        .strip_suffix(" (PC Audio)")
        .unwrap_or(&device_name)
        .to_string();

    let device = find_device_by_name(&clean_name, host_name.as_deref(), loopback)
        .map_err(|e| e.to_string())?;

    let supported = if loopback {
        device
            .default_output_config()
            .map_err(|e| format!("No output config for loopback: {e}"))?
    } else {
        device
            .default_input_config()
            .map_err(|e| format!("No input config: {e}"))?
    };

    let sample_format = supported.sample_format();
    let config: cpal::StreamConfig = supported.clone().into();
    let sample_rate: u32 = config.sample_rate;
    let channels: u16 = config.channels;

    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
    let stop_flag = Arc::new(AtomicBool::new(false));
    write_level(0.0);

    // ~2s of chunks before producer blocks — absorbs peak-UI lock spikes.
    let (chunk_tx, chunk_rx) = std::sync::mpsc::sync_channel::<Vec<f32>>(64);
    let pre_reserve = (sample_rate as usize)
        .saturating_mul(channels as usize)
        .saturating_mul(120); // ~2 min headroom
    let collector_join = spawn_collector(chunk_rx, samples.clone(), pre_reserve);

    let stop_c = stop_flag.clone();
    let (ready_tx, ready_rx): (Sender<Result<(), String>>, _) = channel();

    let stream_join = thread::spawn(move || {
        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                build_stream_f32(&device, &config, chunk_tx, stop_c.clone())
            }
            cpal::SampleFormat::I16 => {
                build_stream_i16(&device, &config, chunk_tx, stop_c.clone())
            }
            other => Err(format!("Unsupported sample format: {other:?}")),
        };

        match stream {
            Ok(s) => {
                if let Err(e) = s.play() {
                    let _ = ready_tx.send(Err(e.to_string()));
                    return;
                }
                let _ = ready_tx.send(Ok(()));
                // Keep stream alive until stop (dropping sender closes collector).
                while !stop_c.load(Ordering::Relaxed) {
                    thread::sleep(std::time::Duration::from_millis(50));
                }
                drop(s);
            }
            Err(e) => {
                let _ = ready_tx.send(Err(e));
            }
        }
    });

    ready_rx
        .recv()
        .map_err(|e| e.to_string())?
        .map_err(|e| e)?;

    // Level meter ticker
    let app_level = app.clone();
    let stop_level = stop_flag.clone();
    thread::spawn(move || {
        while !stop_level.load(Ordering::Relaxed) {
            let bits = LEVEL.load(Ordering::Relaxed);
            let level = f32::from_bits(bits);
            let _ = app_level.emit("record_level", level);
            thread::sleep(std::time::Duration::from_millis(50));
        }
    });

    *guard = Some(ActiveRecording {
        stop_flag,
        samples,
        sample_rate,
        channels,
        _stream_join: Some(stream_join),
        _collector_join: Some(collector_join),
    });

    Ok(RecordStartInfo {
        sample_rate,
        channels,
    })
}

#[tauri::command]
pub fn stop_recording(app: AppHandle) -> Result<String, String> {
    let mut guard = active().lock().map_err(|e| e.to_string())?;
    let rec = guard.take().ok_or("Not recording")?;
    rec.stop_flag.store(true, Ordering::Relaxed);
    // Stream drop closes chunk sender → collector drains + exits.
    if let Some(j) = rec._stream_join {
        let _ = j.join();
    }
    if let Some(j) = rec._collector_join {
        let _ = j.join();
    }
    write_level(0.0);
    let _ = app.emit("record_level", 0.0f32);

    let samples = rec.samples.lock().map_err(|e| e.to_string())?.clone();
    if samples.is_empty() {
        return Err("No audio captured".into());
    }

    let dir = temp_record_dir(&app)?;
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let path = dir.join(format!("recording_{ts}.wav"));

    let spec = WavSpec {
        channels: rec.channels,
        sample_rate: rec.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = WavWriter::create(&path, spec).map_err(|e| e.to_string())?;
    for s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i = (clamped * i16::MAX as f32) as i16;
        writer.write_sample(i).map_err(|e| e.to_string())?;
    }
    writer.finalize().map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

/// Live min/max peaks while recording: `[min0, max0, min1, max1, …]` plus duration.
#[tauri::command]
pub fn get_live_record_peaks(
    buckets: usize,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<(f64, Vec<f32>), String> {
    let buckets = buckets.max(1);
    let guard = active().lock().map_err(|e| e.to_string())?;
    let Some(rec) = guard.as_ref() else {
        return Ok((0.0, vec![0.0; buckets * 2]));
    };
    // Safe to block here: capture callback no longer shares this mutex.
    let samples = rec.samples.lock().map_err(|e| e.to_string())?;
    let ch = rec.channels.max(1) as usize;
    let frames = samples.len() / ch;
    let duration = if rec.sample_rate == 0 || ch == 0 {
        0.0
    } else {
        frames as f64 / rec.sample_rate as f64
    };
    if frames == 0 {
        return Ok((duration, vec![0.0; buckets * 2]));
    }
    let start_s = start_sec.unwrap_or(0.0).clamp(0.0, duration);
    let end_s = end_sec.unwrap_or(duration).clamp(start_s, duration);
    let start_frame = ((start_s * rec.sample_rate as f64).round() as usize).min(frames);
    let end_frame = ((end_s * rec.sample_rate as f64).round() as usize)
        .min(frames)
        .max(start_frame + 1);
    let span = end_frame - start_frame;
    let mut peaks = Vec::with_capacity(buckets * 2);
    for b in 0..buckets {
        let start = start_frame + b * span / buckets;
        let end = (start_frame + (b + 1) * span / buckets).max(start + 1);
        let mut min_v = 0.0f32;
        let mut max_v = 0.0f32;
        for f in start..end {
            let mut frame_peak = 0.0f32;
            for c in 0..ch {
                let idx = f * ch + c;
                if idx < samples.len() {
                    let a = samples[idx].abs();
                    if a > frame_peak {
                        frame_peak = a;
                    }
                }
            }
            min_v = min_v.min(-frame_peak);
            max_v = max_v.max(frame_peak);
        }
        peaks.push(min_v);
        peaks.push(max_v);
    }
    Ok((duration, peaks))
}

#[tauri::command]
pub fn get_record_level() -> f32 {
    f32::from_bits(LEVEL.load(Ordering::Relaxed))
}

#[tauri::command]
pub fn is_recording() -> bool {
    active()
        .lock()
        .map(|g| g.is_some())
        .unwrap_or(false)
}
