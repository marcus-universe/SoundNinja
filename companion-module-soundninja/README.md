# Companion Module: Sound Ninja

Bitfocus Companion module for [Sound Ninja](https://github.com/marcus-universe/SoundNinja). Trigger sounds by ID, stop one sound or everything, and show playing state on Stream Deck buttons.

## Install

1. Enable **Settings → Remote** in Sound Ninja (default port `7331`).
2. In Companion: **Developer → Modules** (or drop this folder into Companion's custom modules path).
3. Add a **Sound Ninja** connection. Paste the PC IP from Sound Ninja **About** / **Remote**.

```bash
cd companion-module-soundninja
npm install
```

See [companion/HELP.md](companion/HELP.md) for actions, feedbacks, and variables.

## Package

```bash
npm run package
```
