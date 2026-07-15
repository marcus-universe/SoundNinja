use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::cpal;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn device_display_name(d: &cpal::Device) -> Option<String> {
    d.description().ok().map(|desc| {
        desc.extended()
            .first()
            .cloned()
            .unwrap_or_else(|| desc.name().to_string())
    })
}

/// Returns output devices, optionally restricted to one named host.
/// `host_name = None` → merges all available hosts (backward-compat).
pub fn get_output_devices_for_host_name(host_name: Option<&str>) -> Result<Vec<cpal::Device>> {
    match host_name {
        Some(name) => {
            let host_id = cpal::available_hosts()
                .into_iter()
                .find(|id| id.name() == name)
                .ok_or_else(|| anyhow::anyhow!("Host '{}' not found", name))?;
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

/// Alias for use by playback.rs (all hosts merged).
pub fn get_output_devices() -> Result<Vec<cpal::Device>> {
    get_output_devices_for_host_name(None)
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
