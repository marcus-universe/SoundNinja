# Sound Ninja

![Sound Ninja Logo Animated](./designs/Logo_Animated.gif)

<div align="center">

[![Website](https://img.shields.io/badge/Website-GitHub_Pages-29d4ff?style=for-the-badge)](https://marcus-universe.github.io/SoundNinja/)
[![Docs](https://img.shields.io/badge/Docs-User_guide-29d4ff?style=for-the-badge)](https://marcus-universe.github.io/SoundNinja/docs)
<br>
![License](https://img.shields.io/github/license/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Version](https://img.shields.io/github/package-json/v/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Stars](https://img.shields.io/github/stars/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Forks](https://img.shields.io/github/forks/marcus-universe/SoundNinja.svg)

</div>
<b>Sound Ninja</b> is an <b>Open Source Soundboard App</b> with maximal customizability option to create your best Soundboard. Perfect for **live streaming, chatting with friends or podcast recording**. 😉👍


## 🥷 Why SoundNinja?
The core pillars of SoundNinja are high performance and deep customizability.

Most existing soundboard apps are difficult to read and clunky to control on small displays or inside VR environments. SoundNinja solves this by providing a highly accessible, customizable, and fully open-source solution designed to adapt to your specific setup.

### 🚀 Performance First
Built in Rust with **Tauri v2** and versitile Frontend with **Nuxt 4**, SoundNinja is incredibly lightweight. Unlike heavy alternative soundboard apps, it keeps its storage footprint tiny and stays ultra-lean on RAM & CPU usage.

### 🎯 Built for Speed & Customizability
The core mission was to build a soundboard that empowers you to organize massive libraries efficiently. Find the exact sound you need in milliseconds—right when you need it most.


| Feature                      | Description                                               | Status |
| ---------------------------- | --------------------------------------------------------- | ------ |
| **Soundboard**               | Create your own Soundboard with your favorite sounds      | ✅     |
| **Tab System**               | Organize your sounds in Tabs                              | ✅     |
| **Searchbar**                | Find your sounds quick and easy                           | ✅     |
| **Themes**                   | Design your Soundboard how you like it                    | ✅     |
| **Custom Profiles**          | Create multiple Soundboards with different settings       | ✅     |
| **Recorder**                 | Record your PC-Audio directly in Sound Ninja              | ✅     |
| **Gifs & Images**            | Give your Sounds an Image/Gif                             | ✅     |
| **Customizable Hotkeys**     | Customize the Hotkeys to your needs                       | ✅     |
| **Companion Remote**         | Trigger sounds from Bitfocus Companion over HTTP/WebSocket | ✅     |
| **Soundboard Sharing**       | Share your Soundboard with your friends                   | ✅     |
| **Soundboard Import/Export** | Import and Export your Soundboard                         | ✅     |
| **Tag System**               | Tag your favorite sounds in to cathegories                | ✅     |
| **Midi Support**             | Control your Soundboard via Midi                          | ⛔     |
| **Speech Search**            | Find sounds with the power of your voice                  | ⛔     |
| **Twitch Chat Control**      | Let your community decide what to play                    | ⛔     |
| **AI Assistant**             | Recommends you sounds that works in the current situation | ⛔     |

## Docs

- [Website](https://marcus-universe.github.io/SoundNinja/)
- [User guide](https://marcus-universe.github.io/SoundNinja/docs)
- [Build from source](./docs/build-from-source.md)
- [Development setup](./docs/development.md)
- [Companion / Remote](./docs/companion.md)
- [Stem separation model](./docs/stems-model.md)
- [App updater](./docs/updater.md)

---
<img src="./designs/download.png" alt="Downloads" width="200" />

[![Windows NSIS](https://img.shields.io/badge/Windows-NSIS-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-windows-x64-setup.exe)
<br>
<br>
[![macOS Apple Silicon](https://img.shields.io/badge/macOS-Apple_Silicon-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-macos-arm64.dmg)
[![macOS Intel](https://img.shields.io/badge/macOS-Intel-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-macos-x64.dmg)
<br>
<br>
[![Linux deb](https://img.shields.io/badge/Linux-.deb-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-linux-amd64.deb)
[![Linux AppImage](https://img.shields.io/badge/Linux-AppImage-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-linux-amd64.AppImage)
<br>
<br>
[![Companion](https://img.shields.io/badge/Companion-Module-111111?style=for-the-badge)](https://github.com/marcus-universe/companion-module-soundninja/releases/latest)
<br>
<br>
[![Build from source](https://img.shields.io/badge/Build-from_source-29d4ff?style=for-the-badge)](./docs/build-from-source.md)
<br>
<br>

App installers live on `/releases/latest/download/…`. The Companion module `.tgz` is on the [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja/releases/latest) release page.

Run **Actions → Release** — it creates a GitHub Release **draft** (`vX.Y.Z`) if none exists, then attaches only platforms that are not already on the draft. Rebuild one target with **Release Windows**, **Release Linux**, **Release macOS ARM**, or **Release macOS Intel**. Each action uploads the installer (plus a stable alias) onto that draft. Companion module: [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja) (`SoundNinja-Companion.tgz` and `SoundNinja-Companion-{Version}.tgz`).

## Platforms

Official builds (x86_64 unless noted):

| Platform | Support |
| :------- | :------ |
| Windows 10+ | NSIS installer (WebView2) |
| macOS 10.15+ | DMG per architecture (Apple Silicon, Intel) |
| Linux x86_64 | `.deb` (Debian/Ubuntu, WebKitGTK 4.1) or AppImage. PipeWire or PulseAudio for sound. |

Early development — not tested on every distro. Please report any issues you encounter so we can fix them.

## Credits

- [lucidrains/BS-RoFormer](https://github.com/lucidrains/BS-RoFormer) — Band-Split RoPE Transformer, the architecture powering Sound Ninja's AI stem separation in the Record Editor. MIT licensed.
- Weights trained by viperx; ONNX export tooling by [ZFTurbo/MSS_ONNX_TensorRT](https://github.com/ZFTurbo/MSS_ONNX_TensorRT).
