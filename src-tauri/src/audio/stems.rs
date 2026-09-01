//! BS-RoFormer stem separation (ONNX Runtime + host STFT).
//! Model downloads on first use into `{app_data}/models/bs_roformer/`.

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use super::dsp::SessionInfo;

pub const STEMS_MODEL_NAME: &str = "bs_roformer_ep317_sdr12.9755";
pub const STEMS_MODEL_PAGE: &str = "https://huggingface.co/xycld/BS-RoFormer-ONNX";
pub const STEMS_MODEL_LABEL: &str = "BS-RoFormer (ep317, SDR 12.98)";
pub const STEMS_MODEL_SIZE: &str = "~158 MB";

#[cfg(feature = "stems")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(feature = "stems")]
static BUSY: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Serialize)]
pub struct StemsProgress {
    pub stage: String,
    pub percent: f32,
    pub message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StemsStatus {
    /// True when this binary was compiled with `--features stems`.
    pub available: bool,
    /// True when the ONNX model file is present and looks valid.
    pub model_ready: bool,
    /// Installer checkbox (Windows) or pending first-run ask.
    pub install_intent: bool,
    /// True when the UI may auto-prompt for download.
    pub can_prompt: bool,
    /// True after the user has been asked (declined or accepted).
    pub has_been_asked: bool,
    pub model_name: String,
    pub model_label: String,
    pub model_page_url: String,
    pub model_size_hint: String,
}

fn asked_marker_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("models")
        .join("bs_roformer");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("stems-asked"))
}

fn has_been_asked(app: &AppHandle) -> bool {
    asked_marker_path(app)
        .map(|p| p.exists())
        .unwrap_or(false)
}

fn mark_asked(app: &AppHandle) -> Result<(), String> {
    let path = asked_marker_path(app)?;
    fs::write(path, b"1").map_err(|e| e.to_string())
}

/// Windows: HKCU\Software\com.soundninja.dev\WantStemsModel = 1
fn read_windows_install_intent() -> bool {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey("Software\\com.soundninja.dev") {
            if let Ok(v) = key.get_value::<u32, _>("WantStemsModel") {
                return v != 0;
            }
            if let Ok(v) = key.get_value::<String, _>("WantStemsModel") {
                return v == "1" || v.eq_ignore_ascii_case("true");
            }
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

fn clear_windows_install_intent() {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey_with_flags(
            "Software\\com.soundninja.dev",
            winreg::enums::KEY_WRITE,
        ) {
            let _ = key.delete_value("WantStemsModel");
            let _ = key.set_value("WantStemsModel", &0u32);
        }
    }
}

fn model_ready_status(app: &AppHandle) -> bool {
    #[cfg(feature = "stems")]
    {
        crate::audio::roformer::model_ready(app)
    }
    #[cfg(not(feature = "stems"))]
    {
        let _ = app;
        // Still detect a previously downloaded model so UI can show status.
        if let Ok(dir) = app.path().app_data_dir() {
            let path = dir
                .join("models")
                .join("bs_roformer")
                .join("bs_roformer_ep317_sdr12.9755_quantized_uint8.onnx");
            if let Ok(meta) = fs::metadata(path) {
                return meta.is_file() && meta.len() > 50_000_000;
            }
        }
        false
    }
}

#[tauri::command(async)]
pub fn get_stems_status(app: AppHandle) -> StemsStatus {
    let available = cfg!(feature = "stems");
    let model_ready = model_ready_status(&app);
    let asked = has_been_asked(&app);
    let win_intent = read_windows_install_intent();
    let is_macos = cfg!(target_os = "macos");

    // Intent: Windows checkbox still pending, or not-yet-asked on Linux/macOS.
    let install_intent = if model_ready {
        false
    } else if win_intent && !asked {
        true
    } else if !cfg!(target_os = "windows") && !asked {
        // Linux + macOS: first-run may prompt.
        true
    } else {
        false
    };

    // Auto-prompt: macOS always (until asked or ready); Win/Linux only with intent.
    let can_prompt = available
        && !model_ready
        && !asked
        && (is_macos || win_intent || !cfg!(target_os = "windows"));

    StemsStatus {
        available,
        model_ready,
        install_intent,
        can_prompt,
        has_been_asked: asked,
        model_name: STEMS_MODEL_NAME.to_string(),
        model_label: STEMS_MODEL_LABEL.to_string(),
        model_page_url: STEMS_MODEL_PAGE.to_string(),
        model_size_hint: STEMS_MODEL_SIZE.to_string(),
    }
}

#[tauri::command(async)]
pub fn dismiss_stems_intent(app: AppHandle) -> Result<(), String> {
    mark_asked(&app)?;
    clear_windows_install_intent();
    Ok(())
}

/// Downloads ~158 MB — blocking pool, not an async worker.
#[cfg(feature = "stems")]
#[tauri::command]
pub async fn ensure_stems_model(app: AppHandle) -> Result<(), String> {
    crate::task::run_blocking(move || {
        crate::audio::roformer::download_model(&app)?;
        mark_asked(&app)?;
        clear_windows_install_intent();
        Ok(())
    })
    .await
}

/// ONNX inference runs for minutes on long clips — strictly blocking pool.
#[cfg(feature = "stems")]
#[tauri::command]
pub async fn split_session(
    app: AppHandle,
    session_id: String,
    mode: String,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<SessionInfo, String> {
    crate::task::run_blocking(move || {
        split_session_blocking(app, session_id, mode, start_sec, end_sec)
    })
    .await
}

#[cfg(feature = "stems")]
fn split_session_blocking(
    app: AppHandle,
    session_id: String,
    mode: String,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<SessionInfo, String> {
    if BUSY.swap(true, Ordering::SeqCst) {
        return Err("Stem separation already running".into());
    }
    let result = (|| {
        if !crate::audio::roformer::model_ready(&app) {
            crate::audio::roformer::download_model(&app)?;
        }
        let (samples, sr, ch) =
            super::dsp::clone_session_range(&session_id, start_sec, end_sec)?;

        let vocals =
            crate::audio::roformer::separate_vocals(&app, &samples, sr, ch)?;

        let mode_l = mode.to_lowercase();
        let out = if mode_l == "vocals" || mode_l == "voice" {
            vocals
        } else {
            // music / instrumental = mix − vocals
            if vocals.len() != samples.len() {
                return Err("Vocals length mismatch vs mix".into());
            }
            samples
                .iter()
                .zip(vocals.iter())
                .map(|(m, v)| (m - v).clamp(-1.0, 1.0))
                .collect()
        };

        super::dsp::replace_session_range(&session_id, start_sec, end_sec, out, sr, ch)?;
        mark_asked(&app)?;
        super::dsp::session_info(&session_id)
    })();
    BUSY.store(false, Ordering::SeqCst);
    result
}

#[cfg(feature = "stems")]
#[tauri::command(async)]
pub fn stems_busy() -> bool {
    BUSY.load(Ordering::SeqCst)
}

#[cfg(feature = "stems")]
#[tauri::command(async)]
pub fn cancel_stems_model_download() {
    crate::audio::roformer::cancel_download();
}

#[cfg(not(feature = "stems"))]
#[tauri::command(async)]
pub fn ensure_stems_model(_app: AppHandle) -> Result<(), String> {
    Err(
        "STEMS_ENGINE_UNAVAILABLE: Stem separation engine is not compiled into this build. \
Rebuild with --features stems."
            .into(),
    )
}

#[cfg(not(feature = "stems"))]
#[tauri::command(async)]
pub fn split_session(
    _app: AppHandle,
    _session_id: String,
    _mode: String,
    _start_sec: Option<f64>,
    _end_sec: Option<f64>,
) -> Result<SessionInfo, String> {
    Err(
        "STEMS_ENGINE_UNAVAILABLE: Stem separation engine is not compiled into this build. \
Rebuild with --features stems."
            .into(),
    )
}

#[cfg(not(feature = "stems"))]
#[tauri::command(async)]
pub fn stems_busy() -> bool {
    false
}

#[cfg(not(feature = "stems"))]
#[tauri::command(async)]
pub fn cancel_stems_model_download() {}
