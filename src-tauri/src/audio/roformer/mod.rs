//! BS-RoFormer vocal stem separation via ONNX Runtime + host-side STFT/iSTFT.
//!
//! Architecture: lucidrains/BS-RoFormer (MIT).
//! Weights: viperx ep_317_sdr_12.9755 (uint8 ONNX export by xycld).

#![cfg(feature = "stems")]

mod download;
mod infer;
mod stft;

pub use download::{
    cancel_download, download_model, model_dir, model_file_path, model_ready,
};
pub use infer::separate_vocals;

/// Hugging Face resolve URL for the uint8 quantized ONNX graph (~158 MB).
pub const MODEL_URL: &str =
    "https://huggingface.co/xycld/BS-RoFormer-ONNX/resolve/main/bs_roformer_ep317_sdr12.9755_quantized_uint8.onnx";
pub const MODEL_FILENAME: &str = "bs_roformer_ep317_sdr12.9755_quantized_uint8.onnx";
pub const MODEL_NAME: &str = "bs_roformer_ep317_sdr12.9755";
pub const MODEL_PAGE: &str = "https://huggingface.co/xycld/BS-RoFormer-ONNX";
pub const MODEL_LABEL: &str = "BS-RoFormer (ep317, SDR 12.98)";
pub const MODEL_SIZE_HINT: &str = "~158 MB";
/// Expected minimum size (bytes). Full SHA is verified when present in sidecar.
pub const MODEL_MIN_BYTES: u64 = 50_000_000;
/// Optional SHA-256 hex (lowercase). Empty = size-only check.
pub const MODEL_SHA256: &str = "";

pub const SAMPLE_RATE: u32 = 44_100;
pub const N_FFT: usize = 2048;
pub const HOP_LENGTH: usize = 441;
pub const WIN_LENGTH: usize = 2048;
pub const CHUNK_SIZE: usize = 352_800;
/// Matches ZFTurbo config `inference.num_overlap`.
pub const NUM_OVERLAP: usize = 2;
pub const FREQ_BINS: usize = N_FFT / 2 + 1; // 1025
pub const AUDIO_CHANNELS: usize = 2; // stereo
/// Frames per chunk: floor(chunk/hop)+1 with center STFT ≈ 801.
pub const DIM_T: usize = CHUNK_SIZE / HOP_LENGTH + 1;
