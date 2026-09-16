# Build from source

Repo: [github.com/marcus-universe/SoundNinja](https://github.com/marcus-universe/SoundNinja)

Needs [Node.js](https://nodejs.org/) (or [Bun](https://bun.sh/) / [pnpm](https://pnpm.io/) / [Deno](https://deno.com/)) plus the [Tauri v2 prerequisites](https://v2.tauri.app/start/prerequisites/) (Rust, platform WebView / GTK).

Scripts, stems flag, and frontend-only: [development.md](./development.md).

```bash
git clone https://github.com/marcus-universe/SoundNinja.git
cd SoundNinja
bun install          # or: npm install / pnpm install / deno install
bun run tauri:serve  # hot-reload desktop app
# production installer:
bun run tauri:build
```

Linux and macOS stop there. They use PipeWire/PulseAudio (Linux) or Core Audio (macOS). **Do not run `setup:asio` on those platforms** — ASIO is a Windows host API.

## Linux

- System packages from the [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux): WebKitGTK 4.1, GTK, appindicator, `librsvg`, `patchelf`.
- Audio hosts: PipeWire and PulseAudio. Also install `libasound2-dev`, `libpipewire-0.3-dev`, `libpulse-dev`.
- **No ASIO.** Skip `bun run setup:asio`.
- WebKit DMA-BUF issues: `bun run tauri:serve:linux`.

## macOS

- Xcode Command Line Tools + the [Tauri macOS prerequisites](https://v2.tauri.app/start/prerequisites/#macos).
- Audio: Core Audio only. **No ASIO**, no Steinberg SDK, no `setup:asio`.
- Universal installer: `bun run tauri:build:macos`.

## Windows ASIO (Steinberg SDK)

Every Windows build — including `bun run tauri:serve` — compiles ASIO into the app. That is a **target dependency** (`cpal`/`rodio` features `asio` under `cfg(windows)`), not a Cargo feature you toggle. Linux and macOS never enable that feature, so they never compile [asio-sys](https://crates.io/crates/asio-sys).

ASIO is Steinberg's Windows audio host API. Compiling the host needs the official **ASIO SDK** (C headers + host sample that `asio-sys` bindgen-wraps). A hardware vendor's ASIO *driver* (Focusrite USB ASIO, RME, ASIO4ALL, …) is required at **runtime** only and is not enough to compile.

| What | When | Where |
| --- | --- | --- |
| [Steinberg ASIO SDK](https://www.steinberg.net/asiosdk) (currently 2.3.4) | compile time | `vendor/asiosdk/` via `CPAL_ASIO_DIR` |
| LLVM `libclang.dll` | compile time (bindgen) | `LIBCLANG_PATH` from `setup:asio` or CI |
| Vendor ASIO driver | runtime | Windows registry / device installer |

The SDK is **not** committed (Steinberg license). `.gitignore` excludes `/vendor/asiosdk/` and `/vendor/libclang/`.

One-time on a Windows machine:

```bash
bun run setup:asio
```

`scripts/setup-asio-sdk.ps1` downloads the SDK zip from Steinberg ([direct 2.3.4](https://download.steinberg.net/sdk_downloads/ASIO-SDK_2.3.4_2025-10-15.zip) or [https://www.steinberg.net/asiosdk](https://www.steinberg.net/asiosdk)), extracts so `vendor/asiosdk` contains `common/` and `host/`, installs LLVM if `libclang.dll` is missing, and sets the User `LIBCLANG_PATH`. `.cargo/config.toml` sets `CPAL_ASIO_DIR=vendor/asiosdk` (project-root relative). A user-set `CPAL_ASIO_DIR` or `LIBCLANG_PATH` wins.

Manual fallback: download the SDK yourself, extract to `vendor/asiosdk` (must contain `common/` + `host/`), install [LLVM](https://github.com/llvm/llvm-project/releases), set `LIBCLANG_PATH` to the folder that contains `libclang.dll`.

Then:

```bash
bun run tauri:serve
# or
bun run tauri:build
```

CI (`.github/workflows/_tauri-build.yml`) fetches the same SDK and LLVM **only** when `kind == windows`. Linux and macOS jobs do not set `CPAL_ASIO_DIR` / `LIBCLANG_PATH`.
