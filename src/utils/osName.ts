import { ref } from 'vue'
import { platform } from '@tauri-apps/plugin-os'

/**
 * OS name. Seeded from the user agent so the first paint already knows Linux.
 * WebKitGTK draws a native `<select>` popup that ignores the theme; waiting
 * on the async plugin left that white menu up on the Steam Frame.
 * The Tauri plugin overwrites this once it answers.
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
  platform()
    .then((name) => { osName.value = name })
    .catch(() => { /* keep the sync guess */ })
}
