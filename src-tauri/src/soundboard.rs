//! Soundboard ZIP export / import with progress events.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipCopyEntry {
    pub src: String,
    pub dest_rel: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferProgress {
    pub percent: u32,
    pub current: u32,
    pub total: u32,
}

fn emit_progress(app: &AppHandle, event: &str, current: u32, total: u32) {
    let percent = if total == 0 {
        100
    } else {
        ((current as u64 * 100) / total as u64) as u32
    };
    let _ = app.emit(
        event,
        TransferProgress {
            percent,
            current,
            total,
        },
    );
}

fn sanitize_zip_rel(rel: &str) -> Result<String, String> {
    let norm = rel.replace('\\', "/");
    if norm.starts_with('/') || norm.contains("..") {
        return Err(format!("invalid zip path: {rel}"));
    }
    Ok(norm)
}

fn safe_extract_path(dest: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel = sanitize_zip_rel(rel)?;
    let out = dest.join(Path::new(&rel));
    let dest_abs = dest
        .canonicalize()
        .unwrap_or_else(|_| dest.to_path_buf());
    // Parent may not exist yet — compare prefix after normalizing.
    let out_norm = out.components().collect::<PathBuf>();
    let dest_norm = dest_abs.components().collect::<PathBuf>();
    if !out_norm.starts_with(&dest_norm) && !out.starts_with(dest) {
        return Err("zip path escapes destination".into());
    }
    Ok(out)
}

#[tauri::command]
pub fn make_temp_dir() -> Result<String, String> {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("soundninja-{ms}"));
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().to_string())
}

#[tauri::command]
pub fn export_soundboard_zip(
    app: AppHandle,
    zip_path: String,
    entries: Vec<ZipCopyEntry>,
) -> Result<(), String> {
    if let Some(parent) = Path::new(&zip_path).parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = File::create(&zip_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(file);
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let total = entries.len() as u32;
    emit_progress(&app, "soundboard_export_progress", 0, total.max(1));

    for (i, entry) in entries.iter().enumerate() {
        let dest = sanitize_zip_rel(&entry.dest_rel)?;
        let mut src = File::open(&entry.src).map_err(|e| format!("{}: {e}", entry.src))?;
        let mut buf = Vec::new();
        src.read_to_end(&mut buf)
            .map_err(|e| format!("{}: {e}", entry.src))?;
        zip.start_file(&dest, opts)
            .map_err(|e| e.to_string())?;
        zip.write_all(&buf).map_err(|e| e.to_string())?;
        emit_progress(&app, "soundboard_export_progress", (i + 1) as u32, total.max(1));
    }

    zip.finish().map_err(|e| e.to_string())?;
    emit_progress(&app, "soundboard_export_progress", total.max(1), total.max(1));
    Ok(())
}

#[tauri::command]
pub fn import_soundboard_zip(
    app: AppHandle,
    zip_path: String,
    dest_dir: String,
) -> Result<String, String> {
    let dest = PathBuf::from(&dest_dir);
    fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    let file = File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    let mut has_project = false;
    for i in 0..archive.len() {
        let name = archive
            .by_index(i)
            .map_err(|e| e.to_string())?
            .name()
            .replace('\\', "/");
        if name == "project.sninja" || name.ends_with("/project.sninja") {
            has_project = true;
            break;
        }
    }
    if !has_project {
        return Err("ZIP is missing project.sninja".into());
    }

    let total = archive.len() as u32;
    emit_progress(&app, "soundboard_import_progress", 0, total.max(1));

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().replace('\\', "/");
        if name.ends_with('/') {
            let dir = safe_extract_path(&dest, name.trim_end_matches('/'))?;
            fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        } else {
            let out = safe_extract_path(&dest, &name)?;
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let mut outfile = File::create(&out).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut outfile).map_err(|e| e.to_string())?;
        }
        emit_progress(&app, "soundboard_import_progress", (i + 1) as u32, total.max(1));
    }

    let db = dest.join("project.sninja");
    if !db.exists() {
        // Nested project.sninja (folder zip)
        for entry in walkdir_one(&dest) {
            if entry.file_name().and_then(|n| n.to_str()) == Some("project.sninja") {
                return Ok(entry.to_string_lossy().to_string());
            }
        }
        return Err("project.sninja not found after extract".into());
    }
    Ok(db.to_string_lossy().to_string())
}

fn walkdir_one(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.push(p);
            }
        }
    }
    out
}
