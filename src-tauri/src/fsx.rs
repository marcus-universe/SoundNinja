//! Unrestricted absolute-path file helpers.
//!
//! The Projects and Themes folders can live anywhere on disk (portable exe
//! dir or a user-chosen location), which is outside the tauri-plugin-fs
//! AppData scope. These commands run in Rust — unaffected by the frontend
//! filesystem scope — so the UI can read/write theme CSS, list fonts and copy
//! font files regardless of where the data folders were relocated.

use std::fs;
use std::path::{Path, PathBuf};

#[tauri::command]
pub fn read_text_file_abs(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

/// Reads a binary file and returns its base64-encoded contents. Used to load
/// uploaded font files into the frontend as `@font-face` data URLs.
#[tauri::command]
pub fn read_file_base64_abs(path: String) -> Result<String, String> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    Ok(STANDARD.encode(bytes))
}

#[tauri::command]
pub fn write_text_file_abs(path: String, contents: String) -> Result<(), String> {
    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&path, contents).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn path_exists_abs(path: String) -> bool {
    Path::new(&path).exists()
}

#[tauri::command]
pub fn make_dir_abs(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}

/// Lists file names in `dir` whose extension is in `exts` (case-insensitive).
/// Pass an empty `exts` to list every file. Non-existent dir → empty list.
#[tauri::command]
pub fn list_dir_files_abs(dir: String, exts: Vec<String>) -> Result<Vec<String>, String> {
    let path = Path::new(&dir);
    if !path.exists() {
        return Ok(vec![]);
    }
    let lower: Vec<String> = exts.iter().map(|e| e.to_lowercase()).collect();
    let mut out = Vec::new();
    for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if lower.is_empty() {
            out.push(name);
            continue;
        }
        let ext = Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if lower.contains(&ext) {
            out.push(name);
        }
    }
    out.sort();
    Ok(out)
}

/// Copies `src` into `dst_dir`, keeping the original file name.
/// Returns the destination absolute path.
#[tauri::command]
pub fn copy_file_abs(src: String, dst_dir: String) -> Result<String, String> {
    let src_path = PathBuf::from(&src);
    let name = src_path
        .file_name()
        .ok_or_else(|| "source has no file name".to_string())?;
    fs::create_dir_all(&dst_dir).map_err(|e| e.to_string())?;
    let dst = Path::new(&dst_dir).join(name);
    fs::copy(&src_path, &dst).map_err(|e| e.to_string())?;
    Ok(dst.to_string_lossy().to_string())
}

/// Deletes a file if it exists. No error when already absent.
#[tauri::command]
pub fn delete_file_abs(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if p.exists() {
        fs::remove_file(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}
