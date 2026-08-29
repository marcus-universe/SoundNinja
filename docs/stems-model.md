# Stem Separation Model (BS-RoFormer)

SoundNinja’s Record Editor can keep **vocals** or **music** (mix − vocals) using AI stem separation.

The model is **not bundled** with the app. It is a third-party ONNX model (~**158 MB**) downloaded on first use (or when the Windows installer checkbox / first-run prompt is accepted).

## Model details

| | |
| --- | --- |
| **Name** | BS-RoFormer (`bs_roformer_ep317_sdr12.9755`) |
| **Type** | Band-Split RoPE Transformer, uint8 ONNX (no-STFT host pipeline) |
| **Architecture** | [lucidrains/BS-RoFormer](https://github.com/lucidrains/BS-RoFormer) |
| **Weights** | viperx `ep_317_sdr_12.9755` (vocals SDR ≈ 12.9 dB on MUSDB18HQ) |
| **ONNX export** | [xycld/BS-RoFormer-ONNX](https://huggingface.co/xycld/BS-RoFormer-ONNX) |
| **Runtime** | ONNX Runtime via Rust `ort` + host STFT/iSTFT (`rustfft` / `realfft`) |
| **Size** | ~158 MB (single-file uint8 quantized) |
| **Stems** | vocals; music = residual (`mix − vocals`) |

Model page: https://huggingface.co/xycld/BS-RoFormer-ONNX  
Direct file: `bs_roformer_ep317_sdr12.9755_quantized_uint8.onnx`

## Requirements

1. **Stem engine in the build**  
   The app must be compiled with the Cargo feature `stems`:

   ```bash
   npm run tauri:serve:stems
   # or
   npm run tauri:build:stems
   ```

   Release CI passes `--features stems` for all platforms.

2. **Windows / MSVC**  
   The stem engine loads ONNX Runtime **dynamically** (`ort` `load-dynamic` + `copy-dylibs`), so older VS 2022 toolsets can link the app. A working MSVC + Windows SDK is still required to build.

3. **Disk space & network**  
   First download is ~158 MB. Keep a stable connection.

## How users get the model

### Windows installer checkbox

The NSIS installer shows a page:

> Download the AI stem separation model on first launch (~158 MB)

Default: **checked**. This only writes  
`HKCU\Software\com.soundninja.dev\WantStemsModel = 1`  
— the download happens inside the app (resumable, with progress UI).

### Linux / macOS first-run prompt

On first launch (deb, AppImage, AUR, DMG), if the model is missing, the same download dialog appears. A `stems-asked` marker under the model directory prevents re-asking after Yes/No.

### Settings escape hatch

**Settings → Stem Separation → Download model** reinstalls or installs after a decline. On Windows/Linux the Record Editor **Stems** button stays hidden until the model is present (or installer intent is still pending).

### Record Editor

1. Open **Record Editor** from the player bar.
2. Record or load a session.
3. Click **Stems** → **Keep Vocals** or **Keep Music**.
4. If the model is missing (and the button is visible), confirm the download dialog.
5. Separation runs after the model is ready.

### Cache location

| OS | Path |
| --- | --- |
| Windows | `%APPDATA%\com.soundninja.dev\models\bs_roformer\` (or portable app-data) |
| macOS | `~/Library/Application Support/com.soundninja.dev/models/bs_roformer/` |
| Linux | `~/.local/share/com.soundninja.dev/models/bs_roformer/` |

Exact base follows Tauri `app_data_dir` / portable-first layout in `paths.rs`.

## Visibility rules

| Platform | Model missing, no intent | Model missing + intent / first-run | Model ready |
| --- | --- | --- | --- |
| Windows | Stems button **hidden** | Prompt / button visible | Visible |
| Linux | Stems button **hidden** (after decline) | First-run prompt | Visible |
| macOS | Button visible; download on use | First-run prompt | Visible |

## Troubleshooting

| Problem | What to do |
| --- | --- |
| “Stem engine not compiled” / dialog says engine missing | Rebuild with `npm run tauri:serve:stems` after updating VS/MSVC. |
| Stems button missing on Windows/Linux | Install via **Settings → Stem Separation**, or reinstall with the checkbox enabled. |
| Download stuck / fails | Check network, disk space, firewall; retry from Settings. |
| Missing `onnxruntime.dll` / `.so` next to the exe | Rebuild with `--features stems` so `copy-dylibs` stages the runtime. |
| Separation slow | uint8 dynamic quantization runs on **CPU** EP; first run warms ORT. |

## License / attribution

- App code: see the SoundNinja repository license.
- Architecture: [lucidrains/BS-RoFormer](https://github.com/lucidrains/BS-RoFormer) (MIT).
- Training / tooling: [ZFTurbo/Music-Source-Separation-Training](https://github.com/ZFTurbo/Music-Source-Separation-Training), [ZFTurbo/MSS_ONNX_TensorRT](https://github.com/ZFTurbo/MSS_ONNX_TensorRT) (MIT).
- Weights: community (viperx); confirm training-data license before commercial redistribution.
- SoundNinja does **not** host or redistribute the model weights.
