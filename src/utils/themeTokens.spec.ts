import { describe, expect, it } from 'vitest'
import { readableTextColor } from '~/utils/contrast'
import {
  GIF_OVERLAY_DEFAULTS,
  THEME_TOKEN_DEFAULTS,
  lightenHex,
  parseThemeCss,
  resolveThemeTokens,
  withAlpha,
} from '~/utils/themeTokens'

describe('lightenHex', () => {
  it('moves toward white', () => {
    expect(lightenHex('#000000', 0.5)).toBe('#808080')
  })

  it('treats unparseable hex as black', () => {
    expect(lightenHex('not-a-color', 0.12)).toBe(lightenHex('#000000', 0.12))
  })
})

describe('withAlpha', () => {
  it('appends aa for 0.5', () => {
    expect(withAlpha('#00d4ff', 0.5)).toBe('#00d4ff80')
  })
})

describe('resolveThemeTokens', () => {
  it('returns defaults for empty settings', () => {
    const tokens = resolveThemeTokens({})
    expect(tokens.primaryColor).toBe(THEME_TOKEN_DEFAULTS.primaryColor)
    expect(tokens.bg).toBe(THEME_TOKEN_DEFAULTS.bg)
  })

  it('keeps flat tokens when present', () => {
    const tokens = resolveThemeTokens({
      primaryColor: '#ff0000',
      bg: '#111111',
      btnBg: '#222222',
      btnBorder: '#ff0000',
    })
    expect(tokens.primaryColor).toBe('#ff0000')
    expect(tokens.bg).toBe('#111111')
    expect(tokens.bg2).toBeTruthy()
  })
})

describe('parseThemeCss', () => {
  it('reads flat CSS vars', () => {
    const css = `:root { --primary_color: #00d4ff; --color-bg: #222831; }`
    const parsed = parseThemeCss(css)
    expect(parsed.primaryColor).toBe('#00d4ff')
    expect(parsed.bg).toBe('#222831')
  })
})

describe('overlay defaults', () => {
  it('hover is lighter wash than idle', () => {
    expect(GIF_OVERLAY_DEFAULTS.gifOverlayHover).toBeLessThan(
      GIF_OVERLAY_DEFAULTS.gifOverlay
    )
  })
})

describe('readableTextColor', () => {
  it('picks light text on dark bg', () => {
    expect(readableTextColor('#222831', '#eeeeee', '#222831')).toBe('#eeeeee')
  })
})
