//! Host-side STFT / iSTFT matching torch.stft / torch.istft
//! (periodic Hann, center=True, normalized=False) used by BS_roformer_processor.

use realfft::{ComplexToReal, RealFftPlanner, RealToComplex};
use rustfft::num_complex::Complex;
use std::f32::consts::PI;
use std::sync::Arc;

use super::{FREQ_BINS, HOP_LENGTH, N_FFT, WIN_LENGTH};

pub struct StftEngine {
    forward: Arc<dyn RealToComplex<f32>>,
    inverse: Arc<dyn ComplexToReal<f32>>,
    window: Vec<f32>,
    /// Window² used for envelope normalisation on inverse.
    window_sq: Vec<f32>,
}

impl StftEngine {
    pub fn new() -> Self {
        let mut planner = RealFftPlanner::<f32>::new();
        let forward = planner.plan_fft_forward(N_FFT);
        let inverse = planner.plan_fft_inverse(N_FFT);
        let window = hann_periodic(WIN_LENGTH);
        let window_sq: Vec<f32> = window.iter().map(|w| w * w).collect();
        Self {
            forward,
            inverse,
            window,
            window_sq,
        }
    }

    /// STFT of one mono channel. Returns complex frames: `frames * FREQ_BINS`.
    pub fn stft_mono(&self, samples: &[f32]) -> Vec<Complex<f32>> {
        let pad = N_FFT / 2;
        let mut padded = Vec::with_capacity(samples.len() + 2 * pad);
        // Reflect pad (torch center=True).
        reflect_pad(samples, pad, &mut padded);

        let n_frames = if padded.len() >= N_FFT {
            1 + (padded.len() - N_FFT) / HOP_LENGTH
        } else {
            0
        };

        let mut out = Vec::with_capacity(n_frames * FREQ_BINS);
        let mut input = self.forward.make_input_vec();
        let mut spectrum = self.forward.make_output_vec();

        for frame in 0..n_frames {
            let start = frame * HOP_LENGTH;
            for i in 0..N_FFT {
                input[i] = padded[start + i] * self.window[i];
            }
            self.forward
                .process(&mut input, &mut spectrum)
                .expect("rfft forward");
            out.extend_from_slice(&spectrum);
        }
        out
    }

    /// Inverse STFT with overlap-add + window envelope normalisation.
    pub fn istft_mono(&self, spectrum: &[Complex<f32>], out_len: usize) -> Vec<f32> {
        let n_frames = spectrum.len() / FREQ_BINS;
        let pad = N_FFT / 2;
        let full_len = (n_frames.saturating_sub(1)) * HOP_LENGTH + N_FFT;
        let mut acc = vec![0.0f32; full_len];
        let mut weight = vec![0.0f32; full_len];

        let mut time_buf = self.inverse.make_output_vec();
        let mut spec_buf = self.inverse.make_input_vec();

        for frame in 0..n_frames {
            let src = &spectrum[frame * FREQ_BINS..(frame + 1) * FREQ_BINS];
            spec_buf.copy_from_slice(src);
            self.inverse
                .process(&mut spec_buf, &mut time_buf)
                .expect("irfft inverse");
            // realfft inverse is unnormalized (sum of forward*inverse = N).
            let scale = 1.0 / N_FFT as f32;
            let start = frame * HOP_LENGTH;
            for i in 0..N_FFT {
                let s = time_buf[i] * scale * self.window[i];
                acc[start + i] += s;
                weight[start + i] += self.window_sq[i];
            }
        }

        for i in 0..full_len {
            if weight[i] > 1e-8 {
                acc[i] /= weight[i];
            }
        }

        // Remove center padding and truncate/pad to out_len.
        let end = (pad + out_len).min(acc.len());
        let mut out = vec![0.0f32; out_len];
        if end > pad {
            out[..end - pad].copy_from_slice(&acc[pad..end]);
        }
        out
    }
}

/// Pack stereo STFT into BS-RoFormer input layout:
/// `stft_repr: b s f t c -> b (f s) t c`, then `-> b t (f c)`.
///
/// Returns flat `(T, F*S*2)` row-major for ONNX input shape `[1, T, 4100]`,
/// plus the complex spectrogram kept for mask multiply: shape `(F*S, T)` complex
/// stored as interleaved real/imag per `(freq_ch, time)`.
pub fn pack_stereo_input(
    left: &[Complex<f32>],
    right: &[Complex<f32>],
    n_frames: usize,
) -> (Vec<f32>, Vec<f32>) {
    let f_merged = FREQ_BINS * 2; // (f s)
    let feat = f_merged * 2; // (f c) with c=real/imag
    let mut input = vec![0.0f32; n_frames * feat];
    // Keep stft_repr as [f_merged][t][2] for mask apply.
    let mut stft_repr = vec![0.0f32; f_merged * n_frames * 2];

    for t in 0..n_frames {
        for f in 0..FREQ_BINS {
            let l = left[t * FREQ_BINS + f];
            let r = right[t * FREQ_BINS + f];
            // merged freq index: even = left, odd = right  (f*s with s leading after f)
            // rearrange 'b s f t c -> b (f s) t c' means for each f, channels are adjacent:
            // index = f * 2 + s
            let i_l = f * 2;
            let i_r = f * 2 + 1;

            // stft_repr[freq_ch, t, c]
            let base_l = (i_l * n_frames + t) * 2;
            let base_r = (i_r * n_frames + t) * 2;
            stft_repr[base_l] = l.re;
            stft_repr[base_l + 1] = l.im;
            stft_repr[base_r] = r.re;
            stft_repr[base_r + 1] = r.im;

            // input[t, (f c)] where f here is merged (f*s)
            let row = t * feat;
            input[row + i_l * 2] = l.re;
            input[row + i_l * 2 + 1] = l.im;
            input[row + i_r * 2] = r.re;
            input[row + i_r * 2 + 1] = r.im;
        }
    }
    (input, stft_repr)
}

/// Apply complex mask to stored STFT and return per-channel spectrograms.
///
/// `mask` is flat `[n_stems, f_merged, T, 2]` or `[f_merged, T, 2]` (stem dim squeezed).
/// Returns (left_complex_frames, right_complex_frames) each `T * FREQ_BINS`.
pub fn apply_mask(
    stft_repr: &[f32],
    mask: &[f32],
    n_frames: usize,
    mask_has_stem_dim: bool,
) -> (Vec<Complex<f32>>, Vec<Complex<f32>>) {
    let f_merged = FREQ_BINS * 2;
    let mut left = vec![Complex::new(0.0, 0.0); n_frames * FREQ_BINS];
    let mut right = vec![Complex::new(0.0, 0.0); n_frames * FREQ_BINS];

    // Mask layout after typical export: [1, 1, f_merged, T, 2] or [1, f_merged, T, 2]
    // We accept flat data matching f_merged * T * 2 (stem 0).
    let mask_elems = f_merged * n_frames * 2;
    let mask_offset = if mask_has_stem_dim && mask.len() >= mask_elems * 2 {
        // [stems, f, t, 2] — use stem 0; if longer than one stem, skip nothing extra
        0
    } else {
        0
    };
    let mask = &mask[mask_offset..mask_offset + mask_elems.min(mask.len())];

    for t in 0..n_frames {
        for f in 0..FREQ_BINS {
            let i_l = f * 2;
            let i_r = f * 2 + 1;
            let s_base_l = (i_l * n_frames + t) * 2;
            let s_base_r = (i_r * n_frames + t) * 2;
            let m_base_l = (i_l * n_frames + t) * 2;
            let m_base_r = (i_r * n_frames + t) * 2;

            if s_base_l + 1 >= stft_repr.len() || m_base_l + 1 >= mask.len() {
                continue;
            }
            let sr = Complex::new(stft_repr[s_base_l], stft_repr[s_base_l + 1]);
            let mr = Complex::new(mask[m_base_l], mask[m_base_l + 1]);
            let si = Complex::new(stft_repr[s_base_r], stft_repr[s_base_r + 1]);
            let mi = Complex::new(mask[m_base_r], mask[m_base_r + 1]);
            let ol = sr * mr;
            let or = si * mi;
            left[t * FREQ_BINS + f] = ol;
            right[t * FREQ_BINS + f] = or;
        }
    }
    (left, right)
}

fn hann_periodic(len: usize) -> Vec<f32> {
    // torch.hann_window(periodic=True): period = len
    (0..len)
        .map(|i| 0.5 - 0.5 * (2.0 * PI * i as f32 / len as f32).cos())
        .collect()
}

/// Torch/numpy `reflect` pad (edge not repeated). Requires `pad < n` for audio chunks.
fn reflect_pad(samples: &[f32], pad: usize, out: &mut Vec<f32>) {
    out.clear();
    let n = samples.len();
    if n == 0 {
        out.resize(2 * pad, 0.0);
        return;
    }
    if n == 1 {
        out.resize(n + 2 * pad, samples[0]);
        return;
    }
    // Left: samples[pad], samples[pad-1], …, samples[1]
    for i in (1..=pad).rev() {
        let idx = {
            let mut k = i;
            while k >= n {
                k = 2 * (n - 1) - k;
                if k >= n {
                    k = (k % (2 * (n - 1))).min(n - 1);
                    break;
                }
            }
            k.min(n - 1)
        };
        out.push(samples[idx]);
    }
    out.extend_from_slice(samples);
    // Right: samples[n-2], samples[n-3], …, samples[n-pad-1]
    for i in 1..=pad {
        let idx = {
            let mut k = n - 1 - i;
            if (k as isize) < 0 {
                k = i % (n - 1);
            }
            k.min(n - 1)
        };
        out.push(samples[idx]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hann_peak_is_one() {
        let w = hann_periodic(2048);
        assert!((w[1024] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn stft_istft_roundtrip_sine() {
        let eng = StftEngine::new();
        let sr = 44100usize;
        let n = 352_800usize;
        let mut mono = vec![0.0f32; n];
        for (i, s) in mono.iter_mut().enumerate() {
            *s = (2.0 * PI * 440.0 * i as f32 / sr as f32).sin() * 0.5;
        }
        let spec = eng.stft_mono(&mono);
        assert_eq!(spec.len() / FREQ_BINS, 801);
        let out = eng.istft_mono(&spec, n);
        // Center region should reconstruct; edges of long clips are softer.
        let mid = n / 2;
        let err: f32 = (0..1000)
            .map(|k| (out[mid + k] - mono[mid + k]).abs())
            .sum::<f32>()
            / 1000.0;
        assert!(err < 0.05, "mean abs err {err}");
    }
}
