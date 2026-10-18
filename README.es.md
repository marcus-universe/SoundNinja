# Sound Ninja

[English](./README.md) · [Deutsch](./README.de.md) · **Español** · [Français](./README.fr.md) · [日本語](./README.ja.md) · [简体中文](./README.zh-Hans.md)

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
<b>Sound Ninja</b> es una <b>app de soundboard de código abierto</b> con la máxima personalización para crear tu mejor soundboard. Perfecta para **directos, charlar con amigos o grabar pódcasts**. 😉👍


## 🥷 ¿Por qué SoundNinja?
Los pilares de SoundNinja son el rendimiento y la personalización.

La mayoría de las apps de soundboard se leen mal y se controlan con torpeza en pantallas pequeñas o dentro de VR. SoundNinja lo resuelve con una solución muy accesible, personalizable y totalmente de código abierto, pensada para adaptarse a tu setup.

### 🚀 Rendimiento primero
Hecho en Rust con **Tauri v2** y un frontend versátil con **Nuxt 4**. SoundNinja es muy ligero. A diferencia de las alternativas pesadas, ocupa poco disco y gasta muy poco RAM y CPU.

### 🎯 Hecho para la velocidad y la personalización
La misión era un soundboard con el que organices bibliotecas enormes sin fricción. Encuentra el sonido exacto en milisegundos, justo cuando lo necesitas.


| Función                      | Descripción                                               | Estado |
| ---------------------------- | --------------------------------------------------------- | ------ |
| **Soundboard**               | Crea tu soundboard con tus sonidos favoritos              | ✅     |
| **Sistema de pestañas**      | Organiza los sonidos en pestañas                          | ✅     |
| **Buscador**                 | Encuentra tus sonidos rápido y fácil                      | ✅     |
| **Temas**                    | Diseña el soundboard como quieras                         | ✅     |
| **Perfiles propios**         | Varios soundboards con ajustes distintos                  | ✅     |
| **Grabadora**                | Graba el audio del PC directamente en Sound Ninja         | ✅     |
| **GIFs e imágenes**          | Pon una imagen o un GIF a tus sonidos                     | ✅     |
| **Atajos personalizables**   | Ajusta los atajos a lo que necesitas                      | ✅     |
| **Control Companion**        | Dispara sonidos desde Bitfocus Companion por HTTP/WebSocket | ✅     |
| **Compartir soundboard**     | Comparte tu soundboard con tus amigos                     | ✅     |
| **Importar/exportar**        | Importa y exporta tu soundboard                           | ✅     |
| **Sistema de etiquetas**     | Etiqueta tus sonidos favoritos por categorías             | ✅     |
| **Soporte MIDI**             | Controla el soundboard por MIDI                           | ⛔     |
| **Búsqueda por voz**         | Encuentra sonidos con tu voz                              | ⛔     |
| **Control por chat de Twitch** | Tu comunidad decide qué se reproduce                    | ⛔     |
| **Asistente de IA**          | Recomienda sonidos que encajan en la situación            | ⛔     |

## Docs

- [Web](https://soundninja.marcus-universe.com/)
- [Guía de usuario](https://soundninja.marcus-universe.com/docs)
- [Compilar desde el código](./docs/build-from-source.md) (inglés)
- [Entorno de desarrollo](./docs/development.md) (inglés)
- [Companion / Remote](./docs/companion.md) (inglés)
- [Modelo de separación de stems](./docs/stems-model.md) (inglés)
- [Actualizador de la app](./docs/updater.md) (inglés)

---
<img src="./designs/download.png" alt="Descargas" width="200" />

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

Los instaladores de la app están en `/releases/latest/download/…`. El módulo de Companion en `.tgz` está en la página de releases de [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja/releases/latest).

### Arch Linux

Los paquetes precompilados `soundninja-bin` (`x86_64` y `aarch64`) se publican en GitHub Releases. **Aún no** están en [aur.archlinux.org](https://aur.archlinux.org), así que `yay -S soundninja` no los encuentra. Instala con curl:

```bash
arch=$(uname -m)   # x86_64 or aarch64
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-bin-[^"]+-'"${arch}"'\.pkg\.tar\.zst' | head -n1)
curl -fLO "$url"
sudo pacman -U ./soundninja-bin-*-"${arch}".pkg.tar.zst
```

O compila desde el tarball de fuentes AUR (`makepkg`):

```bash
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-aur-[^"]+\.tar\.gz' | head -n1)
curl -fLO "$url"
mkdir -p soundninja-bin && tar xf soundninja-aur-*.tar.gz -C soundninja-bin
cd soundninja-bin && makepkg -si
```

**Actions → Release** crea un **borrador** de GitHub Release (`vX.Y.Z`) si no existe, y adjunta solo las plataformas que todavía no están en el borrador. Recompila un objetivo con **Release Windows**, **Release Linux**, **Release macOS ARM** o **Release macOS Intel**. Cada action sube el instalador (más un alias estable) a ese borrador. Módulo de Companion: [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja) (`SoundNinja-Companion.tgz` y `SoundNinja-Companion-{Version}.tgz`).

## Plataformas

Builds oficiales (x86_64 salvo que se indique otra cosa):

| Plataforma | Soporte |
| :------- | :------ |
| Windows 10+ | Instalador NSIS (WebView2) |
| macOS 10.15+ | DMG por arquitectura (Apple Silicon, Intel) |
| Linux x86_64 | `.deb` (Debian/Ubuntu, WebKitGTK 4.1) o AppImage. PipeWire o PulseAudio para el sonido. |
| Arch Linux | `soundninja-bin` desde GitHub Releases (`pacman -U`, x86_64 + aarch64). Aún no en aur.archlinux.org. |
| Linux ARM | `.deb` para `arm64` / `armhf`. |

Desarrollo temprano: no está probado en todas las distros. Informa de cualquier problema para que podamos corregirlo.

## Créditos

- [lucidrains/BS-RoFormer](https://github.com/lucidrains/BS-RoFormer) — Band-Split RoPE Transformer, la arquitectura de la separación de stems por IA de Sound Ninja en el Record Editor. Licencia MIT.
- Pesos entrenados por viperx; exportación ONNX de [ZFTurbo/MSS_ONNX_TensorRT](https://github.com/ZFTurbo/MSS_ONNX_TensorRT).
- [KLIPY](https://klipy.com) — búsqueda de GIFs en línea (tu propia API key).
- [GifSnap](https://gifsnap.com) — búsqueda de GIFs sin key (API pública, best effort). Los resultados se guardan en caché local para reducir peticiones.
