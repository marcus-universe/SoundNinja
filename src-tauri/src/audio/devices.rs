use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::cpal;

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
                desc.extended()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| desc.name().to_string())
            })
        })
        .inspect(|n| println!("Device: {}", n))
        .collect()
}
