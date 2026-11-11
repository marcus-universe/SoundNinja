/** Translate a key from Pinia / plain utils (Vue setup not required). */
export function tt(key: string, params?: Record<string, string | number>): string {
  try {
    const { $i18n } = useNuxtApp()
    return String(params ? $i18n.t(key, params) : $i18n.t(key))
  } catch {
    return key
  }
}
