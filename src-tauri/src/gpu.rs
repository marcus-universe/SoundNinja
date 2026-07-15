use std::sync::atomic::{AtomicBool, Ordering};

static GPU_AUDIO_ENABLED: AtomicBool = AtomicBool::new(false);

/// Returns `true` if a dedicated (discrete) GPU is available on this system.
///
/// Uses `wgpu` adapter enumeration to check for a `DiscreteGpu` device type.
/// Falls back to `false` on any error so the feature is safely hidden when
/// GPU info is unavailable.
#[tauri::command]
pub fn has_dedicated_gpu() -> bool {
    use wgpu::{Backends, DeviceType, Instance, InstanceDescriptor};
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });
    // `enumerate_adapters` is synchronous and fast — it only queries the
    // driver registry, it does not open a device connection.
    instance
        .enumerate_adapters(Backends::all())
        .into_iter()
        .any(|a| a.get_info().device_type == DeviceType::DiscreteGpu)
}

/// Enables or disables GPU-accelerated DSP for audio processing.
///
/// NOTE: Full GPU DSP implementation (WGPU compute shaders for convolution
/// reverb / FFT effects) is deferred to a dedicated feature PR.
/// This command stores the preference and makes it available to the audio
/// pipeline — currently it is a no-op on the processing path.
#[tauri::command]
pub fn set_gpu_audio(enabled: bool) {
    GPU_AUDIO_ENABLED.store(enabled, Ordering::Relaxed);
    println!(
        "[gpu] GPU audio processing {}",
        if enabled { "enabled (stub)" } else { "disabled" }
    );
}

/// Returns the current GPU audio enabled state (for re-sync after app restart).
#[tauri::command]
pub fn get_gpu_audio_enabled() -> bool {
    GPU_AUDIO_ENABLED.load(Ordering::Relaxed)
}
