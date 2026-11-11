//! Input / loopback recording via cpal.
//!
//! WASAPI loopback: call `build_input_stream` on an *output* device — cpal sets
//! `AUDCLNT_STREAMFLAGS_LOOPBACK` automatically.
//!
//! Capture callback never locks the sample buffer (live peak UI used to block it
//! after ~8s → pops/crackles). Chunks go through an mpsc channel; a collector
//! thread owns the `Vec<f32>` and keeps running waveform peaks next to it, so
//! the live waveform never folds the take or touches the sample buffer.

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
use super::peaks::PeakAcc;
use crate::task::run_blocking;

#[derive(Clone, Serialize)]
pub struct RecordStartInfo {
    pub sample_rate: u32,
    pub channels: u16,
}

/// Waveform state of the take in progress, shared between the collector
/// (writer) and the live-peaks command (reader).
struct LiveTake {
    peaks: Mutex<PeakAcc>,
    sample_rate: u32,
    channels: u16,
}

/// Frames per peak window at the start of a take.
const LIVE_PEAK_WINDOW_FRAMES: usize = 16;
/// Peak windows kept for a take (1 MiB). More than file waveforms get,
/// because the record view zooms into a time range of the take.
const LIVE_PEAK_MAX_WINDOWS: usize = 1 << 18;

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
/// Mirrors `ACTIVE.is_some()`. `ACTIVE` stays locked for the whole of a start
/// or stop (device open, thread joins, WAV write); status polls read this
/// flag instead of queueing behind that.
static RECORDING: AtomicBool = AtomicBool::new(false);
/// Peaks of the take in progress. Its own lock for the same reason.
static LIVE: Mutex<Option<Arc<LiveTake>>> = Mutex::new(None);
static LEVEL: AtomicU32 = AtomicU32::new(0);
/// Capture gain applied while recording (0.0–2.0, default 1.0). Stored as f32 bits.
static INPUT_VOLUME: AtomicU32 = AtomicU32::new(0x3F800000);
/// Hardware channel count of the open capture stream (before ASIO extract).
static INPUT_HW_CHANNELS: AtomicU32 = AtomicU32::new(0);
/// When true, pick ASIO L/R from the hardware buffer and store stereo.
static INPUT_EXTRACT: AtomicBool = AtomicBool::new(false);

fn active() -> &'static Mutex<Option<ActiveRecording>> {
    ACTIVE.get_or_init(|| Mutex::new(None))
}

fn set_live(take: Option<Arc<LiveTake>>) {
    *LIVE.lock().unwrap_or_else(|e| e.into_inner()) = take;
}

fn live() -> Option<Arc<LiveTake>> {
    LIVE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

fn current_input_volume() -> f32 {
    f32::from_bits(INPUT_VOLUME.load(Ordering::Relaxed))
}

fn write_level(peak: f32) {
    LEVEL.store(peak.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

/// Apply gain, update level meter, enqueue chunk. Never touches the sample `Mutex`.
fn enqueue_f32(tx: &SyncSender<Vec<f32>>, data: &[f32]) {
    if INPUT_EXTRACT.load(Ordering::Relaxed) {
        let hw = INPUT_HW_CHANNELS.load(Ordering::Relaxed) as u16;
        let stereo = super::channel_map::extract_stereo(data, hw);
        enqueue_gained(tx, &stereo);
    } else {
        enqueue_gained(tx, data);
    }
}

fn enqueue_gained(tx: &SyncSender<Vec<f32>>, data: &[f32]) {
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
    let mut converted = Vec::with_capacity(data.len());
    for &s in data {
        converted.push(s as f32 / i16::MAX as f32);
    }
    enqueue_f32(tx, &converted);
}

#[tauri::command(async)]
pub fn set_input_volume(volume: f32) {
    // Allow up to +6 dB (~2.0) for quiet mics.
    INPUT_VOLUME.store(volume.clamp(0.0, 2.0).to_bits(), Ordering::Relaxed);
}

#[tauri::command(async)]
pub fn get_input_volume() -> f32 {
    current_input_volume()
}

fn build_stream_f32(
    device: &cpal::Device,
    config: cpal::StreamConfig,
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
    config: cpal::StreamConfig,
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
    live: Arc<LiveTake>,
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
            if let Ok(mut peaks) = live.peaks.lock() {
                peaks.push(&chunk, live.channels);
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

/// Opening a capture device can take a noticeable moment (ASIO driver load,
/// PipeWire connect), so it runs on the blocking pool, not the main thread.
#[tauri::command]
pub async fn start_recording(
    app: AppHandle,
    device_name: String,
    host_name: Option<String>,
    loopback: bool,
) -> Result<RecordStartInfo, String> {
    run_blocking(move || start_recording_blocking(app, device_name, host_name, loopback)).await
}

fn start_recording_blocking(
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
    let config: cpal::StreamConfig = supported.into();
    let sample_rate: u32 = config.sample_rate;
    let hw_channels: u16 = config.channels;
    let extract = super::channel_map::input_enabled() && !loopback;
    INPUT_EXTRACT.store(extract, Ordering::Relaxed);
    INPUT_HW_CHANNELS.store(hw_channels as u32, Ordering::Relaxed);
    let channels: u16 = if extract { 2 } else { hw_channels };

    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));
    let stop_flag = Arc::new(AtomicBool::new(false));
    write_level(0.0);

    // ~2s of chunks before producer blocks — absorbs peak-UI lock spikes.
    let (chunk_tx, chunk_rx) = std::sync::mpsc::sync_channel::<Vec<f32>>(64);
    let pre_reserve = (sample_rate as usize)
        .saturating_mul(channels as usize)
        .saturating_mul(120); // ~2 min headroom
    let live_take = Arc::new(LiveTake {
        peaks: Mutex::new(PeakAcc::with_limits(
            LIVE_PEAK_WINDOW_FRAMES,
            LIVE_PEAK_MAX_WINDOWS,
        )),
        sample_rate,
        channels,
    });
    let collector_join =
        spawn_collector(chunk_rx, samples.clone(), Arc::clone(&live_take), pre_reserve);

    let stop_c = stop_flag.clone();
    let (ready_tx, ready_rx): (Sender<Result<(), String>>, _) = channel();

    let stream_join = thread::spawn(move || {
        let stream = match sample_format {
            cpal::SampleFormat::F32 => {
                build_stream_f32(&device, config, chunk_tx, stop_c.clone())
            }
            cpal::SampleFormat::I16 => {
                build_stream_i16(&device, config, chunk_tx, stop_c.clone())
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

    ready_rx.recv().map_err(|e| e.to_string())??;

    // Level meter ticker
    let app_level = app;
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
    set_live(Some(live_take));
    RECORDING.store(true, Ordering::Release);

    Ok(RecordStartInfo {
        sample_rate,
        channels,
    })
}

/// Joins the capture threads and writes the take to a WAV — seconds for a
/// long recording, so it runs on the blocking pool, not the main thread.
#[tauri::command]
pub async fn stop_recording(app: AppHandle) -> Result<String, String> {
    run_blocking(move || stop_recording_blocking(app)).await
}

fn stop_recording_blocking(app: AppHandle) -> Result<String, String> {
    let mut guard = active().lock().map_err(|e| e.to_string())?;
    let rec = guard.take().ok_or("Not recording")?;
    RECORDING.store(false, Ordering::Release);
    set_live(None);
    rec.stop_flag.store(true, Ordering::Relaxed);
    INPUT_EXTRACT.store(false, Ordering::Relaxed);
    // Stream drop closes chunk sender → collector drains + exits.
    if let Some(j) = rec._stream_join {
        let _ = j.join();
    }
    if let Some(j) = rec._collector_join {
        let _ = j.join();
    }
    write_level(0.0);
    let _ = app.emit("record_level", 0.0f32);

    // Both threads are joined, so nothing else uses the buffer: take it
    // instead of copying a take that can be hundreds of megabytes.
    let samples = std::mem::take(&mut *rec.samples.lock().map_err(|e| e.to_string())?);
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

/// Tear down capture without writing a file. Used on shutdown, where the
/// recorded audio is discarded anyway.
pub fn abort_recording() {
    let Ok(mut guard) = active().lock() else { return };
    let Some(rec) = guard.take() else { return };
    RECORDING.store(false, Ordering::Release);
    set_live(None);
    rec.stop_flag.store(true, Ordering::Relaxed);
    INPUT_EXTRACT.store(false, Ordering::Relaxed);
    if let Some(j) = rec._stream_join {
        let _ = j.join();
    }
    if let Some(j) = rec._collector_join {
        let _ = j.join();
    }
    write_level(0.0);
}

/// Live min/max peaks while recording: `[min0, max0, min1, max1, …]` plus duration.
/// Served from the collector's running peaks, so the cost does not grow with
/// the length of the take and the sample buffer is never locked.
#[tauri::command(async)]
pub fn get_live_record_peaks(
    buckets: usize,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<(f64, Vec<f32>), String> {
    let buckets = buckets.max(1);
    let Some(take) = live() else {
        return Ok((0.0, vec![0.0; buckets * 2]));
    };
    let peaks = take.peaks.lock().map_err(|e| e.to_string())?;
    Ok(live_peaks(&peaks, take.sample_rate, buckets, start_sec, end_sec))
}

/// `(duration, peaks)` of the optional time range of a take.
fn live_peaks(
    peaks: &PeakAcc,
    sample_rate: u32,
    buckets: usize,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> (f64, Vec<f32>) {
    let frames = peaks.frames();
    if frames == 0 || sample_rate == 0 {
        return (0.0, vec![0.0; buckets * 2]);
    }
    let rate = sample_rate as f64;
    let duration = frames as f64 / rate;
    let start_s = start_sec.unwrap_or(0.0).clamp(0.0, duration);
    let end_s = end_sec.unwrap_or(duration).clamp(start_s, duration);
    let start_frame = ((start_s * rate).round() as usize).min(frames);
    let end_frame = ((end_s * rate).round() as usize).min(frames);
    (duration, peaks.range_to_buckets(buckets, start_frame, end_frame))
}

#[tauri::command]
pub fn get_record_level() -> f32 {
    f32::from_bits(LEVEL.load(Ordering::Relaxed))
}

#[tauri::command]
pub fn is_recording() -> bool {
    RECORDING.load(Ordering::Acquire)
}

#[cfg(test)]
mod tests {
    use super::{live_peaks, PeakAcc, LIVE_PEAK_MAX_WINDOWS, LIVE_PEAK_WINDOW_FRAMES};

    fn take(samples: &[f32], channels: u16) -> PeakAcc {
        let mut peaks = PeakAcc::with_limits(LIVE_PEAK_WINDOW_FRAMES, LIVE_PEAK_MAX_WINDOWS);
        // Capture hands the collector small chunks; feed it the same way.
        for chunk in samples.chunks(480 * channels as usize) {
            peaks.push(chunk, channels);
        }
        peaks
    }

    #[test]
    fn live_peaks_reports_duration_and_covers_the_whole_take() {
        // One second of mono at 8 kHz: loud first half, quiet second half.
        let samples: Vec<f32> = (0..8_000).map(|i| if i < 4_000 { 0.8 } else { 0.1 }).collect();
        let (duration, peaks) = live_peaks(&take(&samples, 1), 8_000, 4, None, None);
        assert!((duration - 1.0).abs() < 1e-9);
        assert_eq!(peaks.len(), 8);
        assert!(peaks[1] > 0.7 && peaks[3] > 0.7);
        assert!(peaks[5] < 0.2 && peaks[7] < 0.2);
    }

    #[test]
    fn live_peaks_zooms_into_a_time_range() {
        let samples: Vec<f32> = (0..8_000).map(|i| if i < 4_000 { 0.8 } else { 0.1 }).collect();
        let (duration, peaks) = live_peaks(&take(&samples, 1), 8_000, 4, Some(0.5), Some(1.0));
        assert!((duration - 1.0).abs() < 1e-9);
        assert!(peaks.chunks_exact(2).all(|p| p[1] < 0.2 && p[1] > 0.0), "{peaks:?}");
    }

    #[test]
    fn live_peaks_is_silent_for_an_empty_take_or_an_empty_range() {
        let empty = PeakAcc::with_limits(LIVE_PEAK_WINDOW_FRAMES, LIVE_PEAK_MAX_WINDOWS);
        assert_eq!(live_peaks(&empty, 8_000, 3, None, None), (0.0, vec![0.0; 6]));

        let samples = vec![0.5f32; 800];
        let (duration, peaks) = live_peaks(&take(&samples, 1), 8_000, 3, Some(0.05), Some(0.05));
        assert!((duration - 0.1).abs() < 1e-9);
        assert!(peaks.iter().all(|&p| p == 0.0));
    }

    #[test]
    fn live_peaks_counts_stereo_frames_not_samples() {
        let samples = vec![0.5f32; 2 * 8_000];
        let (duration, _) = live_peaks(&take(&samples, 2), 8_000, 4, None, None);
        assert!((duration - 1.0).abs() < 1e-9);
    }
}
