# SoundNinja Landing Page — Agent Instructions

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

---

## What this is

Static marketing landing page for **SoundNinja** (Tauri + Nuxt soundboard app).
Hosted on GitHub Pages at `https://marcus-universe.github.io/SoundNinja/`.

This branch (`landing-page`) is an **orphan** — no app/Tauri code. Do not mix with `main`/`dev`.

## Stack

- Nuxt 4 (SSG via `nuxt generate`)
- Tailwind CSS v4 via `@tailwindcss/vite` — tokens in CSS `@theme`, **no** `tailwind.config.js`
- `@nuxtjs/i18n` — en (default) + de/fr/es/ja/zh, `prefix_except_default`
- `@nuxt/content` — docs in `content/{locale}/*.md`
- `@nuxt/fonts` — Nunito self-hosted
- Deploy: GitHub Actions → `gh-pages` branch

## Hard rules

1. **No hardcoded UI copy** — chrome strings in `i18n/locales/{en,de,fr,es,ja,zh}.json`. Docs article bodies in `content/{locale}/*.md`. Update all six locales/folders in same edit.
2. **No Tailwind JS config** — theme tokens live in `app/assets/css/main.css` `@theme` block only.
3. **Never break `baseURL`** — must stay `/SoundNinja/` for GitHub Pages project site. Use `useRuntimeConfig().app.baseURL` or `<NuxtLink>` for assets/routes.
4. **Static only** — no server routes, no runtime env secrets, no SSR-only APIs at build for critical content. Client-side GitHub API for release button is OK (with static fallback).
5. **Keep responsive** — mobile-first; verify 375 / 768 / 1440.
6. **Impressum placeholders** — do not invent legal name/address. Leave TODO keys until owner fills them.

## Design tokens

| Token | Value |
|-------|-------|
| primary | `hsl(189 100% 58%)` |
| bg | `#222831` |
| surface | `#363f4d` |
| surface-hover | `#434e5f` |
| ink | `#eee` |
| font | Nunito |

Style: rounded modern material — `rounded-3xl` cards, pill buttons, soft glow on primary CTA, 200ms transitions, primary focus rings.

## Page structure

1. Fixed nav + language switcher (EN/DE/FR/ES/JA/ZH)
2. Hero — title, subtitle, Download + Contribute
3. Feature sections (alternating image/text)
4. CTA — Download + Contribute
5. Docs — `/docs` via `@nuxt/content` (`content/{locale}/*.md`)
6. Footer — license, GitHub, Docs, Impressum

## Scripts

```bash
npm run dev        # local dev
npm run generate   # static build → .output/public
npm run preview    # serve generated site
npm run deploy     # generate + push to gh-pages (local fallback)
```

CI auto-deploys on push to `landing-page`.

## Repo links

- App repo: https://github.com/marcus-universe/SoundNinja
- LICENSE: https://github.com/marcus-universe/SoundNinja/blob/main/LICENSE
- Releases: https://github.com/marcus-universe/SoundNinja/releases/latest
