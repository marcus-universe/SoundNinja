# Sound Ninja

[English](./README.md) · **Deutsch** · [Español](./README.es.md) · [Français](./README.fr.md) · [日本語](./README.ja.md) · [简体中文](./README.zh-Hans.md)

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
<b>Sound Ninja</b> ist eine <b>Open-Source-Soundboard-App</b> mit maximaler Anpassbarkeit, damit du dein bestes Soundboard bauen kannst. Perfekt für **Livestreams, Chats mit Freunden oder Podcast-Aufnahmen**. 😉👍


## 🥷 Warum SoundNinja?
Die Grundpfeiler von SoundNinja sind hohe Performance und tiefe Anpassbarkeit.

Die meisten Soundboard-Apps sind schwer lesbar und auf kleinen Displays oder in VR umständlich zu bedienen. SoundNinja löst das mit einer gut zugänglichen, anpassbaren und vollständig quelloffenen Lösung, die sich an dein Setup anpasst.

### 🚀 Performance zuerst
Gebaut in Rust mit **Tauri v2** und einem vielseitigen Frontend auf **Nuxt 4**. SoundNinja ist extrem leicht. Im Gegensatz zu schweren Soundboard-Alternativen bleibt der Speicherbedarf klein, und RAM sowie CPU werden kaum belastet.

### 🎯 Gebaut für Tempo und Anpassbarkeit
Ziel war ein Soundboard, mit dem du große Bibliotheken effizient organisierst. Den richtigen Sound findest du in Millisekunden — genau dann, wenn du ihn brauchst.


| Funktion                     | Beschreibung                                              | Status |
| ---------------------------- | --------------------------------------------------------- | ------ |
| **Soundboard**               | Eigenes Soundboard mit deinen Lieblingssounds             | ✅     |
| **Tab-System**               | Sounds in Tabs organisieren                               | ✅     |
| **Suchleiste**               | Sounds schnell und einfach finden                         | ✅     |
| **Themes**                   | Soundboard so gestalten, wie du es willst                 | ✅     |
| **Eigene Profile**           | Mehrere Soundboards mit unterschiedlichen Einstellungen   | ✅     |
| **Recorder**                 | PC-Audio direkt in Sound Ninja aufnehmen                  | ✅     |
| **GIFs und Bilder**          | Sounds ein Bild oder GIF geben                            | ✅     |
| **Anpassbare Hotkeys**       | Hotkeys an deine Bedürfnisse anpassen                     | ✅     |
| **Companion-Fernsteuerung**  | Sounds über Bitfocus Companion per HTTP/WebSocket auslösen | ✅     |
| **Soundboard teilen**        | Soundboard mit Freunden teilen                            | ✅     |
| **Import/Export**            | Soundboard importieren und exportieren                    | ✅     |
| **Tag-System**               | Lieblingssounds in Kategorien taggen                      | ✅     |
| **MIDI-Unterstützung**       | Soundboard per MIDI steuern                               | ⛔     |
| **Sprachsuche**              | Sounds mit der Stimme finden                              | ⛔     |
| **Twitch-Chat-Steuerung**    | Die Community entscheidet, was gespielt wird              | ⛔     |
| **KI-Assistent**             | Schlägt Sounds vor, die zur aktuellen Situation passen    | ⛔     |

## Docs

- [Website](https://soundninja.marcus-universe.com/)
- [Benutzerhandbuch](https://soundninja.marcus-universe.com/docs)
- [Aus dem Quellcode bauen](./docs/build-from-source.md) (Englisch)
- [Entwicklungs-Setup](./docs/development.md) (Englisch)
- [Companion / Remote](./docs/companion.md) (Englisch)
- [Stem-Separation-Modell](./docs/stems-model.md) (Englisch)
- [App-Updater](./docs/updater.md) (Englisch)

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
[![Arch](https://img.shields.io/badge/Arch-pkg.tar.zst-1793D1?style=for-the-badge&logo=archlinux&logoColor=white)](#arch-linux)
<br>
<br>
[![Companion](https://img.shields.io/badge/Companion-Module-111111?style=for-the-badge)](https://github.com/marcus-universe/companion-module-soundninja/releases/latest)
<br>
<br>
[![Build from source](https://img.shields.io/badge/Build-from_source-29d4ff?style=for-the-badge)](./docs/build-from-source.md)
<br>
<br>

Die App-Installer liegen unter `/releases/latest/download/…`. Das Companion-Modul als `.tgz` liegt auf der Release-Seite von [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja/releases/latest).

### Arch Linux

Fertige `soundninja-bin`-Pakete (`x86_64` und `aarch64`) gibt es auf GitHub Releases. Sie sind **noch nicht** auf [aur.archlinux.org](https://aur.archlinux.org), deshalb findet `yay -S soundninja` sie nicht. Installation mit curl:

```bash
arch=$(uname -m)   # x86_64 or aarch64
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-bin-[^"]+-'"${arch}"'\.pkg\.tar\.zst' | head -n1)
curl -fLO "$url"
sudo pacman -U ./soundninja-bin-*-"${arch}".pkg.tar.zst
```

Oder aus dem AUR-Quell-Tarball bauen (`makepkg`):

```bash
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-aur-[^"]+\.tar\.gz' | head -n1)
curl -fLO "$url"
mkdir -p soundninja-bin && tar xf soundninja-aur-*.tar.gz -C soundninja-bin
cd soundninja-bin && makepkg -si
```

**Actions → Release** legt einen GitHub-Release-**Entwurf** (`vX.Y.Z`) an, falls noch keiner existiert, und hängt nur Plattformen an, die noch nicht im Entwurf sind. Ein einzelnes Ziel neu bauen mit **Release Windows**, **Release Linux**, **Release macOS ARM** oder **Release macOS Intel**. Jede Action lädt den Installer (plus einen stabilen Alias) in diesen Entwurf. Companion-Modul: [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja) (`SoundNinja-Companion.tgz` und `SoundNinja-Companion-{Version}.tgz`).

## Plattformen

Offizielle Builds (x86_64, sofern nicht anders angegeben):

| Plattform | Unterstützung |
| :------- | :------ |
| Windows 10+ | NSIS-Installer (WebView2) |
| macOS 10.15+ | DMG je Architektur (Apple Silicon, Intel) |
| Linux x86_64 | `.deb` (Debian/Ubuntu, WebKitGTK 4.1) oder AppImage. PipeWire oder PulseAudio für den Ton. |
| Arch Linux | `soundninja-bin` von GitHub Releases (`pacman -U`, x86_64 + aarch64). Noch nicht auf aur.archlinux.org. |
| Linux ARM | `.deb` für `arm64` / `armhf`. |

Frühe Entwicklung — nicht auf jeder Distribution getestet. Bitte melde Probleme, damit wir sie beheben können.

## Credits

- [lucidrains/BS-RoFormer](https://github.com/lucidrains/BS-RoFormer) — Band-Split RoPE Transformer, die Architektur hinter der KI-Stem-Separation von Sound Ninja im Record Editor. MIT-Lizenz.
- Gewichte trainiert von viperx; ONNX-Export von [ZFTurbo/MSS_ONNX_TensorRT](https://github.com/ZFTurbo/MSS_ONNX_TensorRT).
- [KLIPY](https://klipy.com) — Online-GIF-Suche (eigener API-Key).
- [GifSnap](https://gifsnap.com) — GIF-Suche ohne Key (öffentliche API, best effort). Suchergebnisse werden lokal gecacht, um Anfragen zu reduzieren.
