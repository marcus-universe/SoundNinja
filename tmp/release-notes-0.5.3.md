# 🥷 SoundNinja v0.5.3 is Here!

A big jump from **0.5.0** — recording & playback tools, deeper theming, GIF button art, named **Groups**, optional **stem separation**, and in-app updates. Same `.sninja` projects; open and go.

## 🚀 What's New & Exciting
* **🎙️ Recording Editor** — Capture from input or loopback, trim, and land clips straight on your board.
* **▶️ Player Bar & Playing List** — See what's playing, pause/seek, and keep multi-sound sessions under control.
* **🎨 Theme Creator Overhaul** — Fresh token-based colors for buttons, tabs, and surfaces. Legacy theme modes cleaned out.
* **🖼️ GIF Button Backgrounds** — Klipy search or local GIF/PNG/WebP, with theme overlay controls so text stays readable.
* **📦 Sound Groups** — Separators grow up: named bordered groups, drag the group (children come along), drop sounds in/out, per-group border/name colors, and button alignment (tab default or group override). Old projects still load.
* **🧩 Stem Separation (BS-RoFormer)** — Optional vocals / instrumental split via ONNX (Windows installer checkbox; Linux/macOS can download the model). See `docs/stems-model.md`.
* **🔄 In-App Updates** — Check on start or from the menu; signed updater artifacts + `latest.json` (see `docs/updater.md`).
* **↩️ Undo / Redo** — Step through soundboard edits with shortcuts and menu actions.

## 🛠️ Polish & Bug Fixes
* **🔊 Long recordings** — Fixed pop/crackle on captures longer than ~8 seconds.
* **🎧 Audio device selection** — Sturdier host/device fallback when outputs change.
* **🖱️ Context menus & i18n** — Group actions, GIF picker, updater strings in **English** and **Deutsch**.
* **💎 UI refinements** — Player visuals, dialogs, navbar, and settings tips tightened across the app.

**Downloads**
- Windows: `.exe` / `.msi` installer
- macOS: universal `.dmg` (Apple Silicon + Intel)
- Linux: `.deb` (Ubuntu/Debian) and `.AppImage`
- Arch: install via AUR (`soundninja-bin`) after this release is published

In-app updates use signed updater artifacts + `latest.json` (see `docs/updater.md`).

> **Note:** This draft’s installers may ship without signed updater payloads until the `TAURI_SIGNING_PRIVATE_KEY` Actions secret is configured. Manual download/install still works.

**Full Changelog**: https://github.com/marcus-universe/SoundNinja/compare/v0.5.0...v0.5.3
