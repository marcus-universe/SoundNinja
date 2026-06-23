#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
pub mod audio;
pub mod menu;
use tauri::Manager;

#[tauri::command]
fn get_system_fonts() -> Vec<String> {
    get_system_fonts_platform()
}

#[cfg(target_os = "windows")]
fn get_system_fonts_platform() -> Vec<String> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;
    use std::collections::BTreeSet;
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key_path = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts";
    let fonts_key = match hklm.open_subkey(key_path) {
        Ok(k) => k,
        Err(_) => return vec![],
    };
    let type_suffixes = [
        " (TrueType)", " (OpenType)", " (TrueType/OpenType)", " (All Res)",
    ];
    let style_suffixes = [
        " Bold Italic", " Bold", " Italic", " Regular", " Light", " Black",
        " Medium", " Thin", " ExtraBold", " ExtraLight", " SemiBold",
        " Semi Bold", " Extra Bold", " Extra Light", " Condensed", " Narrow",
        " Heavy", " Hairline", " Demi", " Book", " Display",
    ];
    let mut families: BTreeSet<String> = BTreeSet::new();
    for item in fonts_key.enum_values().filter_map(|r| r.ok()) {
        let (name, _) = item;
        let mut family = name.clone();
        for s in &type_suffixes {
            family = family.replace(s, "");
        }
        let mut trimmed = family.trim().to_string();
        for s in &style_suffixes {
            if trimmed.ends_with(s) {
                trimmed = trimmed[..trimmed.len() - s.len()].trim().to_string();
                break;
            }
        }
        if !trimmed.is_empty() {
            families.insert(trimmed);
        }
    }
    families.into_iter().collect()
}

#[cfg(not(target_os = "windows"))]
fn get_system_fonts_platform() -> Vec<String> {
    vec![
        "Arial".to_string(),
        "Helvetica".to_string(),
        "sans-serif".to_string(),
        "serif".to_string(),
        "monospace".to_string(),
    ]
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            menu::setup(app)?;
            audio::init_audio_thread(app.handle().clone());
            let data_dir = app.path().app_data_dir().expect("cannot resolve app data dir");
            std::fs::create_dir_all(data_dir.join("themes")).ok();
            std::fs::create_dir_all(data_dir.join("projects")).ok();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            audio::devices::get_out_devices,
            audio::playback::play_sound,
            audio::playback::get_sound_duration,
            audio::cache::warm_sound_cache,
            audio::cache::clear_sound_cache,
            audio::cache::get_cache_stats,
            audio::cache::set_cache_config,
            get_system_fonts,
            menu::rebuild_menu
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
