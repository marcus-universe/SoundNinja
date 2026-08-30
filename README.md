# SoundNinja Landing Page

Static marketing site for [SoundNinja](https://github.com/marcus-universe/SoundNinja) — modern, customizable, open source soundboard.

**Live:** https://marcus-universe.github.io/SoundNinja/

## Stack

- [Nuxt 4](https://nuxt.com/) (static generate)
- [Tailwind CSS v4](https://tailwindcss.com/)
- [@nuxtjs/i18n](https://i18n.nuxtjs.org/) (English + German)
- [@nuxt/fonts](https://fonts.nuxt.com/) (Nunito)

## Develop

```bash
npm install
npm run dev
```

## Build & deploy

```bash
npm run generate   # → .output/public
npm run preview    # serve static output
npm run deploy     # generate + push to gh-pages branch
```

Push to `landing-page` also triggers GitHub Actions → deploys `gh-pages`.

### One-time GitHub Pages setup

1. Repo **Settings → Pages**
2. Source: **Deploy from a branch**
3. Branch: `gh-pages` / `/ (root)`
4. Save

## Structure

| Path | Purpose |
|------|---------|
| `app/pages/index.vue` | Hero + features slideshow |
| `app/pages/impressum.vue` | Legal notice (placeholder) |
| `i18n/locales/` | EN / DE copy |
| `public/screenshots/` | Feature image placeholders |

## License

Landing page code in this branch follows the same project. SoundNinja app is [GPL-3.0](https://github.com/marcus-universe/SoundNinja/blob/main/LICENSE).
