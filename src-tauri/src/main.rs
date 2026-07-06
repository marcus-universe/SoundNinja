#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
pub mod audio;
pub mod menu;
pub mod paths;
pub mod fsx;

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
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            menu::setup(app)?;
            audio::init_audio_thread(app.handle().clone());
            let settings = paths::load_settings(app.handle());
            paths::ensure_dirs(&settings);
            // Allow plugin-fs access to the (possibly relocated) data folders.
            use tauri_plugin_fs::FsExt;
            let scope = app.fs_scope();
            let _ = scope.allow_directory(&settings.projects_path, true);
            let _ = scope.allow_directory(&settings.themes_path, true);
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
            audio::playback::set_output_volume,
            get_system_fonts,
            menu::rebuild_menu,
            paths::get_app_settings,
            paths::set_app_settings,
            paths::relocate_data,
            paths::list_projects,
            fsx::read_text_file_abs,
            fsx::read_file_base64_abs,
            fsx::write_text_file_abs,
            fsx::path_exists_abs,
            fsx::make_dir_abs,
            fsx::list_dir_files_abs,
            fsx::copy_file_abs,
            fsx::delete_file_abs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
