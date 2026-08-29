# App updater (Tauri)

Sound Ninja uses [`tauri-plugin-updater`](https://v2.tauri.app/plugin/updater/) against GitHub Releases:

`https://github.com/marcus-universe/SoundNinja/releases/latest/download/latest.json`

## One-time signing setup

A keypair was generated for this project. The **public** key is in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`).

1. Keep the private key secret (never commit it). Default path if generated locally: `~/.tauri/soundninja.key`
2. Add GitHub Actions **repository** secrets (Settings → Secrets → Actions). Names must match exactly:
   - `TAURI_SIGNING_PRIVATE_KEY` — full contents of the private key file (real newlines)
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — only if the key has a password; otherwise omit
3. Set `"createUpdaterArtifacts": true` in `src-tauri/tauri.conf.json` `bundle`.
4. Release workflow signs updater artifacts and uploads `latest.json` (`uploadUpdaterJson: true`).

> **v0.5.3 note:** `createUpdaterArtifacts` is currently `false` because the repo had no `TAURI_SIGNING_PRIVATE_KEY` secret (only `SOUND_NINJA_SECRET`, unrelated). Installers still ship; in-app update signatures do not until the secret is added and artifacts are re-enabled.

If you lose the private key, generate a new pair with `npx tauri signer generate`, replace `pubkey` in `tauri.conf.json`, and ship a **manual** install once — old builds cannot verify new signatures.

## In-app behavior

- **Settings → Check for updates on start** (default on): silent check; popup only when an update exists.
- **Help → Check for Updates**: always shows the dialog (checking / up to date / available / error).
- **Update now**: download + install signed artifact, then relaunch.

Works on installed release builds only — not `tauri dev`.
