import { invoke } from '@tauri-apps/api/core'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

export type SecondaryWindowSpec = {
  label: string
  url: string
  title: string
  width: number
  height: number
  minWidth?: number
  minHeight?: number
}

export const RECORD_EDITOR: SecondaryWindowSpec = {
  label: 'record-editor',
  url: '#/record-editor',
  title: 'Record Editor',
  width: 960,
  height: 780,
  minWidth: 720,
  minHeight: 560,
}

export const THEME_CREATOR: SecondaryWindowSpec = {
  label: 'theme-creator',
  url: '#/theme-creator',
  title: 'Theme Creator',
  width: 940,
  height: 720,
  minWidth: 720,
  minHeight: 560,
}

export const PLAYING_LIST: SecondaryWindowSpec = {
  label: 'playing-list',
  url: '#/playing-list',
  title: 'Playing List',
  width: 520,
  height: 560,
  minWidth: 380,
  minHeight: 320,
}

const inflight = new Map<string, Promise<WebviewWindow>>()

function waitCreated(win: WebviewWindow): Promise<void> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`Window '${win.label}' create timeout`))
    }, 60_000)
    win.once('tauri://created', () => {
      clearTimeout(timer)
      resolve()
    })
    win.once('tauri://error', (e) => {
      clearTimeout(timer)
      reject(e)
    })
  })
}

/** Match main-window chrome prefs when spawning tool windows. */
function chromeOptions(): { decorations: boolean; nativeChrome: boolean; hidden: boolean } {
  try {
    const s = useAppSettingsStore()
    const hidden = !!s.hideTitlebar
    const nativeChrome = s.titlebarMode === 'system'
    return {
      decorations: nativeChrome && !hidden,
      nativeChrome,
      hidden,
    }
  } catch {
    return { decorations: false, nativeChrome: false, hidden: false }
  }
}

async function applyChromeToAll(nativeChrome: boolean, hidden: boolean) {
  try {
    await invoke('set_window_chrome', { nativeChrome, hidden })
  } catch (e) {
    console.warn('set_window_chrome failed', e)
  }
}

async function stripMenu(label: string) {
  try {
    await invoke('strip_window_menu_for', { label })
  } catch (e) {
    console.warn('strip_window_menu_for failed', e)
  }
}

/** Create (hidden) if needed. Keeps SPA warm for fast show later. */
export async function ensureSecondaryWindow(spec: SecondaryWindowSpec): Promise<WebviewWindow> {
  const existing = await WebviewWindow.getByLabel(spec.label)
  if (existing) return existing

  const pending = inflight.get(spec.label)
  if (pending) return pending

  const create = (async () => {
    try {
      const again = await WebviewWindow.getByLabel(spec.label)
      if (again) return again

      const chrome = chromeOptions()
      const win = new WebviewWindow(spec.label, {
        url: spec.url,
        title: spec.title,
        width: spec.width,
        height: spec.height,
        minWidth: spec.minWidth,
        minHeight: spec.minHeight,
        resizable: true,
        visible: false,
        focus: false,
        decorations: chrome.decorations,
      })
      await waitCreated(win)
      // Re-apply chrome so decorations/menu match main for every window.
      await applyChromeToAll(chrome.nativeChrome, chrome.hidden)
      await stripMenu(spec.label)
      return win
    } finally {
      inflight.delete(spec.label)
    }
  })()

  inflight.set(spec.label, create)
  return create
}

/** Show + focus a secondary window (creates hidden first if missing). */
export async function openSecondaryWindow(spec: SecondaryWindowSpec): Promise<WebviewWindow> {
  const win = await ensureSecondaryWindow(spec)
  const chrome = chromeOptions()
  await applyChromeToAll(chrome.nativeChrome, chrome.hidden)
  await stripMenu(spec.label)
  try {
    if (await win.isMinimized()) await win.unminimize()
  } catch { /* permission / platform may lack unminimize */ }
  await win.show()
  await win.setFocus()
  // Windows/Linux: force raise when caller still holds focus.
  try {
    if (!(await win.isFocused())) {
      await win.setAlwaysOnTop(true)
      await win.setFocus()
      await win.setAlwaysOnTop(false)
    }
  } catch { /* best-effort */ }
  return win
}

export async function hideSecondaryWindow(label: string): Promise<void> {
  const win = await WebviewWindow.getByLabel(label)
  if (!win) return
  await win.hide()
}

/** Background-warm secondary tool windows after main is ready. */
export function prewarmSecondaryWindows(): void {
  void ensureSecondaryWindow(RECORD_EDITOR).catch((e) => {
    console.warn('prewarm record-editor failed', e)
  })
  void ensureSecondaryWindow(THEME_CREATOR).catch((e) => {
    console.warn('prewarm theme-creator failed', e)
  })
  void ensureSecondaryWindow(PLAYING_LIST).catch((e) => {
    console.warn('prewarm playing-list failed', e)
  })
}
