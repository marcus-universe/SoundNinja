// Built-in theme presets (flat token sets). Default id: soundninja.

import {
  THEME_TOKEN_DEFAULTS,
  GIF_OVERLAY_DEFAULTS,
  lightenHex,
  darkenHex,
  withAlpha,
  type ThemeTokens,
  type ThemeGifOverlay,
} from '~/utils/themeTokens'

export type ThemePreset = {
  id: string
  label: string
  tokens: ThemeTokens
  extras: ThemeGifOverlay
}

function preset(
  id: string,
  label: string,
  partial: Partial<ThemeTokens> & Pick<ThemeTokens, 'primaryColor' | 'bg' | 'btnBg'>,
  overlay: Partial<ThemeGifOverlay> = {}
): ThemePreset {
  const primary = partial.primaryColor
  const btnBg = partial.btnBg
  const btnText = partial.btnText ?? '#eeeeee'
  const bg = partial.bg
  const tokens: ThemeTokens = {
    ...THEME_TOKEN_DEFAULTS,
    primaryColor: primary,
    primaryHover: partial.primaryHover ?? lightenHex(primary),
    bg,
    bg2: partial.bg2 ?? darkenHex(bg),
    btnBg,
    btnBgHover: partial.btnBgHover ?? lightenHex(btnBg),
    btnText,
    btnTextHover: partial.btnTextHover ?? primary,
    btnBorder: partial.btnBorder ?? primary,
    btnBorderHover: partial.btnBorderHover ?? lightenHex(primary),
    tabBg: partial.tabBg ?? withAlpha(primary, 0.2),
    tabBgHover: partial.tabBgHover ?? withAlpha(primary, 0.4),
    tabText: partial.tabText ?? btnText,
    tabTextHover: partial.tabTextHover ?? btnText,
    tabBorder: partial.tabBorder ?? primary,
    tabBorderHover: partial.tabBorderHover ?? lightenHex(primary),
  }
  return {
    id,
    label,
    tokens,
    extras: { ...GIF_OVERLAY_DEFAULTS, ...overlay },
  }
}

export const THEME_PRESETS: ThemePreset[] = [
  preset('soundninja', 'SoundNinja', {
    primaryColor: '#00d4ff',
    bg: '#222831',
    bg2: '#1a1e25',
    btnBg: '#363f4d',
    btnBgHover: '#434e5f',
    btnText: '#eeeeee',
  }, { gifOverlay: 0.72, gifOverlayHover: 0.38 }),
  preset('soundninja-light', 'SoundNinja-Light', {
    primaryColor: '#0088aa',
    bg: '#eeeeee',
    bg2: '#d4d4d4',
    btnBg: '#7184a2',
    btnBgHover: '#8294b0',
    btnText: '#ffffff',
    tabText: '#222831',
    tabTextHover: '#222831',
  }, { gifOverlay: 0.55, gifOverlayHover: 0.22 }),
  preset('solarized', 'Solarized', {
    primaryColor: '#268bd2',
    bg: '#002b36',
    bg2: '#001f28',
    btnBg: '#073642',
    btnBgHover: '#0a4a5a',
    btnText: '#93a1a1',
    btnTextHover: '#268bd2',
  }, { gifOverlay: 0.78, gifOverlayHover: 0.42 }),
  preset('monokai', 'Monokai', {
    primaryColor: '#a6e22e',
    bg: '#272822',
    bg2: '#1d1e19',
    btnBg: '#3e3d32',
    btnBgHover: '#49483e',
    btnText: '#f8f8f2',
    btnTextHover: '#a6e22e',
  }, { gifOverlay: 0.70, gifOverlayHover: 0.36 }),
  preset('dracula', 'Dracula', {
    primaryColor: '#bd93f9',
    bg: '#282a36',
    bg2: '#1e2029',
    btnBg: '#44475a',
    btnBgHover: '#565a73',
    btnText: '#f8f8f2',
    btnTextHover: '#bd93f9',
  }, { gifOverlay: 0.74, gifOverlayHover: 0.38 }),
  preset('nord', 'Nord', {
    primaryColor: '#88c0d0',
    bg: '#2e3440',
    bg2: '#22272f',
    btnBg: '#3b4252',
    btnBgHover: '#434c5e',
    btnText: '#eceff4',
    btnTextHover: '#88c0d0',
  }, { gifOverlay: 0.74, gifOverlayHover: 0.40 }),
  preset('gruvbox', 'Gruvbox', {
    primaryColor: '#fabd2f',
    bg: '#282828',
    bg2: '#1e1e1e',
    btnBg: '#3c3836',
    btnBgHover: '#504945',
    btnText: '#ebdbb2',
    btnTextHover: '#fabd2f',
  }, { gifOverlay: 0.70, gifOverlayHover: 0.34 }),
  preset('cobalt', 'Cobalt', {
    primaryColor: '#ff9d00',
    bg: '#193549',
    bg2: '#132837',
    btnBg: '#1f4662',
    btnBgHover: '#29587a',
    btnText: '#ffffff',
    btnTextHover: '#ff9d00',
  }, { gifOverlay: 0.68, gifOverlayHover: 0.32 }),
]

export const DEFAULT_THEME_ID = 'soundninja'

const LEGACY_THEME_IDS = new Set([
  'dark-cyan',
  'dark-purple',
  'dark-orange',
  'dark-green',
  'dark-pink',
])

export function getPreset(id: string): ThemePreset | undefined {
  return THEME_PRESETS.find((p) => p.id === id)
}

/** Resolve a persisted theme id; legacy dark-* ids → soundninja. */
export function normalizeThemeId(id: string | undefined | null): string {
  if (!id) return DEFAULT_THEME_ID
  if (LEGACY_THEME_IDS.has(id)) return DEFAULT_THEME_ID
  if (id === 'custom' || id.startsWith('file:')) return id
  if (getPreset(id)) return id
  return DEFAULT_THEME_ID
}
