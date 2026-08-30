---
title: Installation
description: Installe SoundNinja sur Windows, macOS ou Linux.
order: 2
---

# Installation

Récupère un build sur la [page Télécharger](/download) ou sur [GitHub Releases](https://github.com/marcus-universe/SoundNinja/releases/latest). SoundNinja est en développement précoce — traite les builds comme des bêtas.

## Windows

- **Installateur (.exe)** ou **MSI (.msi)** pour Windows 64 bits
- Cible de la landing page : Windows 10 et plus
- Le README de l’app mentionne aussi Windows 8 et supérieur

Lance l’installateur, puis ouvre SoundNinja depuis le menu Démarrer.

## macOS

- **.dmg universel** pour Apple Silicon et Intel
- Cible de la landing page : macOS 10.14 et plus
- Le README de l’app mentionne macOS 10.15 et supérieur

Ouvre l’image disque et glisse SoundNinja dans Applications.

## Linux

- **Debian (.deb)** pour Ubuntu / Debian
- **AppImage** pour les autres distros
- Après publication d’une release, les utilisateurs Arch peuvent installer `soundninja-bin` depuis l’AUR
- Testé sur Ubuntu 20.04

Les builds desktop ont besoin des paquets WebKit et GTK. Sous Debian/Ubuntu, ça veut souvent dire `libwebkit2gtk`, `libgtk-3-0` et une bibliothèque app-indicator.

## Mises à jour

Dans l’app, ouvre **Aide → Rechercher les mises à jour**. Tu peux aussi laisser SoundNinja vérifier au démarrage dans les Paramètres.

## Compiler depuis les sources

Besoin d’un build custom ? Clone [le repo](https://github.com/marcus-universe/SoundNinja), installe les [prérequis Tauri](https://v2.tauri.app/start/prerequisites/), puis :

```bash
bun install
bun run tauri:serve
```

`npm`, `pnpm` ou `deno` marchent aussi. Build de production : `bun run tauri:build`.
