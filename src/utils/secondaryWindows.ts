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

const inflight = new Map<string, Promise<void>>()

/** Match main-window chrome prefs when spawning tool windows. */
function chromeDecorations(): boolean {
  try {
    const s = useAppSettingsStore()
    return s.titlebarMode === 'system' && !s.hideTitlebar
  } catch {
    return false
  }
}

/**
 * Show + focus a tool window, creating it if needed.
 *
 * Creation happens in Rust (`open_tool_window`): a window built from JS gets
 * wry's default WebView2 command line, which no longer matches the environment
 * the main window created, and WebView2 then silently drops the window.
 */
export async function openSecondaryWindow(spec: SecondaryWindowSpec): Promise<void> {
  const pending = inflight.get(spec.label)
  if (pending) return pending

  const open = invoke('open_tool_window', {
    spec: {
      label: spec.label,
      url: spec.url,
      title: spec.title,
      width: spec.width,
      height: spec.height,
      minWidth: spec.minWidth ?? null,
      minHeight: spec.minHeight ?? null,
      decorations: chromeDecorations(),
    },
  }).finally(() => {
    inflight.delete(spec.label)
  }) as Promise<void>

  inflight.set(spec.label, open)
  return open
}

/** Destroy a secondary window so its WebView2 process is released. */
export async function destroySecondaryWindow(label: string): Promise<void> {
  const win = await WebviewWindow.getByLabel(label)
  if (!win) return
  await win.destroy()
}
