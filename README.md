# Companion Module: Sound Ninja

Bitfocus Companion module for [Sound Ninja](https://github.com/marcus-universe/SoundNinja). Trigger sounds by ID, stop one sound or everything, and show playing state on Stream Deck buttons.

**Version:** 1.0.0 (independent of the Sound Ninja app version).

Module-only tree. App code lives on Sound Ninja `main` / `dev`.

## Install

1. Enable **Settings → Remote** in Sound Ninja (default port `7331`).
2. Download the latest packaged module: [SoundNinja-Companion.tgz](https://github.com/marcus-universe/SoundNinja/releases/latest/download/SoundNinja-Companion.tgz).
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

Produces `soundninja-1.0.0.tgz`. Push to `main` / `companion-module` (or run **Actions → Companion Module**) to attach `SoundNinja-Companion-{Version}.tgz` and the stable alias `SoundNinja-Companion.tgz` to the [latest SoundNinja GitHub release](https://github.com/marcus-universe/SoundNinja/releases/latest).

### GitHub Actions

`workflow_dispatch` only shows **Run workflow** when this YAML lives on the repo **default branch**. Cross-repo upload (if this tree is its own GitHub repo) needs a PAT with **Contents: Read and write** on `marcus-universe/SoundNinja`, stored as repository secret `SOUNDNINJA_RELEASE_TOKEN`. Same-repo runs can use `GITHUB_TOKEN`.
