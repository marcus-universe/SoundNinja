#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::cpal;

pub fn get_output_devices() -> Result<Vec<cpal::Device>> {
    let mut devices = Vec::new();

    for host_id in cpal::available_hosts() {
        let host = cpal::host_from_id(host_id)?;

        devices.extend(
            host.devices()?
                .filter(|device| device.default_output_config().is_ok()),
        );
    }

    Ok(devices)
}

#[tauri::command]
fn get_out_devices() -> Vec<String> {
    let devices = get_output_devices().unwrap_or_else(|_| Vec::new());

    // Create a mutable `Vec<String>` variable called `output` and initialize
    // it to an empty vector
    let mut output = Vec::new();

    for device in devices {
        let name = device.name().unwrap();
        println!("Device: {:?}", device.name().unwrap());
        // Append the device name to the `output` vector
        output.push(name);
    }

    // Return the `output` vector
    output
}

// fn get_out_devices() -> String {
//     let devices = get_output_devices().unwrap_or_else(|_| Vec::new());

//     // Create a mutable `String` variable called `output` and initialize
//     // it to an empty string
//     let mut output = String::new();

//     for device in devices {
//         let name = device.name().unwrap();
//         println!("Device: {:?}", device.name().unwrap());
//         // Append the device name to the `output` string, followed by a newline character
//         output.push_str(&name);
//         output.push('\n');
//     }

//     // Return the `output` string
//     output.into()
// }
// fn getOutputDevices() -> String {
//     let devices = get_output_devices().unwrap_or_else(|_| Vec::new());
//     for device in devices {
//         println!("Device: {:?}", device.name().unwrap());
//         // return ("{:?}", device.name().unwrap());
//     }
// }

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_out_devices])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
