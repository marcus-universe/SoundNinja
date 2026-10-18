# Sound Ninja

[English](./README.md) · [Deutsch](./README.de.md) · [Español](./README.es.md) · [Français](./README.fr.md) · [日本語](./README.ja.md) · **简体中文**

![Sound Ninja Logo Animated](./designs/Logo_Animated.gif)

<div align="center">

[![Website](https://img.shields.io/badge/Website-soundninja.marcus--universe.com-29d4ff?style=for-the-badge)](https://soundninja.marcus-universe.com/)
[![Docs](https://img.shields.io/badge/Docs-User_guide-29d4ff?style=for-the-badge)](https://soundninja.marcus-universe.com/docs)
<br>
![License](https://img.shields.io/github/license/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Version](https://img.shields.io/github/package-json/v/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Stars](https://img.shields.io/github/stars/marcus-universe/SoundNinja?style=for-the-badge.svg)
![Forks](https://img.shields.io/github/forks/marcus-universe/SoundNinja.svg)

</div>
<b>Sound Ninja</b> 是一款可高度自定义的 <b>开源音效板应用</b>，用来做出你自己的音效板。适合**直播、和朋友聊天，或录播客**。😉👍


## 🥷 为什么是 SoundNinja？
SoundNinja 的核心是高性能和深度自定义。

现有的音效板应用大多难读，在小屏幕或 VR 里也不好操作。SoundNinja 用一套易用、可定制、完全开源的方案来解决这个问题，并适应你的设备环境。

### 🚀 性能优先
用 Rust、**Tauri v2** 和灵活的 **Nuxt 4** 前端写成。SoundNinja 非常轻。和笨重的同类应用不同，它占的磁盘很小，RAM 和 CPU 占用也很低。

### 🎯 为速度和自定义而做
目标是让你高效整理超大音效库。需要某个音效时，几毫秒内就能找到。


| 功能                         | 说明                                                      | 状态 |
| ---------------------------- | --------------------------------------------------------- | ------ |
| **音效板**                   | 用喜欢的音效做自己的音效板                                | ✅     |
| **标签页**                   | 用标签页整理音效                                          | ✅     |
| **搜索栏**                   | 快速、简单地找到音效                                      | ✅     |
| **主题**                     | 按自己的喜好设计音效板                                    | ✅     |
| **自定义配置**               | 做多套设置不同的音效板                                    | ✅     |
| **录音器**                   | 在 Sound Ninja 里直接录制电脑音频                         | ✅     |
| **GIF 与图片**               | 给音效配上图片或 GIF                                      | ✅     |
| **可自定义热键**             | 按需要改热键                                              | ✅     |
| **Companion 遥控**           | 用 Bitfocus Companion 通过 HTTP/WebSocket 触发音效        | ✅     |
| **分享音效板**               | 把音效板分享给朋友                                        | ✅     |
| **导入 / 导出**              | 导入和导出音效板                                          | ✅     |
| **标签系统**                 | 把喜欢的音效按分类打标签                                  | ✅     |
| **MIDI 支持**                | 用 MIDI 控制音效板                                        | ⛔     |
| **语音搜索**                 | 用声音找音效                                              | ⛔     |
| **Twitch 聊天控制**          | 让社区决定播放什么                                        | ⛔     |
| **AI 助手**                  | 推荐适合当前场景的音效                                    | ⛔     |

## 文档

- [网站](https://soundninja.marcus-universe.com/)
- [用户指南](https://soundninja.marcus-universe.com/docs)
- [从源码构建](./docs/build-from-source.md)（英文）
- [开发环境](./docs/development.md)（英文）
- [Companion / Remote](./docs/companion.md)（英文）
- [分轨模型](./docs/stems-model.md)（英文）
- [应用更新器](./docs/updater.md)（英文）

---
<img src="./designs/download.png" alt="下载" width="200" />

[![Windows NSIS](https://img.shields.io/badge/Windows-NSIS-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-windows-x64-setup.exe)
<br>
<br>
[![macOS Apple Silicon](https://img.shields.io/badge/macOS-Apple_Silicon-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-macos-arm64.dmg)
[![macOS Intel](https://img.shields.io/badge/macOS-Intel-000000?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-macos-x64.dmg)
<br>
<br>
[![Linux deb](https://img.shields.io/badge/Linux-.deb-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-linux-amd64.deb)
[![Linux AppImage](https://img.shields.io/badge/Linux-AppImage-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/marcus-universe/SoundNinja/releases/latest/download/soundninja-linux-amd64.AppImage)
[![Arch](https://img.shields.io/badge/Arch-pkg.tar.zst-1793D1?style=for-the-badge&logo=archlinux&logoColor=white)](#arch-linux)
<br>
<br>
[![Companion](https://img.shields.io/badge/Companion-Module-111111?style=for-the-badge)](https://github.com/marcus-universe/companion-module-soundninja/releases/latest)
<br>
<br>
[![Build from source](https://img.shields.io/badge/Build-from_source-29d4ff?style=for-the-badge)](./docs/build-from-source.md)
<br>
<br>

应用安装包在 `/releases/latest/download/…`。Companion 模块的 `.tgz` 在 [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja/releases/latest) 的发布页。

### Arch Linux

预编译的 `soundninja-bin` 包（`x86_64` 和 `aarch64`）发在 GitHub Releases 上。它们**还没有**上 [aur.archlinux.org](https://aur.archlinux.org)，所以 `yay -S soundninja` 找不到。用 curl 安装：

```bash
arch=$(uname -m)   # x86_64 or aarch64
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-bin-[^"]+-'"${arch}"'\.pkg\.tar\.zst' | head -n1)
curl -fLO "$url"
sudo pacman -U ./soundninja-bin-*-"${arch}".pkg.tar.zst
```

也可以用 AUR 源码包构建（`makepkg`）：

```bash
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-aur-[^"]+\.tar\.gz' | head -n1)
curl -fLO "$url"
mkdir -p soundninja-bin && tar xf soundninja-aur-*.tar.gz -C soundninja-bin
cd soundninja-bin && makepkg -si
```

**Actions → Release** 会在还没有 GitHub Release **草稿**（`vX.Y.Z`）时创建一个，并且只附上草稿里还没有的平台。用 **Release Windows**、**Release Linux**、**Release macOS ARM** 或 **Release macOS Intel** 重编某一个目标。每个 Action 会把安装包（以及一个稳定别名）上传到这份草稿。Companion 模块：[companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja)（`SoundNinja-Companion.tgz` 和 `SoundNinja-Companion-{Version}.tgz`）。

## 平台

官方构建（除非另注，均为 x86_64）：

| 平台 | 支持 |
| :------- | :------ |
| Windows 10+ | NSIS 安装程序（WebView2） |
| macOS 10.15+ | 按架构提供 DMG（Apple Silicon、Intel） |
| Linux x86_64 | `.deb`（Debian/Ubuntu，WebKitGTK 4.1）或 AppImage。声音用 PipeWire 或 PulseAudio。 |
| Arch Linux | GitHub Releases 上的 `soundninja-bin`（`pacman -U`，x86_64 + aarch64）。尚未上 aur.archlinux.org。 |
| Linux ARM | 面向 `arm64` / `armhf` 的 `.deb`。 |

还在早期开发，没有在每个发行版上测试。遇到问题请反馈，方便我们修。

## 致谢

- [lucidrains/BS-RoFormer](https://github.com/lucidrains/BS-RoFormer) — Band-Split RoPE Transformer，Sound Ninja 在 Record Editor 里做 AI 分轨用的架构。MIT 许可。
- 权重由 viperx 训练；ONNX 导出工具来自 [ZFTurbo/MSS_ONNX_TensorRT](https://github.com/ZFTurbo/MSS_ONNX_TensorRT)。
- [KLIPY](https://klipy.com) — 在线 GIF 搜索（使用你自己的 API key）。
- [GifSnap](https://gifsnap.com) — 无需 key 的 GIF 搜索（尽力而为的公开 API）。搜索结果会缓存在本地，以减少请求。
