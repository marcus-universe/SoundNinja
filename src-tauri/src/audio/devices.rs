use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::cpal;
use serde::Serialize;
use std::sync::{Mutex, MutexGuard};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Friendly, unique label for UI + settings persistence.
///
/// PipeWire often puts a short card nick in `name()` and the useful route/port
/// label (e.g. "Scarlett 18i20 3rd Gen Headphones 1") in `extended()`. Prefer that.
pub fn device_display_name(d: &cpal::Device) -> Option<String> {
    if let Ok(desc) = d.description() {
        if let Some(ext) = desc.extended().next() {
            let ext = ext.trim();
            if !ext.is_empty() {
                return Some(ext.to_string());
            }
        }
        let primary = desc.name().trim();
        if !primary.is_empty() && !is_internal_device_label(primary) {
            return Some(primary.to_string());
        }
    }
    d.id().ok().map(|id| id.to_string()).filter(|s| !is_internal_device_label(s))
}

fn is_internal_device_label(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "sink_default"
            | "input_default"
            | "output_default"
            | "unknown"
            | "null"
            | "discard all samples (playback) or generate zero samples (capture)"
    ) || name.to_ascii_lowercase().starts_with("loopback-")
}

/// cpal's `HostId::name()` is the enum variant (`"Wasapi"`), while the UI historically
/// stores `"WASAPI"`. Treat those as equal (case-insensitive).
fn host_name_matches(id_name: &str, wanted: &str) -> bool {
    id_name.eq_ignore_ascii_case(wanted)
}

/// UI / default preference: PipeWire > PulseAudio > Alsa on Linux; Wasapi first on Windows.
fn host_sort_key(name: &str) -> u8 {
    match name.to_ascii_lowercase().as_str() {
        "pipewire" => 0,
        "pulseaudio" => 1,
        "wasapi" => 0,
        "coreaudio" => 0,
        "asio" => 1,
        "jack" => 3,
        "alsa" => 4,
        _ => 10,
    }
}

pub fn is_asio_host_name(host_name: Option<&str>) -> bool {
    host_name
        .map(str::trim)
        .is_some_and(|h| !h.is_empty() && h.eq_ignore_ascii_case("asio"))
}

fn host_id_is_asio(id: &cpal::HostId) -> bool {
    id.name().eq_ignore_ascii_case("asio")
}

/// ASIO allows one loaded driver process-wide. All load/enumerate/open calls
/// share this lock so UI listing cannot race the audio thread.
static ASIO_API: Mutex<()> = Mutex::new(());
static ASIO_META: Mutex<Vec<AsioMeta>> = Mutex::new(Vec::new());

#[derive(Clone)]
struct AsioMeta {
    name: String,
    outputs: u16,
    inputs: u16,
}

fn remember_asio_meta(name: &str, outputs: u16, inputs: u16) {
    if name.is_empty() {
        return;
    }
    let mut cache = ASIO_META.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(hit) = cache.iter_mut().find(|m| m.name == name) {
        if outputs > 0 {
            hit.outputs = outputs;
        }
        if inputs > 0 {
            hit.inputs = inputs;
        }
        return;
    }
    cache.push(AsioMeta {
        name: name.to_string(),
        outputs,
        inputs,
    });
}

pub fn cache_asio_device_channels(name: &str, outputs: u16, inputs: u16) {
    remember_asio_meta(name, outputs, inputs);
}

pub fn lock_asio_api() -> MutexGuard<'static, ()> {
    ASIO_API.lock().unwrap_or_else(|e| e.into_inner())
}

/// Driver names from the registry — does not `ASIOInit` any DLL.
#[cfg(target_os = "windows")]
fn asio_registry_names() -> Vec<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let mut names = Vec::new();
    for path in [r"SOFTWARE\ASIO", r"SOFTWARE\WOW6432Node\ASIO"] {
        let Ok(key) = hklm.open_subkey(path) else { continue };
        for name in key.enum_keys().flatten() {
            let name = name.trim().to_string();
            if name.is_empty() {
                continue;
            }
            if !names.iter().any(|n: &String| n.eq_ignore_ascii_case(&name)) {
                names.push(name);
            }
        }
    }
    names
}

#[cfg(not(target_os = "windows"))]
fn asio_registry_names() -> Vec<String> {
    Vec::new()
}

fn asio_host() -> Option<cpal::Host> {
    let id = cpal::available_hosts()
        .into_iter()
        .find(|id| host_id_is_asio(id))?;
    cpal::host_from_id(id).ok()
}

#[cfg(target_os = "windows")]
fn cached_asio_meta(want: &str) -> Option<AsioMeta> {
    let cache = ASIO_META.lock().unwrap_or_else(|e| e.into_inner());
    if is_default_name(want) {
        cache.first().cloned()
    } else {
        cache.iter().find(|m| m.name == want).cloned()
    }
}

/// Load ASIO drivers only until `want` matches. Does not `collect` the rest.
#[cfg(target_os = "windows")]
fn probe_asio_device(want: &str) -> Option<AsioMeta> {
    if let Some(hit) = cached_asio_meta(want) {
        if hit.outputs > 0 || hit.inputs > 0 {
            return Some(hit);
        }
    }
    let _guard = lock_asio_api();
    if let Some(hit) = cached_asio_meta(want) {
        if hit.outputs > 0 || hit.inputs > 0 {
            return Some(hit);
        }
    }
    let host = asio_host()?;
    let devices = host.devices().ok()?;
    for d in devices {
        let Some(name) = device_display_name(&d) else {
            continue;
        };
        let outputs = d.default_output_config().map(|c| c.channels()).unwrap_or(0);
        let inputs = d.default_input_config().map(|c| c.channels()).unwrap_or(0);
        remember_asio_meta(&name, outputs, inputs);
        if is_default_name(want) || name == want {
            return Some(AsioMeta {
                name,
                outputs,
                inputs,
            });
        }
    }
    None
}

/// Resolve a host id by name. Returns `None` when `host_name` is empty / "default"
/// (caller should search all hosts). Errors only when a concrete name was given
/// and nothing matches.
fn resolve_host_id(host_name: Option<&str>) -> Result<Option<cpal::HostId>, anyhow::Error> {
    let Some(wanted) = host_name.map(str::trim).filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("default")) else {
        return Ok(None);
    };
    let hosts = cpal::available_hosts();
    if let Some(id) = hosts.iter().find(|id| host_name_matches(id.name(), wanted)).copied() {
        return Ok(Some(id));
    }
    // Soft fallback: unknown/legacy host labels (e.g. old "WASAPI" vs "Wasapi") —
    // search all hosts instead of failing hard.
    eprintln!(
        "Audio host '{}' not in {:?}; falling back to all hosts",
        wanted,
        hosts.iter().map(|h| h.name()).collect::<Vec<_>>()
    );
    Ok(None)
}

/// Returns output devices, optionally restricted to one named host.
/// `host_name = None` → merges all available hosts (backward-compat).
pub fn get_output_devices_for_host_name(host_name: Option<&str>) -> Result<Vec<cpal::Device>> {
    match resolve_host_id(host_name)? {
        Some(host_id) => {
            if host_id_is_asio(&host_id) {
                // Callers that need a live Device must use resolve_asio_device.
                // Collecting here would ASIOInit every installed driver.
                return Ok(Vec::new());
            }
            let host = cpal::host_from_id(host_id)?;
            Ok(host
                .devices()?
                .filter(|d| d.default_output_config().is_ok())
                .collect())
        }
        None => {
            let mut devices = Vec::new();
            for host_id in cpal::available_hosts() {
                // Never probe ASIO while merging hosts — loading every ASIO DLL
                // just to list WASAPI devices is a common heap-corruption trigger.
                if host_id_is_asio(&host_id) {
                    continue;
                }
                if let Ok(host) = cpal::host_from_id(host_id) {
                    if let Ok(devs) = host.devices() {
                        devices.extend(devs.filter(|d| d.default_output_config().is_ok()));
                    }
                }
            }
            Ok(devices)
        }
    }
}

/// Returns input devices, optionally restricted to one named host.
pub fn get_input_devices_for_host_name(host_name: Option<&str>) -> Result<Vec<cpal::Device>> {
    match resolve_host_id(host_name)? {
        Some(host_id) => {
            if host_id_is_asio(&host_id) {
                return Ok(Vec::new());
            }
            let host = cpal::host_from_id(host_id)?;
            Ok(host
                .devices()?
                .filter(|d| d.default_input_config().is_ok())
                .collect())
        }
        None => {
            let mut devices = Vec::new();
            for host_id in cpal::available_hosts() {
                if host_id_is_asio(&host_id) {
                    continue;
                }
                if let Ok(host) = cpal::host_from_id(host_id) {
                    if let Ok(devs) = host.devices() {
                        devices.extend(devs.filter(|d| d.default_input_config().is_ok()));
                    }
                }
            }
            Ok(devices)
        }
    }
}

/// Alias for use by playback.rs (all hosts merged).
pub fn get_output_devices() -> Result<Vec<cpal::Device>> {
    get_output_devices_for_host_name(None)
}

pub fn is_default_name(name: &str) -> bool {
    name.is_empty() || name.eq_ignore_ascii_case("default")
}

/// Resolve a playback device. `"default"` / empty uses the host's default
/// output — the frontend stores that sentinel before the user picks a device.
pub fn resolve_output_device(device_name: &str, host_name: Option<&str>) -> Result<cpal::Device> {
    if is_asio_host_name(host_name) {
        return resolve_asio_device(device_name, false);
    }
    if is_default_name(device_name) {
        if let Some(host_id) = resolve_host_id(host_name)? {
            let host = cpal::host_from_id(host_id)?;
            if let Some(d) = host.default_output_device() {
                return Ok(d);
            }
        }
        return cpal::default_host()
            .default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No default output device"));
    }
    get_output_devices_for_host_name(host_name)?
        .into_iter()
        .find(|d| device_display_name(d).as_deref() == Some(device_name))
        .ok_or_else(|| anyhow::anyhow!("Device '{}' not found", device_name))
}

fn resolve_asio_device(device_name: &str, input: bool) -> Result<cpal::Device> {
    let host = asio_host().ok_or_else(|| anyhow::anyhow!("No ASIO host"))?;
    if is_default_name(device_name) {
        let d = if input {
            host.default_input_device()
        } else {
            host.default_output_device()
        };
        return d.ok_or_else(|| anyhow::anyhow!("No ASIO device"));
    }
    host.devices()?
        .find(|d| device_display_name(d).as_deref() == Some(device_name))
        .ok_or_else(|| anyhow::anyhow!("Device '{}' not found", device_name))
}

/// Sample rate and channel count a device would be opened with, without
/// actually opening a stream. Used to pre-decode PCM into the right format
/// before the first sound plays.
pub fn default_output_format(
    device_name: &str,
    host_name: Option<&str>,
) -> Option<(rodio::SampleRate, rodio::ChannelCount)> {
    let device = resolve_output_device(device_name, host_name).ok()?;
    let config = device.default_output_config().ok()?;
    Some((
        std::num::NonZero::new(config.sample_rate())?,
        std::num::NonZero::new(config.channels())?,
    ))
}

/// Native output channel count (ASIO default is already the hardware max).
pub fn max_output_channels(device: &cpal::Device) -> Option<u16> {
    device
        .default_output_config()
        .ok()
        .map(|c| c.channels())
        .filter(|&n| n > 0)
        .or_else(|| {
            device
                .supported_output_configs()
                .ok()?
                .map(|c| c.channels())
                .max()
                .filter(|&n| n > 0)
        })
}

/// Native input channel count.
pub fn max_input_channels(device: &cpal::Device) -> Option<u16> {
    device
        .default_input_config()
        .ok()
        .map(|c| c.channels())
        .filter(|&n| n > 0)
        .or_else(|| {
            device
                .supported_input_configs()
                .ok()?
                .map(|c| c.channels())
                .max()
                .filter(|&n| n > 0)
        })
}

/// Find a device by display name among input or output devices.
/// `device_name` of `"default"` / empty → platform default input (or default output for loopback).
pub fn find_device_by_name(
    device_name: &str,
    host_name: Option<&str>,
    loopback: bool,
) -> Result<cpal::Device> {
    if is_asio_host_name(host_name) {
        let _guard = lock_asio_api();
        return resolve_asio_device(device_name, !loopback);
    }
    let want_default = device_name.is_empty() || device_name.eq_ignore_ascii_case("default");

    if loopback {
        if want_default {
            // Prefer the host's default output for loopback capture.
            if let Some(host_id) = resolve_host_id(host_name)? {
                let host = cpal::host_from_id(host_id)?;
                if let Some(d) = host.default_output_device() {
                    return Ok(d);
                }
            }
            return cpal::default_host()
                .default_output_device()
                .ok_or_else(|| anyhow::anyhow!("No default output device for loopback"));
        }
        get_output_devices_for_host_name(host_name)?
            .into_iter()
            .find(|d| device_display_name(d).as_deref() == Some(device_name))
            .ok_or_else(|| anyhow::anyhow!("Loopback device '{}' not found", device_name))
    } else if want_default {
        if let Some(host_id) = resolve_host_id(host_name)? {
            let host = cpal::host_from_id(host_id)?;
            if let Some(d) = host.default_input_device() {
                return Ok(d);
            }
        }
        cpal::default_host()
            .default_input_device()
            .ok_or_else(|| anyhow::anyhow!("No default input device"))
    } else {
        get_input_devices_for_host_name(host_name)?
            .into_iter()
            .find(|d| device_display_name(d).as_deref() == Some(device_name))
            .ok_or_else(|| anyhow::anyhow!("Input device '{}' not found", device_name))
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Returns the names of all audio hosts available on this platform
/// (e.g. "PipeWire"/"Alsa" on Linux; "Wasapi"; "ASIO" on Windows).
#[tauri::command(async)]
pub fn get_audio_hosts() -> Vec<String> {
    let mut hosts: Vec<String> = cpal::available_hosts()
        .into_iter()
        .map(|id| id.name().to_string())
        .collect();
    hosts.sort_by_key(|h| host_sort_key(h));
    hosts
}

/// Returns output-device names across all hosts (backward-compat: no params).
#[tauri::command(async)]
pub fn get_out_devices() -> Vec<String> {
    unique_preserve_order(
        get_output_devices_for_host_name(None)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|d| device_display_name(&d))
            .inspect(|n| println!("Device: {}", n))
            .collect(),
    )
}

/// Returns output-device names for a specific driver/host.
#[tauri::command(async)]
pub fn get_out_devices_host(host: String) -> Vec<String> {
    if host.eq_ignore_ascii_case("asio") {
        return unique_preserve_order(asio_registry_names());
    }
    unique_preserve_order(
        get_output_devices_for_host_name(Some(&host))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|d| device_display_name(&d))
            .collect(),
    )
}

fn unique_preserve_order(names: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    names
        .into_iter()
        .filter(|n| seen.insert(n.clone()))
        .collect()
}

/// Returns input-device names across all hosts.
#[tauri::command(async)]
pub fn get_in_devices() -> Vec<String> {
    get_input_devices_for_host_name(None)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|d| device_display_name(&d))
        .collect()
}

/// Returns input-device names for a specific driver/host.
#[tauri::command(async)]
pub fn get_in_devices_host(host: String) -> Vec<String> {
    if host.eq_ignore_ascii_case("asio") {
        return unique_preserve_order(asio_registry_names());
    }
    get_input_devices_for_host_name(Some(&host))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|d| device_display_name(&d))
        .collect()
}

#[derive(Clone, Serialize)]
pub struct CaptureDeviceInfo {
    pub name: String,
    pub loopback: bool,
}

/// Returns capture sources: real mics + loopback (PC Audio) outputs.
#[tauri::command(async)]
pub fn get_loopback_devices(host: Option<String>) -> Vec<CaptureDeviceInfo> {
    if host.as_deref().is_some_and(|h| h.eq_ignore_ascii_case("asio")) {
        return asio_registry_names()
            .into_iter()
            .map(|name| CaptureDeviceInfo {
                name,
                loopback: false,
            })
            .collect();
    }
    let host_ref = host.as_deref();
    let mut out = Vec::new();

    for d in get_input_devices_for_host_name(host_ref).unwrap_or_default() {
        if let Some(name) = device_display_name(&d) {
            out.push(CaptureDeviceInfo {
                name,
                loopback: false,
            });
        }
    }

    for d in get_output_devices_for_host_name(host_ref).unwrap_or_default() {
        if let Some(name) = device_display_name(&d) {
            // Avoid listing the same PipeWire duplex sink twice (sinks are often duplex).
            if out.iter().any(|e| e.name == name) {
                continue;
            }
            out.push(CaptureDeviceInfo {
                name: format!("{} (PC Audio)", name),
                loopback: true,
            });
        }
    }

    out
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AsioDeviceChannels {
    pub outputs: Vec<String>,
    pub inputs: Vec<String>,
}

#[cfg(target_os = "windows")]
fn channel_labels(count: u16) -> Vec<String> {
    (1..=count).map(|i| format!("Channel {i}")).collect()
}

/// Output and input channel names for an ASIO device.
/// Empty lists on non-Windows (no ASIO host).
#[tauri::command(async)]
#[allow(unused_variables)]
pub fn get_asio_device_channels(
    device_name: String,
    input_device_name: Option<String>,
) -> AsioDeviceChannels {
    #[cfg(target_os = "windows")]
    {
        let in_name = input_device_name
            .as_deref()
            .unwrap_or(&device_name)
            .to_string();
        let out_meta = probe_asio_device(&device_name);
        let in_meta = if in_name == device_name {
            out_meta.clone()
        } else {
            probe_asio_device(&in_name).or_else(|| out_meta.clone())
        };
        let outputs = out_meta
            .as_ref()
            .map(|m| channel_labels(m.outputs))
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| channel_labels(2));
        let inputs = in_meta
            .as_ref()
            .map(|m| channel_labels(m.inputs))
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| channel_labels(2));
        return AsioDeviceChannels { outputs, inputs };
    }
    #[cfg(not(target_os = "windows"))]
    AsioDeviceChannels {
        outputs: vec![],
        inputs: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_asio_host_name_detects_asio() {
        assert!(is_asio_host_name(Some("ASIO")));
        assert!(is_asio_host_name(Some("asio")));
        assert!(!is_asio_host_name(Some("Wasapi")));
        assert!(!is_asio_host_name(None));
    }

    #[test]
    fn is_default_name_treats_empty_and_default_as_sentinel() {
        assert!(is_default_name(""));
        assert!(is_default_name("default"));
        assert!(is_default_name("Default"));
        assert!(!is_default_name("Speakers"));
    }

    #[test]
    fn linux_host_lists_real_sinks() {
        if std::env::var("RUN_SLOW_TESTS").is_err() {
            return;
        }
        let hosts = get_audio_hosts();
        let host = hosts
            .iter()
            .find(|h| h.eq_ignore_ascii_case("pipewire"))
            .or_else(|| hosts.iter().find(|h| h.eq_ignore_ascii_case("pulseaudio")))
            .cloned();
        let Some(host) = host else {
            // CI / machines without PipeWire — ALSA-only is fine.
            eprintln!("skip: no PipeWire/PulseAudio host in {hosts:?}");
            return;
        };
        let outs = get_out_devices_host(host);
        assert!(
            !outs.is_empty(),
            "expected PipeWire/Pulse sinks, got empty list"
        );
        // On this Arch setup we expect at least one named sink (not only ALSA plugins).
        assert!(
            outs.iter().any(|n| {
                let l = n.to_ascii_lowercase();
                l.contains("scarlett")
                    || l.contains("headphones")
                    || l.contains("soundboard")
                    || l.contains("hdmi")
                    || l.contains("line output")
            }),
            "expected real system sinks, got {outs:?}"
        );
    }
}
