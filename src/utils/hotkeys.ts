export const APP_HOTKEY_ACTIONS = [
  'save',
  'saveAs',
  'undo',
  'redo',
  'search',
  'stopAll',
] as const

export type AppHotkeyAction = (typeof APP_HOTKEY_ACTIONS)[number]

export const DEFAULT_APP_HOTKEYS: Record<AppHotkeyAction, string> = {
  save: 'Ctrl+S',
  saveAs: 'Ctrl+Shift+S',
  undo: 'Ctrl+Z',
  redo: 'Ctrl+Shift+Z',
  search: 'Ctrl+F',
  stopAll: '',
}

export interface SoundHotkey {
  id: string
  soundId: string
  combo: string
}

const MOD_KEYS = new Set(['Control', 'Shift', 'Alt', 'Meta', 'OS'])

function normalizeKey(e: KeyboardEvent): string | null {
  if (MOD_KEYS.has(e.key)) return null
  const code = e.code || ''
  if (code.startsWith('Numpad')) return code
  if (/^F\d{1,2}$/.test(e.key)) return e.key
  if (e.key === ' ') return 'Space'
  if (e.key === 'Escape') return 'Escape'
  if (e.key === 'Backspace') return 'Backspace'
  if (e.key === 'Delete') return 'Delete'
  if (e.key === 'Enter') return 'Enter'
  if (e.key === 'Tab') return 'Tab'
  if (e.key.startsWith('Arrow')) return e.key
  if (e.key.length === 1) return e.key.toUpperCase()
  if (code.startsWith('Key') && code.length === 4) return code.slice(3)
  if (code.startsWith('Digit') && code.length === 6) return code.slice(5)
  return e.key.length ? e.key : null
}

/** Canonical combo, e.g. `Ctrl+Shift+K`, `F8`, `Numpad1`. */
export function eventToCombo(e: KeyboardEvent): string | null {
  const key = normalizeKey(e)
  if (!key) return null
  const parts: string[] = []
  if (e.ctrlKey || e.metaKey) parts.push('Ctrl')
  if (e.altKey) parts.push('Alt')
  if (e.shiftKey && key !== 'Shift') parts.push('Shift')
  parts.push(key)
  return parts.join('+')
}

/** Tauri global-shortcut accelerator. */
export function comboToAccelerator(combo: string): string {
  return combo.replace(/^Ctrl\+/, 'CommandOrControl+').replace(/\+Ctrl\+/g, '+CommandOrControl+')
}

export function parseAppHotkeys(raw: unknown): Record<AppHotkeyAction, string> {
  const out = { ...DEFAULT_APP_HOTKEYS }
  if (!raw || typeof raw !== 'object') return out
  const obj = raw as Record<string, unknown>
  for (const key of APP_HOTKEY_ACTIONS) {
    if (typeof obj[key] === 'string') out[key] = obj[key] as string
  }
  return out
}

export function parseSoundHotkeys(raw: unknown): SoundHotkey[] {
  if (!Array.isArray(raw)) return []
  const out: SoundHotkey[] = []
  for (const row of raw) {
    if (!row || typeof row !== 'object') continue
    const r = row as Record<string, unknown>
    const id = typeof r.id === 'string' ? r.id : ''
    const soundId = typeof r.soundId === 'string' ? r.soundId : ''
    const combo = typeof r.combo === 'string' ? r.combo : ''
    if (!id) continue
    out.push({ id, soundId, combo })
  }
  return out
}

export function newHotkeyRowId(): string {
  const buf = new Uint8Array(6)
  crypto.getRandomValues(buf)
  return Array.from(buf, (b) => b.toString(16).padStart(2, '0')).join('')
}
