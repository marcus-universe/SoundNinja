#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
pub mod audio;
pub mod gpu;
pub mod menu;
pub mod paths;
pub mod fsx;
pub mod httpx;
pub mod soundboard;
pub mod hotkeys;

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

/// Open http(s)/mailto/tel links in the system browser (works from secondary windows).
#[tauri::command]
fn open_external_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("Empty URL".into());
    }
    let lower = trimmed.to_lowercase();
    if !(lower.starts_with("https://")
        || lower.starts_with("http://")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:"))
    {
        return Err("URL scheme not allowed".into());
    }
    app.opener()
        .open_url(trimmed, None::<&str>)
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // On macOS keep native decorations (traffic lights) — the custom
            // HTML title bar in TitleBar.vue only renders on Windows/Linux.
            #[cfg(target_os = "macos")]
            {
                use tauri::Manager;
                if let Some(win) = app.get_webview_window("main") {
                    let _ = win.set_decorations(true);
                }
            }
            menu::setup(app)?;
            audio::init_audio_thread(app.handle().clone());
            let base = paths::default_base_dir(app.handle());
            paths::ensure_default_dirs(&base);
            // Allow plugin-fs access to the portable-first data folders.
            use tauri_plugin_fs::FsExt;
            let scope = app.fs_scope();
            let _ = scope.allow_directory(base.join("projects"), true);
            let _ = scope.allow_directory(base.join("themes"), true);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            audio::devices::get_out_devices,
            audio::devices::get_audio_hosts,
            audio::devices::get_out_devices_host,
            audio::devices::get_asio_device_channels,
            audio::devices::get_in_devices,
            audio::devices::get_in_devices_host,
            audio::devices::get_loopback_devices,
            audio::playback::play_sound,
            audio::playback::get_sound_duration,
            audio::playback::pause_sound,
            audio::playback::resume_sound,
            audio::playback::pause_all,
            audio::playback::resume_all,
            audio::playback::stop_all,
            audio::playback::get_playing_sounds,
            audio::playback::seek_playing,
            audio::playback::set_playing_loop,
            audio::playback::set_output_volume,
            audio::cache::warm_sound_cache,
            audio::cache::clear_sound_cache,
            audio::cache::get_cache_stats,
            audio::cache::set_cache_config,
            audio::record::start_recording,
            audio::record::stop_recording,
            audio::record::get_live_record_peaks,
            audio::record::get_record_level,
            audio::record::is_recording,
            audio::record::set_input_volume,
            audio::record::get_input_volume,
            audio::dsp::load_edit_session,
            audio::dsp::get_waveform_peaks,
            audio::dsp::get_file_waveform_peaks,
            audio::dsp::trim_session,
            audio::dsp::delete_range,
            audio::dsp::normalize_session,
            audio::dsp::denoise_session,
            audio::dsp::undo_session,
            audio::dsp::redo_session,
            audio::dsp::export_session,
            audio::dsp::stage_session_clip,
            audio::dsp::preview_session,
            audio::dsp::stop_preview,
            audio::dsp::pause_preview,
            audio::dsp::resume_preview,
            audio::stems::get_stems_status,
            audio::stems::ensure_stems_model,
            audio::stems::cancel_stems_model_download,
            audio::stems::dismiss_stems_intent,
            audio::stems::split_session,
            audio::stems::stems_busy,
            get_system_fonts,
            open_external_url,
            menu::rebuild_menu,
            menu::set_recent_projects,
            menu::set_window_chrome,
            menu::strip_window_menu,
            menu::strip_window_menu_for,
            paths::get_default_paths,
            paths::read_install_language,
            paths::relocate_data,
            paths::list_projects,
            fsx::read_text_file_abs,
            fsx::read_file_base64_abs,
            fsx::write_text_file_abs,
            fsx::path_exists_abs,
            fsx::paths_exist_abs,
            fsx::find_files_by_names,
            fsx::make_dir_abs,
            fsx::list_dir_files_abs,
            fsx::copy_file_abs,
            fsx::copy_file_to_abs,
            fsx::delete_file_abs,
            fsx::delete_dir_abs,
            fsx::collect_audio_buckets_abs,
            httpx::download_url_bytes,
            httpx::http_get_text,
            gpu::has_dedicated_gpu,
            gpu::set_gpu_audio,
            gpu::get_gpu_audio_enabled,
            soundboard::make_temp_dir,
            soundboard::export_soundboard_zip,
            soundboard::import_soundboard_zip,
            hotkeys::set_global_sound_hotkeys
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
