use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::cpal;
use serde::Serialize;

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
            let host = cpal::host_from_id(host_id)?;
            Ok(host
                .devices()?
                .filter(|d| d.default_output_config().is_ok())
                .collect())
        }
        None => {
            let mut devices = Vec::new();
            for host_id in cpal::available_hosts() {
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
            let host = cpal::host_from_id(host_id)?;
            Ok(host
                .devices()?
                .filter(|d| d.default_input_config().is_ok())
                .collect())
        }
        None => {
            let mut devices = Vec::new();
            for host_id in cpal::available_hosts() {
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

/// Find a device by display name among input or output devices.
/// `device_name` of `"default"` / empty → platform default input (or default output for loopback).
pub fn find_device_by_name(
    device_name: &str,
    host_name: Option<&str>,
    loopback: bool,
) -> Result<cpal::Device> {
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
/// (e.g. "PipeWire"/"Alsa" on Linux; "Wasapi"; "ASIO" only with `--features asio`).
#[tauri::command]
pub fn get_audio_hosts() -> Vec<String> {
    let mut hosts: Vec<String> = cpal::available_hosts()
        .into_iter()
        .map(|id| id.name().to_string())
        .collect();
    hosts.sort_by_key(|h| host_sort_key(h));
    hosts
}

/// Returns output-device names across all hosts (backward-compat: no params).
#[tauri::command]
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
#[tauri::command]
pub fn get_out_devices_host(host: String) -> Vec<String> {
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
#[tauri::command]
pub fn get_in_devices() -> Vec<String> {
    get_input_devices_for_host_name(None)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|d| device_display_name(&d))
        .collect()
}

/// Returns input-device names for a specific driver/host.
#[tauri::command]
pub fn get_in_devices_host(host: String) -> Vec<String> {
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
#[tauri::command]
pub fn get_loopback_devices(host: Option<String>) -> Vec<CaptureDeviceInfo> {
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

/// Returns mono output channel names for an ASIO device.
/// Always returns an empty list on non-ASIO builds (no `--features asio`).
#[tauri::command]
#[allow(unused_variables)]
pub fn get_asio_device_channels(device_name: String) -> Vec<String> {
    #[cfg(feature = "asio")]
    {
        let asio_id = match cpal::available_hosts()
            .into_iter()
            .find(|id| id.name() == "ASIO")
        {
            Some(id) => id,
            None => return vec![],
        };
        let host = match cpal::host_from_id(asio_id) {
            Ok(h) => h,
            Err(_) => return vec![],
        };
        let device = match host.devices().ok().and_then(|devs| {
            devs.find(|d| {
                device_display_name(d)
                    .map(|n| n == device_name)
                    .unwrap_or(false)
            })
        }) {
            Some(d) => d,
            None => return vec![],
        };
        let max_ch = device
            .supported_output_configs()
            .map(|cfgs| cfgs.map(|c| c.channels()).max().unwrap_or(0))
            .unwrap_or(0);
        return (1..=max_ch).map(|i| format!("Channel {}", i)).collect();
    }
    #[cfg(not(feature = "asio"))]
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_host_lists_real_sinks() {
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
