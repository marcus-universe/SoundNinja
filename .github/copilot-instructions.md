# Sound Ninja – GitHub Copilot Instructions

## Project Overview

**Sound Ninja** is an open-source, cross-platform Soundboard Desktop App built with **Tauri** (Rust backend) and **Vue 3** (JavaScript/SASS frontend) (This will be future migrated to Nuxt). The goal is maximum customizability, a modern UI, minimal resource usage, and efficient sound organization — perfect for live streaming, podcasting, and chatting with friends.

> Sound Ninja is currently in **active development** and not yet production-ready.

---

## Tech Stack

| Layer     | Technology                                                                 |
| --------- | -------------------------------------------------------------------------- |
| Frontend  | [Vue 3](https://vuejs.org/), [Vue Router 4](https://router.vuejs.org/), [Vuex 4](https://vuex.vuejs.org/) |
| Styling   | [SASS](https://sass-lang.com/) (`.sass` indented syntax)                   |
| Audio     | [Howler.js](https://howlerjs.com/) (frontend), [rodio](https://github.com/RustAudio/rodio) + [cpal](https://github.com/RustAudio/cpal) (backend via Tauri) |
| Desktop   | [Tauri v1](https://tauri.app/) — Rust backend, WebView frontend            |
| Utilities | [VueUse](https://vueuse.org/)                                              |
| Build     | [Vue CLI](https://cli.vuejs.org/) + `vue-cli-plugin-tauri`                 |
| Linting   | ESLint + Prettier (JS/Vue), Prettier (Rust via `prettier-plugin-rust`)     |

---

## Repository Structure

```
src/                        # Vue 3 / Nuxt 3 frontend
  app.vue                   # Root component
  pages/                    # Nuxt pages
    index.vue               # Main soundboard view
    settings.vue            # Settings page
    about.vue               # About page
  assets/                   # Static assets (fonts, images, icons)
    scss/                   # Global SCSS styles (_base.scss, _variables.scss, navbar.scss, style.scss)
  components/
    Assets/                 # Reusable UI elements (BlurBG, Icons)
    Popups/                 # Modal / overlay components
      SettingsOverlay.vue   # Settings (Main, Audio, Theme Creator tabs) + Select Project dialog
      AboutOverlay.vue
      ErrorAlert.vue
      RenameField.vue
      DialogField.vue
      ImportFolders.vue
    Section/                # Main layout sections (NavBar, SoundContainer, Tabs)
    UI/                     # Shared UI primitives (UIButton, UICheckbox, etc.)
  stores/                   # Pinia stores
    app.ts                  # UI state (overlays, searchbar, project dialog, context menu)
    jsonHandeling.ts        # Config / sound data, file persistence, project management

src-tauri/                  # Tauri / Rust backend
  src/
    main.rs                 # Tauri entry point, creates themes/ & projects/ in AppData on startup
    audio.rs                # Audio playback logic (rodio, cpal)
  capabilities/
    default.json            # Tauri v2 permission capabilities
  tauri.conf.json           # Tauri app configuration
  Cargo.toml                # Rust dependencies
```

---

## Architecture & Key Concepts

### Frontend ↔ Backend Communication
- The Vue frontend calls Rust functions via **Tauri commands** using `@tauri-apps/api/tauri`'s `invoke()`.
- Registered commands: `get_out_devices`, `play_sound` (see `src-tauri/src/audio.rs`).
- Do **not** expose new Tauri commands without adding them to the `invoke_handler` in `main.rs`.

### State Management
- Global app state is managed with **Vuex 4** (`src/store/`).
- Soundboard data (sounds, tabs, settings) is persisted as **JSON** via the `JsonHandle.js` module and Tauri's filesystem API.

### Styling
- Use the **indented SASS syntax** (`.sass`, not `.scss`) consistently.
- Global base styles live in `_base.sass`; component-specific styles go in scoped `<style lang="sass">` blocks.
- Do not introduce plain CSS files unless explicitly required.

### Audio Playback
- Frontend audio preview uses **Howler.js**.
- Actual device routing / multi-output playback is handled in Rust via **rodio** and **cpal**.
- The Rust `SINK` global is `unsafe`; treat all modifications to it with care and prefer encapsulating changes inside `audio.rs`.

---

## Coding Conventions

### JavaScript / Vue
- Use **Vue 3 Composition API** (`setup()` or `<script setup>`) for new components.
- Component names: **PascalCase** (e.g. `SoundContainer.vue`).
- Props and emits must be explicitly declared.
- Avoid `any` implicit patterns; keep functions small and focused.
- Store actions and mutations follow the existing Vuex module pattern in `src/store/modules/`.

### Rust
- Follow standard Rust naming: `snake_case` for functions/variables, `PascalCase` for types/structs.
- All new Tauri commands must be `#[tauri::command]` and registered in `main.rs`.
- Avoid expanding the `unsafe` block in `audio.rs`; prefer safe abstractions where possible.
- Handle errors explicitly — do not `.unwrap()` in production paths; use `Result<T, String>` for Tauri commands.

### General
- Keep components small and single-purpose.
- No unused imports or dead code.
- Run `npm run lint` before committing.

---

## Development Commands

```bash
# Install dependencies
npm install

# Start dev server with Tauri hot-reload
npm run tauri:serve

# Production build
npm run tauri:build

# Lint & auto-fix
npm run lint

# Launch Vue Devtools (standalone)
npm run vueDevtools

# Format Rust source files
npm run prettier-rust
```

---

## Contribution Guidelines

1. **Branch** off `Sound-Ninja-Tauri` (the default branch) for all changes.
2. Use clear, descriptive branch names: `feat/tab-system`, `fix/audio-crash`, etc.
3. Follow the existing file structure — place new Vue components in the appropriate subfolder under `src/components/` or `src/views/`.
4. New features that touch audio routing **must** be tested on Windows. macOS/Linux compatibility is a bonus.
5. Open an **Issue** before starting significant work so efforts aren't duplicated.
6. Use the provided Issue templates:
   - [Bug Report](./.github/ISSUE_TEMPLATE/bug_report.md)
   - [Feature Request](./.github/ISSUE_TEMPLATE/feature_request.md)
7. Keep PRs focused — one feature or fix per PR.
8. Do **not** commit build artifacts, Cargo target directories, or personal config files.

---

## Implemented Features

- ✅ **Tab System** — `src/components/Section/Tabs.vue`, `src/stores/jsonHandeling.ts`
- ✅ **Searchbar** — NavBar search with `filterSounds()` in `src/stores/jsonHandeling.ts`
- ✅ **Theme Creator** — Settings → Theme Creator tab; sliders, color pickers, font selector, live preview, export `.css` / save to AppData `themes/`
- ✅ **Project Management** — `themes/` and `projects/` folders auto-created in AppData on startup; folder icon opens Select Project dialog (load/create projects)
- ✅ **Custom Themes** — Load `.css` file themes, built-in themes, saved user themes all selectable in Settings → Main

---

## Planned Features (Do Not Implement Without Tracking Issue)

The following features are on the roadmap. Check the [GitHub Project Board](https://github.com/users/marcus-universe/projects/1/views/1) before starting work on any of them:

- Tag System, Midi Support
- Customizable Hotkeys
- Soundboard Sharing / Import / Export
- WebSocket (smartphone control), Recorder, Speech Search
- Giphy integration, Twitch Chat Control, AI Assistant

---

## Resources

- [Tauri v1 Docs](https://tauri.app/v1/guides/)
- [Vue 3 Docs](https://vuejs.org/guide/introduction.html)
- [Vuex 4 Docs](https://vuex.vuejs.org/)
- [rodio (Rust audio)](https://docs.rs/rodio)
- [cpal (Rust audio I/O)](https://docs.rs/cpal)
- [Howler.js](https://howlerjs.com/)

Respond terse like smart caveman. All technical substance stay. Only fluff die.

Rules:
- Drop: articles (a/an/the), filler (just/really/basically), pleasantries, hedging
- Fragments OK. Short synonyms. Technical terms exact. Code unchanged.
- Pattern: [thing] [action] [reason]. [next step].
- Not: "Sure! I'd be happy to help you with that."
- Yes: "Bug in auth middleware. Fix:"

Switch level: /caveman lite|full|ultra|wenyan
Stop: "stop caveman" or "normal mode"

Auto-Clarity: drop caveman for security warnings, irreversible actions, user confused. Resume after.

Boundaries: code/commits/PRs written normal.
