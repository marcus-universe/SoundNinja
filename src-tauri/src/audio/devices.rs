use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::cpal;
use serde::Serialize;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn device_display_name(d: &cpal::Device) -> Option<String> {
    d.description().ok().map(|desc| {
        desc.extended()
            .first()
            .cloned()
            .unwrap_or_else(|| desc.name().to_string())
    })
}

/// cpal's `HostId::name()` is the enum variant (`"Wasapi"`), while the UI historically
/// stores `"WASAPI"`. Treat those as equal (case-insensitive).
fn host_name_matches(id_name: &str, wanted: &str) -> bool {
    id_name.eq_ignore_ascii_case(wanted)
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
/// (e.g. "WASAPI"; "ASIO" only when built with `--features asio`).
#[tauri::command]
pub fn get_audio_hosts() -> Vec<String> {
    cpal::available_hosts()
        .into_iter()
        .map(|id| id.name().to_string())
        .collect()
}

/// Returns output-device names across all hosts (backward-compat: no params).
#[tauri::command]
pub fn get_out_devices() -> Vec<String> {
    get_output_devices_for_host_name(None)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|d| device_display_name(&d))
        .inspect(|n| println!("Device: {}", n))
        .collect()
}

/// Returns output-device names for a specific driver/host.
#[tauri::command]
pub fn get_out_devices_host(host: String) -> Vec<String> {
    get_output_devices_for_host_name(Some(&host))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|d| device_display_name(&d))
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

/// Returns capture sources: real mics + WASAPI loopback (PC Audio) outputs.
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
