pub mod audio;
pub mod fsx;
pub mod gifcache;
pub mod hotkeys;
pub mod httpx;
pub mod menu;
pub mod paths;
pub mod remote;
pub mod soundboard;
pub mod task;
pub mod toolwin;

#[tauri::command(async)]
fn get_system_fonts() -> Vec<String> {
    get_system_fonts_platform()
}

/// Strip registry / OS font-name suffixes down to a family.
#[cfg(any(target_os = "windows", test))]
pub(crate) fn strip_font_family_name(name: &str) -> String {
    const TYPE_SUFFIXES: &[&str] = &[
        " (TrueType)",
        " (OpenType)",
        " (TrueType/OpenType)",
        " (All Res)",
    ];
    const STYLE_SUFFIXES: &[&str] = &[
        " Bold Italic",
        " Bold",
        " Italic",
        " Regular",
        " Light",
        " Black",
        " Medium",
        " Thin",
        " ExtraBold",
        " ExtraLight",
        " SemiBold",
        " Semi Bold",
        " Extra Bold",
        " Extra Light",
        " Condensed",
        " Narrow",
        " Heavy",
        " Hairline",
        " Demi",
        " Book",
        " Display",
    ];
    let mut family = name.to_string();
    for suffix in TYPE_SUFFIXES {
        family = family.replace(suffix, "");
    }
    let mut trimmed = family.trim().to_string();
    for suffix in STYLE_SUFFIXES {
        if trimmed.ends_with(suffix) {
            trimmed = trimmed[..trimmed.len() - suffix.len()].trim().to_string();
            break;
        }
    }
    trimmed
}

#[cfg(target_os = "windows")]
fn get_system_fonts_platform() -> Vec<String> {
    use std::collections::BTreeSet;
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key_path = "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts";
    let fonts_key = match hklm.open_subkey(key_path) {
        Ok(k) => k,
        Err(_) => return vec![],
    };
    let mut families: BTreeSet<String> = BTreeSet::new();
    for item in fonts_key.enum_values().filter_map(|r| r.ok()) {
        let (name, _) = item;
        let trimmed = strip_font_family_name(&name);
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

/// Allow only http(s)/mailto/tel. Returns the trimmed URL or an error string.
pub(crate) fn allowed_external_url(url: &str) -> Result<&str, String> {
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
    Ok(trimmed)
}

/// Open http(s)/mailto/tel links in the system browser (works from secondary windows).
#[tauri::command]
fn open_external_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let trimmed = allowed_external_url(&url)?;
    app.opener()
        .open_url(trimmed, None::<&str>)
        .map_err(|e| e.to_string())
}

/// Release everything the app holds outside the webview: audio device streams,
/// decoded PCM, the remote server and OS-level hotkey registrations. Without
/// this the process can linger with a live output stream after the last window
/// is gone.
fn shutdown(app: &tauri::AppHandle) {
    let _ = audio::playback::stop_all();
    audio::record::abort_recording();
    let _ = audio::cache::clear_sound_cache();

    {
        use tauri_plugin_global_shortcut::GlobalShortcutExt;
        let _ = app.global_shortcut().unregister_all();
    }

    // The remote server owns a tokio task; give it a bounded moment to close
    // its listener so the port is free if the app is restarted right away.
    let _ = tauri::async_runtime::block_on(async {
        tokio::time::timeout(std::time::Duration::from_secs(2), remote::remote_stop()).await
    });
}

/// Desktop entry used by `main.rs`. Same plugins and commands as before.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
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
        .on_window_event(|window, event| {
            use tauri::{Manager, WindowEvent};
            // Only the main window owns the shared audio/network state; tool
            // windows come and go without touching it.
            if !matches!(event, WindowEvent::Destroyed) || window.label() != "main" {
                return;
            }
            shutdown(window.app_handle());
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
            audio::playback::set_asio_channel_map,
            audio::playback::set_output_volume,
            audio::playback::set_sound_volume,
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
            audio::dsp::is_sound_pumping,
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
            toolwin::open_tool_window,
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
            fsx::get_sound_file_meta,
            fsx::find_files_by_names,
            fsx::make_dir_abs,
            fsx::list_dir_files_abs,
            fsx::list_image_files_abs,
            fsx::copy_file_abs,
            fsx::copy_file_to_abs,
            fsx::delete_file_abs,
            fsx::delete_dir_abs,
            fsx::inspect_import_drop,
            httpx::download_url_bytes,
            httpx::http_get_text,
            gifcache::gif_cache_paths,
            gifcache::gif_cache_put,
            gifcache::gif_cache_clear,
            soundboard::make_temp_dir,
            soundboard::export_soundboard_zip,
            soundboard::import_soundboard_zip,
            hotkeys::set_global_sound_hotkeys,
            remote::remote_start,
            remote::remote_stop,
            remote::remote_status,
            remote::remote_publish_state,
            remote::get_local_ips
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{allowed_external_url, strip_font_family_name};

    #[test]
    fn allowed_external_url_rejects_empty() {
        assert!(allowed_external_url("   ").is_err());
    }

    #[test]
    fn allowed_external_url_rejects_file_scheme() {
        assert!(allowed_external_url("file:///etc/passwd").is_err());
    }

    #[test]
    fn allowed_external_url_accepts_https() {
        assert_eq!(
            allowed_external_url(" https://example.com ").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn strip_font_family_name_drops_truetype_and_bold() {
        assert_eq!(
            strip_font_family_name("Nunito Bold (TrueType)"),
            "Nunito"
        );
    }
}
