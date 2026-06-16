use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::{self, cpal, Source, Player, MixerDeviceSink, DeviceSinkBuilder};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

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
        .filter_map(|d| {
            d.description().ok().map(|desc| {
                // On Windows, DEVPKEY_Device_FriendlyName (the full name like
                // "Lautsprecher (3- Focusrite USB Audio)") is stored in extended()[0].
                // DEVPKEY_Device_DeviceDesc ("Lautsprecher") is stored in name().
                desc.extended()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| desc.name().to_string())
            })
        })
        .inspect(|n| println!("Device: {}", n))
        .collect()
}

// --- Duration query ---

#[tauri::command]
pub fn get_sound_duration(sound_path: String) -> Result<f64, String> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    // Try the symphonia probe first – it reads XING/VBR headers and container
    // metadata for accurate duration across MP3, WAV, OGG, and FLAC.
    let file = File::open(&sound_path).map_err(|e| e.to_string())?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = std::path::Path::new(&sound_path)
        .extension()
        .and_then(|e| e.to_str())
    {
        hint.with_extension(ext);
    }

    let probe_result = symphonia::default::get_probe().format(
        &hint,
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    );
    if let Ok(probed) = probe_result {
        for track in probed.format.tracks() {
            let params = &track.codec_params;
            if let (Some(n_frames), Some(tb)) = (params.n_frames, params.time_base) {
                let secs = n_frames as f64 * tb.numer as f64 / tb.denom as f64;
                if secs > 0.0 {
                    return Ok(secs);
                }
            }
        }
    }

    // Fall back to rodio's Decoder (works reliably for WAV / CBR formats
    // where total_duration() returns Some).
    let file = File::open(&sound_path).map_err(|e| e.to_string())?;
    let decoder = rodio::Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())?;
    Ok(decoder.total_duration().map(|d| d.as_secs_f64()).unwrap_or(0.0))
}

// --- Audio thread message ---

enum AudioMsg {
    Play { path: String, device_name: String },
    Stop,
}

// --- Global audio sender ---

static AUDIO_SENDER: OnceLock<Mutex<Option<Sender<AudioMsg>>>> = OnceLock::new();

fn audio_sender() -> &'static Mutex<Option<Sender<AudioMsg>>> {
    AUDIO_SENDER.get_or_init(|| Mutex::new(None))
}

/// Persistent MixerDeviceSink kept alive across multiple sounds on the same device.
/// We NEVER drop MixerDeviceSink while audio is playing - this prevents the
/// "slice::get_unchecked_mut" WASAPI crash. Only the cheap Player is swapped per sound.
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

// --- Audio helpers ---

fn load_source(path: &str) -> Result<rodio::Decoder<BufReader<File>>, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    rodio::Decoder::new(BufReader::new(file)).map_err(|e| e.to_string())
}

/// Ensures the stream is open and targets `device_name`, reopening if necessary.
fn ensure_stream(stream: &mut Option<AudioStream>, device_name: &str) -> Result<(), String> {
    let needs_new = stream
        .as_ref()
        .map(|s| s.device_name != device_name)
        .unwrap_or(true);

    if !needs_new {
        return Ok(());
    }

    if stream.is_some() {
        thread::sleep(Duration::from_millis(100));
    }
    *stream = Some(AudioStream::open(device_name)?);
    Ok(())
}

/// Receive the next message. Returns `Ok(None)` on timeout, `Err(())` on disconnect.
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
        let mut player: Option<Player> = None;

        loop {
            let is_playing = player.as_ref().map(|p| !p.empty()).unwrap_or(false);

            // When idle, drain any lingering player before blocking on recv.
            if !is_playing && player.is_some() {
                player = None;
                let _ = app_handle.emit("sound_finished", ());
            }

            let msg = match recv_msg(&rx, is_playing) {
                Ok(Some(msg)) => msg,
                Ok(None) => {
                    // Timeout while playing: check if player finished naturally.
                    if player.as_ref().map(|p| p.empty()).unwrap_or(false) {
                        player = None;
                        let _ = app_handle.emit("sound_finished", ());
                    }
                    continue;
                }
                Err(()) => break,
            };

            match msg {
                AudioMsg::Stop => {
                    if let Some(ref p) = player {
                        p.stop();
                    }
                    player = None;
                    let _ = app_handle.emit("sound_finished", ());
                }

                AudioMsg::Play { path, device_name } => {
                    if let Some(p) = player.take() {
                        p.stop();
                        // Allow WASAPI callbacks to drain before reusing the audio
                        // device. Without this delay the WASAPI backend may still be
                        // processing the old buffer when a new Player is created,
                        // triggering the "slice::get_unchecked_mut" panic.
                        thread::sleep(Duration::from_millis(50));
                    }

                    if let Err(e) = ensure_stream(&mut stream, &device_name) {
                        eprintln!("Failed to open audio device: {}", e);
                        let _ = app_handle.emit("sound_error", e);
                        continue;
                    }

                    // stream is guaranteed Some after ensure_stream returns Ok.
                    let st = stream.as_ref().expect("ensure_stream guarantees Some on Ok");

                    let new_player = Player::connect_new(st.device_sink.mixer());

                    match load_source(&path) {
                        Ok(source) => {
                            new_player.append(source);
                            player = Some(new_player);
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
) -> Result<String, String> {
    let guard = audio_sender().lock().map_err(|e| e.to_string())?;
    let tx = (*guard).clone().ok_or("Audio thread not initialized")?;

    if active {
        tx.send(AudioMsg::Stop).map_err(|e| e.to_string())?;
        Ok("stopped".to_string())
    } else {
        tx.send(AudioMsg::Play { path: sound_path, device_name })
            .map_err(|e| e.to_string())?;
        Ok("playing".to_string())
    }
}