/** Browser mirror vs the desktop webview. The bridge script sets the flag before the SPA boots. */

export const REMOTE_SCALE_KEY = 'sn-remote-ui-scale'
export const REMOTE_LOCALE_KEY = 'sn-remote-locale'
export const REMOTE_WAKE_KEY = 'sn-remote-wake'

type RemoteWindow = Window & { __SN_REMOTE__?: boolean }

export const isRemote =
  import.meta.client
  && typeof window !== 'undefined'
  && !!(window as RemoteWindow).__SN_REMOTE__

export function clampUiScale(n: number): number {
  if (!Number.isFinite(n)) return 1
  return Math.min(2, Math.max(0.5, Math.round(n * 100) / 100))
}

export function applyUiScale(n: number): number {
  const v = clampUiScale(n)
  if (typeof document !== 'undefined') {
    document.documentElement.style.setProperty('--ui-scale', String(v))
  }
  return v
}

let wakeLock: WakeLockSentinel | null = null

export async function syncWakeLock(): Promise<void> {
  if (!isRemote || typeof navigator === 'undefined' || !('wakeLock' in navigator)) return
  const on = localStorage.getItem(REMOTE_WAKE_KEY) === '1'
  if (!on) {
    try { await wakeLock?.release() } catch { /* already released */ }
    wakeLock = null
    return
  }
  try {
    wakeLock = await navigator.wakeLock.request('screen')
  } catch { /* browser denied it */ }
}
