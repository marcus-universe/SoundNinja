# Companion Module: Sound Ninja

Bitfocus Companion module for [Sound Ninja](https://github.com/marcus-universe/SoundNinja). Trigger sounds by ID, stop one sound or everything, and show playing state on Stream Deck buttons.

**Version:** 1.0.0 (independent of the Sound Ninja app version).

This branch is module-only. App code lives on `main` / `dev`.

## Install

1. Enable **Settings → Remote** in Sound Ninja (default port `7331`).
2. Download the latest packaged module: [companion-module-soundninja.tgz](https://github.com/marcus-universe/SoundNinja/releases/latest/download/companion-module-soundninja.tgz).
3. In Companion: **Modules → Load module package** and pick the `.tgz`.
4. Add a **Sound Ninja** connection. Paste the PC IP from Sound Ninja **About** / **Remote**.

For local development:

```bash
npm install
```

See [companion/HELP.md](companion/HELP.md) for actions, feedbacks, and variables.

## Version

Independent of the Sound Ninja app. Bump only `package.json`:

```bash
npm run version:patch
npm run version:minor
npm run version:major
```

`companion/manifest.json` stays `0.0.0` — `companion-module-build` stamps the package version at pack time.

## Package

```bash
npm run package
```

Produces `soundninja-1.0.0.tgz`. Push to `companion-module` (or run **Actions → Companion Module**) to attach `companion-module-soundninja.tgz` to the [latest GitHub release](https://github.com/marcus-universe/SoundNinja/releases/latest).
