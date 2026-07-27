//! HTDemucs stem separation via stem-splitter-core (ONNX Runtime).
//! Model downloads on first use into the crate's default cache.

use serde::Serialize;
use tauri::AppHandle;

use super::dsp::SessionInfo;

pub const STEMS_MODEL_NAME: &str = "htdemucs_ort_v1";
pub const STEMS_MODEL_PAGE: &str = "https://huggingface.co/gentij/htdemucs-ort";
pub const STEMS_MODEL_LABEL: &str = "HTDemucs-ORT (htdemucs_ort_v1)";
pub const STEMS_MODEL_SIZE: &str = "~200 MB";

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
    /// True when a cached ONNX model file looks present.
    pub model_ready: bool,
    pub model_name: String,
    pub model_label: String,
    pub model_page_url: String,
    pub model_size_hint: String,
}

fn model_cache_ready() -> bool {
    #[cfg(feature = "stems")]
    {
        if let Ok(dir) = stem_splitter_core::io::paths::models_cache_dir() {
            if let Ok(rd) = std::fs::read_dir(dir) {
                for entry in rd.flatten() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if !name.contains("htdemucs") {
                        continue;
                    }
                    if let Ok(meta) = entry.metadata() {
                        // Model is ~200MB; ignore tiny leftovers.
                        if meta.is_file() && meta.len() > 50_000_000 {
                            return true;
                        }
                    }
                }
            }
        }
        return false;
    }
    #[cfg(not(feature = "stems"))]
    {
        // Same cache layout stem-splitter-core uses (directories crate).
        let Some(proj) = directories::ProjectDirs::from("dev", "StemSplitter", "stem-splitter-core")
        else {
            return false;
        };
        let dir = proj.cache_dir().join("models");
        let Ok(rd) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if !name.contains("htdemucs") {
                continue;
            }
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() && meta.len() > 50_000_000 {
                    return true;
                }
            }
        }
        false
    }
}

#[tauri::command]
pub fn get_stems_status() -> StemsStatus {
    StemsStatus {
        available: cfg!(feature = "stems"),
        model_ready: model_cache_ready(),
        model_name: STEMS_MODEL_NAME.to_string(),
        model_label: STEMS_MODEL_LABEL.to_string(),
        model_page_url: STEMS_MODEL_PAGE.to_string(),
        model_size_hint: STEMS_MODEL_SIZE.to_string(),
    }
}

#[cfg(feature = "stems")]
mod imp {
    use super::StemsProgress;
    use crate::audio::dsp::{
        export_session_to_temp, replace_session_range, session_info, SessionInfo,
    };
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, Once, OnceLock};
    use std::thread;
    use tauri::{AppHandle, Emitter, Manager};

    static BUSY: AtomicBool = AtomicBool::new(false);
    static APP_FOR_PROGRESS: OnceLock<Mutex<Option<AppHandle>>> = OnceLock::new();

    fn app_slot() -> &'static Mutex<Option<AppHandle>> {
        APP_FOR_PROGRESS.get_or_init(|| Mutex::new(None))
    }

    fn emit_progress(stage: &str, percent: f32, message: &str) {
        if let Ok(guard) = app_slot().lock() {
            if let Some(app) = guard.as_ref() {
                let _ = app.emit(
                    "stems_progress",
                    StemsProgress {
                        stage: stage.to_string(),
                        percent,
                        message: message.to_string(),
                    },
                );
            }
        }
    }

    fn install_progress_hooks() {
        use stem_splitter_core::{
            set_download_progress_callback, set_split_progress_callback, SplitProgress,
        };

        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            set_download_progress_callback(|done, total| {
                let pct = if total > 0 {
                    (done as f32 / total as f32) * 100.0
                } else {
                    0.0
                };
                emit_progress(
                    "download",
                    pct,
                    &format!("Downloading model… {done}/{total}"),
                );
                if let Ok(g) = app_slot().lock() {
                    if let Some(app) = g.as_ref() {
                        let _ = app.emit("stems_model_progress", pct);
                    }
                }
            });

            set_split_progress_callback(|p: SplitProgress| match p {
                SplitProgress::Stage(s) => emit_progress(s, 0.0, s),
                SplitProgress::Chunks {
                    done,
                    total,
                    percent,
                } => {
                    emit_progress("infer", percent, &format!("Separating… {done}/{total}"));
                }
                SplitProgress::Writing {
                    stem,
                    done,
                    total,
                    percent,
                } => {
                    emit_progress(
                        "write",
                        percent,
                        &format!("Writing {stem} ({done}/{total})"),
                    );
                }
                SplitProgress::Finished => emit_progress("done", 100.0, "Done"),
            });
        });
    }

    fn decode_wav_f32(path: &str) -> Result<(Vec<f32>, u32, u16), String> {
        let mut reader = hound::WavReader::open(path).map_err(|e| e.to_string())?;
        let spec = reader.spec();
        let samples: Result<Vec<f32>, _> = match spec.sample_format {
            hound::SampleFormat::Float => reader.samples::<f32>().collect(),
            hound::SampleFormat::Int => reader
                .samples::<i16>()
                .map(|r| r.map(|s| s as f32 / i16::MAX as f32))
                .collect(),
        };
        Ok((
            samples.map_err(|e| e.to_string())?,
            spec.sample_rate,
            spec.channels,
        ))
    }

    fn sum_wav_files(paths: &[&str]) -> Result<(Vec<f32>, u32, u16), String> {
        let mut sum: Option<(Vec<f32>, u32, u16)> = None;
        for p in paths {
            let (samples, sr, ch) = decode_wav_f32(p)?;
            match &mut sum {
                None => sum = Some((samples, sr, ch)),
                Some((acc, acc_sr, acc_ch)) => {
                    if *acc_sr != sr || *acc_ch != ch {
                        return Err("Stem sample rate/channel mismatch".into());
                    }
                    let len = acc.len().min(samples.len());
                    for i in 0..len {
                        acc[i] = (acc[i] + samples[i]).clamp(-1.0, 1.0);
                    }
                    acc.truncate(len);
                }
            }
        }
        sum.ok_or_else(|| "No stems to sum".into())
    }

    pub fn ensure_stems_model_impl(app: AppHandle) -> Result<(), String> {
        *app_slot().lock().map_err(|e| e.to_string())? = Some(app);
        install_progress_hooks();
        emit_progress("download", 0.0, "Preparing HTDemucs model…");
        stem_splitter_core::prepare_model(super::STEMS_MODEL_NAME, None).map_err(|e| e.to_string())?;
        emit_progress("download", 100.0, "Model ready");
        Ok(())
    }

    pub fn split_session_impl(
        app: AppHandle,
        session_id: String,
        mode: String,
        start_sec: Option<f64>,
        end_sec: Option<f64>,
    ) -> Result<SessionInfo, String> {
        if BUSY.swap(true, Ordering::SeqCst) {
            return Err("Stem separation already running".into());
        }

        *app_slot().lock().map_err(|e| e.to_string())? = Some(app.clone());
        install_progress_hooks();

        let result = (|| {
            let input = export_session_to_temp(&app, &session_id, start_sec, end_sec)?;
            let out_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| e.to_string())?
                .join("recordings")
                .join("stems");
            std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

            emit_progress("start", 0.0, "Starting stem separation…");

            let opts = stem_splitter_core::SplitOptions {
                output_dir: out_dir.to_string_lossy().to_string(),
                model_name: super::STEMS_MODEL_NAME.into(),
                manifest_url_override: None,
            };

            let input_str = input.to_string_lossy().to_string();
            let join = thread::spawn(move || {
                stem_splitter_core::split_file(&input_str, opts).map_err(|e| e.to_string())
            });
            let split = join
                .join()
                .map_err(|_| "Stem thread panicked".to_string())??;

            let mode_l = mode.to_lowercase();
            let (samples, sr, ch) = if mode_l == "vocals" || mode_l == "voice" {
                decode_wav_f32(&split.vocals_path)?
            } else {
                sum_wav_files(&[
                    split.drums_path.as_str(),
                    split.bass_path.as_str(),
                    split.other_path.as_str(),
                ])?
            };

            replace_session_range(&session_id, start_sec, end_sec, samples, sr, ch)?;
            for p in [
                &split.vocals_path,
                &split.drums_path,
                &split.bass_path,
                &split.other_path,
            ] {
                let _ = std::fs::remove_file(p);
            }
            let _ = std::fs::remove_file(&input);

            session_info(&session_id)
        })();

        BUSY.store(false, Ordering::SeqCst);
        result
    }

    pub fn stems_busy_impl() -> bool {
        BUSY.load(Ordering::SeqCst)
    }
}

#[cfg(feature = "stems")]
#[tauri::command]
pub fn ensure_stems_model(app: AppHandle) -> Result<(), String> {
    imp::ensure_stems_model_impl(app)
}

#[cfg(feature = "stems")]
#[tauri::command]
pub fn split_session(
    app: AppHandle,
    session_id: String,
    mode: String,
    start_sec: Option<f64>,
    end_sec: Option<f64>,
) -> Result<SessionInfo, String> {
    imp::split_session_impl(app, session_id, mode, start_sec, end_sec)
}

#[cfg(feature = "stems")]
#[tauri::command]
pub fn stems_busy() -> bool {
    imp::stems_busy_impl()
}

#[cfg(not(feature = "stems"))]
#[tauri::command]
pub fn ensure_stems_model(_app: AppHandle) -> Result<(), String> {
    Err(
        "STEMS_ENGINE_UNAVAILABLE: Stem separation engine is not compiled into this build. \
Update Visual Studio 2022 to 17.14+ (MSVC 14.44+) and rebuild with --features stems."
            .into(),
    )
}

#[cfg(not(feature = "stems"))]
#[tauri::command]
pub fn split_session(
    _app: AppHandle,
    _session_id: String,
    _mode: String,
    _start_sec: Option<f64>,
    _end_sec: Option<f64>,
) -> Result<SessionInfo, String> {
    Err(
        "STEMS_ENGINE_UNAVAILABLE: Stem separation engine is not compiled into this build. \
Update Visual Studio 2022 to 17.14+ (MSVC 14.44+) and rebuild with --features stems."
            .into(),
    )
}

#[cfg(not(feature = "stems"))]
#[tauri::command]
pub fn stems_busy() -> bool {
    false
}
