use rodio::buffer::SamplesBuffer;
use rodio::{
    self, ChannelCount, DeviceSinkBuilder, MixerDeviceSink, Player, SampleRate, Source,
};
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tauri::Emitter;

use super::devices::{is_default_name, resolve_output_device};
use super::stream::PumpHandle;

// --- Global output volume (stored as f32 bits in an atomic) ---

static OUTPUT_VOLUME: AtomicU32 = AtomicU32::new(0x3F800000); // 1.0f32 bits

fn current_volume() -> f32 {
    f32::from_bits(OUTPUT_VOLUME.load(Ordering::Relaxed))
}

#[tauri::command(async)]
pub fn set_output_volume(volume: f32) {
    let volume = volume.clamp(0.0, 1.0);
    OUTPUT_VOLUME.store(volume.to_bits(), Ordering::Relaxed);
    // Best effort: sounds already running follow the slider too.
    let _ = send_msg(AudioMsg::SetVolume(volume));
}

// --- Duration query ---

#[tauri::command(async)]
pub fn get_sound_duration(sound_path: String) -> Result<f64, String> {
    probe_duration(&sound_path)
}

/// Track length from container metadata. Builds a decoder but never pulls
/// samples through it, so this only parses headers.
fn probe_duration(sound_path: &str) -> Result<f64, String> {
    if let Some(secs) = super::cache::cached_duration(sound_path) {
        return Ok(secs);
    }
    let cursor = super::cache::open_cursor(sound_path)?;
    let decoder = rodio::Decoder::new(cursor).map_err(|e| e.to_string())?;
    let secs = decoder.total_duration().map(|d| d.as_secs_f64()).unwrap_or(0.0);
    if secs > 0.0 {
        super::cache::store_duration(sound_path, secs);
    }
    Ok(secs)
}

// --- Output format (what PCM buffers are pre-converted to) ---

static OUTPUT_RATE: AtomicU32 = AtomicU32::new(0);
static OUTPUT_CHANNELS: AtomicU32 = AtomicU32::new(0);

fn publish_output_format(rate: SampleRate, channels: ChannelCount) {
    OUTPUT_RATE.store(rate.get(), Ordering::Relaxed);
    OUTPUT_CHANNELS.store(channels.get() as u32, Ordering::Relaxed);
}

/// Format of the currently open stream, if one has ever been opened.
/// Callers that need it before first playback fall back to querying the device.
pub fn output_format() -> Option<(SampleRate, ChannelCount)> {
    let rate = SampleRate::new(OUTPUT_RATE.load(Ordering::Relaxed))?;
    let channels = ChannelCount::new(OUTPUT_CHANNELS.load(Ordering::Relaxed) as u16)?;
    Some((rate, channels))
}

// --- Playing list snapshot (shared with frontend) ---

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayingInfo {
    pub path: String,
    pub paused: bool,
    pub position_secs: f64,
    pub looping: bool,
    pub duration_secs: f64,
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
            duration_secs: s.duration_secs,
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
        ready: Ready,
        duration_secs: f64,
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
    /// Apply the master volume to sounds that are already running.
    SetVolume(f32),
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
    /// Output format sources are pre-converted to, so the mixer never resamples.
    #[allow(dead_code)]
    sample_rate: SampleRate,
    #[allow(dead_code)]
    channels: ChannelCount,
}

impl AudioStream {
    fn open(device_name: &str, host_name: Option<&str>) -> Result<Self, String> {
        let device = resolve_output_device(device_name, host_name).map_err(|e| e.to_string())?;
        let resolved_name = super::devices::device_display_name(&device)
            .unwrap_or_else(|| device_name.to_owned());
        let device_sink = DeviceSinkBuilder::from_device(device)
            .map_err(|e| e.to_string())?
            .open_stream()
            .map_err(|e| e.to_string())?;
        let sample_rate = device_sink.config().sample_rate();
        let channels = device_sink.config().channel_count();
        publish_output_format(sample_rate, channels);
        Ok(Self {
            device_sink,
            device_name: resolved_name,
            host_name: host_name.map(str::to_owned),
            sample_rate,
            channels,
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
    ready: Ready,
}

/// Fully cached PCM, or a live progressive pump.
#[derive(Clone)]
pub enum Ready {
    Cached(SamplesBuffer),
    Growing(PumpHandle),
}

impl Ready {
    fn cancel(&self) {
        if let Self::Growing(h) = self {
            h.cancel();
        }
    }

    fn append_to(&self, player: &Player) {
        match self {
            Self::Cached(buf) => player.append(buf.clone()),
            Self::Growing(h) => {
                if let Some(buf) = h.snapshot() {
                    player.append(buf);
                } else {
                    player.append(h.source());
                }
            }
        }
    }
}

// --- Helpers ---

/// Cache hit is instant. Miss starts a pump and waits only for preroll (~300 ms).
pub fn prepare_play(
    path: &str,
    device_name: &str,
    host_name: Option<&str>,
) -> Result<(Ready, f64), String> {
    let (rate, channels) = super::devices::default_output_format(device_name, host_name)
        .or_else(output_format)
        .ok_or_else(|| "Cannot resolve output format".to_string())?;
    if let Some((buffer, duration)) = super::pcm::peek(path, rate, channels) {
        if duration > 0.0 {
            super::cache::store_duration(path, duration);
        }
        return Ok((Ready::Cached(buffer), duration));
    }
    let handle = super::stream::start(path.to_owned(), rate, channels)?;
    let duration = handle.duration_secs();
    if duration > 0.0 {
        super::cache::store_duration(path, duration);
    }
    Ok((Ready::Growing(handle), duration))
}

/// Ensures the stream targets `device_name`/`host_name`, reopening only when they changed.
fn ensure_stream(
    stream: &mut Option<AudioStream>,
    device_name: &str,
    host_name: Option<&str>,
) -> Result<(), String> {
    let needs_new = stream
        .as_ref()
        .map(|s| {
            let host_changed = s.host_name.as_deref() != host_name;
            let device_changed = s.device_name != device_name && !is_default_name(device_name);
            host_changed || device_changed
        })
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

    // 2) Rebuild from the cached buffer or the live pump. Never full-decode here.
    let file_path = playing[i].path.clone();
    let looping = playing[i].looping;
    let ready = playing[i].ready.clone();
    if duration_secs <= 0.0 {
        duration_secs = match &ready {
            Ready::Cached(b) => b.total_duration().map(|d| d.as_secs_f64()).unwrap_or(0.0),
            Ready::Growing(h) => h.duration_secs().max(h.ready_secs()),
        };
    }
    let max_pos = match &ready {
        Ready::Growing(h) if !h.is_done() => h.ready_secs(),
        _ => duration_secs,
    };
    let clamped = clamp_seek_pos(position_secs, max_pos);
    let target = Duration::from_secs_f64(clamped);

    let new_player = Player::connect_new(stream.device_sink.mixer());
    new_player.set_volume(current_volume());
    let origin = match &ready {
        Ready::Cached(buf) => {
            let mut source = buf.clone();
            match source.try_seek(target) {
                Ok(()) => {
                    new_player.append(source);
                    clamped
                }
                Err(_) if clamped < 2.0 => {
                    new_player.append(buf.clone().skip_duration(Duration::from_secs_f64(clamped)));
                    clamped
                }
                Err(e) => {
                    eprintln!("Seek unsupported at {clamped:.2}s for '{file_path}': {e}");
                    return;
                }
            }
        }
        Ready::Growing(h) => {
            let mut source = h.source();
            let _ = source.try_seek(target);
            new_player.append(source);
            clamped
        }
    };

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
            ready,
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
                        let was_paused = playing[i].player.is_paused();
                        let sink_pos = playing[i].player.get_pos().as_secs_f64();
                        playing[i].position_origin_secs = -sink_pos;
                        playing[i].ready.append_to(&playing[i].player);
                        if was_paused {
                            playing[i].player.pause();
                        }
                        finished_any = true;
                        i += 1;
                    } else {
                        let finished = playing.remove(i);
                        let _ = app_handle.emit("sound_finished", finished.path);
                        finished_any = true;
                    }
                } else {
                    i += 1;
                }
            }
            for s in &mut playing {
                if let Ready::Growing(h) = &s.ready {
                    let d = h.duration_secs();
                    if d > s.duration_secs {
                        s.duration_secs = d;
                        finished_any = true;
                    }
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
                        s.ready.cancel();
                        s.player.stop();
                        let _ = app_handle.emit("sound_finished", s.path);
                    }
                    sync_playing_list(&app_handle, &playing);
                }

                AudioMsg::StopOne { path } => {
                    if let Some(pos) = playing.iter().position(|s| s.path == path) {
                        let s = playing.remove(pos);
                        s.ready.cancel();
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

                AudioMsg::Play {
                    path,
                    device_name,
                    host_name,
                    overlap,
                    ready,
                    duration_secs,
                } => {
                    if !overlap {
                        for s in playing.drain(..) {
                            s.ready.cancel();
                            s.player.stop();
                            let _ = app_handle.emit("sound_finished", s.path);
                        }
                        thread::sleep(Duration::from_millis(50));
                    }

                    if let Err(e) = ensure_stream(&mut stream, &device_name, host_name.as_deref()) {
                        eprintln!("Failed to open audio device: {}", e);
                        ready.cancel();
                        let _ = app_handle.emit("sound_error", e);
                        continue;
                    }

                    let st = stream.as_ref().expect("ensure_stream guarantees Some on Ok");
                    let new_player = Player::connect_new(st.device_sink.mixer());
                    new_player.set_volume(current_volume());
                    ready.append_to(&new_player);
                    playing.push(PlayingSound {
                        player: new_player,
                        path: path.clone(),
                        looping: false,
                        duration_secs,
                        position_origin_secs: 0.0,
                        ready,
                    });
                    let keep: Vec<String> = playing.iter().map(|s| s.path.clone()).collect();
                    super::cache::evict_idle(&keep);
                    super::pcm::evict_idle(&keep);
                    sync_playing_list(&app_handle, &playing);
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

                AudioMsg::SetVolume(volume) => {
                    for s in &playing {
                        s.player.set_volume(volume);
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

/// Decode on the calling thread, then hand a ready buffer to the audio thread.
/// Used by the record-editor preview (already on a worker) and by [`play_sound`].
pub fn enqueue_play(
    sound_path: String,
    device_name: String,
    host_name: Option<String>,
    overlap: bool,
) -> Result<f64, String> {
    let (ready, duration) = prepare_play(&sound_path, &device_name, host_name.as_deref())?;
    send_msg(AudioMsg::Play {
        path: sound_path,
        device_name,
        host_name,
        overlap,
        ready,
        duration_secs: duration,
    })?;
    Ok(duration)
}

#[tauri::command]
pub async fn play_sound(
    sound_path: String,
    device_name: String,
    host_name: Option<String>,
    active: bool,
    overlap: bool,
) -> Result<f64, String> {
    if active {
        send_msg(AudioMsg::StopOne { path: sound_path })?;
        return Ok(0.0);
    }
    crate::task::run_blocking(move || enqueue_play(sound_path, device_name, host_name, overlap))
        .await
}

#[tauri::command(async)]
pub fn pause_sound(sound_path: String) -> Result<(), String> {
    send_msg(AudioMsg::PauseOne { path: sound_path })
}

#[tauri::command(async)]
pub fn resume_sound(sound_path: String) -> Result<(), String> {
    send_msg(AudioMsg::ResumeOne { path: sound_path })
}

#[tauri::command(async)]
pub fn pause_all() -> Result<(), String> {
    send_msg(AudioMsg::PauseAll)
}

#[tauri::command(async)]
pub fn resume_all() -> Result<(), String> {
    send_msg(AudioMsg::ResumeAll)
}

#[tauri::command(async)]
pub fn stop_all() -> Result<(), String> {
    send_msg(AudioMsg::Stop)
}

#[tauri::command(async)]
pub fn get_playing_sounds() -> Vec<PlayingInfo> {
    playing_list()
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default()
}

#[tauri::command(async)]
pub fn seek_playing(position_secs: f64, sound_path: Option<String>) -> Result<(), String> {
    send_msg(AudioMsg::Seek {
        path: sound_path,
        position_secs,
    })
}

#[tauri::command(async)]
pub fn set_playing_loop(looping: bool, sound_path: Option<String>) -> Result<(), String> {
    send_msg(AudioMsg::SetLoop {
        path: sound_path,
        looping,
    })
}
