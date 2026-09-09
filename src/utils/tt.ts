/** Translate a key from Pinia / plain utils (Vue setup not required). */
export function tt(key: string): string {
  try {
    const { $i18n } = useNuxtApp()
    return String($i18n.t(key))
  } catch {
    return key
  }
}
