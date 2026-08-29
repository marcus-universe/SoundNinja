import { invoke } from '@tauri-apps/api/core'
import { openUrl } from '@tauri-apps/plugin-opener'

export const KLIPY_HOME_URL = 'https://klipy.com'
export const KLIPY_PARTNER_URL = 'https://partner.klipy.com'

/** Open https URL in the OS browser — never the app WebView. */
export async function openInSystemBrowser(url: string): Promise<void> {
  try {
    await invoke('open_external_url', { url })
  } catch (e1) {
    try {
      await openUrl(url)
    } catch (e2) {
      console.error('open_external_url failed', e1, e2)
    }
  }
}
