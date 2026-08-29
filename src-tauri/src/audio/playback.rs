use rodio::{self, Player, MixerDeviceSink, DeviceSinkBuilder, Source};
use serde::Serialize;
use std::io::Cursor;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

use super::devices::{device_display_name, get_output_devices_for_host_name};

// --- Global output volume (stored as f32 bits in an atomic) ---

static OUTPUT_VOLUME: AtomicU32 = AtomicU32::new(0x3F800000); // 1.0f32 bits

fn current_volume() -> f32 {
    f32::from_bits(OUTPUT_VOLUME.load(Ordering::Relaxed))
}

#[tauri::command]
pub fn set_output_volume(volume: f32) {
    OUTPUT_VOLUME.store(volume.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

// --- Duration query ---

#[tauri::command]
pub fn get_sound_duration(sound_path: String) -> Result<f64, String> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;

    // Load from cache (or disk on first access — result is cached for next time).
    let bytes = super::cache::load_bytes(&sound_path)?;

    let mss = MediaSourceStream::new(
        Box::new(Cursor::new(bytes.clone())),
        Default::default(),
    );

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(&sound_path)
        .extension()
        .and_then(|e| e.to_str())
    {
        hint.with_extension(ext);
    }

    let probe_result = symphonia::default::get_probe().probe(
        &hint,
        mss,
        FormatOptions::default(),
        MetadataOptions::default(),
    );
    if let Ok(format) = probe_result {
        for track in format.tracks() {
            if let (Some(n_frames), Some(tb)) = (track.num_frames, track.time_base) {
                let secs = n_frames as f64 * tb.numer.get() as f64 / tb.denom.get() as f64;
                if secs > 0.0 {
                    return Ok(secs);
                }
            }
        }
    }

    // Fallback: let rodio decode from the in-memory bytes.
    let decoder = rodio::Decoder::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    Ok(decoder.total_duration().map(|d| d.as_secs_f64()).unwrap_or(0.0))
}

// --- Playing list snapshot (shared with frontend) ---

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayingInfo {
    pub path: String,
    pub paused: bool,
    pub position_secs: f64,
    pub looping: bool,
}

static PLAYING_LIST: OnceLock<Mutex<Vec<PlayingInfo>>> = OnceLock::new();

fn playing_list() -> &'static Mutex<Vec<PlayingInfo>> {
    PLAYING_LIST.get_or_init(|| Mutex::new(Vec::new()))
}

fn sync_playing_list(app: &tauri::AppHandle, playing: &[PlayingSound]) {
    let snapshot: Vec<PlayingInfo> = playing
        .iter()
        .map(|s| PlayingInfo {
            path: s.path.clone(),
            paused: s.player.is_paused(),
            // get_pos() is relative to the current Source. After skip_duration
            // seek rebuilds, that starts at 0 — add origin for absolute time.
            position_secs: s.position_origin_secs + s.player.get_pos().as_secs_f64(),
            looping: s.looping,
        })
        .collect();
    if let Ok(mut guard) = playing_list().lock() {
        *guard = snapshot.clone();
    }
    let _ = app.emit("playing_changed", snapshot);
}

// --- Audio thread message ---

pub enum AudioMsg {
    /// Play a sound. `overlap` = keep existing sounds running.
    Play {
        path: String,
        device_name: String,
        /// Optional audio host name (e.g. "WASAPI", "ASIO"). None = search all hosts.
        host_name: Option<String>,
        overlap: bool,
    },
    /// Stop a single playing sound matched by its file path.
    StopOne {
        path: String,
    },
    /// Stop every playing sound.
    Stop,
    PauseOne {
        path: String,
    },
    ResumeOne {
        path: String,
    },
    PauseAll,
    ResumeAll,
    /// Seek the first matching (or only) playing sound.
    Seek {
        path: Option<String>,
        position_secs: f64,
    },
    /// Enable/disable seamless restart when the current sound ends.
    SetLoop {
        path: Option<String>,
        looping: bool,
    },
}

// --- Global audio sender ---

static AUDIO_SENDER: OnceLock<Mutex<Option<Sender<AudioMsg>>>> = OnceLock::new();

fn audio_sender() -> &'static Mutex<Option<Sender<AudioMsg>>> {
    AUDIO_SENDER.get_or_init(|| Mutex::new(None))
}

fn send_msg(msg: AudioMsg) -> Result<(), String> {
    let guard = audio_sender().lock().map_err(|e| e.to_string())?;
    let tx = (*guard).clone().ok_or("Audio thread not initialized")?;
    tx.send(msg).map_err(|e| e.to_string())
}

// --- Audio stream wrapper ---

/// Persistent MixerDeviceSink kept alive across sounds on the same device.
/// Multiple Player instances connected to the same mixer play simultaneously.
struct AudioStream {
    device_sink: MixerDeviceSink,
    device_name: String,
    host_name: Option<String>,
}

impl AudioStream {
    fn open(device_name: &str, host_name: Option<&str>) -> Result<Self, String> {
        let device = get_output_devices_for_host_name(host_name)
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|d| device_display_name(d).as_deref() == Some(device_name))
            .ok_or_else(|| format!("Device '{}' not found", device_name))?;
        let device_sink = DeviceSinkBuilder::from_device(device)
            .map_err(|e| e.to_string())?
            .open_stream()
            .map_err(|e| e.to_string())?;
        Ok(Self {
            device_sink,
            device_name: device_name.to_owned(),
            host_name: host_name.map(str::to_owned),
        })
    }
}

// --- Playing sound slot ---

struct PlayingSound {
    player: Player,
    /// Original file path — sent back in the `sound_finished` event so the
    /// frontend can deactivate exactly the sound that finished.
    path: String,
    looping: bool,
    /// Known duration used to clamp seeks away from EOF (avoids empty source).
    duration_secs: f64,
    /// Absolute file offset corresponding to player position 0 (set on seek).
    position_origin_secs: f64,
}

// --- Helpers ---

fn load_source(path: &str) -> Result<rodio::Decoder<Cursor<Arc<[u8]>>>, String> {
    let bytes = super::cache::load_bytes(path)?;
    rodio::Decoder::new(Cursor::new(bytes)).map_err(|e| e.to_string())
}

/// Ensures the stream targets `device_name`/`host_name`, reopening only when they changed.
fn ensure_stream(
    stream: &mut Option<AudioStream>,
    device_name: &str,
    host_name: Option<&str>,
) -> Result<(), String> {
    let needs_new = stream
        .as_ref()
        .map(|s| s.device_name != device_name || s.host_name.as_deref() != host_name)
        .unwrap_or(true);

    if !needs_new {
        return Ok(());
    }

    if stream.is_some() {
        // Allow WASAPI callbacks to drain before opening a new stream.
        thread::sleep(Duration::from_millis(100));
    }
    *stream = Some(AudioStream::open(device_name, host_name)?);
    Ok(())
}

/// Non-blocking recv when sounds are playing; blocking recv when idle.
fn recv_msg(rx: &Receiver<AudioMsg>, is_playing: bool) -> Result<Option<AudioMsg>, ()> {
    if is_playing {
        match rx.recv_timeout(Duration::from_millis(50)) {
            Ok(msg) => Ok(Some(msg)),
            Err(RecvTimeoutError::Timeout) => Ok(None),
            Err(RecvTimeoutError::Disconnected) => Err(()),
        }
    } else {
        rx.recv().map(Some).map_err(|_| ())
    }
}

/// Drop obsolete Seek messages so Stop/Play are not stuck behind a scrub flood.
/// Keeps message order; for each seek path key, only the latest Seek remains.
fn coalesce_seeks(batch: &mut Vec<AudioMsg>) {
    if batch.len() < 2 {
        return;
    }
    let mut drop_idx = vec![false; batch.len()];
    for i in 0..batch.len() {
        let key_i = match &batch[i] {
            AudioMsg::Seek { path, .. } => Some(path.clone()),
            _ => None,
        };
        let Some(key_i) = key_i else { continue };
        for j in (i + 1)..batch.len() {
            if let AudioMsg::Seek { path, .. } = &batch[j] {
                if *path == key_i {
                    drop_idx[i] = true;
                    break;
                }
            }
        }
    }
    let mut kept = Vec::with_capacity(batch.len());
    for (i, msg) in batch.drain(..).enumerate() {
        if !drop_idx[i] {
            kept.push(msg);
        }
    }
    *batch = kept;
}

/// Clamp seek away from EOF so the source is not exhausted.
fn clamp_seek_pos(position_secs: f64, duration_secs: f64) -> f64 {
    let max_pos = if duration_secs > 0.08 {
        duration_secs - 0.05
    } else {
        0.0
    };
    position_secs.clamp(0.0, max_pos.max(0.0))
}

/// Seek playing slot: prefer in-place demuxer seek; rebuild only on failure.
fn seek_playing_slot(
    playing: &mut [PlayingSound],
    i: usize,
    stream: &AudioStream,
    position_secs: f64,
) {
    let was_paused = playing[i].player.is_paused();
    let mut duration_secs = playing[i].duration_secs;
    let clamped = clamp_seek_pos(position_secs, duration_secs);
    let target = Duration::from_secs_f64(clamped);

    // 1) In-place seek while playing (typically 0–5 ms when supported).
    //    Skip when paused: rodio applies seek via periodic_access which needs
    //    sample pulls, and briefly unpausing would click. Rebuild instead.
    if !was_paused {
        let in_place_ok = playing[i].player.try_seek(target).is_ok()
            && (playing[i].player.get_pos().as_secs_f64() - clamped).abs() <= 0.35;
        if in_place_ok {
            // Player::try_seek sets absolute get_pos — no origin offset needed.
            playing[i].position_origin_secs = 0.0;
            return;
        }
    }

    // 2) Rebuild decoder and demuxer-seek before append (no eager skip_duration).
    //    New Player get_pos starts at 0, so origin must equal the seek offset.
    let file_path = playing[i].path.clone();
    let looping = playing[i].looping;
    let Ok(mut source) = load_source(&file_path) else {
        eprintln!("Seek reload failed for '{file_path}'");
        return;
    };
    if duration_secs <= 0.0 {
        duration_secs = source
            .total_duration()
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
    }
    let clamped = clamp_seek_pos(position_secs, duration_secs);
    let target = Duration::from_secs_f64(clamped);

    let new_player = Player::connect_new(stream.device_sink.mixer());
    let origin = match source.try_seek(target) {
        Ok(()) => {
            new_player.append(source.amplify(current_volume()));
            // Decoder already advanced; player clock starts at 0.
            clamped
        }
        Err(_) if clamped < 2.0 => {
            // try_seek may leave the decoder dirty — reload before short skip.
            let Ok(fresh) = load_source(&file_path) else {
                eprintln!("Seek reload failed for '{file_path}'");
                return;
            };
            new_player.append(
                fresh
                    .skip_duration(Duration::from_secs_f64(clamped))
                    .amplify(current_volume()),
            );
            clamped
        }
        Err(e) => {
            eprintln!("Seek unsupported at {clamped:.2}s for '{file_path}': {e}");
            return;
        }
    };

    // Pause before swap so the new source never audibly starts if we were paused.
    if was_paused {
        new_player.pause();
    }
    let old = std::mem::replace(
        &mut playing[i],
        PlayingSound {
            player: new_player,
            path: file_path,
            looping,
            duration_secs,
            position_origin_secs: origin,
        },
    );
    old.player.stop();
}

// --- Audio thread ---

pub fn init_audio_thread(app_handle: tauri::AppHandle) {
    let (tx, rx) = channel::<AudioMsg>();
    *audio_sender().lock().expect("mutex poisoned") = Some(tx);

    thread::spawn(move || {
        let mut stream: Option<AudioStream> = None;
        let mut playing: Vec<PlayingSound> = Vec::new();

        loop {
            // Drain naturally-finished sounds and notify frontend.
            let mut finished_any = false;
            let mut i = 0;
            while i < playing.len() {
                if playing[i].player.empty() {
                    if playing[i].looping {
                        let path = playing[i].path.clone();
                        let was_paused = playing[i].player.is_paused();
                        match load_source(&path) {
                            Ok(source) => {
                                // Sink::get_pos is cumulative across appended
                                // sources. Offset origin so absolute position
                                // restarts at 0 and the UI playhead/progress
                                // can loop with the audio.
                                let sink_pos = playing[i].player.get_pos().as_secs_f64();
                                playing[i].position_origin_secs = -sink_pos;
                                playing[i].player.append(source.amplify(current_volume()));
                                if was_paused {
                                    playing[i].player.pause();
                                }
                                finished_any = true; // refresh position → 0
                                i += 1;
                            }
                            Err(_) => {
                                let finished = playing.remove(i);
                                let _ = app_handle.emit("sound_finished", finished.path);
                                finished_any = true;
                            }
                        }
                    } else {
                        let finished = playing.remove(i);
                        let _ = app_handle.emit("sound_finished", finished.path);
                        finished_any = true;
                    }
                } else {
                    i += 1;
                }
            }
            if finished_any {
                sync_playing_list(&app_handle, &playing);
            }

            // Use timeout-based recv while sounds are active so we keep draining.
            let is_playing = !playing.is_empty();

            let first = match recv_msg(&rx, is_playing) {
                Ok(Some(msg)) => msg,
                Ok(None) => continue, // timeout — loop back to drain check
                Err(()) => break,      // channel closed
            };
            let mut batch = vec![first];
            while let Ok(m) = rx.try_recv() {
                batch.push(m);
            }
            coalesce_seeks(&mut batch);

            for msg in batch {
            match msg {
                AudioMsg::Stop => {
                    for s in playing.drain(..) {
                        s.player.stop();
                        let _ = app_handle.emit("sound_finished", s.path);
                    }
                    sync_playing_list(&app_handle, &playing);
                }

                AudioMsg::StopOne { path } => {
                    if let Some(pos) = playing.iter().position(|s| s.path == path) {
                        let s = playing.remove(pos);
                        s.player.stop();
                        let _ = app_handle.emit("sound_finished", s.path);
                        sync_playing_list(&app_handle, &playing);
                    }
                }

                AudioMsg::PauseOne { path } => {
                    if let Some(s) = playing.iter().find(|s| s.path == path) {
                        s.player.pause();
                        sync_playing_list(&app_handle, &playing);
                    }
                }

                AudioMsg::ResumeOne { path } => {
                    if let Some(s) = playing.iter().find(|s| s.path == path) {
                        s.player.play();
                        sync_playing_list(&app_handle, &playing);
                    }
                }

                AudioMsg::PauseAll => {
                    for s in &playing {
                        s.player.pause();
                    }
                    sync_playing_list(&app_handle, &playing);
                }

                AudioMsg::ResumeAll => {
                    for s in &playing {
                        s.player.play();
                    }
                    sync_playing_list(&app_handle, &playing);
                }

                AudioMsg::Play { path, device_name, host_name, overlap } => {
                    if !overlap {
                        // Stop all currently playing sounds before starting the new one.
                        for s in playing.drain(..) {
                            s.player.stop();
                            let _ = app_handle.emit("sound_finished", s.path);
                        }
                        // Allow WASAPI callbacks to drain before reusing the device.
                        thread::sleep(Duration::from_millis(50));
                    }

                    if let Err(e) = ensure_stream(&mut stream, &device_name, host_name.as_deref()) {
                        eprintln!("Failed to open audio device: {}", e);
                        let _ = app_handle.emit("sound_error", e);
                        continue;
                    }

                    let st = stream.as_ref().expect("ensure_stream guarantees Some on Ok");
                    // Each Player connects to the shared MixerDeviceSink — sounds mix.
                    let new_player = Player::connect_new(st.device_sink.mixer());

                    match load_source(&path) {
                        Ok(source) => {
                            let duration_secs = source
                                .total_duration()
                                .map(|d| d.as_secs_f64())
                                .unwrap_or(0.0);
                            new_player.append(source.amplify(current_volume()));
                            playing.push(PlayingSound {
                                player: new_player,
                                path,
                                looping: false,
                                duration_secs,
                                position_origin_secs: 0.0,
                            });
                            sync_playing_list(&app_handle, &playing);
                        }
                        Err(e) => {
                            eprintln!("Failed to decode audio: {}", e);
                            let _ = app_handle.emit("sound_error", e);
                        }
                    }
                }

                AudioMsg::Seek {
                    path,
                    position_secs,
                } => {
                    let idx = match path.as_deref() {
                        Some(p) => playing.iter().position(|s| s.path == p),
                        None => (!playing.is_empty()).then_some(0),
                    };
                    if let (Some(i), Some(st)) = (idx, stream.as_ref()) {
                        seek_playing_slot(&mut playing, i, st, position_secs);
                        sync_playing_list(&app_handle, &playing);
                    }
                }

                AudioMsg::SetLoop { path, looping } => {
                    let targets: Vec<usize> = match path.as_deref() {
                        Some(p) => playing
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| s.path == p)
                            .map(|(i, _)| i)
                            .collect(),
                        None => (0..playing.len()).collect(),
                    };
                    for i in targets {
                        playing[i].looping = looping;
                    }
                    sync_playing_list(&app_handle, &playing);
                }
            }
            } // for msg in batch
        }
    });
}

// --- Tauri commands ---

#[tauri::command]
pub fn play_sound(
    sound_path: String,
    device_name: String,
    host_name: Option<String>,
    active: bool,
    overlap: bool,
) -> Result<String, String> {
    if active {
        send_msg(AudioMsg::StopOne { path: sound_path })?;
        Ok("stopped".to_string())
    } else {
        send_msg(AudioMsg::Play {
            path: sound_path,
            device_name,
            host_name,
            overlap,
        })?;
        Ok("playing".to_string())
    }
}

#[tauri::command]
pub fn pause_sound(sound_path: String) -> Result<(), String> {
    send_msg(AudioMsg::PauseOne { path: sound_path })
}

#[tauri::command]
pub fn resume_sound(sound_path: String) -> Result<(), String> {
    send_msg(AudioMsg::ResumeOne { path: sound_path })
}

#[tauri::command]
pub fn pause_all() -> Result<(), String> {
    send_msg(AudioMsg::PauseAll)
}

#[tauri::command]
pub fn resume_all() -> Result<(), String> {
    send_msg(AudioMsg::ResumeAll)
}

#[tauri::command]
pub fn stop_all() -> Result<(), String> {
    send_msg(AudioMsg::Stop)
}

#[tauri::command]
pub fn get_playing_sounds() -> Vec<PlayingInfo> {
    playing_list()
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
}

#[tauri::command]
pub fn seek_playing(position_secs: f64, sound_path: Option<String>) -> Result<(), String> {
    send_msg(AudioMsg::Seek {
        path: sound_path,
        position_secs,
    })
}

#[tauri::command]
pub fn set_playing_loop(looping: bool, sound_path: Option<String>) -> Result<(), String> {
    send_msg(AudioMsg::SetLoop {
        path: sound_path,
        looping,
    })
}
