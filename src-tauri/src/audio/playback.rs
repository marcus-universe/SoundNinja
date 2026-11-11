use rodio::buffer::SamplesBuffer;
use rodio::{
    self, ChannelCount, DeviceSinkBuilder, MixerDeviceSink, Player, SampleRate, Source,
};
use serde::Serialize;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use tauri::Emitter;

use super::devices::{is_default_name, resolve_output_device};
use super::stream::PumpHandle;

// --- Global output volume (stored as f32 bits in an atomic) ---

static OUTPUT_VOLUME: AtomicU32 = AtomicU32::new(0x3F800000); // 1.0f32 bits

fn current_volume() -> f32 {
    f32::from_bits(OUTPUT_VOLUME.load(Ordering::Relaxed))
}

fn effective_volume(sound_volume: f32) -> f32 {
    (current_volume() * sound_volume.clamp(0.0, 1.0)).clamp(0.0, 1.0)
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
/// samples through it, so this only parses headers — a few buffered reads,
/// not the whole file.
fn probe_duration(sound_path: &str) -> Result<f64, String> {
    if let Some(secs) = super::cache::cached_duration(sound_path) {
        return Ok(secs);
    }
    let decoder = super::decode::open_decoder(sound_path)?;
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

/// PCM cache / decode target: stereo when ASIO channel mapping is on, otherwise
/// the device's native channel count.
pub fn mix_output_format(rate: SampleRate, channels: ChannelCount) -> (SampleRate, ChannelCount) {
    (rate, super::channel_map::mix_channels(channels))
}

// --- Identity of the open stream ---

/// What the open output stream was opened for and the format it runs at.
/// Published by the audio thread so `prepare_play` and the cache warmer can
/// pick the decode format without enumerating devices again (on PipeWire that
/// enumeration is a full server connect, on every play).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StreamIdent {
    /// Resolved display name of the device, never the "default" sentinel.
    device_name: String,
    host_name: Option<String>,
    /// True when the stream was opened with ASIO L/R mapping (max channels).
    mapped: bool,
    rate: SampleRate,
    channels: ChannelCount,
}

static STREAM_IDENT: Mutex<Option<StreamIdent>> = Mutex::new(None);

fn set_stream_ident(ident: Option<StreamIdent>) {
    // Poisoning cannot leave this plain value half-written; keep going.
    *STREAM_IDENT.lock().unwrap_or_else(|e| e.into_inner()) = ident;
}

/// True when a stream described by `ident` can serve a request for
/// `device_name` / `host_name` with the current channel-map setting. The
/// "default" sentinel matches whatever device is open on that host.
fn stream_matches(
    ident: &StreamIdent,
    device_name: &str,
    host_name: Option<&str>,
    mapped: bool,
) -> bool {
    let host_same = ident.host_name.as_deref() == host_name;
    let device_same = ident.device_name == device_name || is_default_name(device_name);
    host_same && device_same && ident.mapped == mapped
}

/// Device format of the open stream when it serves this device/host, else `None`.
pub(crate) fn stream_format_for(
    device_name: &str,
    host_name: Option<&str>,
) -> Option<(SampleRate, ChannelCount)> {
    let mapped = super::channel_map::output_enabled();
    let guard = STREAM_IDENT.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .as_ref()
        .filter(|ident| stream_matches(ident, device_name, host_name, mapped))
        .map(|ident| (ident.rate, ident.channels))
}

/// Decode target for a play on this device: `(rate, channels, from_stream)`.
/// The open stream's own format wins; otherwise the device is asked.
fn resolve_play_format(
    device_name: &str,
    host_name: Option<&str>,
) -> Option<(SampleRate, ChannelCount, bool)> {
    if let Some((rate, channels)) = stream_format_for(device_name, host_name) {
        let (rate, channels) = mix_output_format(rate, channels);
        return Some((rate, channels, true));
    }
    // Do not ASIOInit a driver just to pick a decode format — that races the
    // audio thread and is a common Windows heap-corruption trigger.
    let (rate, device_ch) = if super::devices::is_asio_host_name(host_name) {
        output_format().or_else(|| Some((SampleRate::new(48_000)?, ChannelCount::new(2)?)))
    } else {
        super::devices::default_output_format(device_name, host_name).or_else(output_format)
    }?;
    let (rate, channels) = mix_output_format(rate, device_ch);
    Some((rate, channels, false))
}

/// Format the cache warmer should decode to so its keys match what a later
/// play asks for. Unlike a play this never guesses a format for ASIO: warming
/// a whole board into a guessed rate would be wasted work.
pub(crate) fn warm_format(
    device_name: Option<&str>,
    host_name: Option<&str>,
) -> Option<(SampleRate, ChannelCount)> {
    let Some(name) = device_name else {
        return output_format().map(|(rate, ch)| mix_output_format(rate, ch));
    };
    if super::devices::is_asio_host_name(host_name) && stream_format_for(name, host_name).is_none()
    {
        return output_format().map(|(rate, ch)| mix_output_format(rate, ch));
    }
    resolve_play_format(name, host_name).map(|(rate, channels, _)| (rate, channels))
}

fn unix_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
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
    /// True when `duration_secs` is the real decoded length (cached PCM, or a
    /// pump that decoded the whole file). False while it is header metadata,
    /// an estimate, or still growing.
    pub duration_exact: bool,
    /// Unix epoch milliseconds at which `position_secs` was sampled, so the
    /// frontend can extrapolate the playhead without polling.
    pub as_of_ms: u64,
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
            duration_exact: s.duration_exact,
            as_of_ms: unix_millis(),
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
        /// Per-sound gain 0–1, multiplied with the master output volume.
        volume: f32,
        /// Restart from the start when this instance ends.
        looping: bool,
        /// When the message was queued; only read for `SN_PERF` timings.
        queued_at: Instant,
    },
    /// Open the output stream ahead of the first play so that play does not
    /// pay for the device open. Ignored while anything is playing.
    Prewarm {
        device_name: String,
        host_name: Option<String>,
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
    /// Update per-sound gain on already-playing instances of `path`.
    SetSoundVolume {
        path: String,
        volume: f32,
    },
    /// Settings changed the ASIO L/R map. Drop an idle stream so the next
    /// play opens at the new channel count.
    ApplyChannelMap,
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
    /// Device, host, channel-map state and the output format sources are
    /// pre-converted to, so the mixer never resamples.
    ident: StreamIdent,
}

impl AudioStream {
    fn open(device_name: &str, host_name: Option<&str>) -> Result<Self, String> {
        let mapped = super::channel_map::output_enabled();
        let asio = super::devices::is_asio_host_name(host_name);
        // Hold the process-wide ASIO lock for load + create_buffers. Drop it
        // after the stream exists so the callback is not behind the mutex.
        let _asio_guard = asio.then(super::devices::lock_asio_api);
        let device = resolve_output_device(device_name, host_name).map_err(|e| e.to_string())?;
        let resolved_name = super::devices::device_display_name(&device)
            .unwrap_or_else(|| device_name.to_owned());
        // The channel counts only feed the ASIO channel picker. Probing them on
        // other hosts cost two config queries per open and put non-ASIO device
        // names into the ASIO metadata list.
        if asio {
            let outs = super::devices::max_output_channels(&device).unwrap_or(0);
            let ins = super::devices::max_input_channels(&device).unwrap_or(0);
            super::devices::cache_asio_device_channels(&resolved_name, outs, ins);
        }
        // rodio's from_device() pins a 50ms Fixed buffer. ASIO drivers reject
        // or heap-corrupt on sizes outside their quantum — use Default instead.
        let mut builder = DeviceSinkBuilder::from_device(device).map_err(|e| e.to_string())?;
        if asio {
            builder = builder.with_buffer_size(rodio::cpal::BufferSize::Default);
        }
        let device_sink = builder.open_stream().map_err(|e| e.to_string())?;
        let sample_rate = device_sink.config().sample_rate();
        let channels = device_sink.config().channel_count();
        publish_output_format(sample_rate, channels);
        let ident = StreamIdent {
            device_name: resolved_name,
            host_name: host_name.map(str::to_owned),
            mapped,
            rate: sample_rate,
            channels,
        };
        set_stream_ident(Some(ident.clone()));
        Ok(Self { device_sink, ident })
    }
}

/// Drop the stream and withdraw its published identity in one step, so no
/// caller can pick a decode format from a stream that is already gone.
fn close_stream(stream: &mut Option<AudioStream>) {
    *stream = None;
    set_stream_ident(None);
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
    /// `duration_secs` is the decoded length, not a header value or estimate.
    duration_exact: bool,
    /// Absolute file offset corresponding to player position 0 (set on seek).
    position_origin_secs: f64,
    ready: Ready,
    /// Per-sound gain 0–1 (master is applied separately).
    sound_volume: f32,
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

    /// Swap a finished pump for the buffer it stored in the PCM cache and
    /// return that buffer's exact duration. Restarts (loops) then share the
    /// cached samples instead of copying the pump's chunks every time.
    /// `None` when there is nothing to swap: already cached, still decoding,
    /// or the result was too large to be kept.
    fn settle(&mut self) -> Option<f64> {
        let Self::Growing(h) = self else { return None };
        if !h.is_done() {
            return None;
        }
        let (rate, channels) = h.format();
        let (buffer, duration) = super::pcm::peek(h.path(), rate, channels)?;
        *self = Self::Cached(buffer);
        Some(duration)
    }

    /// Queue this sound from its start. Both variants share their samples:
    /// a cached buffer by `Arc`, a pump through its published chunks.
    fn append_to(&self, player: &Player, dst_ch: ChannelCount) {
        match self {
            Self::Cached(buf) => append_mapped(player, buf.clone(), dst_ch),
            Self::Growing(h) => append_mapped(player, h.source(), dst_ch),
        }
    }
}

/// Follow the pump's view of the track length. While it decodes, the length
/// only grows (metadata, then the buffer); once the whole file is decoded the
/// real length replaces the estimate, even when that is shorter. Returns
/// whether anything the frontend sees changed.
fn adopt_pump_duration(s: &mut PlayingSound) -> bool {
    let Ready::Growing(h) = &s.ready else {
        return false;
    };
    let duration = h.duration_secs();
    if h.is_complete() && duration > 0.0 {
        let changed = !s.duration_exact || duration != s.duration_secs;
        s.duration_secs = duration;
        s.duration_exact = true;
        return changed;
    }
    if duration > s.duration_secs {
        s.duration_secs = duration;
        return true;
    }
    false
}

fn append_mapped<S: Source + Send + 'static>(player: &Player, source: S, dst_ch: ChannelCount) {
    if super::channel_map::output_enabled() {
        player.append(super::channel_map::ChannelScatter::new(source, dst_ch.get()));
    } else {
        player.append(source);
    }
}

// --- Helpers ---

/// Cache hit is instant. Miss starts a pump and waits only for preroll (~300 ms).
pub fn prepare_play(
    path: &str,
    device_name: &str,
    host_name: Option<&str>,
) -> Result<(Ready, f64), String> {
    let started = Instant::now();
    let (rate, channels, from_stream) = resolve_play_format(device_name, host_name)
        .ok_or_else(|| "Cannot resolve output format".to_string())?;
    let format_ms = crate::perf::ms_since(started);
    if let Some((buffer, duration)) = super::pcm::peek(path, rate, channels) {
        if duration > 0.0 {
            super::cache::store_duration(path, duration);
        }
        crate::perf::log(|| {
            format!(
                "prepare_play cache=hit format={format_ms:.2}ms (from open stream: {from_stream}) \
                 total={:.2}ms '{path}'",
                crate::perf::ms_since(started)
            )
        });
        return Ok((Ready::Cached(buffer), duration));
    }
    let preroll_started = Instant::now();
    let handle = super::stream::start(path.to_owned(), rate, channels)?;
    crate::perf::log(|| {
        format!(
            "prepare_play cache=miss format={format_ms:.2}ms (from open stream: {from_stream}) \
             preroll={:.2}ms total={:.2}ms '{path}'",
            crate::perf::ms_since(preroll_started),
            crate::perf::ms_since(started)
        )
    });
    let duration = handle
        .duration_secs()
        .max(super::cache::cached_duration(path).unwrap_or(0.0));
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
    let mapped = super::channel_map::output_enabled();
    let asio = super::devices::is_asio_host_name(host_name);
    let reusable = stream
        .as_ref()
        .is_some_and(|s| stream_matches(&s.ident, device_name, host_name, mapped));
    if reusable {
        return Ok(());
    }

    let had_stream = stream.is_some();
    close_stream(stream);
    if had_stream {
        thread::sleep(Duration::from_millis(if asio { 250 } else { 100 }));
    }
    *stream = Some(AudioStream::open(device_name, host_name)?);
    Ok(())
}

/// Pause after stopping players in a non-overlap play. See the Play handler.
const STOP_SETTLE: Duration = Duration::from_millis(10);

/// Whether opening the output stream ahead of the first play is safe on this
/// host. ASIO drivers are exclusive and must only be loaded for a real play.
/// On Linux only the sound servers share a device; a raw ALSA or JACK stream
/// would hold the device (or a JACK port) while the app sits idle.
fn prewarm_allowed(os: &str, host_name: Option<&str>) -> bool {
    if super::devices::is_asio_host_name(host_name) {
        return false;
    }
    match os {
        "windows" | "macos" => true,
        "linux" => host_name.map(str::trim).is_some_and(|h| {
            h.eq_ignore_ascii_case("pipewire") || h.eq_ignore_ascii_case("pulseaudio")
        }),
        _ => false,
    }
}

/// True when a `Prewarm` sent now would be acted on: the host allows it and
/// nothing is playing. The cache warmer uses this to decide whether waiting
/// for the stream's format is worthwhile.
pub(crate) fn prewarm_expected(host_name: Option<&str>) -> bool {
    prewarm_allowed(std::env::consts::OS, host_name)
        && playing_list().lock().map(|g| g.is_empty()).unwrap_or(false)
}

/// Diagnostics that are only useful while developing; silent in release builds.
fn debug_log(message: impl FnOnce() -> String) {
    if cfg!(debug_assertions) {
        eprintln!("{}", message());
    }
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
        drop_idx[i] = batch[i + 1..]
            .iter()
            .any(|later| matches!(later, AudioMsg::Seek { path, .. } if *path == key_i));
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
    adopt_pump_duration(&mut playing[i]);
    let mut duration_secs = playing[i].duration_secs;
    // Audio can only land inside decoded samples while the pump is still
    // filling. Timeline duration (metadata) stays on PlayingSound; seek target
    // is clamped to ready so the emitted playhead is not a fake 100%.
    let max_pos = match &playing[i].ready {
        Ready::Growing(h) if !h.is_done() => {
            let ready = h.ready_secs();
            if duration_secs > 0.0 {
                ready.min(duration_secs)
            } else {
                ready
            }
        }
        _ => duration_secs,
    };
    let clamped = clamp_seek_pos(position_secs, max_pos);
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
            Ready::Growing(h) => h.duration_secs(),
        };
    }

    let new_player = Player::connect_new(stream.device_sink.mixer());
    new_player.set_volume(effective_volume(playing[i].sound_volume));
    let origin = match &ready {
        Ready::Cached(buf) => {
            let mut source = buf.clone();
            match source.try_seek(target) {
                Ok(()) => {
                    append_mapped(&new_player, source, stream.ident.channels);
                    clamped
                }
                Err(_) if clamped < 2.0 => {
                    append_mapped(
                        &new_player,
                        buf.clone().skip_duration(Duration::from_secs_f64(clamped)),
                        stream.ident.channels,
                    );
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
            append_mapped(&new_player, source, stream.ident.channels);
            clamped
        }
    };

    if was_paused {
        new_player.pause();
    }
    let sound_volume = playing[i].sound_volume;
    let duration_exact = playing[i].duration_exact;
    let old = std::mem::replace(
        &mut playing[i],
        PlayingSound {
            player: new_player,
            path: file_path,
            looping,
            duration_secs,
            duration_exact,
            position_origin_secs: origin,
            ready,
            sound_volume,
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
            let mut finished_any = false;
            // Adopt metadata/hint, or the exact length once the pump reached
            // EOF. Runs before the restart pass below, which may replace a
            // finished pump with its cached buffer.
            for s in &mut playing {
                finished_any |= adopt_pump_duration(s);
            }

            // Drain naturally-finished sounds and notify frontend.
            let mut i = 0;
            while i < playing.len() {
                if playing[i].player.empty() {
                    if playing[i].looping {
                        let was_paused = playing[i].player.is_paused();
                        let sink_pos = playing[i].player.get_pos().as_secs_f64();
                        playing[i].position_origin_secs = -sink_pos;
                        if let Some(exact) = playing[i].ready.settle() {
                            playing[i].duration_secs = exact;
                            playing[i].duration_exact = true;
                        }
                        let dst_ch = stream.as_ref().map(|s| s.ident.channels);
                        if let Some(ch) = dst_ch {
                            playing[i].ready.append_to(&playing[i].player, ch);
                        }
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
                    mut ready,
                    mut duration_secs,
                    volume,
                    looping,
                    queued_at,
                } => {
                    if !overlap && !playing.is_empty() {
                        for s in playing.drain(..) {
                            s.ready.cancel();
                            s.player.stop();
                            let _ = app_handle.emit("sound_finished", s.path);
                        }
                        // The mixer stays open across sounds, so there is no
                        // device to wait for. A short pause only lets the
                        // stopped players leave the mixer before the new one
                        // starts; with nothing stopped there is nothing to wait on.
                        thread::sleep(STOP_SETTLE);
                    }

                    if let Err(e) = ensure_stream(&mut stream, &device_name, host_name.as_deref()) {
                        eprintln!("Failed to open audio device: {}", e);
                        ready.cancel();
                        let _ = app_handle.emit("sound_error", e);
                        // The frontend marked this sound active when it sent the
                        // play; without this it would stay lit forever.
                        let _ = app_handle.emit("sound_finished", path);
                        sync_playing_list(&app_handle, &playing);
                        continue;
                    }

                    let st = stream.as_ref().expect("ensure_stream guarantees Some on Ok");
                    let new_player = Player::connect_new(st.device_sink.mixer());
                    let sound_volume = volume.clamp(0.0, 1.0);
                    new_player.set_volume(effective_volume(sound_volume));
                    // Cached PCM is the whole decoded file. A pump only knows
                    // the real length once it reaches the end; a short clip
                    // may already be there (and in the cache) by now.
                    let mut duration_exact = matches!(ready, Ready::Cached(_));
                    if let Some(exact) = ready.settle() {
                        duration_secs = exact;
                        duration_exact = true;
                    }
                    ready.append_to(&new_player, st.ident.channels);
                    crate::perf::log(|| {
                        format!(
                            "play message->appended {:.2}ms '{path}'",
                            crate::perf::ms_since(queued_at)
                        )
                    });
                    playing.push(PlayingSound {
                        player: new_player,
                        path: path.clone(),
                        looping,
                        duration_secs,
                        duration_exact,
                        position_origin_secs: 0.0,
                        ready,
                        sound_volume,
                    });
                    let keep: Vec<String> = playing.iter().map(|s| s.path.clone()).collect();
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

                AudioMsg::SetVolume(_) => {
                    for s in &playing {
                        s.player.set_volume(effective_volume(s.sound_volume));
                    }
                }

                AudioMsg::SetSoundVolume { path, volume } => {
                    let volume = volume.clamp(0.0, 1.0);
                    for s in &mut playing {
                        if s.path == path {
                            s.sound_volume = volume;
                            s.player.set_volume(effective_volume(volume));
                        }
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

                AudioMsg::ApplyChannelMap => {
                    if playing.is_empty() && stream.is_some() {
                        thread::sleep(Duration::from_millis(100));
                        close_stream(&mut stream);
                    }
                }

                AudioMsg::Prewarm {
                    device_name,
                    host_name,
                } => {
                    let host = host_name.as_deref();
                    if playing.is_empty() && prewarm_allowed(std::env::consts::OS, host) {
                        // Best effort: a device that cannot be opened now is
                        // reported by the play that actually needs it.
                        if let Err(e) = ensure_stream(&mut stream, &device_name, host) {
                            debug_log(|| format!("[audio] prewarm skipped: {e}"));
                        }
                    }
                }
            }
            } // for msg in batch
        }
        // The command channel closed (shutdown): withdraw the stream's
        // identity together with the stream itself.
        close_stream(&mut stream);
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
    volume: f32,
    looping: bool,
) -> Result<f64, String> {
    let (ready, duration) = prepare_play(&sound_path, &device_name, host_name.as_deref())?;
    send_msg(AudioMsg::Play {
        path: sound_path,
        device_name,
        host_name,
        overlap,
        ready,
        duration_secs: duration,
        volume,
        looping,
        queued_at: Instant::now(),
    })?;
    Ok(duration)
}

/// Ask the audio thread to open the output stream for this device now.
pub(crate) fn request_prewarm(device_name: &str, host_name: Option<&str>) {
    let _ = send_msg(AudioMsg::Prewarm {
        device_name: device_name.to_owned(),
        host_name: host_name.map(str::to_owned),
    });
}

#[tauri::command]
pub async fn play_sound(
    sound_path: String,
    device_name: String,
    host_name: Option<String>,
    active: bool,
    overlap: bool,
    volume: Option<f32>,
    looping: Option<bool>,
) -> Result<f64, String> {
    if active {
        send_msg(AudioMsg::StopOne { path: sound_path })?;
        return Ok(0.0);
    }
    let volume = volume.unwrap_or(1.0).clamp(0.0, 1.0);
    let looping = looping.unwrap_or(false);
    crate::task::run_blocking(move || {
        enqueue_play(sound_path, device_name, host_name, overlap, volume, looping)
    })
    .await
}

#[tauri::command(async)]
pub fn set_sound_volume(sound_path: String, volume: f32) -> Result<(), String> {
    send_msg(AudioMsg::SetSoundVolume {
        path: sound_path,
        volume: volume.clamp(0.0, 1.0),
    })
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

#[tauri::command(async)]
pub fn set_asio_channel_map(
    enabled: bool,
    out_left: u16,
    out_right: u16,
    in_left: u16,
    in_right: u16,
) -> Result<(), String> {
    super::channel_map::configure(enabled, out_left, out_right, in_left, in_right);
    let _ = send_msg(AudioMsg::ApplyChannelMap);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        close_stream, ensure_stream, prewarm_allowed, resolve_play_format, stream_format_for,
        stream_matches, StreamIdent,
    };
    use rodio::{ChannelCount, SampleRate};

    fn ident(device: &str, host: Option<&str>, mapped: bool) -> StreamIdent {
        StreamIdent {
            device_name: device.to_owned(),
            host_name: host.map(str::to_owned),
            mapped,
            rate: SampleRate::new(48_000).unwrap(),
            channels: ChannelCount::new(2).unwrap(),
        }
    }

    #[test]
    fn stream_matches_follows_device_host_and_channel_map() {
        let open = ident("Speakers", Some("Wasapi"), false);
        // (requested device, requested host, requested map, expected)
        let cases = [
            ("Speakers", Some("Wasapi"), false, true),
            // The sentinel means "whatever is open on this host".
            ("default", Some("Wasapi"), false, true),
            ("", Some("Wasapi"), false, true),
            ("Headphones", Some("Wasapi"), false, false),
            ("Speakers", Some("ASIO"), false, false),
            ("Speakers", None, false, false),
            ("default", Some("ASIO"), false, false),
            ("Speakers", Some("Wasapi"), true, false),
        ];
        for (device, host, mapped, expected) in cases {
            assert_eq!(
                stream_matches(&open, device, host, mapped),
                expected,
                "device={device:?} host={host:?} mapped={mapped}"
            );
        }
    }

    #[test]
    fn stream_matches_treats_missing_host_as_its_own_host() {
        let open = ident("Speakers", None, true);
        assert!(stream_matches(&open, "default", None, true));
        assert!(!stream_matches(&open, "default", Some("Wasapi"), true));
    }

    /// Opens the real default output device (silently, nothing is queued).
    /// Needs audio hardware, so it only runs with `RUN_SLOW_TESTS` set, like
    /// the device listing test in `devices.rs`.
    #[test]
    fn open_stream_publishes_the_format_a_play_resolves_to() {
        if std::env::var("RUN_SLOW_TESTS").is_err() {
            return;
        }
        let mut stream = None;
        if let Err(e) = ensure_stream(&mut stream, "default", None) {
            eprintln!("skip: no usable output device ({e})");
            return;
        }
        let opened = stream.as_ref().map(|s| s.ident.clone()).unwrap();
        let format = Some((opened.rate, opened.channels));
        assert_eq!(stream_format_for("default", None), format);
        assert_eq!(stream_format_for(&opened.device_name, None), format);
        assert_eq!(stream_format_for("default", Some("ASIO")), None);

        // A play on this device takes its format from the stream, without
        // asking the device again.
        let (rate, _, from_stream) = resolve_play_format("default", None).unwrap();
        assert!(from_stream);
        assert_eq!(rate, opened.rate);

        // Asking again for the same device keeps the stream.
        ensure_stream(&mut stream, "default", None).unwrap();
        assert_eq!(stream.as_ref().map(|s| s.ident.clone()), Some(opened));

        close_stream(&mut stream);
        assert_eq!(stream_format_for("default", None), None);
    }

    #[test]
    fn prewarm_allowed_only_for_shared_hosts() {
        let cases = [
            ("windows", Some("Wasapi"), true),
            ("windows", None, true),
            ("windows", Some("ASIO"), false),
            ("macos", None, true),
            ("linux", Some("PipeWire"), true),
            ("linux", Some("pulseaudio"), true),
            ("linux", Some("Alsa"), false),
            ("linux", Some("Jack"), false),
            // Unknown host on Linux may resolve to raw ALSA.
            ("linux", None, false),
            ("freebsd", None, false),
        ];
        for (os, host, expected) in cases {
            assert_eq!(prewarm_allowed(os, host), expected, "os={os} host={host:?}");
        }
    }
}
