# Sound Ninja

[English](./README.md) · [Deutsch](./README.de.md) · [Español](./README.es.md) · **Français** · [日本語](./README.ja.md) · [简体中文](./README.zh-Hans.md)

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
<b>Sound Ninja</b> est une <b>application soundboard open source</b> avec un maximum de personnalisation pour créer ton meilleur soundboard. Parfaite pour le **streaming, discuter entre amis ou enregistrer un podcast**. 😉👍


## 🥷 Pourquoi SoundNinja ?
Les piliers de SoundNinja sont la performance et la personnalisation.

La plupart des apps soundboard sont difficiles à lire et pénibles à contrôler sur un petit écran ou en VR. SoundNinja répond avec une solution très accessible, personnalisable et entièrement open source, conçue pour s’adapter à ton setup.

### 🚀 La performance d’abord
Écrit en Rust avec **Tauri v2** et un frontend polyvalent en **Nuxt 4**. SoundNinja est très léger. Contrairement aux alternatives lourdes, il prend peu de place sur le disque et reste très sobre en RAM et en CPU.

### 🎯 Conçu pour la vitesse et la personnalisation
La mission : un soundboard qui permet d’organiser d’énormes bibliothèques sans friction. Tu trouves le son exact en quelques millisecondes, au moment où tu en as besoin.


| Fonction                     | Description                                               | Statut |
| ---------------------------- | --------------------------------------------------------- | ------ |
| **Soundboard**               | Crée ton soundboard avec tes sons préférés                | ✅     |
| **Système d’onglets**        | Organise tes sons dans des onglets                        | ✅     |
| **Barre de recherche**       | Retrouve tes sons vite et simplement                      | ✅     |
| **Thèmes**                   | Dessine le soundboard comme tu le veux                    | ✅     |
| **Profils personnalisés**    | Plusieurs soundboards avec des réglages différents        | ✅     |
| **Enregistreur**             | Enregistre l’audio du PC directement dans Sound Ninja     | ✅     |
| **GIF et images**            | Donne une image ou un GIF à tes sons                      | ✅     |
| **Raccourcis personnalisables** | Adapte les raccourcis à tes besoins                    | ✅     |
| **Télécommande Companion**   | Déclenche des sons depuis Bitfocus Companion en HTTP/WebSocket | ✅     |
| **Partage de soundboard**    | Partage ton soundboard avec tes amis                      | ✅     |
| **Import / export**          | Importe et exporte ton soundboard                         | ✅     |
| **Système de tags**          | Classe tes sons préférés par catégories                   | ✅     |
| **Support MIDI**             | Contrôle le soundboard en MIDI                            | ⛔     |
| **Recherche vocale**         | Trouve des sons avec ta voix                              | ⛔     |
| **Contrôle par chat Twitch** | Ta communauté choisit ce qui est joué                     | ⛔     |
| **Assistant IA**             | Propose des sons adaptés à la situation                   | ⛔     |

## Docs

- [Site](https://soundninja.marcus-universe.com/)
- [Guide utilisateur](https://soundninja.marcus-universe.com/docs)
- [Compiler depuis les sources](./docs/build-from-source.md) (anglais)
- [Environnement de développement](./docs/development.md) (anglais)
- [Companion / Remote](./docs/companion.md) (anglais)
- [Modèle de séparation de stems](./docs/stems-model.md) (anglais)
- [Mise à jour de l’app](./docs/updater.md) (anglais)

---
<img src="./designs/download.png" alt="Téléchargements" width="200" />

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

Les installeurs de l’app sont sur `/releases/latest/download/…`. Le module Companion en `.tgz` est sur la page de releases de [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja/releases/latest).

### Arch Linux

Les paquets précompilés `soundninja-bin` (`x86_64` et `aarch64`) sont publiés sur GitHub Releases. Ils ne sont **pas encore** sur [aur.archlinux.org](https://aur.archlinux.org), donc `yay -S soundninja` ne les trouve pas. Installation avec curl :

```bash
arch=$(uname -m)   # x86_64 or aarch64
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-bin-[^"]+-'"${arch}"'\.pkg\.tar\.zst' | head -n1)
curl -fLO "$url"
sudo pacman -U ./soundninja-bin-*-"${arch}".pkg.tar.zst
```

Ou compile depuis l’archive source AUR (`makepkg`) :

```bash
url=$(curl -fsSL https://api.github.com/repos/marcus-universe/SoundNinja/releases/latest \
  | grep -oE 'https://[^"]+/soundninja-aur-[^"]+\.tar\.gz' | head -n1)
curl -fLO "$url"
mkdir -p soundninja-bin && tar xf soundninja-aur-*.tar.gz -C soundninja-bin
cd soundninja-bin && makepkg -si
```

**Actions → Release** crée un **brouillon** de GitHub Release (`vX.Y.Z`) s’il n’en existe pas, puis n’attache que les plateformes absentes du brouillon. Recompile une cible avec **Release Windows**, **Release Linux**, **Release macOS ARM** ou **Release macOS Intel**. Chaque action envoie l’installeur (plus un alias stable) sur ce brouillon. Module Companion : [companion-module-soundninja](https://github.com/marcus-universe/companion-module-soundninja) (`SoundNinja-Companion.tgz` et `SoundNinja-Companion-{Version}.tgz`).

## Plateformes

Builds officiels (x86_64 sauf mention contraire) :

| Plateforme | Prise en charge |
| :------- | :------ |
| Windows 10+ | Installeur NSIS (WebView2) |
| macOS 10.15+ | DMG par architecture (Apple Silicon, Intel) |
| Linux x86_64 | `.deb` (Debian/Ubuntu, WebKitGTK 4.1) ou AppImage. PipeWire ou PulseAudio pour le son. |
| Arch Linux | `soundninja-bin` depuis GitHub Releases (`pacman -U`, x86_64 + aarch64). Pas encore sur aur.archlinux.org. |
| Linux ARM | `.deb` pour `arm64` / `armhf`. |

Développement précoce — pas testé sur toutes les distros. Signale les problèmes pour qu’on puisse les corriger.

## Crédits

- [lucidrains/BS-RoFormer](https://github.com/lucidrains/BS-RoFormer) — Band-Split RoPE Transformer, l’architecture de la séparation de stems par IA de Sound Ninja dans le Record Editor. Licence MIT.
- Poids entraînés par viperx ; export ONNX par [ZFTurbo/MSS_ONNX_TensorRT](https://github.com/ZFTurbo/MSS_ONNX_TensorRT).
- [KLIPY](https://klipy.com) — recherche de GIF en ligne (ta propre clé API).
- [GifSnap](https://gifsnap.com) — recherche de GIF sans clé (API publique, best effort). Les résultats sont mis en cache localement pour limiter les requêtes.
