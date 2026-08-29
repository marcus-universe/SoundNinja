// Flat (dark-only) theme token model. Settings keys map 1:1 to CSS variables.
// Legacy light/dark pair CSS is still readable via parseThemeCss().

import { hexToRgb, readableTextColor } from '~/utils/contrast'

export const THEME_TOKEN_DEFAULTS = {
  primaryColor: '#00d4ff',
  primaryHover: '#33ddff',
  bg: '#222831',
  /** Secondary surfaces (settings sidebar, tool windows). Matches former rgba(0,0,0,0.25) over bg. */
  bg2: '#1a1e25',
  btnBg: '#363f4d',
  btnBgHover: '#434e5f',
  btnText: '#eeeeee',
  btnTextHover: '#00d4ff',
  btnBorder: '#00d4ff',
  btnBorderHover: '#33ddff',
  tabBg: '#00d4ff33',
  tabBgHover: '#00d4ff66',
  tabText: '#eeeeee',
  tabTextHover: '#eeeeee',
  tabBorder: '#00d4ff',
  tabBorderHover: '#33ddff',
} as const

export type ThemeTokenKey = keyof typeof THEME_TOKEN_DEFAULTS
export type ThemeTokens = Record<ThemeTokenKey, string>

/** Settings/CSS-var map used by apply + export. */
export const TOKEN_CSS_VARS: Record<ThemeTokenKey, string> = {
  primaryColor: '--primary_color',
  primaryHover: '--primary-hover',
  bg: '--color-bg',
  bg2: '--color-bg-2',
  btnBg: '--color-btn',
  btnBgHover: '--btn-bg-hover',
  btnText: '--sound-text',
  btnTextHover: '--btn-text-hover',
  btnBorder: '--btn-border',
  btnBorderHover: '--btn-border-hover',
  tabBg: '--tab-bg',
  tabBgHover: '--tab-bg-hover',
  tabText: '--tab-text',
  tabTextHover: '--tab-text-hover',
  tabBorder: '--tab-border',
  tabBorderHover: '--tab-border-hover',
}

/** Color wash over GIF/image buttons (0 = media only, 1 = solid). Hover must differ or GIF hover is invisible. */
export type ThemeGifOverlay = {
  gifOverlay: number
  gifOverlayHover: number
}

export const GIF_OVERLAY_DEFAULTS: ThemeGifOverlay = {
  gifOverlay: 0.72,
  gifOverlayHover: 0.38,
}

/** All CSS vars that may be set inline by the theme system (cleared before file themes). */
export const THEME_INLINE_VARS = [
  ...Object.values(TOKEN_CSS_VARS),
  '--color-text',
  '--gif-overlay',
  '--gif-overlay-hover',
]

/** Lighten a #rrggbb hex by `amount` (0–1). Falls back to input on parse fail. */
export function lightenHex(hex: string, amount = 0.12): string {
  try {
    const [r, g, b] = hexToRgb(hex)
    const lift = (c: number) => Math.min(255, Math.round(c + (255 - c) * amount))
    return (
      '#' +
      [lift(r), lift(g), lift(b)].map((x) => x.toString(16).padStart(2, '0')).join('')
    )
  } catch {
    return hex
  }
}

/** Darken hex by mixing toward black (`amount` 0–1). 0.25 ≈ former settings sidebar tint. */
export function darkenHex(hex: string, amount = 0.25): string {
  try {
    const [r, g, b] = hexToRgb(hex)
    const sink = (c: number) => Math.max(0, Math.round(c * (1 - amount)))
    return (
      '#' +
      [sink(r), sink(g), sink(b)].map((x) => x.toString(16).padStart(2, '0')).join('')
    )
  } catch {
    return hex
  }
}

/** Append alpha (0–1) to an opaque hex → #rrggbbaa. */
export function withAlpha(hex: string, alpha: number): string {
  const a = Math.max(0, Math.min(255, Math.round(alpha * 255)))
    .toString(16)
    .padStart(2, '0')
  const base = (hex || '').replace('#', '').slice(0, 6)
  if (base.length < 6) return hex
  return `#${base}${a}`
}

function g(v: unknown, d: string): string {
  return (v && String(v).trim()) || d
}

/**
 * Resolve flat tokens from project settings. Migrates legacy pair fields
 * (bgDark/btnDark/textLight/textDark/primaryColor) when flat keys are absent.
 */
export function resolveThemeTokens(
  settings: Record<string, unknown> | undefined | null
): ThemeTokens {
  const s = settings ?? {}
  const primary = g(s.primaryColor, THEME_TOKEN_DEFAULTS.primaryColor)
  const hasFlat = !!(s.bg || s.btnBg || s.btnBorder)

  if (hasFlat) {
    const bg = g(s.bg, THEME_TOKEN_DEFAULTS.bg)
    return {
      primaryColor: primary,
      primaryHover: g(s.primaryHover, lightenHex(primary)),
      bg,
      bg2: g(s.bg2, darkenHex(bg)),
      btnBg: g(s.btnBg, THEME_TOKEN_DEFAULTS.btnBg),
      btnBgHover: g(s.btnBgHover, lightenHex(g(s.btnBg, THEME_TOKEN_DEFAULTS.btnBg))),
      btnText: g(s.btnText, THEME_TOKEN_DEFAULTS.btnText),
      btnTextHover: g(s.btnTextHover, primary),
      btnBorder: g(s.btnBorder, primary),
      btnBorderHover: g(s.btnBorderHover, lightenHex(primary)),
      tabBg: g(s.tabBg, withAlpha(primary, 0.2)),
      tabBgHover: g(s.tabBgHover, withAlpha(primary, 0.4)),
      tabText: g(s.tabText, THEME_TOKEN_DEFAULTS.tabText),
      tabTextHover: g(s.tabTextHover, THEME_TOKEN_DEFAULTS.tabTextHover),
      tabBorder: g(s.tabBorder, primary),
      tabBorderHover: g(s.tabBorderHover, lightenHex(primary)),
    }
  }

  // Legacy pair migration: dark variant wins.
  const bg = g(s.bgDark, THEME_TOKEN_DEFAULTS.bg)
  const btnBg = g(s.btnDark, THEME_TOKEN_DEFAULTS.btnBg)
  const textLight = g(s.textLight, '#eeeeee')
  const textDark = g(s.textDark, '#222831')
  const btnText = readableTextColor(btnBg, textLight, textDark)

  return {
    primaryColor: primary,
    primaryHover: lightenHex(primary),
    bg,
    bg2: darkenHex(bg),
    btnBg,
    btnBgHover: lightenHex(btnBg),
    btnText,
    btnTextHover: primary,
    btnBorder: primary,
    btnBorderHover: lightenHex(primary),
    tabBg: withAlpha(primary, 0.2),
    tabBgHover: withAlpha(primary, 0.4),
    tabText: textLight,
    tabTextHover: textLight,
    tabBorder: primary,
    tabBorderHover: lightenHex(primary),
  }
}

/** Apply resolved tokens as inline CSS vars on <html>. */
export function applyThemeTokens(
  settings: Record<string, unknown> | undefined | null,
  overlay?: Partial<ThemeGifOverlay>
): void {
  if (typeof document === 'undefined') return
  const tokens = resolveThemeTokens(settings)
  const root = document.documentElement
  for (const key of Object.keys(TOKEN_CSS_VARS) as ThemeTokenKey[]) {
    root.style.setProperty(TOKEN_CSS_VARS[key], tokens[key])
  }
  // UI chrome text (settings, dialogs) — contrast against page bg.
  root.style.setProperty(
    '--color-text',
    readableTextColor(tokens.bg, '#eeeeee', '#222831')
  )
  const idle = overlay?.gifOverlay ?? GIF_OVERLAY_DEFAULTS.gifOverlay
  const hover = overlay?.gifOverlayHover ?? GIF_OVERLAY_DEFAULTS.gifOverlayHover
  root.style.setProperty('--gif-overlay', String(idle))
  root.style.setProperty('--gif-overlay-hover', String(hover))
}

/** Build a SoundNinja theme CSS file from flat tokens (+ optional layout extras). */
export function buildThemeCss(
  name: string,
  tokens: ThemeTokens,
  extras: Record<string, string> = {}
): string {
  const lines = Object.entries(TOKEN_CSS_VARS).map(
    ([key, cssVar]) => `  ${cssVar}: ${tokens[key as ThemeTokenKey]};`
  )
  lines.push(
    `  --color-text: ${readableTextColor(tokens.bg, '#eeeeee', '#222831')};`
  )
  for (const [cssVar, value] of Object.entries(extras)) {
    lines.push(`  ${cssVar}: ${value};`)
  }
  return `/* SoundNinja Theme: ${name} */
:root {
${lines.join('\n')}
}
`
}

function parseCssVars(css: string): Record<string, string> {
  const result: Record<string, string> = {}
  const re = /(--[\w-]+)\s*:\s*([^;]+)/g
  let m: RegExpExecArray | null
  while ((m = re.exec(css)) !== null) {
    result[m[1].trim()] = m[2].trim()
  }
  return result
}

/** Convert any CSS color string to #rrggbb when possible. */
export function cssColorToHex(color: string): string {
  const c = color.trim()
  if (/^#[0-9a-f]{3,8}$/i.test(c)) {
    if (c.length === 4) {
      return (
        '#' +
        c
          .slice(1)
          .split('')
          .map((ch) => ch + ch)
          .join('')
      )
    }
    return c.length >= 7 ? c.slice(0, 7) : c
  }
  if (typeof document === 'undefined') return c
  try {
    const canvas = document.createElement('canvas')
    canvas.width = canvas.height = 1
    const ctx = canvas.getContext('2d')
    if (!ctx) return c
    ctx.fillStyle = c
    ctx.fillRect(0, 0, 1, 1)
    const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data
    return '#' + [r, g, b].map((x) => x.toString(16).padStart(2, '0')).join('')
  } catch {
    return c
  }
}

/**
 * Parse theme CSS into flat tokens. Accepts new flat vars and legacy
 * --color-bg-dark / --color-btn-dark / html.theme-* pair files.
 */
export function parseThemeCss(css: string): Partial<ThemeTokens> {
  const vars = parseCssVars(css)
  const out: Partial<ThemeTokens> = {}

  const setFrom = (key: ThemeTokenKey, ...candidates: string[]) => {
    for (const name of candidates) {
      if (vars[name]) {
        const raw = vars[name]
        // Keep 8-digit hex (with alpha) for tab tints; otherwise normalize to #rrggbb.
        if (/^#[0-9a-f]{8}$/i.test(raw.trim())) {
          out[key] = raw.trim()
        } else {
          out[key] = cssColorToHex(raw)
        }
        return
      }
    }
  }

  setFrom('primaryColor', '--primary_color')
  setFrom('primaryHover', '--primary-hover')
  setFrom('bg', '--color-bg', '--color-bg-dark')
  setFrom('bg2', '--color-bg-2')
  setFrom('btnBg', '--color-btn', '--color-btn-dark')
  setFrom('btnBgHover', '--btn-bg-hover')
  setFrom('btnText', '--sound-text', '--text-light')
  setFrom('btnTextHover', '--btn-text-hover')
  setFrom('btnBorder', '--btn-border', '--primary_color')
  setFrom('btnBorderHover', '--btn-border-hover')
  setFrom('tabBg', '--tab-bg')
  setFrom('tabBgHover', '--tab-bg-hover')
  setFrom('tabText', '--tab-text', '--sound-text', '--text-light')
  setFrom('tabTextHover', '--tab-text-hover')
  setFrom('tabBorder', '--tab-border', '--primary_color')
  setFrom('tabBorderHover', '--tab-border-hover')

  // Fill derived defaults when only primary + bg/btn present (legacy files).
  const primary = out.primaryColor || THEME_TOKEN_DEFAULTS.primaryColor
  if (!out.primaryHover) out.primaryHover = lightenHex(primary)
  if (!out.bg2 && out.bg) out.bg2 = darkenHex(out.bg)
  if (!out.btnBgHover && out.btnBg) out.btnBgHover = lightenHex(out.btnBg)
  if (!out.btnTextHover) out.btnTextHover = primary
  if (!out.btnBorder) out.btnBorder = primary
  if (!out.btnBorderHover) out.btnBorderHover = lightenHex(primary)
  if (!out.tabBg) out.tabBg = withAlpha(primary, 0.2)
  if (!out.tabBgHover) out.tabBgHover = withAlpha(primary, 0.4)
  if (!out.tabText) out.tabText = out.btnText || THEME_TOKEN_DEFAULTS.tabText
  if (!out.tabTextHover) out.tabTextHover = out.tabText
  if (!out.tabBorder) out.tabBorder = primary
  if (!out.tabBorderHover) out.tabBorderHover = lightenHex(primary)

  return out
}

export function parseThemeName(css: string): string {
  const m = css.match(/\/\*\s*SoundNinja Theme:\s*(.+?)\s*\*\//i)
  return m ? m[1].trim() : ''
}
