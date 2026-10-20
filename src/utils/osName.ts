import { ref } from 'vue'
import { platform } from '@tauri-apps/plugin-os'

/**
 * OS name. Seeded from the user agent so the first paint already knows Linux.
 * WebKitGTK draws a native `<select>` popup that ignores the theme, so the
 * first paint must already know Linux. plugin-os `platform()` is synchronous.
 */
function detectOsSync(): string {
  if (typeof navigator === 'undefined') return ''
  const blob = `${navigator.userAgent || ''} ${navigator.platform || ''}`
  if (/android/i.test(blob)) return 'android'
  if (/linux/i.test(blob)) return 'linux'
  if (/windows|win32|win64/i.test(blob)) return 'windows'
  if (/mac os|macintosh/i.test(blob)) return 'macos'
  return ''
}

export const osName = ref(detectOsSync())

let started = false

export function ensureOs() {
  if (started || !import.meta.client) return
  started = true
  if (!osName.value) osName.value = detectOsSync()
  // plugin-os `platform()` is synchronous (a string). Calling `.then` on it
  // throws in WebKit and the Theme Creator page dies with a 500.
  try {
    const name = platform() as string | Promise<string> | undefined
    if (typeof name === 'string' && name) osName.value = name
    else if (name && typeof (name as Promise<string>).then === 'function') {
      ;(name as Promise<string>).then((next) => { osName.value = next }).catch(() => {})
    }
  } catch { /* keep the sync guess */ }
}
