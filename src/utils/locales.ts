export const APP_LOCALES = ['en', 'de', 'es', 'fr', 'ja', 'zh-Hans'] as const

export type AppLocale = (typeof APP_LOCALES)[number]

export function isAppLocale(value: string | null | undefined): value is AppLocale {
  return !!value && (APP_LOCALES as readonly string[]).includes(value)
}

/** Map OS / installer language tags onto a supported locale. */
export function resolveAppLocale(raw: string | null | undefined, fallback: AppLocale = 'en'): AppLocale {
  if (!raw) return fallback
  const lower = raw.trim().toLowerCase().replace('_', '-')
  if (isAppLocale(raw)) return raw
  if (lower.startsWith('zh')) return 'zh-Hans'
  const short = lower.split('-')[0]
  if (isAppLocale(short)) return short
  return fallback
}
