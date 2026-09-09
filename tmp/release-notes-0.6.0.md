# 🥷 SoundNinja v0.6.0 is Here!

A **massive performance** jump from **0.5.3**. Same `.sninja` projects — open and go. Long clips start fast, and the board stays lighter.

## 🚀 What's New & Exciting
* **⚡ Progressive Playback** — Long sounds (1–4+ min) start after a short preroll instead of decoding the whole file first. A single pump thread fills a few seconds ahead of the playhead; the audio callback only copies samples. Second play is instant from the PCM cache.
* **🖼️ GIF Cache, No 11 MB IPC** — Blobs stay in the project DB, but Rust writes them to a content-addressed file cache and the webview loads via `convertFileSrc`. Hover/scroll no longer ships base64 through `plugin:sql|select`.
* **🧠 Leaner WebView2** — GPU rasterization on, fewer extra Edge processes (`process-per-site`, renderer limit). Hover and navigation cost less CPU.
* **📦 Smaller Frontend** — Locales code-split, settings/overlays lazy-loaded, vendor chunks split. Splash uses a lighter animated asset.
* **🧹 Cleaner Shutdown** — Closing the main window stops audio, drops caches, stops remote, and unregisters hotkeys.

## 🛠️ Polish & Bug Fixes
* **▶️ Missed Clicks / Empty Waveform** — `"default"` output now resolves to the real device. Play no longer blocks the audio thread on a full decode. Duration is reported after preroll so the player wave can draw.
* **🔊 Quiet 24-bit WAVs** — Integer samples scale by bit depth (`2^(bits-1)`), not `i32::MAX`. 16-bit and 24-bit clips play at the same loudness.
* **🌡️ Tab-Open CPU Spike** — Prefetch only warms clips under ~30 s, one file at a time. Long files are byte-warmed, not fully decoded.

## ⚙️ Under the Hood
* **🏗️ Release Profile** — `lto`, `codegen-units = 1`, `panic = "abort"`, stripped symbols.
* **🤖 Faster CI** — Rust cache + sccache, faster linkers, macOS built per arch (not one slow universal job). Windows ships **NSIS only** (no MSI).
* **💾 Bigger Sound Cache** — Defaults raised to **256 / 128 MiB** (old 64/16 projects bump automatically) so multi-minute clips can stay decoded.

**Downloads**
- Windows: `.exe` installer (NSIS)
- macOS: `.dmg` for Apple Silicon and for Intel
- Linux: `.deb` (Ubuntu/Debian) and `.AppImage`
- Companion: `companion-module-soundninja.tgz`

In-app updates use signed updater artifacts + `latest.json` (see `docs/updater.md`).

**Full Changelog**: https://github.com/marcus-universe/SoundNinja/compare/v0.5.3...v0.6.0
