//! Data-location management.
//!
//! Sound Ninja stores its Projects and Themes folders "portable-first":
//! next to the executable when that directory is writable (portable / dev
//! builds), otherwise it falls back to the OS app-data directory (installed
//! builds where the program directory is read-only).
//!
//! App-level preferences (the configurable Projects/Themes paths, the navbar
//! side, the last opened project) always live in a fixed, guaranteed-writable
//! `app-settings.json` inside the app-data directory — the app must always be
//! able to find these even after the user relocates the data folders.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub projects_path: String,
    pub themes_path: String,
    /// "left" | "right"
    pub navbar_side: String,
    pub last_project_path: Option<String>,
    pub locale: Option<String>,
}

/// Returns true when a temp file can be created inside `dir` (i.e. writable).
fn is_writable(dir: &Path) -> bool {
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".sn_write_probe");
    match fs::write(&probe, b"") {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// Directory that contains the running executable.
fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// Portable-first base directory: the exe dir when writable, else app-data.
pub fn default_base_dir(app: &tauri::AppHandle) -> PathBuf {
    if let Some(dir) = exe_dir() {
        if is_writable(&dir) {
            return dir;
        }
    }
    app.path()
        .app_data_dir()
        .expect("cannot resolve app data dir")
}

/// Location of the always-writable app-settings file (app-data dir).
fn settings_file(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("cannot resolve app data dir");
    let _ = fs::create_dir_all(&dir);
    dir.join("app-settings.json")
}

fn default_settings(app: &tauri::AppHandle) -> AppSettings {
    let base = default_base_dir(app);
    AppSettings {
        projects_path: base.join("projects").to_string_lossy().to_string(),
        themes_path: base.join("themes").to_string_lossy().to_string(),
        navbar_side: "left".to_string(),
        last_project_path: None,
        locale: None,
    }
}

/// Loads app settings, creating defaults on first run.
pub fn load_settings(app: &tauri::AppHandle) -> AppSettings {
    let file = settings_file(app);
    if let Ok(txt) = fs::read_to_string(&file) {
        if let Ok(mut s) = serde_json::from_str::<AppSettings>(&txt) {
            if s.navbar_side != "right" {
                s.navbar_side = "left".to_string();
            }
            return s;
        }
    }
    let defaults = default_settings(app);
    let _ = save_settings_inner(app, &defaults);
    defaults
}

fn save_settings_inner(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    let file = settings_file(app);
    let txt = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&file, txt).map_err(|e| e.to_string())
}

/// Creates the projects/, themes/ and themes/fonts/ folders.
pub fn ensure_dirs(settings: &AppSettings) {
    let _ = fs::create_dir_all(&settings.projects_path);
    let _ = fs::create_dir_all(&settings.themes_path);
    let _ = fs::create_dir_all(Path::new(&settings.themes_path).join("fonts"));
}

/// Recursively copies the contents of `src` into `dst`.
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

// ── Tauri commands ────────────────────────────────────────────────────────────

#[tauri::command]
pub fn get_app_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let settings = load_settings(&app);
    ensure_dirs(&settings);
    Ok(settings)
}

#[tauri::command]
pub fn set_app_settings(app: tauri::AppHandle, settings: AppSettings) -> Result<(), String> {
    ensure_dirs(&settings);
    save_settings_inner(&app, &settings)
}

/// Relocates the projects or themes folder to `target`.
/// `kind` = "projects" | "themes". `mode` = "copy" | "blank".
/// Returns the new absolute path.
#[tauri::command]
pub fn relocate_data(
    app: tauri::AppHandle,
    kind: String,
    target: String,
    mode: String,
) -> Result<String, String> {
    let mut settings = load_settings(&app);
    let old_path = match kind.as_str() {
        "projects" => settings.projects_path.clone(),
        "themes" => settings.themes_path.clone(),
        _ => return Err(format!("unknown kind: {kind}")),
    };
    let target_path = PathBuf::from(&target);
    fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;

    if mode == "copy" {
        let old = Path::new(&old_path);
        if old.exists() && old != target_path.as_path() {
            copy_dir_all(old, &target_path).map_err(|e| e.to_string())?;
        }
    }

    match kind.as_str() {
        "projects" => settings.projects_path = target.clone(),
        "themes" => settings.themes_path = target.clone(),
        _ => {}
    }
    ensure_dirs(&settings);
    save_settings_inner(&app, &settings)?;
    Ok(target)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    pub name: String,
    pub db_path: String,
}

/// Lists all projects (sub-folders of `projects_path` containing a project.db).
#[tauri::command]
pub fn list_projects(projects_path: String) -> Result<Vec<ProjectInfo>, String> {
    let root = Path::new(&projects_path);
    if !root.exists() {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let db = entry.path().join("project.db");
        if db.exists() {
            out.push(ProjectInfo {
                name: entry.file_name().to_string_lossy().to_string(),
                db_path: db.to_string_lossy().to_string(),
            });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}
