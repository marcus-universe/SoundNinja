#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
pub mod audio;
use audio::{ get_out_devices, get_sound_duration, init_audio_thread, play_sound };
use tauri::menu::{ Menu, MenuItemBuilder, SubmenuBuilder };
use tauri::{ Emitter, Manager };

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let file_menu = SubmenuBuilder::new(app, "File")
                .item(&MenuItemBuilder::with_id("open_project", "Open Project").build(app)?)
                .item(&MenuItemBuilder::with_id("save", "Save").build(app)?)
                .item(&MenuItemBuilder::with_id("save_as", "Save As...").build(app)?)
                .separator()
                .item(&MenuItemBuilder::with_id("import_audio", "Import Audio Files").build(app)?)
                .item(&MenuItemBuilder::with_id("import_folders", "Import Folders").build(app)?)
                .separator()
                .item(&MenuItemBuilder::with_id("quit", "Quit").build(app)?)
                .build()?;

            let settings_menu = SubmenuBuilder::new(app, "Edit")
                .item(&MenuItemBuilder::with_id("open_settings", "Settings").build(app)?)
                .item(
                    &MenuItemBuilder::with_id("open_config", "Open Config File").build(app)?,
                )
                .build()?;

            let help_menu = SubmenuBuilder::new(app, "Help")
                .item(
                    &MenuItemBuilder::with_id("about", "About Sound Ninja").build(app)?,
                )
                .build()?;

            let menu = Menu::with_items(app, &[&file_menu, &settings_menu, &help_menu])?;
            app.set_menu(menu)?;

            app.on_menu_event(|app, event| {
                match event.id.as_ref() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "open_project" => {
                        app.emit("menu_open_project", ()).unwrap_or_default();
                    }
                    "save" => {
                        app.emit("menu_save", ()).unwrap_or_default();
                    }
                    "save_as" => {
                        app.emit("menu_save_as", ()).unwrap_or_default();
                    }
                    "import_audio" => {
                        app.emit("menu_import_audio", ()).unwrap_or_default();
                    }
                    "import_folders" => {
                        app.emit("menu_import_folders", ()).unwrap_or_default();
                    }
                    "open_settings" => {
                        app.emit("menu_open_settings", ()).unwrap_or_default();
                    }
                    "open_config" => {
                        use tauri_plugin_opener::OpenerExt;
                        if let Ok(app_data_dir) = app.path().app_data_dir() {
                            let config_path = app_data_dir.join("config.json");
                            let _ = app.opener().open_path(
                                config_path.to_string_lossy().to_string(),
                                None::<String>,
                            );
                        }
                    }
                    "about" => {
                        app.emit("menu_open_about", ()).unwrap_or_default();
                    }
                    _ => {}
                }
            });

            init_audio_thread(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_out_devices, play_sound, get_sound_duration])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
