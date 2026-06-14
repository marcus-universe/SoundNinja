use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use lazy_static::lazy_static;
use rodio::{self, cpal, Source};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::Manager;

// --- Device enumeration ---

pub fn get_output_devices() -> Result<Vec<cpal::Device>> {
    let mut devices = Vec::new();
    for host_id in cpal::available_hosts() {
        let host = cpal::host_from_id(host_id)?;
        devices.extend(host.devices()?.filter(|d| d.default_output_config().is_ok()));
    }
    Ok(devices)
}

#[tauri::command]
pub fn get_out_devices() -> Vec<String> {
    get_output_devices()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|d| d.name().ok())
        .inspect(|n| println!("Device: {}", n))
        .collect()
}

// --- Duration query ---

#[tauri::command]
pub fn get_sound_duration(sound_path: String) -> Result<f64, String> {
    let file = File::open(&sound_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let decoder = rodio::Decoder::new(reader).map_err(|e| e.to_string())?;
    let secs = decoder
        .total_duration()
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    Ok(secs)
}

// --- Audio thread types ---

enum AudioMsg {
    Play { path: String, device_name: String },
    Stop,
}

lazy_static! {
    static ref AUDIO_SENDER: Mutex<Option<Sender<AudioMsg>>> = Mutex::new(None);
}

/// Persistent OutputStream kept alive across multiple sounds on the same device.
/// We NEVER drop OutputStream while audio is playing - this prevents the
/// "slice::get_unchecked_mut" WASAPI crash. Only the cheap Sink is swapped per sound.
struct AudioStream {
    _stream: rodio::OutputStream,
    handle: rodio::OutputStreamHandle,
    device_name: String,
}

impl AudioStream {
    fn open(device_name: &str) -> Result<Self, String> {
        let devices = get_output_devices().map_err(|e| e.to_string())?;
        let device = devices
            .iter()
            .find(|d| d.name().map(|n| n == device_name).unwrap_or(false))
            .ok_or_else(|| format!("Device '{}' not found", device_name))?;
        let (stream, handle) =
            rodio::OutputStream::try_from_device(device).map_err(|e| e.to_string())?;
        Ok(AudioStream {
            _stream: stream,
            handle,
            device_name: device_name.to_owned(),
        })
    }
}

// --- Audio thread ---

pub fn init_audio_thread(app_handle: tauri::AppHandle) {
    let (tx, rx) = channel::<AudioMsg>();
    *AUDIO_SENDER.lock().expect("mutex poisoned") = Some(tx);

    thread::spawn(move || {
        let mut stream: Option<AudioStream> = None;
        let mut sink: Option<rodio::Sink> = None;

        loop {
            let is_playing = sink.as_ref().map(|s| !s.empty()).unwrap_or(false);

            let msg: Option<AudioMsg> = if is_playing {
                match rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(m) => Some(m),
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if sink.as_ref().map(|s| s.empty()).unwrap_or(false) {
                            sink = None;
                            let _ = app_handle.emit_all("sound_finished", ());
                        }
                        None
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            } else {
                if sink.is_some() {
                    sink = None;
                    let _ = app_handle.emit_all("sound_finished", ());
                }
                match rx.recv() {
                    Ok(m) => Some(m),
                    Err(_) => break,
                }
            };

            if let Some(msg) = msg {
                match msg {
                    AudioMsg::Stop => {
                        if let Some(ref s) = sink {
                            s.stop();
                        }
                        sink = None;
                        let _ = app_handle.emit_all("sound_finished", ());
                    }
                    AudioMsg::Play { path, device_name } => {
                        if let Some(ref s) = sink {
                            s.stop();
                        }
                        sink = None;

                        let need_new_stream = stream
                            .as_ref()
                            .map(|st| st.device_name != device_name)
                            .unwrap_or(true);

                        if need_new_stream {
                            if stream.is_some() {
                                thread::sleep(Duration::from_millis(100));
                            }
                            stream = None;
                            match AudioStream::open(&device_name) {
                                Ok(s) => stream = Some(s),
                                Err(e) => {
                                    eprintln!("Failed to open audio device: {}", e);
                                    let _ = app_handle.emit_all("sound_error", e);
                                    continue;
                                }
                            }
                        }

                        if let Some(ref st) = stream {
                            match rodio::Sink::try_new(&st.handle) {
                                Ok(s) => {
                                    match File::open(&path)
                                        .map_err(|e| e.to_string())
                                        .and_then(|f| {
                                            rodio::Decoder::new(BufReader::new(f))
                                                .map_err(|e| e.to_string())
                                        }) {
                                        Ok(source) => {
                                            s.append(source);
                                            sink = Some(s);
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to decode audio: {}", e);
                                            let _ = app_handle.emit_all("sound_error", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to create sink: {}", e);
                                    let _ = app_handle.emit_all("sound_error", e.to_string());
                                }
                            }
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
) -> Result<String, String> {
    let guard = AUDIO_SENDER.lock().map_err(|e| e.to_string())?;
    let tx = guard.as_ref().ok_or("Audio thread not initialized")?;

    if active {
        tx.send(AudioMsg::Stop).map_err(|e| e.to_string())?;
        Ok("stopped".to_string())
    } else {
        tx.send(AudioMsg::Play { path: sound_path, device_name })
            .map_err(|e| e.to_string())?;
        Ok("playing".to_string())
    }
}