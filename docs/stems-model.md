# Stem Separation Model (HTDemucs-ORT)

SoundNinja’s Record Editor can split a recording into **vocals** or **music** (drums + bass + other) using AI stem separation.

The model is **not bundled** with the app. It is a third-party ONNX model that must be downloaded (about **~200 MB**) before the feature works.

## Model details

| | |
| --- | --- |
| **Name** | HTDemucs-ORT (`htdemucs_ort_v1`) |
| **Type** | Hybrid Transformer Demucs (Meta Demucs v4 lineage), converted for ONNX Runtime |
| **Provider** | [gentij/htdemucs-ort on Hugging Face](https://huggingface.co/gentij/htdemucs-ort) |
| **Library** | [stem-splitter-core](https://github.com/gentij/stem-splitter-core) |
| **Size** | ~200 MB |
| **Stems** | vocals, drums, bass, other |

Model page: https://huggingface.co/gentij/htdemucs-ort  
Manifest (used by the downloader): https://huggingface.co/gentij/htdemucs-ort/resolve/main/manifest.json

## Requirements

1. **Stem engine in the build**  
   The app must be compiled with the Cargo feature `stems`:

   ```bash
   npm run tauri:serve:stems
   # or
   npm run tauri:build:stems
   ```

2. **Windows / MSVC**  
   Prebuilt ONNX Runtime libs need a recent MSVC toolchain:
   - Visual Studio 2022 **17.14+**
   - MSVC toolset **14.44+**

   With older toolsets (e.g. 14.43), linking fails with unresolved symbols such as `__std_find_first_of_trivial_pos_*`. Update Visual Studio, then rebuild with `--features stems`.

3. **Disk space & network**  
   First use downloads ~200 MB. Keep a stable connection.

## Install via the Record Editor (recommended)

1. Open **Record Editor** from the player bar.
2. Record or load a session.
3. Click **Stems** → **Keep Vocals** or **Keep Music**.
4. If the model is missing, a dialog appears:
   - Model name and size
   - Third-party notice
   - Link to the Hugging Face page
5. Click **Yes** / **Ja**.
6. Wait for the download progress to finish.
7. Separation runs automatically after the model is ready.

Next time the model is already cached, the dialog is skipped and stems run directly.

### Cache location

`stem-splitter-core` stores models under the OS cache for project  
`dev.StemSplitter.stem-splitter-core`, typically:

| OS | Path |
| --- | --- |
| Windows | `%LOCALAPPDATA%\StemSplitter\stem-splitter-core\cache\models\` |
| macOS | `~/Library/Caches/dev.StemSplitter.stem-splitter-core/models/` |
| Linux | `~/.cache/stem-splitter-core/models/` |

Files are named from the manifest (name + SHA prefix). Do not rename them if you place files manually.

## Manual / offline notes

SoundNinja’s normal path is **in-app download** via `ensure_stems_model` / `prepare_model("htdemucs_ort_v1")`.

If you need to inspect or mirror the model:

1. Open https://huggingface.co/gentij/htdemucs-ort
2. Check the repo files and `manifest.json` for the exact ONNX artifact URL and SHA-256.
3. Prefer letting the app download so checksum verification matches the library.

Manual copy into the cache folder only works if the filename and checksum match what `stem-splitter-core` expects.

## Troubleshooting

| Problem | What to do |
| --- | --- |
| “Stem engine not compiled” / dialog says engine missing | Rebuild with `npm run tauri:serve:stems` after updating VS/MSVC. |
| Link / “Open model page” does nothing | Needs `opener` permission (app capability). Restart the app after updating. Or open https://huggingface.co/gentij/htdemucs-ort in a browser. |
| Download stuck / fails | Check network, disk space, firewall; retry **Yes**. |
| Link error `__std_find_first_of_trivial_pos_*` | Update Visual Studio 2022 to 17.14+ and rebuild with `--features stems`. |
| Separation slow | First run may warm ORT; GPU providers (e.g. DirectML) are used when available. |

## License / attribution

- App code: see the SoundNinja repository license.
- Model and ONNX export: follow the terms on the Hugging Face model page and upstream Demucs / stem-splitter-core projects. SoundNinja does **not** host or redistribute the model weights.