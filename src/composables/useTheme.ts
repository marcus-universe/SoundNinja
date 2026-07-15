// Runtime application of the per-project light/dark theme color model.
// Sets the CSS variable pairs on :root and resolves the active mode into the
// concrete --color-bg / --color-btn / --sound-text used by the UI.
import { readableTextColor } from '~/utils/contrast'

const DEFAULTS = {
  primaryColor: '#00d4ff',
  bgLight: '#eeeeee',
  bgDark: '#222831',
  btnLight: '#7184a2',
  btnDark: '#363f4d',
  textLight: '#eeeeee',
  textDark: '#222831',
}

type ThemeColors = typeof DEFAULTS & { themeMode: 'light' | 'dark' }

function resolve(settings: Record<string, unknown> | undefined | null): ThemeColors {
  const g = (v: unknown, d: string) => (v && String(v).trim()) || d
  return {
    primaryColor: g(settings?.primaryColor, DEFAULTS.primaryColor),
    bgLight: g(settings?.bgLight, DEFAULTS.bgLight),
    bgDark: g(settings?.bgDark, DEFAULTS.bgDark),
    btnLight: g(settings?.btnLight, DEFAULTS.btnLight),
    btnDark: g(settings?.btnDark, DEFAULTS.btnDark),
    textLight: g(settings?.textLight, DEFAULTS.textLight),
    textDark: g(settings?.textDark, DEFAULTS.textDark),
    themeMode: settings?.themeMode === 'light' ? 'light' : 'dark',
  }
}

/** Applies the full color model (both pairs + accent) and resolves the mode. */
export function applyThemeColors(settings: Record<string, unknown> | undefined | null): void {
  if (typeof document === 'undefined') return
  const c = resolve(settings)
  const root = document.documentElement
  root.style.setProperty('--primary_color', c.primaryColor)
  root.style.setProperty('--color-bg-light', c.bgLight)
  root.style.setProperty('--color-bg-dark', c.bgDark)
  root.style.setProperty('--color-btn-light', c.btnLight)
  root.style.setProperty('--color-btn-dark', c.btnDark)
  root.style.setProperty('--text-light', c.textLight)
  root.style.setProperty('--text-dark', c.textDark)
  applyThemeMode(c.themeMode, settings)
}

/** Resolves the given mode to the concrete bg/btn/text vars + root class. */
export function applyThemeMode(
  mode: 'light' | 'dark',
  settings: Record<string, unknown> | undefined | null
): void {
  if (typeof document === 'undefined') return
  const c = resolve(settings)
  const light = mode === 'light'
  const root = document.documentElement
  root.classList.toggle('theme-light', light)
  root.classList.toggle('theme-dark', !light)
  const bg = light ? c.bgLight : c.bgDark
  const btn = light ? c.btnLight : c.btnDark
  root.style.setProperty('--color-bg', bg)
  root.style.setProperty('--color-btn', btn)
  root.style.setProperty('--sound-text', readableTextColor(btn, c.textLight, c.textDark))
}

export const THEME_COLOR_DEFAULTS = DEFAULTS
