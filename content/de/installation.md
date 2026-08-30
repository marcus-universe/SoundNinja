---
title: Installation
description: SoundNinja unter Windows, macOS oder Linux installieren.
order: 2
---

# Installation

Hol dir einen Build von der [Download-Seite](/download) oder von [GitHub Releases](https://github.com/marcus-universe/SoundNinja/releases/latest). SoundNinja ist in früher Entwicklung — behandle Builds als Beta.

## Windows

- **Installer (.exe)** oder **MSI (.msi)** für 64-Bit-Windows
- Landing-Page-Ziel: Windows 10 und neuer
- Die App-README nennt auch Windows 8 und höher

Installer starten, dann SoundNinja über das Startmenü öffnen.

## macOS

- **Universelles .dmg** für Apple Silicon und Intel
- Landing-Page-Ziel: macOS 10.14 und neuer
- Die App-README nennt macOS 10.15 und höher

Disk-Image öffnen und SoundNinja nach Programme ziehen.

## Linux

- **Debian (.deb)** für Ubuntu / Debian
- **AppImage** für andere Distros
- Nach Veröffentlichung einer Release können Arch-Nutzer `soundninja-bin` aus dem AUR installieren
- Getestet unter Ubuntu 20.04

Desktop-Builds brauchen WebKit- und GTK-Pakete. Unter Debian/Ubuntu meist `libwebkit2gtk`, `libgtk-3-0` und eine App-Indicator-Bibliothek.

## Updates

In der App: **Hilfe → Nach Updates suchen**. Optional prüft SoundNinja beim Start in den Einstellungen.

## Aus dem Quellcode bauen

Eigenen Build? [Repo klonen](https://github.com/marcus-universe/SoundNinja), [Tauri-Voraussetzungen](https://v2.tauri.app/start/prerequisites/) installieren, dann:

```bash
bun install
bun run tauri:serve
```

`npm`, `pnpm` oder `deno` gehen auch. Produktionsbuild: `bun run tauri:build`.
