//! Download + verify BS-RoFormer ONNX into app data.

use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter, Manager};

use super::{MODEL_FILENAME, MODEL_MIN_BYTES, MODEL_SHA256, MODEL_URL};

pub fn model_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("models")
        .join("bs_roformer");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn model_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(model_dir(app)?.join(MODEL_FILENAME))
}

pub fn model_ready(app: &AppHandle) -> bool {
    let Ok(path) = model_file_path(app) else {
        return false;
    };
    file_looks_ready(&path)
}

fn file_looks_ready(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() || meta.len() < MODEL_MIN_BYTES {
        return false;
    }
    if MODEL_SHA256.is_empty() {
        return true;
    }
    verify_sha256(path, MODEL_SHA256).unwrap_or(false)
}

fn verify_sha256(path: &Path, expected_hex: &str) -> Result<bool, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1024 * 256];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let got = format!("{:x}", hasher.finalize());
    Ok(got.eq_ignore_ascii_case(expected_hex))
}

/// Stream-download model to `.part`, verify, rename into place.
/// Emits `stems_progress` / `stems_model_progress` on `app`.
pub fn download_model(app: &AppHandle) -> Result<(), String> {
    let dest = model_file_path(app)?;
    if file_looks_ready(&dest) {
        emit(app, "download", 100.0, "Model ready");
        let _ = app.emit("stems_model_progress", 100.0f32);
        return Ok(());
    }

    let part = dest.with_extension("onnx.part");
    if part.exists() {
        let _ = fs::remove_file(&part);
    }

    emit(app, "download", 0.0, "Downloading BS-RoFormer model…");
    let _ = app.emit("stems_model_progress", 0.0f32);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60 * 30))
        .build()
        .map_err(|e| e.to_string())?;

    let mut resp = client
        .get(MODEL_URL)
        .header("User-Agent", "SoundNinja/stems")
        .send()
        .map_err(|e| format!("Download request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Download failed: HTTP {}", resp.status()));
    }

    let total = resp.content_length().unwrap_or(0);
    let mut file = File::create(&part).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut downloaded: u64 = 0;
    let mut buf = [0u8; 1024 * 64];

    loop {
        let n = resp.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        hasher.update(&buf[..n]);
        downloaded += n as u64;
        let pct = if total > 0 {
            (downloaded as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        emit(
            app,
            "download",
            pct,
            &format!("Downloading model… {downloaded}/{total}"),
        );
        let _ = app.emit("stems_model_progress", pct);
    }
    file.flush().map_err(|e| e.to_string())?;
    drop(file);

    if downloaded < MODEL_MIN_BYTES {
        let _ = fs::remove_file(&part);
        return Err(format!(
            "Downloaded file too small ({downloaded} bytes). Check network and retry."
        ));
    }

    if !MODEL_SHA256.is_empty() {
        let got = format!("{:x}", hasher.finalize());
        if !got.eq_ignore_ascii_case(MODEL_SHA256) {
            let _ = fs::remove_file(&part);
            return Err(format!(
                "SHA-256 mismatch (got {got}, expected {MODEL_SHA256})"
            ));
        }
    }

    if dest.exists() {
        let _ = fs::remove_file(&dest);
    }
    fs::rename(&part, &dest).map_err(|e| e.to_string())?;
    emit(app, "download", 100.0, "Model ready");
    let _ = app.emit("stems_model_progress", 100.0f32);
    Ok(())
}

fn emit(app: &AppHandle, stage: &str, percent: f32, message: &str) {
    #[derive(Clone, serde::Serialize)]
    struct Progress {
        stage: String,
        percent: f32,
        message: String,
    }
    let _ = app.emit(
        "stems_progress",
        Progress {
            stage: stage.to_string(),
            percent,
            message: message.to_string(),
        },
    );
}
