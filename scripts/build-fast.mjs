// Release build without whole-program optimisation.
//
// `[profile.release]` in src-tauri/Cargo.toml sets `lto = true` and
// `codegen-units = 1`, which is what shipped builds want but makes the link
// step dominate local iteration. Cargo lets any profile key be overridden by an
// environment variable, so nothing in the manifest has to change.
//
// Use this while testing a packaged build; use `npm run tauri:build` for
// anything you actually hand to a user.

import { spawn } from 'node:child_process'

const args = process.argv.slice(2)

const child = spawn('npm', ['run', 'tauri', '--', 'build', ...args], {
  stdio: 'inherit',
  shell: true,
  env: {
    ...process.env,
    CARGO_PROFILE_RELEASE_LTO: 'off',
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS: '16',
  },
})

child.on('exit', (code) => process.exit(code ?? 1))
