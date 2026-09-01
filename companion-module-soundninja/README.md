# Companion Module: Sound Ninja

Bitfocus Companion module for [Sound Ninja](https://github.com/marcus-universe/SoundNinja). Trigger sounds by ID, stop one sound or everything, and show playing state on Stream Deck buttons.

**Version:** 1.0.0 (independent of the Sound Ninja app version).

## Install

1. Enable **Settings → Remote** in Sound Ninja (default port `7331`).
2. Download the latest packaged module: [companion-module-soundninja.tgz](https://github.com/marcus-universe/SoundNinja/releases/latest/download/companion-module-soundninja.tgz).
3. In Companion: **Modules → Load module package** and pick the `.tgz`.
4. Add a **Sound Ninja** connection. Paste the PC IP from Sound Ninja **About** / **Remote**.

For local development, clone this repo and point Companion at `companion-module-soundninja/`:

```bash
cd companion-module-soundninja
npm install
```

See [companion/HELP.md](companion/HELP.md) for actions, feedbacks, and variables.

## Package

```bash
npm run package
```

Produces `companion-module-soundninja-1.0.0.tgz`. CI uploads that file plus a stable alias `companion-module-soundninja.tgz` on each app release.
