#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
pub mod audio;
use audio::{get_out_devices, play_sound};

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_out_devices, play_sound])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
