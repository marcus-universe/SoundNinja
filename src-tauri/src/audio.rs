use anyhow::Result;
use cpal::traits::{ DeviceTrait, HostTrait };
use rodio::{ self, cpal };
use std::fs::File;
use std::io::BufReader;

pub fn get_output_devices() -> Result<Vec<cpal::Device>> {
    let mut devices = Vec::new();

    for host_id in cpal::available_hosts() {
        let host = cpal::host_from_id(host_id)?;

        devices.extend(host.devices()?.filter(|device| device.default_output_config().is_ok()));
    }

    Ok(devices)
}

#[tauri::command]
pub fn get_out_devices() -> Vec<String> {
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

pub static mut SINK: Option<rodio::Sink> = None;

#[tauri::command]
pub async fn play_sound(
    sound_path: &str,
    device_name: &str,
    active: bool
) -> Result<String, String> {
    let devices: Vec<rodio::Device> = match get_output_devices() {
        Ok(devices) => devices,
        Err(err) => {
            return Err(err.to_string());
        }
    };
    let device = devices
        .iter()
        .find(|device| {
            device
                .name()
                .map(|name| name == device_name)
                .unwrap_or(false)
        })
        .unwrap();
    let _name = device.name().unwrap_or("Unknown".to_string());

    let (_stream, stream_handle) = rodio::OutputStream::try_from_device(&device).unwrap();
    unsafe {
        // unsichere Operationen ausführen
        SINK = Some(rodio::Sink::try_new(&stream_handle).unwrap());
    }

    let file = File::open(sound_path).map_err(|err| err.to_string())?;
    let reader = BufReader::new(file);
    let source = rodio::Decoder::new(reader).map_err(|err| err.to_string())?;

    if active == false {
        unsafe {
            if let Some(sink) = &mut SINK {
                // die SINK-Variable enthält einen Wert
                sink.append(source);
                sink.sleep_until_end();
                // sink.clear();
            } else {
                // die SINK-Variable enthält keinen Wert
                return Err("SINK not initialized".to_string());
            }
        }
    } else {
        unsafe {
            if let Some(sink) = &mut SINK {
                // die SINK-Variable enthält einen Wert
                sink.stop();
            } else {
                // die SINK-Variable enthält keinen Wert
                return Err("SINK not initialized".to_string());
            }
        }
        // sinks.push(rodio::Sink::try_new(&stream_handle).unwrap());
    }
    Ok("".to_string())
}