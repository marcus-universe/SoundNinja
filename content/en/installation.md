---
title: Installation
description: Install SoundNinja on Windows, macOS, or Linux.
order: 2
---

# Installation

Grab a build from the [Download page](/download) or from [GitHub Releases](https://github.com/marcus-universe/SoundNinja/releases/latest). SoundNinja is in early development, so treat builds as beta.

## Windows

- **Installer (.exe)** or **MSI (.msi)** for 64-bit Windows
- Landing page target: Windows 10 and newer
- The app README also lists Windows 8 and above

Run the installer, then launch SoundNinja from the Start menu.

## macOS

- **Universal .dmg** for Apple Silicon and Intel
- Landing page target: macOS 10.14 and newer
- The app README lists macOS 10.15 and above

Open the disk image and drag SoundNinja to Applications.

## Linux

- **Debian (.deb)** for Ubuntu / Debian
- **AppImage** for other distros
- After a release is published, Arch users can install `soundninja-bin` from the AUR
- Tested on Ubuntu 20.04

Desktop builds need WebKit and GTK packages. On Debian/Ubuntu that usually means `libwebkit2gtk`, `libgtk-3-0`, and an app-indicator library.

## Updates

In the app, open **Help → Check for Updates**. You can also let SoundNinja check on startup in Settings.

## Build from source

Need a custom build? Clone [the repo](https://github.com/marcus-universe/SoundNinja), install [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/), then:

```bash
bun install
bun run tauri:serve
```

`npm`, `pnpm`, or `deno` work too. Production build: `bun run tauri:build`.
