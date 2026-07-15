# Sound Ninja – GitHub Copilot Instructions

## Project Overview

**Sound Ninja** is an open-source, cross-platform Soundboard Desktop App built with **Tauri v2** (Rust backend) and **Nuxt 4 / Vue 3** (JavaScript/SCSS frontend). The goal is maximum customizability, a modern UI, minimal resource usage, and efficient sound organization — perfect for live streaming, podcasting, and chatting with friends.

> Sound Ninja is currently in **active development** and not yet production-ready.

---

## Tech Stack

| Layer     | Technology                                                                 |
| --------- | -------------------------------------------------------------------------- |
| Frontend  | [Nuxt 4](https://nuxt.com/), [Vue 3](https://vuejs.org/) (Composition API) |
| State     | [Pinia](https://pinia.vuejs.org/) (auto-imported stores)                   |
| Persistence | [SQLite](https://www.sqlite.org/) via [tauri-plugin-sql](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/sql) (one `project.db` per project) |
| Styling   | [SCSS](https://sass-lang.com/) (`.scss` syntax)                            |
| i18n      | [@nuxtjs/i18n](https://i18n.nuxtjs.org/) (`en`/`de`, strategy `no_prefix`) |
| Audio     | [Howler.js](https://howlerjs.com/) (frontend), [rodio](https://github.com/RustAudio/rodio) + [symphonia](https://github.com/pdeljanov/Symphonia) (backend via Tauri) |
| Desktop   | [Tauri v2](https://tauri.app/) — Rust backend, WebView frontend            |
| Utilities | [VueUse](https://vueuse.org/), [SortableJS](https://sortablejs.github.io/Sortable/) |
| Build     | [Nuxt](https://nuxt.com/) (`nuxt generate`) + [`@tauri-apps/cli`](https://tauri.app/) |
| Linting   | Prettier (JS/Vue), Prettier (Rust via `prettier-plugin-rust`)              |

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
    jsonHandeling.ts        # Sound/tab/settings data + SQLite persistence per project
    appSettings.ts          # App-level prefs (projectsPath, themesPath, navbarSide, locale)
  utils/
    db.ts                   # SQLite schema + load/save config
    projects.ts             # Project folder/DB path helpers
  composables/
    customFonts.ts          # Register uploaded @font-face fonts at runtime
  pages/
    index.vue               # Main soundboard view
    theme-creator.vue       # Theme Creator (opens in its own Tauri window)

src-tauri/                  # Tauri / Rust backend
  src/
    main.rs                 # Tauri entry point, registers plugins + commands
    paths.rs                # Portable-first data paths, app-settings, project listing
    fsx.rs                  # Absolute-path filesystem command helpers
    audio/                  # Audio playback (rodio, cpal) — devices, playback, cache
    menu/                   # Native menu (File > New Project/Open/Save…)
  capabilities/
    default.json            # Tauri v2 permission capabilities
  tauri.conf.json           # Tauri app configuration
  Cargo.toml                # Rust dependencies
```

---

## Architecture & Key Concepts

### Frontend ↔ Backend Communication
- The Vue frontend calls Rust functions via **Tauri commands** using `@tauri-apps/api/tauri`'s `invoke()`.
- Registered commands include `get_out_devices`, `play_sound` (see `src-tauri/src/audio/`), plus data/path helpers in `paths.rs` and `fsx.rs`.
- Do **not** expose new Tauri commands without adding them to the `invoke_handler` in `main.rs`.

### State Management
- Global app state is managed with **Pinia** (`src/stores/`, auto-imported).
- Soundboard data (sounds, tabs, settings, separators) is persisted to a per-project **SQLite** database (`<projectsPath>/<name>/project.db`) via `tauri-plugin-sql`, orchestrated by `jsonHandeling.ts` (`db.ts` holds schema + load/save). Writes are debounced; a `dirty` flag drives save-on-close prompts.
- App-level prefs (project/themes paths, navbar side, locale, last project) live in `appSettings.ts` and a fixed AppData `app-settings.json`.
- Data location is **portable-first**: `themes/` and `projects/` sit next to the executable when writable, else in AppData. Legacy `config.json` is auto-migrated to a `Default` project DB on first launch.

### Styling
- Use **SCSS** (`.scss` syntax). All app styling must live in `src/assets/scss/` as `.scss` files (`_base.scss`, `_variables.scss`, `navbar.scss`, `style.scss`, `settings.scss`, and split feature files as needed).
- Do not add or keep `<style>` blocks in Vue/Nuxt components (`.vue` files). Move component-specific rules into appropriate files under `src/assets/scss/`.
- Theme values are CSS custom properties declared in `:root` (`_base.scss`) with SCSS fallbacks, e.g. `var(--color-bg, #{$color-bg})`. Do not hardcode compile-time theme colors in backgrounds.
- Do not introduce plain CSS files unless explicitly required.

### Audio Playback
- Frontend audio preview uses **Howler.js**.
- Actual device routing / multi-output playback is handled in Rust via **rodio** and **cpal** (`src-tauri/src/audio/`).
- The Rust `SINK` global is `unsafe`; treat all modifications to it with care and prefer encapsulating changes inside the `audio` module.

---

## Coding Conventions

### JavaScript / Vue
- Use **Vue 3 Composition API** (`setup()` or `<script setup>`) for new components.
- Component names: **PascalCase** (e.g. `SoundContainer.vue`).
- Props and emits must be explicitly declared.
- Avoid `any` implicit patterns; keep functions small and focused.
- Store actions follow the existing Pinia option-store pattern in `src/stores/`.

### Rust
- Follow standard Rust naming: `snake_case` for functions/variables, `PascalCase` for types/structs.
- All new Tauri commands must be `#[tauri::command]` and registered in `main.rs`.
- Avoid expanding the `unsafe` block in the audio module; prefer safe abstractions where possible.
- Handle errors explicitly — do not `.unwrap()` in production paths; use `Result<T, String>` for Tauri commands.

### General
- Keep components small and single-purpose.
- No unused imports or dead code.
- Run `npm run build` and `cargo check` before committing.

---

## Development Commands

```bash
# Install dependencies
npm install

# Start dev server with Tauri hot-reload
npm run tauri:serve

# Production build
npm run tauri:build

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
- ✅ **SQLite Persistence** — per-project `project.db` via `tauri-plugin-sql`; `db.ts` schema, `jsonHandeling.ts` load/save, auto-migration of legacy `config.json`
- ✅ **Theme Creator (separate window)** — `src/pages/theme-creator.vue`; sliders (button gap, border width, radius, font sizes), color pickers, font selector + font upload, live preview via cross-window events, export `.css` / save to `themes/`
- ✅ **Project Management** — portable-first `themes/`/`projects/` folders; folder icon opens Select Project dialog; File > New Project with unsaved-changes prompt
- ✅ **Custom Themes & Fonts** — load `.css` themes (with embedded name), built-in + saved themes selectable; upload `.ttf`/`.otf` into `themes/fonts/`, registered at runtime
- ✅ **Navbar Side** — left/right runtime toggle in Settings → Main (CSS vars + html class)
- ✅ **Tab Separators** — right-click a sound → Add Separator; full-width horizontal divider, drag-movable, persisted per tab
- ✅ **Save-on-close** — dirty-check prompt (Save/Discard/Cancel) on New Project and window close

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

- [Tauri v2 Docs](https://tauri.app/start/)
- [Nuxt 4 Docs](https://nuxt.com/docs)
- [Vue 3 Docs](https://vuejs.org/guide/introduction.html)
- [Pinia Docs](https://pinia.vuejs.org/)
- [tauri-plugin-sql](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/sql)
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
