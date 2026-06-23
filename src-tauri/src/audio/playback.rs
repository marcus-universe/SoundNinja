use cpal::traits::DeviceTrait;
use rodio::{self, cpal, Player, MixerDeviceSink, DeviceSinkBuilder, Source};
use std::io::Cursor;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

use super::devices::get_output_devices;

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

// --- Audio thread message ---

pub enum AudioMsg {
    /// Play a sound. `overlap` = keep existing sounds running.
    Play {
        path: String,
        device_name: String,
        overlap: bool,
    },
    /// Stop a single playing sound matched by its file path.
    StopOne {
        path: String,
    },
    /// Stop every playing sound.
    Stop,
}

// --- Global audio sender ---

static AUDIO_SENDER: OnceLock<Mutex<Option<Sender<AudioMsg>>>> = OnceLock::new();

fn audio_sender() -> &'static Mutex<Option<Sender<AudioMsg>>> {
    AUDIO_SENDER.get_or_init(|| Mutex::new(None))
}

// --- Audio stream wrapper ---

/// Persistent MixerDeviceSink kept alive across sounds on the same device.
/// Multiple Player instances connected to the same mixer play simultaneously.
struct AudioStream {
    device_sink: MixerDeviceSink,
    device_name: String,
}

impl AudioStream {
    fn open(device_name: &str) -> Result<Self, String> {
        let device = get_output_devices()
            .map_err(|e| e.to_string())?
            .into_iter()
            .find(|d| {
                d.description()
                    .map(|desc| {
                        let full_name = desc
                            .extended()
                            .first()
                            .cloned()
                            .unwrap_or_else(|| desc.name().to_string());
                        full_name == device_name
                    })
                    .unwrap_or(false)
            })
            .ok_or_else(|| format!("Device '{}' not found", device_name))?;
        let device_sink = DeviceSinkBuilder::from_device(device)
            .map_err(|e| e.to_string())?
            .open_stream()
            .map_err(|e| e.to_string())?;
        Ok(Self { device_sink, device_name: device_name.to_owned() })
    }
}

// --- Playing sound slot ---

struct PlayingSound {
    player: Player,
    /// Original file path — sent back in the `sound_finished` event so the
    /// frontend can deactivate exactly the sound that finished.
    path: String,
}

// --- Helpers ---

fn load_source(path: &str) -> Result<rodio::Decoder<Cursor<Arc<[u8]>>>, String> {
    let bytes = super::cache::load_bytes(path)?;
    rodio::Decoder::new(Cursor::new(bytes)).map_err(|e| e.to_string())
}

/// Ensures the stream targets `device_name`, reopening only when the device changed.
fn ensure_stream(stream: &mut Option<AudioStream>, device_name: &str) -> Result<(), String> {
    let needs_new = stream
        .as_ref()
        .map(|s| s.device_name != device_name)
        .unwrap_or(true);

    if !needs_new {
        return Ok(());
    }

    if stream.is_some() {
        // Allow WASAPI callbacks to drain before opening a new stream.
        thread::sleep(Duration::from_millis(100));
    }
    *stream = Some(AudioStream::open(device_name)?);
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

// --- Audio thread ---

pub fn init_audio_thread(app_handle: tauri::AppHandle) {
    let (tx, rx) = channel::<AudioMsg>();
    *audio_sender().lock().expect("mutex poisoned") = Some(tx);

    thread::spawn(move || {
        let mut stream: Option<AudioStream> = None;
        let mut playing: Vec<PlayingSound> = Vec::new();

        loop {
            // Drain naturally-finished sounds and notify frontend.
            let mut i = 0;
            while i < playing.len() {
                if playing[i].player.empty() {
                    let finished = playing.remove(i);
                    let _ = app_handle.emit("sound_finished", finished.path);
                } else {
                    i += 1;
                }
            }

            // Use timeout-based recv while sounds are active so we keep draining.
            let is_playing = !playing.is_empty();

            let msg = match recv_msg(&rx, is_playing) {
                Ok(Some(msg)) => msg,
                Ok(None) => continue, // timeout — loop back to drain check
                Err(()) => break,      // channel closed
            };

            match msg {
                AudioMsg::Stop => {
                    for s in playing.drain(..) {
                        s.player.stop();
                        let _ = app_handle.emit("sound_finished", s.path);
                    }
                }

                AudioMsg::StopOne { path } => {
                    if let Some(pos) = playing.iter().position(|s| s.path == path) {
                        let s = playing.remove(pos);
                        s.player.stop();
                        let _ = app_handle.emit("sound_finished", s.path);
                    }
                }

                AudioMsg::Play { path, device_name, overlap } => {
                    if !overlap {
                        // Stop all currently playing sounds before starting the new one.
                        for s in playing.drain(..) {
                            s.player.stop();
                            let _ = app_handle.emit("sound_finished", s.path);
                        }
                        // Allow WASAPI callbacks to drain before reusing the device.
                        thread::sleep(Duration::from_millis(50));
                    }

                    if let Err(e) = ensure_stream(&mut stream, &device_name) {
                        eprintln!("Failed to open audio device: {}", e);
                        let _ = app_handle.emit("sound_error", e);
                        continue;
                    }

                    let st = stream.as_ref().expect("ensure_stream guarantees Some on Ok");
                    // Each Player connects to the shared MixerDeviceSink — sounds mix.
                    let new_player = Player::connect_new(st.device_sink.mixer());

                    match load_source(&path) {
                        Ok(source) => {
                            new_player.append(source);
                            playing.push(PlayingSound { player: new_player, path });
                        }
                        Err(e) => {
                            eprintln!("Failed to decode audio: {}", e);
                            let _ = app_handle.emit("sound_error", e);
                        }
                    }
                }
            }
        }
    });
}

// --- Tauri command ---

#[tauri::command]
pub fn play_sound(
    sound_path: String,
    device_name: String,
    active: bool,
    overlap: bool,
) -> Result<String, String> {
    let guard = audio_sender().lock().map_err(|e| e.to_string())?;
    let tx = (*guard).clone().ok_or("Audio thread not initialized")?;

    if active {
        tx.send(AudioMsg::StopOne { path: sound_path }).map_err(|e| e.to_string())?;
        Ok("stopped".to_string())
    } else {
        tx.send(AudioMsg::Play { path: sound_path, device_name, overlap })
            .map_err(|e| e.to_string())?;
        Ok("playing".to_string())
    }
}
