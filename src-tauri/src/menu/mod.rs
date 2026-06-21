use tauri::menu::{Menu, MenuItemBuilder, SubmenuBuilder};
use tauri::Emitter;
use tauri::Manager;

struct MenuLabels {
    file: &'static str,
    open_project: &'static str,
    select_project: &'static str,
    save: &'static str,
    save_as: &'static str,
    import_audio: &'static str,
    import_folders: &'static str,
    quit: &'static str,
    edit: &'static str,
    settings: &'static str,
    open_config: &'static str,
    open_themes_folder: &'static str,
    open_projects_folder: &'static str,
    help: &'static str,
    about: &'static str,
}

fn labels_for(lang: &str) -> MenuLabels {
    match lang {
        "de" => MenuLabels {
            file: "Datei",
            open_project: "Projekt öffnen",
            select_project: "Projekt auswählen",
            save: "Speichern",
            save_as: "Speichern unter…",
            import_audio: "Audiodateien importieren",
            import_folders: "Ordner importieren",
            quit: "Beenden",
            edit: "Bearbeiten",
            settings: "Einstellungen",
            open_config: "Konfigurationsdatei öffnen",
            open_themes_folder: "Themes-Ordner öffnen",
            open_projects_folder: "Projekte-Ordner öffnen",
            help: "Hilfe",
            about: "Über SoundNinja",
        },
        _ => MenuLabels {
            file: "File",
            open_project: "Open Project",
            select_project: "Select Project",
            save: "Save",
            save_as: "Save As...",
            import_audio: "Import Audio Files",
            import_folders: "Import Folders",
            quit: "Quit",
            edit: "Edit",
            settings: "Settings",
            open_config: "Open Config File",
            open_themes_folder: "Open Themes Folder",
            open_projects_folder: "Open Projects Folder",
            help: "Help",
            about: "About",
        },
    }
}

fn build_menu(app: &tauri::AppHandle, lang: &str) -> tauri::Result<Menu<tauri::Wry>> {
    let l = labels_for(lang);

    let file_menu = SubmenuBuilder::new(app, l.file)
        .item(&MenuItemBuilder::with_id("open_project", l.open_project).build(app)?)
        .item(&MenuItemBuilder::with_id("select_project", l.select_project).build(app)?)
        .item(&MenuItemBuilder::with_id("save", l.save).build(app)?)
        .item(&MenuItemBuilder::with_id("save_as", l.save_as).build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("import_audio", l.import_audio).build(app)?)
        .item(&MenuItemBuilder::with_id("import_folders", l.import_folders).build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("quit", l.quit).build(app)?)
        .build()?;

    let settings_menu = SubmenuBuilder::new(app, l.edit)
        .item(&MenuItemBuilder::with_id("open_settings", l.settings).build(app)?)
        .item(&MenuItemBuilder::with_id("open_config", l.open_config).build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("open_themes_folder", l.open_themes_folder).build(app)?)
        .item(&MenuItemBuilder::with_id("open_projects_folder", l.open_projects_folder).build(app)?)
        .build()?;

    let help_menu = SubmenuBuilder::new(app, l.help)
        .item(&MenuItemBuilder::with_id("about", l.about).build(app)?)
        .build()?;

    Menu::with_items(app, &[&file_menu, &settings_menu, &help_menu])
}

pub fn setup(app: &mut tauri::App) -> tauri::Result<()> {
    let menu = build_menu(app.handle(), "en")?;
    app.set_menu(menu)?;

    app.on_menu_event(|app, event| {
        match event.id.as_ref() {
            "quit" => {
                std::process::exit(0);
            }
            "open_project" => {
                app.emit("menu_open_project", ()).unwrap_or_default();
            }
            "select_project" => {
                app.emit("menu_select_project", ()).unwrap_or_default();
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
            "open_themes_folder" => {
                use tauri_plugin_opener::OpenerExt;
                if let Ok(app_data_dir) = app.path().app_data_dir() {
                    let themes_path = app_data_dir.join("themes");
                    std::fs::create_dir_all(&themes_path).ok();
                    let _ = app.opener().open_path(
                        themes_path.to_string_lossy().to_string(),
                        None::<String>,
                    );
                }
            }
            "open_projects_folder" => {
                use tauri_plugin_opener::OpenerExt;
                if let Ok(app_data_dir) = app.path().app_data_dir() {
                    let projects_path = app_data_dir.join("projects");
                    std::fs::create_dir_all(&projects_path).ok();
                    let _ = app.opener().open_path(
                        projects_path.to_string_lossy().to_string(),
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

    Ok(())
}

#[tauri::command]
pub fn rebuild_menu(app: tauri::AppHandle, lang: String) -> Result<(), String> {
    let menu = build_menu(&app, &lang).map_err(|e| e.to_string())?;
    app.set_menu(menu).map_err(|e| e.to_string())?;
    Ok(())
}
