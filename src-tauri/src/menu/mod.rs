use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItemBuilder, SubmenuBuilder};
use tauri::{Emitter, Manager};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
    pub path: String,
    pub name: String,
}

/// Global menu state: current language + the recent-projects list. Kept here so
/// both `rebuild_menu` and `set_recent_projects` can rebuild the whole menu
/// without the frontend having to re-send everything each time.
struct MenuState {
    lang: String,
    recents: Vec<RecentProject>,
    /// When false (styled/hidden chrome on Win/Linux), skip installing the native menu.
    native_menu_enabled: bool,
}

static MENU_STATE: Mutex<MenuState> = Mutex::new(MenuState {
    lang: String::new(),
    recents: Vec::new(),
    // Default matches tauri.conf decorations:false + custom HTML titlebar.
    native_menu_enabled: false,
});

/// Installs or removes the app menu based on `MENU_STATE.native_menu_enabled`.
fn apply_menu_from_state(app: &tauri::AppHandle) -> Result<(), String> {
    let (lang, recents, enabled) = {
        let state = MENU_STATE.lock().unwrap();
        (
            if state.lang.is_empty() {
                "en".to_string()
            } else {
                state.lang.clone()
            },
            state.recents.clone(),
            state.native_menu_enabled,
        )
    };

    #[cfg(target_os = "macos")]
    {
        let _ = enabled; // macOS always keeps the menu bar
        let menu = build_menu(app, &lang, &recents).map_err(|e| e.to_string())?;
        app.set_menu(menu).map_err(|e| e.to_string())?;
        let _ = app.show_menu();
        return Ok(());
    }

    #[cfg(not(target_os = "macos"))]
    {
        if enabled {
            let menu = build_menu(app, &lang, &recents).map_err(|e| e.to_string())?;
            app.set_menu(menu).map_err(|e| e.to_string())?;
            let _ = app.show_menu();
        } else {
            let _ = app.hide_menu();
            let _ = app.remove_menu();
        }
        // Tool windows never show the main File/Edit/Help bar.
        strip_secondary_window_menus(app);
        Ok(())
    }
}

struct MenuLabels {
    file: &'static str,
    new_project: &'static str,
    open_project: &'static str,
    open_recent: &'static str,
    no_recent: &'static str,
    select_project: &'static str,
    save: &'static str,
    save_as: &'static str,
    import_audio: &'static str,
    import_folders: &'static str,
    quit: &'static str,
    edit: &'static str,
    undo: &'static str,
    redo: &'static str,
    settings: &'static str,
    open_themes_folder: &'static str,
    open_projects_folder: &'static str,
    help: &'static str,
    check_updates: &'static str,
    about: &'static str,
}

fn labels_for(lang: &str) -> MenuLabels {
    match lang {
        "de" => MenuLabels {
            file: "Datei",
            new_project: "Neues Projekt",
            open_project: "Projekt öffnen",
            open_recent: "Zuletzt geöffnet",
            no_recent: "Keine kürzlichen Projekte",
            select_project: "Projekt auswählen",
            save: "Speichern",
            save_as: "Speichern unter…",
            import_audio: "Audiodateien importieren",
            import_folders: "Ordner importieren",
            quit: "Beenden",
            edit: "Bearbeiten",
            undo: "Rückgängig",
            redo: "Wiederholen",
            settings: "Einstellungen",
            open_themes_folder: "Themes-Ordner öffnen",
            open_projects_folder: "Projekte-Ordner öffnen",
            help: "Hilfe",
            check_updates: "Nach Updates suchen",
            about: "Über SoundNinja",
        },
        _ => MenuLabels {
            file: "File",
            new_project: "New Project",
            open_project: "Open Project",
            open_recent: "Open Recent",
            no_recent: "No recent projects",
            select_project: "Select Project",
            save: "Save",
            save_as: "Save As...",
            import_audio: "Import Audio Files",
            import_folders: "Import Folders",
            quit: "Quit",
            edit: "Edit",
            undo: "Undo",
            redo: "Redo",
            settings: "Settings",
            open_themes_folder: "Open Themes Folder",
            open_projects_folder: "Open Projects Folder",
            help: "Help",
            check_updates: "Check for Updates",
            about: "About",
        },
    }
}

fn build_menu(
    app: &tauri::AppHandle,
    lang: &str,
    recents: &[RecentProject],
) -> tauri::Result<Menu<tauri::Wry>> {
    let l = labels_for(lang);

    let mut recent_submenu = SubmenuBuilder::new(app, l.open_recent);
    if recents.is_empty() {
        let empty = MenuItemBuilder::with_id("recent_none", l.no_recent)
            .enabled(false)
            .build(app)?;
        recent_submenu = recent_submenu.item(&empty);
    } else {
        for (i, r) in recents.iter().enumerate() {
            let item = MenuItemBuilder::with_id(format!("recent_{i}"), &r.name).build(app)?;
            recent_submenu = recent_submenu.item(&item);
        }
    }
    let recent_menu = recent_submenu.build()?;

    let file_menu = SubmenuBuilder::new(app, l.file)
        .item(&MenuItemBuilder::with_id("new_project", l.new_project).build(app)?)
        .item(&MenuItemBuilder::with_id("open_project", l.open_project).build(app)?)
        .item(&recent_menu)
        .item(&MenuItemBuilder::with_id("select_project", l.select_project).build(app)?)
        .item(&MenuItemBuilder::with_id("save", l.save).accelerator("CmdOrCtrl+S").build(app)?)
        .item(&MenuItemBuilder::with_id("save_as", l.save_as).accelerator("CmdOrCtrl+Shift+S").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("import_audio", l.import_audio).build(app)?)
        .item(&MenuItemBuilder::with_id("import_folders", l.import_folders).build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("quit", l.quit).build(app)?)
        .build()?;

    let settings_menu = SubmenuBuilder::new(app, l.edit)
        .item(
            &MenuItemBuilder::with_id("undo", l.undo)
                .accelerator("CmdOrCtrl+Z")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("redo", l.redo)
                .accelerator("CmdOrCtrl+Shift+Z")
                .build(app)?,
        )
        .separator()
        .item(&MenuItemBuilder::with_id("open_settings", l.settings).build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("open_themes_folder", l.open_themes_folder).build(app)?)
        .item(&MenuItemBuilder::with_id("open_projects_folder", l.open_projects_folder).build(app)?)
        .build()?;

    let help_menu = SubmenuBuilder::new(app, l.help)
        .item(&MenuItemBuilder::with_id("check_updates", l.check_updates).build(app)?)
        .item(&MenuItemBuilder::with_id("about", l.about).build(app)?)
        .build()?;

    Menu::with_items(app, &[&file_menu, &settings_menu, &help_menu])
}

pub fn setup(app: &mut tauri::App) -> tauri::Result<()> {
    {
        let mut state = MENU_STATE.lock().unwrap();
        state.lang = "en".to_string();
        // Styled custom titlebar is the default on Win/Linux; macOS keeps native menu.
        #[cfg(target_os = "macos")]
        {
            state.native_menu_enabled = true;
        }
        #[cfg(not(target_os = "macos"))]
        {
            state.native_menu_enabled = false;
        }
    }
    // Install native menu only when enabled (macOS always; Win/Linux after user opts in).
    let _ = apply_menu_from_state(app.handle());

    app.on_menu_event(|app, event| {
        let id = event.id.as_ref();
        if let Some(rest) = id.strip_prefix("recent_") {
            if let Ok(idx) = rest.parse::<usize>() {
                let path = {
                    let state = MENU_STATE.lock().unwrap();
                    state.recents.get(idx).map(|r| r.path.clone())
                };
                if let Some(path) = path {
                    app.emit("menu_open_recent", path).unwrap_or_default();
                }
            }
            return;
        }
        match id {
            "quit" => {
                std::process::exit(0);
            }
            "new_project" => {
                app.emit("menu_new_project", ()).unwrap_or_default();
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
            "undo" => {
                app.emit("menu_undo", ()).unwrap_or_default();
            }
            "redo" => {
                app.emit("menu_redo", ()).unwrap_or_default();
            }
            "open_settings" => {
                app.emit("menu_open_settings", ()).unwrap_or_default();
            }
            "open_themes_folder" => {
                use tauri_plugin_opener::OpenerExt;
                let base = crate::paths::default_base_dir(app);
                let themes_path = base.join("themes");
                std::fs::create_dir_all(&themes_path).ok();
                let _ = app
                    .opener()
                    .open_path(themes_path.to_string_lossy().to_string(), None::<String>);
            }
            "open_projects_folder" => {
                use tauri_plugin_opener::OpenerExt;
                let base = crate::paths::default_base_dir(app);
                let projects_path = base.join("projects");
                std::fs::create_dir_all(&projects_path).ok();
                let _ = app
                    .opener()
                    .open_path(projects_path.to_string_lossy().to_string(), None::<String>);
            }
            "about" => {
                app.emit("menu_open_about", ()).unwrap_or_default();
            }
            "check_updates" => {
                app.emit("menu_check_updates", ()).unwrap_or_default();
            }
            _ => {}
        }
    });

    Ok(())
}

#[tauri::command]
pub fn rebuild_menu(app: tauri::AppHandle, lang: String) -> Result<(), String> {
    {
        let mut state = MENU_STATE.lock().unwrap();
        state.lang = lang;
    }
    apply_menu_from_state(&app)
}

/// Replaces the recent-projects list and rebuilds the menu, keeping the
/// currently active language.
#[tauri::command]
pub fn set_recent_projects(
    app: tauri::AppHandle,
    recents: Vec<RecentProject>,
) -> Result<(), String> {
    {
        let mut state = MENU_STATE.lock().unwrap();
        state.recents = recents;
        if state.lang.is_empty() {
            state.lang = "en".to_string();
        }
    }
    apply_menu_from_state(&app)
}

/// Applies window chrome: native OS decorations + menu vs undecorated custom UI.
///
/// - `native_chrome`: use the OS title bar and native app menu
/// - `hidden`: hide title bar entirely (no decorations, no menu on Win/Linux)
///
/// Applied to every window (main + secondary). Secondary windows keep an empty
/// explicit menu so File/Edit/Help from the app menu do not appear there.
///
/// macOS always keeps its menu bar (platform convention).
#[tauri::command]
pub fn set_window_chrome(
    app: tauri::AppHandle,
    native_chrome: bool,
    hidden: bool,
) -> Result<(), String> {
    let use_native = native_chrome && !hidden;

    // macOS keeps traffic-light decorations; elsewhere follow the setting.
    #[cfg(target_os = "macos")]
    {
        let _ = use_native;
        for (_label, win) in app.webview_windows() {
            let _ = win.set_decorations(true);
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        for (_label, win) in app.webview_windows() {
            win.set_decorations(use_native).map_err(|e| e.to_string())?;
        }
    }

    {
        let mut state = MENU_STATE.lock().unwrap();
        #[cfg(target_os = "macos")]
        {
            state.native_menu_enabled = true;
            let _ = use_native;
        }
        #[cfg(not(target_os = "macos"))]
        {
            state.native_menu_enabled = use_native;
        }
    }

    apply_menu_from_state(&app)
}

const SECONDARY_WINDOW_LABELS: &[&str] = &["record-editor", "theme-creator", "playing-list"];

fn strip_secondary_window_menus(app: &tauri::AppHandle) {
    for label in SECONDARY_WINDOW_LABELS {
        if let Some(win) = app.get_webview_window(label) {
            let _ = strip_menu_on(app, &win);
        }
    }
}

/// Give this window an empty menu so File/Edit/Help from the app menu do not show.
#[tauri::command]
pub fn strip_window_menu(app: tauri::AppHandle, window: tauri::WebviewWindow) -> Result<(), String> {
    strip_menu_on(&app, &window)
}

/// Strip menu on a window by label (call before show to avoid layout jitter).
#[tauri::command]
pub fn strip_window_menu_for(app: tauri::AppHandle, label: String) -> Result<(), String> {
    use tauri::Manager;
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| format!("Window '{label}' not found"))?;
    strip_menu_on(&app, &window)
}

fn strip_menu_on(app: &tauri::AppHandle, window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(not(target_os = "macos"))]
    {
        // Important: assign an *explicit* empty menu. `remove_menu` alone leaves the
        // window eligible for the app-wide File/Edit/Help menu on the next
        // `app.set_menu` / `show_menu`, which is why secondary windows kept
        // showing the main menu after open.
        let empty = Menu::new(app).map_err(|e| e.to_string())?;
        window.set_menu(empty).map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        let _ = (app, window);
    }
    Ok(())
}
