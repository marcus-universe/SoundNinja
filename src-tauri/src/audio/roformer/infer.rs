//! ONNX Runtime inference + chunked overlap-add for BS-RoFormer.

use ort::session::Session;
use ort::value::Tensor;
use rubato::{
    Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType, WindowFunction,
};
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter};

use super::stft::{apply_mask, pack_stereo_input, StftEngine};
use super::{
    model_file_path, CHUNK_SIZE, NUM_OVERLAP, SAMPLE_RATE,
};

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

fn emit_progress(app: &AppHandle, stage: &str, percent: f32, message: &str) {
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

fn load_session(path: &Path) -> Result<Session, String> {
    Session::builder()
        .map_err(|e| e.to_string())?
        .commit_from_file(path)
        .map_err(|e| format!("Failed to load ONNX model: {e}"))
}

fn get_or_load_session(app: &AppHandle) -> Result<(), String> {
    let mut guard = SESSION.lock().map_err(|e| e.to_string())?;
    if guard.is_some() {
        return Ok(());
    }
    let path = model_file_path(app)?;
    if !path.exists() {
        return Err("BS-RoFormer model not installed".into());
    }
    emit_progress(app, "load", 0.0, "Loading ONNX model…");
    *guard = Some(load_session(&path)?);
    emit_progress(app, "load", 100.0, "Model loaded");
    Ok(())
}

/// Separate vocals from interleaved PCM. Returns interleaved stereo vocals at original rate/channels.
pub fn separate_vocals(
    app: &AppHandle,
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<Vec<f32>, String> {
    get_or_load_session(app)?;

    // Resample to 44.1 kHz stereo for the model.
    let (stereo_441, orig_sr, orig_ch) = to_stereo_441(samples, sample_rate, channels)?;
    let n_frames_total = stereo_441[0].len();

    let step = CHUNK_SIZE / NUM_OVERLAP;
    let fade_size = CHUNK_SIZE / 10;
    let border = CHUNK_SIZE - step;

    // Reflect-pad edges like ZFTurbo demix.
    let mut left = stereo_441[0].clone();
    let mut right = stereo_441[1].clone();
    if n_frames_total > 2 * border && border > 0 {
        left = reflect_extend(&left, border);
        right = reflect_extend(&right, border);
    }
    let padded_len = left.len();

    let window = fade_window(CHUNK_SIZE, fade_size);
    let mut out_l = vec![0.0f32; padded_len];
    let mut out_r = vec![0.0f32; padded_len];
    let mut weight = vec![0.0f32; padded_len];

    let stft = StftEngine::new();
    let mut pos = 0usize;
    let total_steps = padded_len.div_ceil(step).max(1);
    let mut step_i = 0usize;

    emit_progress(app, "start", 0.0, "Starting stem separation…");

    while pos < padded_len {
        let chunk_len = (padded_len - pos).min(CHUNK_SIZE);
        let mut chunk_l = vec![0.0f32; CHUNK_SIZE];
        let mut chunk_r = vec![0.0f32; CHUNK_SIZE];
        chunk_l[..chunk_len].copy_from_slice(&left[pos..pos + chunk_len]);
        chunk_r[..chunk_len].copy_from_slice(&right[pos..pos + chunk_len]);
        if chunk_len < CHUNK_SIZE {
            // Reflect pad remainder when long enough; else zeros.
            if chunk_len > CHUNK_SIZE / 2 {
                for i in chunk_len..CHUNK_SIZE {
                    let src = (2 * chunk_len).saturating_sub(i + 2).min(chunk_len - 1);
                    chunk_l[i] = chunk_l[src];
                    chunk_r[i] = chunk_r[src];
                }
            }
        }

        let (voc_l, voc_r) = infer_chunk(app, &stft, &chunk_l, &chunk_r)?;

        let mut win = window.clone();
        if pos == 0 {
            for w in win.iter_mut().take(fade_size) {
                *w = 1.0;
            }
        }
        if pos + step >= padded_len {
            for w in win.iter_mut().rev().take(fade_size) {
                *w = 1.0;
            }
        }

        for i in 0..chunk_len {
            let w = win[i];
            out_l[pos + i] += voc_l[i] * w;
            out_r[pos + i] += voc_r[i] * w;
            weight[pos + i] += w;
        }

        step_i += 1;
        let pct = (step_i as f32 / total_steps as f32) * 100.0;
        emit_progress(
            app,
            "infer",
            pct,
            &format!("Separating… {step_i}/{total_steps}"),
        );

        pos += step;
    }

    for i in 0..padded_len {
        if weight[i] > 1e-8 {
            out_l[i] /= weight[i];
            out_r[i] /= weight[i];
        }
    }

    // Remove border padding.
    let (final_l, final_r) = if n_frames_total > 2 * border && border > 0 {
        let end = border + n_frames_total;
        (
            out_l[border..end].to_vec(),
            out_r[border..end].to_vec(),
        )
    } else {
        (out_l[..n_frames_total].to_vec(), out_r[..n_frames_total].to_vec())
    };

    emit_progress(app, "done", 100.0, "Done");

    // Convert back to original rate / channel layout.
    from_stereo_441(&final_l, &final_r, orig_sr, orig_ch)
}

fn infer_chunk(
    app: &AppHandle,
    stft: &StftEngine,
    left: &[f32],
    right: &[f32],
) -> Result<(Vec<f32>, Vec<f32>), String> {
    let spec_l = stft.stft_mono(left);
    let spec_r = stft.stft_mono(right);
    let n_frames = spec_l.len() / super::FREQ_BINS;
    if n_frames == 0 {
        return Ok((vec![0.0; left.len()], vec![0.0; right.len()]));
    }

    let (input, stft_repr) = pack_stereo_input(&spec_l, &spec_r, n_frames);
    // Expected ONNX shape [1, T, 4100]
    let feat = super::FREQ_BINS * 2 * 2;
    let shape = [1usize, n_frames, feat];

    let mut guard = SESSION.lock().map_err(|e| e.to_string())?;
    let session = guard
        .as_mut()
        .ok_or_else(|| "ONNX session not loaded".to_string())?;

    let input_name = session
        .inputs()
        .first()
        .map(|i| i.name().to_string())
        .unwrap_or_else(|| "input".into());

    let tensor = Tensor::from_array((shape, input)).map_err(|e| e.to_string())?;

    let outputs = session
        .run(ort::inputs![input_name.as_str() => tensor])
        .map_err(|e| format!("ONNX inference failed: {e}"))?;

    let (_out_name, out_val) = outputs
        .iter()
        .next()
        .ok_or_else(|| "ONNX model returned no outputs".to_string())?;

    let (out_shape, out_data) = out_val
        .try_extract_tensor::<f32>()
        .map_err(|e| format!("Bad ONNX output: {e}"))?;

    let f_merged = super::FREQ_BINS * 2;
    let expected = f_merged * n_frames * 2;
    let mask_has_stem = out_shape.len() >= 5;
    let data = out_data.as_ref();

    // Normalize mask to flat [f_merged, T, 2].
    let mask_flat: Vec<f32> = if out_shape.len() == 3
        && out_shape.get(1).copied() == Some(n_frames as i64)
        && out_shape.get(2).copied() == Some(feat as i64)
    {
        // [b, t, (f c)] → [f, t, 2]
        let mut m = vec![0.0f32; expected];
        for t in 0..n_frames {
            for fc in 0..f_merged {
                let src = t * feat + fc * 2;
                let dst = (fc * n_frames + t) * 2;
                m[dst] = data[src];
                m[dst + 1] = data[src + 1];
            }
        }
        m
    } else if data.len() >= expected {
        // [b, n?, f, t, 2] — take first `expected` values (stem 0).
        data[..expected].to_vec()
    } else {
        let _ = app;
        return Err(format!(
            "Unexpected mask size {} (want {expected}), shape {:?}",
            data.len(),
            out_shape
        ));
    };

    let (masked_l, masked_r) = apply_mask(&stft_repr, &mask_flat, n_frames, mask_has_stem);
    let voc_l = stft.istft_mono(&masked_l, left.len());
    let voc_r = stft.istft_mono(&masked_r, right.len());
    Ok((voc_l, voc_r))
}

fn fade_window(window_size: usize, fade_size: usize) -> Vec<f32> {
    let mut w = vec![1.0f32; window_size];
    if fade_size == 0 || fade_size * 2 > window_size {
        return w;
    }
    for i in 0..fade_size {
        let g = i as f32 / (fade_size as f32 - 1.0).max(1.0);
        w[i] = g;
        w[window_size - 1 - i] = g;
    }
    w
}

fn reflect_extend(x: &[f32], border: usize) -> Vec<f32> {
    let n = x.len();
    let mut out = Vec::with_capacity(n + 2 * border);
    for i in 0..border {
        let idx = (border - i).min(n - 1).max(1);
        // mirror: x[border], … → use x[1..=border] reversed
        out.push(x[idx.min(n - 1)]);
    }
    // cleaner left pad
    out.clear();
    for i in (1..=border).rev() {
        out.push(x[i.min(n - 1)]);
    }
    out.extend_from_slice(x);
    for i in 1..=border {
        out.push(x[n.saturating_sub(1 + i).min(n - 1)]);
    }
    out
}

fn to_stereo_441(
    samples: &[f32],
    sample_rate: u32,
    channels: u16,
) -> Result<(Vec<Vec<f32>>, u32, u16), String> {
    let ch = channels.max(1) as usize;
    let frames = samples.len() / ch;
    let mut planes = vec![vec![0.0f32; frames]; 2];
    for i in 0..frames {
        if ch == 1 {
            planes[0][i] = samples[i];
            planes[1][i] = samples[i];
        } else {
            planes[0][i] = samples[i * ch];
            planes[1][i] = samples[i * ch + 1];
        }
    }

    if sample_rate == SAMPLE_RATE {
        return Ok((planes, sample_rate, channels));
    }

    let resampled = resample_planes(&planes, sample_rate, SAMPLE_RATE)?;
    Ok((resampled, sample_rate, channels))
}

fn from_stereo_441(
    left: &[f32],
    right: &[f32],
    orig_sr: u32,
    orig_ch: u16,
) -> Result<Vec<f32>, String> {
    let mut planes = vec![left.to_vec(), right.to_vec()];
    if orig_sr != SAMPLE_RATE {
        planes = resample_planes(&planes, SAMPLE_RATE, orig_sr)?;
    }
    let frames = planes[0].len();
    if orig_ch <= 1 {
        Ok(planes[0][..frames].to_vec())
    } else {
        let mut out = Vec::with_capacity(frames * 2);
        for i in 0..frames {
            out.push(planes[0][i]);
            out.push(planes[1][i]);
        }
        Ok(out)
    }
}

fn resample_planes(
    planes: &[Vec<f32>],
    from_sr: u32,
    to_sr: u32,
) -> Result<Vec<Vec<f32>>, String> {
    if from_sr == to_sr {
        return Ok(planes.to_vec());
    }
    let mut out = Vec::with_capacity(planes.len());
    for plane in planes {
        let params = SincInterpolationParameters {
            sinc_len: 128,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Linear,
            oversampling_factor: 16,
            window: WindowFunction::BlackmanHarris2,
        };
        let mut resampler = SincFixedIn::<f32>::new(
            to_sr as f64 / from_sr as f64,
            2.0,
            params,
            plane.len(),
            1,
        )
        .map_err(|e| e.to_string())?;
        let waves_in = vec![plane.as_slice()];
        let waves_out = resampler.process(&waves_in, None).map_err(|e| e.to_string())?;
        out.push(waves_out.into_iter().next().unwrap_or_default());
    }
    Ok(out)
}
