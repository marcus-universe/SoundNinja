use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait};
use rodio::{self, cpal};
use std::fs::File;
use std::io::BufReader;

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


    #[tauri::command]
    pub fn play_sound(sound_path: &str, device_name: &str) -> Result<String, String> {
        let devices = match get_output_devices() {
            Ok(devices) => devices,
            Err(err) => return Err(err.to_string()),
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

let name = device.name().unwrap_or("Unknown".to_string());
            println!("{}", name);
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_handle).unwrap();

    //     // Open the sound file and create a BufReader
        let file = File::open(sound_path).map_err(|err| err.to_string())?;
        let reader = BufReader::new(file);

    //     // Load the sound file and play it on the sink
        let source = rodio::Decoder::new(reader).map_err(|err| err.to_string())?;
        sink.append(source);
        sink.sleep_until_end();

        Ok("".to_string())
    }



// #[tauri::command]
// pub fn play_sound(sound_path: &str, device_name: &str) -> Result<String, String> {
//     let devices = match get_output_devices() {
//         Ok(devices) => devices,
//         Err(err) => return Err(err.to_string()),
//     };

//     let device = devices
//         .iter()
//         .find(|device| {
//             device
//                 .name()
//                 .map(|name| name == device_name)
//                 .unwrap_or(false)
//         })
//         .unwrap();

//     // Create a Rodio sink for the chosen device
//     let stream = rodio::OutputStream::new(device);
//     let sink = rodio::Sink::new(stream);

//     // Open the sound file and create a BufReader
//     let file = File::open(sound_path).map_err(|err| err.to_string())?;
//     let reader = BufReader::new(file);

//     // Load the sound file and play it on the sink
//     let source = rodio::Decoder::new(reader).map_err(|err| err.to_string())?;
//     sink.append(source);
//     sink.sleep_until_end();

//     Ok("".to_string())
// }




// #[tauri::command]
// pub fn play_sound(sound_path: &str, device_name: &str) -> Result<String, String> {
//     let devices = match get_output_devices() {
//         Ok(devices) => devices,
//         Err(err) => return Err(err.to_string()),
//     };

//     let device = devices
//         .iter()
//         .find(|device| {
//             device
//                 .name()
//                 .map(|name| name == device_name)
//                 .unwrap_or(false)
//         })
//         .ok_or_else(|| {
//             std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!("Device with name {} not found", device_name),
//             )
//         });

//     // Create a Rodio sink for the chosen device
//     let stream = cpal::OutputStream::new(device);
//     let sink = rodio::Sink::new(stream);

//     // Open the sound file and create a BufReader
//     let file = File::open(sound_path);
//     let reader = BufReader::new(file)?;

//     // Load the sound file and play it on the sink
//     let source = rodio::Decoder::new(reader);
//     sink.append(source);
//     sink.sleep_until_end();

//     Ok("".to_string())
// }

// pub fn play_sound(sound_path: &str, device_name: &str) -> Result<String, String> {
//     let devices = get_output_devices().unwrap_or_else(|_| Vec::new());
//     let device = devices
//         .iter()
//         .find(|device| device.name() == device_name)
//         .ok_or_else(|| {
//             std::io::Error::new(
//                 std::io::ErrorKind::Other,
//                 format!("Device with id {} not found", device_name),
//             )
//         })?;

//     // Create a Rodio sink for the chosen device
//     let sink = rodio::Sink::new_output_from_device(device)?;

//     // Open the sound file and create a BufReader
//     let file = File::open(sound_path)?;
//     let reader = BufReader::new(file);

//     // Load the sound file and play it on the sink
//     let source = rodio::Decoder::new(reader)?;
//     sink.append(source);
//     sink.sleep_until_end();

//     Ok("".to_string())
// }
