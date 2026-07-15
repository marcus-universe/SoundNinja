//! Data-location management.
//!
//! Sound Ninja stores its Projects and Themes folders "portable-first":
//! next to the executable when that directory is writable (portable / dev
//! builds), otherwise it falls back to the OS app-data directory (installed
//! builds where the program directory is read-only).
//!
//! App-level preferences (the configurable Projects/Themes paths, the navbar
//! side, the recent projects list) live in a fixed, guaranteed-writable
//! `app-config.db` SQLite database inside the app-data directory — the app
//! must always be able to find these even after the user relocates the data
//! folders. The database itself is owned by the frontend (`utils/appConfig.ts`);
//! Rust only resolves the portable-first default paths.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

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

/// The always-writable app-data directory (holds `app-config.db`).
fn app_data_dir(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("cannot resolve app data dir");
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Creates the projects/, themes/ and themes/fonts/ folders under `base`.
pub fn ensure_default_dirs(base: &Path) {
    let _ = fs::create_dir_all(base.join("projects"));
    let _ = fs::create_dir_all(base.join("themes"));
    let _ = fs::create_dir_all(base.join("themes").join("fonts"));
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DefaultPaths {
    pub projects_path: String,
    pub themes_path: String,
    pub app_config_db_path: String,
    pub app_settings_json_path: String,
}

/// Resolves the portable-first default data paths plus the fixed app-config DB
/// location. The frontend uses these as defaults and to locate `app-config.db`.
#[tauri::command]
pub fn get_default_paths(app: tauri::AppHandle) -> Result<DefaultPaths, String> {
    let base = default_base_dir(&app);
    ensure_default_dirs(&base);
    let data = app_data_dir(&app);
    Ok(DefaultPaths {
        projects_path: base.join("projects").to_string_lossy().to_string(),
        themes_path: base.join("themes").to_string_lossy().to_string(),
        app_config_db_path: data.join("app-config.db").to_string_lossy().to_string(),
        app_settings_json_path: data.join("app-settings.json").to_string_lossy().to_string(),
    })
}

/// Relocates a data folder from `oldPath` to `target`.
/// `mode` = "copy" | "blank". Returns the new absolute path. The caller is
/// responsible for persisting the new path in `app-config.db`.
#[tauri::command]
pub fn relocate_data(
    old_path: String,
    target: String,
    mode: String,
) -> Result<String, String> {
    let target_path = PathBuf::from(&target);
    fs::create_dir_all(&target_path).map_err(|e| e.to_string())?;

    if mode == "copy" {
        let old = Path::new(&old_path);
        if old.exists() && old != target_path.as_path() {
            copy_dir_all(old, &target_path).map_err(|e| e.to_string())?;
        }
    }
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
