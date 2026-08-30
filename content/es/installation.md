---
title: Instalación
description: Instala SoundNinja en Windows, macOS o Linux.
order: 2
---

# Instalación

Consigue una build en la [página de descarga](/download) o en [GitHub Releases](https://github.com/marcus-universe/SoundNinja/releases/latest). SoundNinja está en desarrollo temprano, así que trata las builds como beta.

## Windows

- **Instalador (.exe)** o **MSI (.msi)** para Windows de 64 bits
- Objetivo de la landing: Windows 10 y posterior
- El README de la app también lista Windows 8 y superior

Ejecuta el instalador y luego abre SoundNinja desde el menú Inicio.

## macOS

- **.dmg universal** para Apple Silicon e Intel
- Objetivo de la landing: macOS 10.14 y posterior
- El README de la app lista macOS 10.15 y superior

Abre la imagen de disco y arrastra SoundNinja a Aplicaciones.

## Linux

- **Debian (.deb)** para Ubuntu / Debian
- **AppImage** para otras distros
- Tras publicar una release, los usuarios de Arch pueden instalar `soundninja-bin` desde el AUR
- Probado en Ubuntu 20.04

Las builds de escritorio necesitan paquetes WebKit y GTK. En Debian/Ubuntu eso suele ser `libwebkit2gtk`, `libgtk-3-0` y una biblioteca app-indicator.

## Actualizaciones

En la app, abre **Ayuda → Buscar actualizaciones**. También puedes dejar que SoundNinja compruebe al arrancar en Ajustes.

## Compilar desde el código

¿Necesitas una build personalizada? Clona [el repo](https://github.com/marcus-universe/SoundNinja), instala los [requisitos de Tauri](https://v2.tauri.app/start/prerequisites/) y luego:

```bash
bun install
bun run tauri:serve
```

`npm`, `pnpm` o `deno` también sirven. Build de producción: `bun run tauri:build`.
