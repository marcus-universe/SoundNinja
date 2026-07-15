//! Unrestricted absolute-path file helpers.
//!
//! The Projects and Themes folders can live anywhere on disk (portable exe
//! dir or a user-chosen location), which is outside the tauri-plugin-fs
//! AppData scope. These commands run in Rust — unaffected by the frontend
//! filesystem scope — so the UI can read/write theme CSS, list fonts and copy
//! font files regardless of where the data folders were relocated.

use std::fs;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use serde::Serialize;

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

/// Deletes a directory recursively if it exists. No error when already absent.
#[tauri::command]
pub fn delete_dir_abs(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if p.exists() {
        fs::remove_dir_all(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FolderAudioBucket {
    pub dir: String,
    pub name: String,
    pub audio_files: Vec<String>,
}

/// Scans each root directory for audio files and includes subfolders up to
/// `max_depth` levels deep (0 = root only).
#[tauri::command]
pub fn collect_audio_buckets_abs(
    roots: Vec<String>,
    max_depth: usize,
    exts: Vec<String>,
) -> Result<Vec<FolderAudioBucket>, String> {
    let lower_exts: Vec<String> = exts
        .into_iter()
        .map(|e| e.trim_start_matches('.').to_lowercase())
        .collect();

    let mut out: Vec<FolderAudioBucket> = Vec::new();

    for root in roots {
        let root_path = PathBuf::from(&root);
        if !root_path.exists() {
            continue;
        }

        let mut queue: VecDeque<(PathBuf, usize)> = VecDeque::new();
        queue.push_back((root_path, 0));

        while let Some((dir, depth)) = queue.pop_front() {
            let entries = fs::read_dir(&dir)
                .map_err(|e| format!("Failed to read folder '{}': {}", dir.to_string_lossy(), e))?;

            let mut audio_files: Vec<String> = Vec::new();
            let mut subdirs: Vec<PathBuf> = Vec::new();

            for entry_res in entries {
                let entry = entry_res
                    .map_err(|e| format!("Failed to read folder entry in '{}': {}", dir.to_string_lossy(), e))?;
                let path = entry.path();
                let ty = entry
                    .file_type()
                    .map_err(|e| format!("Failed to read entry type in '{}': {}", dir.to_string_lossy(), e))?;

                if ty.is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let ext = Path::new(&name)
                        .extension()
                        .map(|v| v.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    if lower_exts.contains(&ext) {
                        audio_files.push(name);
                    }
                } else if ty.is_dir() && depth < max_depth {
                    subdirs.push(path);
                }
            }

            if !audio_files.is_empty() {
                let name = dir
                    .file_name()
                    .map(|v| v.to_string_lossy().to_string())
                    .unwrap_or_else(|| dir.to_string_lossy().to_string());
                out.push(FolderAudioBucket {
                    dir: dir.to_string_lossy().to_string(),
                    name,
                    audio_files,
                });
            }

            for sub in subdirs {
                queue.push_back((sub, depth + 1));
            }
        }
    }

    Ok(out)
}
