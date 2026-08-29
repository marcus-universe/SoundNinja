use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

static GPU_AUDIO_ENABLED: AtomicBool = AtomicBool::new(false);

struct GpuCtx {
    device: wgpu::Device,
    queue: wgpu::Queue,
    peak_pipe: wgpu::ComputePipeline,
    gain_pipe: wgpu::ComputePipeline,
    bucket_pipe: wgpu::ComputePipeline,
    window_pipe: wgpu::ComputePipeline,
}

static GPU: OnceLock<Mutex<Option<GpuCtx>>> = OnceLock::new();

fn gpu_slot() -> &'static Mutex<Option<GpuCtx>> {
    GPU.get_or_init(|| Mutex::new(None))
}

pub fn is_gpu_audio_enabled() -> bool {
    GPU_AUDIO_ENABLED.load(Ordering::Relaxed)
}

/// Returns `true` if a dedicated (discrete) GPU is available on this system.
#[tauri::command]
pub fn has_dedicated_gpu() -> bool {
    use wgpu::{Backends, DeviceType, Instance, InstanceDescriptor};
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::PRIMARY,
        ..Default::default()
    });
    instance
        .enumerate_adapters(Backends::PRIMARY)
        .into_iter()
        .any(|a| a.get_info().device_type == DeviceType::DiscreteGpu)
}

/// Enables or disables GPU-accelerated DSP. Takes effect immediately — no restart.
#[tauri::command]
pub fn set_gpu_audio(enabled: bool) {
    GPU_AUDIO_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        let _ = ensure_device();
    }
    println!(
        "[gpu] GPU audio processing {}",
        if enabled { "enabled" } else { "disabled" }
    );
}

/// Returns the current GPU audio enabled state.
#[tauri::command]
pub fn get_gpu_audio_enabled() -> bool {
    is_gpu_audio_enabled()
}

fn ensure_device() -> Result<(), String> {
    {
        let guard = gpu_slot().lock().map_err(|e| e.to_string())?;
        if guard.is_some() {
            return Ok(());
        }
    }
    let ctx = tauri::async_runtime::block_on(init_gpu())?;
    let mut guard = gpu_slot().lock().map_err(|e| e.to_string())?;
    *guard = Some(ctx);
    Ok(())
}

async fn init_gpu() -> Result<GpuCtx, String> {
    use wgpu::*;
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::PRIMARY,
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .ok_or_else(|| "No GPU adapter".to_string())?;
    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: Some("sn-dsp"),
                required_features: Features::empty(),
                required_limits: Limits::downlevel_defaults(),
                memory_hints: MemoryHints::Performance,
            },
            None,
        )
        .await
        .map_err(|e| e.to_string())?;

    let peak_mod = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("peak"),
        source: ShaderSource::Wgsl(PEAK_WGSL.into()),
    });
    let gain_mod = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("gain"),
        source: ShaderSource::Wgsl(GAIN_WGSL.into()),
    });
    let bucket_mod = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("bucket"),
        source: ShaderSource::Wgsl(BUCKET_WGSL.into()),
    });
    let window_mod = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("window"),
        source: ShaderSource::Wgsl(WINDOW_WGSL.into()),
    });

    Ok(GpuCtx {
        peak_pipe: device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("peak-pipe"),
            layout: None,
            module: &peak_mod,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        }),
        gain_pipe: device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("gain-pipe"),
            layout: None,
            module: &gain_mod,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        }),
        bucket_pipe: device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("bucket-pipe"),
            layout: None,
            module: &bucket_mod,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        }),
        window_pipe: device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("window-pipe"),
            layout: None,
            module: &window_mod,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        }),
        device,
        queue,
    })
}

fn with_gpu<T>(f: impl FnOnce(&GpuCtx) -> Result<T, String>) -> Result<T, String> {
    if !is_gpu_audio_enabled() {
        return Err("gpu off".into());
    }
    ensure_device()?;
    let guard = gpu_slot().lock().map_err(|e| e.to_string())?;
    let ctx = guard.as_ref().ok_or_else(|| "gpu missing".to_string())?;
    f(ctx)
}

/// Peak scan + gain apply. Returns true if GPU handled it.
pub fn normalize_range(samples: &mut [f32], target: f32) -> bool {
    if samples.is_empty() {
        return true;
    }
    with_gpu(|ctx| {
        let peak = gpu_abs_peak(ctx, samples)?;
        if peak <= 1e-9 {
            return Ok(());
        }
        gpu_apply_gain(ctx, samples, target / peak)
    })
    .is_ok()
}

/// Waveform bucket min/max pairs. `None` = use CPU.
pub fn bucket_peaks(samples: &[f32], buckets: usize) -> Option<Vec<f32>> {
    if samples.is_empty() || buckets == 0 {
        return Some(vec![0.0; buckets.max(1) * 2]);
    }
    with_gpu(|ctx| gpu_bucket_peaks(ctx, samples, buckets)).ok()
}

/// Multiply `frames` windows of `n_fft` samples by `window` (STFT prep).
/// `padded` is the reflect-padded mono signal. Returns `frames * n_fft` windowed samples.
pub fn window_frames(padded: &[f32], window: &[f32], hop: usize, n_fft: usize) -> Option<Vec<f32>> {
    if padded.len() < n_fft || window.len() != n_fft {
        return None;
    }
    let n_frames = 1 + (padded.len() - n_fft) / hop;
    with_gpu(|ctx| gpu_window_frames(ctx, padded, window, hop, n_fft, n_frames)).ok()
}

fn gpu_abs_peak(ctx: &GpuCtx, samples: &[f32]) -> Result<f32, String> {
    const WG: u32 = 256;
    let n = samples.len() as u32;
    let groups = n.div_ceil(WG);
    let params = [n, 0, 0, 0];
    let (staging, _) = dispatch_readback(
        ctx,
        &ctx.peak_pipe,
        bytemuck_bytes(samples),
        bytemuck_bytes(&params),
        (groups as usize) * 4,
        groups,
        1,
    )?;
    let peaks: &[f32] = bytemuck_f32(&staging);
    Ok(peaks.iter().copied().fold(0.0f32, f32::max))
}

fn gpu_apply_gain(ctx: &GpuCtx, samples: &mut [f32], gain: f32) -> Result<(), String> {
    const WG: u32 = 256;
    let n = samples.len() as u32;
    let groups = n.div_ceil(WG);
    let params = [n, gain.to_bits(), 0, 0];
    let (out, _) = dispatch_readback(
        ctx,
        &ctx.gain_pipe,
        bytemuck_bytes(samples),
        bytemuck_bytes(&params),
        samples.len() * 4,
        groups,
        1,
    )?;
    samples.copy_from_slice(bytemuck_f32(&out));
    Ok(())
}

fn gpu_bucket_peaks(ctx: &GpuCtx, samples: &[f32], buckets: usize) -> Result<Vec<f32>, String> {
    let n = samples.len() as u32;
    let b = buckets as u32;
    let params = [n, b, 0, 0];
    let (out, _) = dispatch_readback(
        ctx,
        &ctx.bucket_pipe,
        bytemuck_bytes(samples),
        bytemuck_bytes(&params),
        buckets * 2 * 4,
        b,
        1,
    )?;
    Ok(bytemuck_f32(&out).to_vec())
}

fn gpu_window_frames(
    ctx: &GpuCtx,
    padded: &[f32],
    window: &[f32],
    hop: usize,
    n_fft: usize,
    n_frames: usize,
) -> Result<Vec<f32>, String> {
    let params = [
        padded.len() as u32,
        window.len() as u32,
        hop as u32,
        n_fft as u32,
        n_frames as u32,
        0,
        0,
        0,
    ];
    // Extra bind: window buffer packed after params via a second input? Use concatenated input.
    let mut input = Vec::with_capacity(padded.len() + window.len());
    input.extend_from_slice(padded);
    input.extend_from_slice(window);
    let (out, _) = dispatch_readback(
        ctx,
        &ctx.window_pipe,
        bytemuck_bytes(&input),
        bytemuck_bytes(&params),
        n_frames * n_fft * 4,
        n_frames as u32,
        1,
    )?;
    Ok(bytemuck_f32(&out).to_vec())
}

fn dispatch_readback(
    ctx: &GpuCtx,
    pipe: &wgpu::ComputePipeline,
    input: &[u8],
    params: &[u8],
    out_bytes: usize,
    x: u32,
    y: u32,
) -> Result<(Vec<u8>, ()), String> {
    use wgpu::*;
    let in_buf = ctx.device.create_buffer(&BufferDescriptor {
        label: Some("in"),
        size: input.len() as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&in_buf, 0, input);

    let param_buf = ctx.device.create_buffer(&BufferDescriptor {
        label: Some("params"),
        size: params.len().max(16) as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&param_buf, 0, params);

    let out_buf = ctx.device.create_buffer(&BufferDescriptor {
        label: Some("out"),
        size: out_bytes as u64,
        usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let read_buf = ctx.device.create_buffer(&BufferDescriptor {
        label: Some("read"),
        size: out_bytes as u64,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let layout = pipe.get_bind_group_layout(0);
    let bg = ctx.device.create_bind_group(&BindGroupDescriptor {
        label: Some("dsp-bg"),
        layout: &layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: in_buf.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: out_buf.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: param_buf.as_entire_binding(),
            },
        ],
    });

    let mut enc = ctx.device.create_command_encoder(&CommandEncoderDescriptor { label: Some("dsp") });
    {
        let mut pass = enc.begin_compute_pass(&ComputePassDescriptor {
            label: Some("dsp-pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipe);
        pass.set_bind_group(0, &bg, &[]);
        pass.dispatch_workgroups(x.max(1), y.max(1), 1);
    }
    enc.copy_buffer_to_buffer(&out_buf, 0, &read_buf, 0, out_bytes as u64);
    ctx.queue.submit(Some(enc.finish()));

    let slice = read_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    ctx.device.poll(Maintain::Wait);
    rx.recv().map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;
    let data = {
        let mapped = slice.get_mapped_range();
        mapped.to_vec()
    };
    read_buf.unmap();
    Ok((data, ()))
}

fn bytemuck_bytes<T>(v: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v)) }
}

fn bytemuck_f32(v: &[u8]) -> &[f32] {
    let n = v.len() / 4;
    unsafe { std::slice::from_raw_parts(v.as_ptr() as *const f32, n) }
}

const PEAK_WGSL: &str = r#"
struct Params { n: u32, _p0: u32, _p1: u32, _p2: u32 }
@group(0) @binding(0) var<storage, read> samples: array<f32>;
@group(0) @binding(1) var<storage, read_write> out_peaks: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

var<workgroup> scratch: array<f32, 256>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>,
        @builtin(local_invocation_id) lid: vec3<u32>,
        @builtin(workgroup_id) wid: vec3<u32>) {
    let i = gid.x;
    var v = 0.0;
    if (i < params.n) { v = abs(samples[i]); }
    scratch[lid.x] = v;
    workgroupBarrier();
    var stride = 128u;
    while (stride > 0u) {
        if (lid.x < stride) {
            scratch[lid.x] = max(scratch[lid.x], scratch[lid.x + stride]);
        }
        workgroupBarrier();
        stride = stride / 2u;
    }
    if (lid.x == 0u) {
        out_peaks[wid.x] = scratch[0];
    }
}
"#;

const GAIN_WGSL: &str = r#"
struct Params { n: u32, gain_bits: u32, _p0: u32, _p1: u32 }
@group(0) @binding(0) var<storage, read> samples: array<f32>;
@group(0) @binding(1) var<storage, read_write> out_samples: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= params.n) { return; }
    let gain = bitcast<f32>(params.gain_bits);
    out_samples[i] = clamp(samples[i] * gain, -1.0, 1.0);
}
"#;

const BUCKET_WGSL: &str = r#"
struct Params { n: u32, buckets: u32, _p0: u32, _p1: u32 }
@group(0) @binding(0) var<storage, read> samples: array<f32>;
@group(0) @binding(1) var<storage, read_write> out_peaks: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let b = gid.x;
    if (b >= params.buckets) { return; }
    let start = b * params.n / params.buckets;
    var end = (b + 1u) * params.n / params.buckets;
    if (end <= start) { end = start + 1u; }
    var mn = 0.0;
    var mx = 0.0;
    var i = start;
    loop {
        if (i >= end || i >= params.n) { break; }
        let a = abs(samples[i]);
        mn = min(mn, -a);
        mx = max(mx, a);
        i = i + 1u;
    }
    out_peaks[b * 2u] = mn;
    out_peaks[b * 2u + 1u] = mx;
}
"#;

const WINDOW_WGSL: &str = r#"
struct Params { padded_len: u32, win_len: u32, hop: u32, n_fft: u32, n_frames: u32, _p0: u32, _p1: u32, _p2: u32 }
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> out_frames: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let frame = gid.x;
    if (frame >= params.n_frames) { return; }
    let start = frame * params.hop;
    let win_off = params.padded_len;
    var i = 0u;
    loop {
        if (i >= params.n_fft) { break; }
        let s = input[start + i];
        let w = input[win_off + i];
        out_frames[frame * params.n_fft + i] = s * w;
        i = i + 1u;
    }
}
"#;
