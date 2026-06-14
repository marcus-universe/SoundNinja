#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]
pub mod audio;
use audio::{ get_out_devices, get_sound_duration, init_audio_thread, play_sound };
use tauri::{ CustomMenuItem, Manager, Menu, MenuItem, Submenu };

fn main() {
    // File menu
    let open_project = CustomMenuItem::new("open_project", "Open Project");
    let save = CustomMenuItem::new("save", "Save");
    let save_as = CustomMenuItem::new("save_as", "Save As...");
    let import_audio = CustomMenuItem::new("import_audio", "Import Audio Files");
    let import_folders = CustomMenuItem::new("import_folders", "Import Folders");
    let quit = CustomMenuItem::new("quit", "Quit");
    let file_menu = Submenu::new(
        "File",
        Menu::new()
            .add_item(open_project)
            .add_item(save)
            .add_item(save_as)
            .add_native_item(MenuItem::Separator)
            .add_item(import_audio)
            .add_item(import_folders)
            .add_native_item(MenuItem::Separator)
            .add_item(quit),
    );

    // Settings menu
    let open_settings = CustomMenuItem::new("open_settings", "Settings");
    let open_config = CustomMenuItem::new("open_config", "Open Config File");
    let settings_menu = Submenu::new(
        "Edit",
        Menu::new().add_item(open_settings).add_item(open_config),
    );

    // Help menu
    let about = CustomMenuItem::new("about", "About Sound Ninja");
    let help_menu = Submenu::new("Help", Menu::new().add_item(about));

    let menu = Menu::new()
        .add_submenu(file_menu)
        .add_submenu(settings_menu)
        .add_submenu(help_menu);

    tauri::Builder
        ::default()
        .menu(menu)
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "quit" => {
                    std::process::exit(0);
                }
                "open_project" => {
                    event.window().emit("menu_open_project", ()).unwrap_or_default();
                }
                "save" => {
                    event.window().emit("menu_save", ()).unwrap_or_default();
                }
                "save_as" => {
                    event.window().emit("menu_save_as", ()).unwrap_or_default();
                }
                "import_audio" => {
                    event.window().emit("menu_import_audio", ()).unwrap_or_default();
                }
                "import_folders" => {
                    event.window().emit("menu_import_folders", ()).unwrap_or_default();
                }
                "open_settings" => {
                    event.window().emit("menu_open_settings", ()).unwrap_or_default();
                }
                "open_config" => {
                    let config = event.window().app_handle().config();
                    if let Some(app_dir) = tauri::api::path::app_data_dir(&config) {
                        let config_path = app_dir.join("config.json");
                        tauri::api::shell::open(
                            &event.window().shell_scope(),
                            config_path.to_string_lossy().to_string(),
                            None,
                        )
                        .unwrap_or_default();
                    }
                }
                "about" => {
                    event.window().emit("menu_open_about", ()).unwrap_or_default();
                }
                _ => {}
            }
        })
        .setup(|app| {
            init_audio_thread(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_out_devices, play_sound, get_sound_duration])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}