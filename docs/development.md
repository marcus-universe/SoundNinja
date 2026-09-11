# Development setup

Requires [Node.js](https://nodejs.org/) (for npm/pnpm) or [Bun](https://bun.sh/) / [Deno](https://deno.com/). Also needs the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for desktop builds.

Install dependencies with any of:

```bash
# bun (recommended)
bun install

# npm
npm install

# pnpm
pnpm install

# deno
deno install
```

## Available dependencies

- [Nuxt 4](https://nuxt.com/) (Vue 3)
- [Tauri v2](https://tauri.app/)
- [Pinia](https://pinia.vuejs.org/)
- [SASS](https://sass-lang.com/)
- [@nuxtjs/i18n](https://i18n.nuxtjs.org/)
- [SortableJS](https://sortablejs.github.io/Sortable/)

## Compiles and hot-reloads for development

```bash
bun run tauri:serve
# or: npm run tauri:serve / pnpm tauri:serve / deno task tauri:serve
```

Stem separation (BS-RoFormer model) needs an extra build flag and a one-time model download. See [stems-model.md](./stems-model.md).

```bash
npm run tauri:serve:stems
```

## Compiles and minifies for production

```bash
bun run tauri:build
# or: npm run tauri:build / pnpm tauri:build / deno task tauri:build
```

## Frontend only (no Tauri shell)

```bash
bun run dev
# or: npm run dev / pnpm dev / deno task dev
```

## Customize configuration

See [Nuxt Configuration](https://nuxt.com/docs/api/configuration/nuxt-config) and [Tauri Configuration](https://v2.tauri.app/reference/config/).
