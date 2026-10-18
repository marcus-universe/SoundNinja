import { reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { useAppSettingsStore } from '~/stores/appSettings'
import { useJsonHandelingStore } from '~/stores/jsonHandeling'
import { isRemote } from '~/utils/runtime'
import { THEME_INLINE_VARS } from '~/utils/themeTokens'

/** App settings mirrored to browsers. Locale and UI scale stay on the device. */
export const WEBUI_SETTING_KEYS = [
  'navbarSide',
  'navbarTooltips',
  'showTagBadges',
  'gifTitleChip',
  'basicButtonColors',
  'outputSource',
  'outputHost',
  'outputVolume',
  'asioLeftChannel',
  'asioRightChannel',
  'asioInLeftChannel',
  'asioInRightChannel',
  'inputSource',
  'inputHost',
  'inputVolume',
  'inputLoopback',
  'audioMigrated',
  'hotkeys',
  'soundTriggersGlobal',
  'themesPath',
  'projectsPath',
  'titlebarMode',
  'hideTitlebar',
  'checkUpdatesOnStart',
] as const

type SettingKey = (typeof WEBUI_SETTING_KEYS)[number]

export const remoteLink = reactive({
  state: 'connecting' as 'connecting' | 'open' | 'closed' | 'rejected' | 'hostDown',
})

let applying = false
export function isApplyingRemote() {
  return applying
}

function normPath(p: string) {
  return (p || '').replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase()
}

function under(path: string, root: string) {
  if (!path || !root) return false
  const a = normPath(path)
  const b = normPath(root)
  return a === b || a.startsWith(b + '/')
}

function esc(seg: string) {
  return seg.replace(/~/g, '~0').replace(/\//g, '~1')
}

function pointer(base: string, key: string) {
  return `${base}/${esc(key)}`
}

function idOf(v: unknown): string | null {
  if (!v || typeof v !== 'object') return null
  const o = v as Record<string, unknown>
  if (typeof o.id === 'string' && o.id) return `id:${o.id}`
  if (typeof o.path === 'string' && o.path) return `path:${o.path}`
  if (typeof o.name === 'string' && o.name) return `name:${o.name}`
  return null
}

type Op = { op: 'set' | 'del'; path: string; value?: unknown }

function diff(prev: unknown, next: unknown, base: string, ops: Op[]) {
  if (Object.is(prev, next)) return
  if (Array.isArray(prev) && Array.isArray(next) && prev.length === next.length && prev.length > 0) {
    const same = prev.every((item, i) => {
      const a = idOf(item)
      return a != null && a === idOf(next[i])
    })
    if (same) {
      for (let i = 0; i < next.length; i++) diff(prev[i], next[i], `${base}/${i}`, ops)
      return
    }
  }
  if (
    prev && next
    && typeof prev === 'object' && typeof next === 'object'
    && !Array.isArray(prev) && !Array.isArray(next)
  ) {
    const a = prev as Record<string, unknown>
    const b = next as Record<string, unknown>
    const keys = new Set([...Object.keys(a), ...Object.keys(b)])
    for (const key of keys) {
      const path = pointer(base, key)
      if (!(key in b)) ops.push({ op: 'del', path })
      else if (!(key in a)) ops.push({ op: 'set', path, value: b[key] })
      else diff(a[key], b[key], path, ops)
    }
    return
  }
  ops.push({ op: 'set', path: base || '/', value: next })
}

function boardState() {
  const json = useJsonHandelingStore()
  const appSettings = useAppSettingsStore()
  const settings: Record<string, unknown> = {}
  for (const key of WEBUI_SETTING_KEYS) settings[key] = appSettings[key]
  settings.locale = appSettings.locale
  return JSON.parse(JSON.stringify({
    config: json.configFile,
    projectPath: json.currentProjectPath,
    settings,
  }))
}

async function publish(kind: string, payload: unknown) {
  try {
    await invoke('webui_publish', { kind, payload })
  } catch (e) {
    console.error('webui_publish failed', kind, e)
  }
}

export function publishThemeFromDom() {
  if (isRemote || typeof document === 'undefined') return
  const css = document.getElementById('sn-custom-theme')?.textContent || ''
  const style = document.documentElement.style
  const vars: Record<string, string> = {}
  for (let i = 0; i < style.length; i++) {
    const name = style.item(i)
    if (!name.startsWith('--') || name === '--ui-scale' || name === '--topbar_height') continue
    vars[name] = style.getPropertyValue(name)
  }
  void publish('theme', { css, vars })
}

function publishAllow() {
  const appSettings = useAppSettingsStore()
  const prefixes = [appSettings.themesPath, appSettings.fontsPath].filter(Boolean)
  void publish('allow', { prefixes })
}

let last: ReturnType<typeof boardState> | null = null
let diffTimer = 0

function flushDiff() {
  diffTimer = 0
  let next: ReturnType<typeof boardState>
  try { next = boardState() } catch { return }
  if (!last) {
    last = next
    void publish('snapshot', next)
    publishThemeFromDom()
    publishAllow()
    return
  }
  const ops: Op[] = []
  diff(last, next, '', ops)
  last = next
  if (!ops.length) return
  if (ops.length > 48) void publish('snapshot', next)
  else void publish('patch', { ops })
}

function scheduleDiff() {
  if (diffTimer) return
  diffTimer = window.setTimeout(flushDiff, 32)
}

function tokens(path: string) {
  return path.split('/').filter(Boolean).map((part) => part.replace(/~1/g, '/').replace(/~0/g, '~'))
}

function writeAt(root: Record<string, unknown>, parts: string[], op: Op) {
  if (!parts.length) return
  let cur: unknown = root
  for (let i = 0; i < parts.length - 1; i++) {
    const key = parts[i]
    if (!cur || typeof cur !== 'object') return
    const obj = cur as Record<string, unknown>
    if (obj[key] == null || typeof obj[key] !== 'object') obj[key] = {}
    cur = obj[key]
  }
  if (!cur || typeof cur !== 'object') return
  const leaf = parts[parts.length - 1]
  const obj = cur as Record<string, unknown>
  if (op.op === 'del') delete obj[leaf]
  else obj[leaf] = op.value
}

function applyOps(ops: Op[]) {
  const json = useJsonHandelingStore()
  const appSettings = useAppSettingsStore()
  for (const op of ops) {
    const parts = tokens(op.path || '')
    if (!parts.length) continue
    if (parts[0] === 'projectPath') {
      if (op.op === 'set') json.currentProjectPath = (op.value as string) || ''
      continue
    }
    if (parts[0] === 'config') writeAt(json.configFile as unknown as Record<string, unknown>, parts.slice(1), op)
    else if (parts[0] === 'settings') writeAt(appSettings as unknown as Record<string, unknown>, parts.slice(1), op)
  }
}

function applyThemeMsg(theme: { css?: string; vars?: Record<string, string> } | null) {
  if (!theme || typeof document === 'undefined') return
  const root = document.documentElement
  for (const name of THEME_INLINE_VARS) root.style.removeProperty(name)
  const vars = theme.vars || {}
  for (const [name, value] of Object.entries(vars)) {
    if (name === '--ui-scale' || name === '--topbar_height') continue
    root.style.setProperty(name, String(value))
  }
  let el = document.getElementById('sn-custom-theme')
  const css = theme.css || ''
  if (!css) {
    el?.remove()
    return
  }
  if (!el) {
    el = document.createElement('style')
    el.id = 'sn-custom-theme'
    document.head.appendChild(el)
  }
  el.textContent = css
}

let seq = 0
let sawSnapshot = false

function onRemoteMessage(msg: {
  type?: string
  seq?: number
  state?: { config?: unknown; projectPath?: string; settings?: Record<string, unknown> }
  theme?: { css?: string; vars?: Record<string, string> }
  ops?: Op[]
}, hooks: { setLocale: (code: string) => Promise<void> | void; onReady: () => void }) {
  if (msg.type === 'hostDown') {
    remoteLink.state = 'hostDown'
    return
  }
  if (msg.type === 'hostUp') {
    if (remoteLink.state !== 'rejected') remoteLink.state = 'open'
    return
  }
  if (msg.type === 'theme') {
    applyThemeMsg(msg.theme || null)
    return
  }
  if (msg.type === 'snapshot') {
    if (typeof msg.seq === 'number' && msg.seq <= seq && sawSnapshot) return
    if (typeof msg.seq === 'number') seq = msg.seq
    const state = msg.state
    if (state?.config) {
      const json = useJsonHandelingStore()
      const appSettings = useAppSettingsStore()
      applying = true
      json.configFile = state.config as typeof json.configFile
      json.currentProjectPath = state.projectPath || ''
      const settings = state.settings || {}
      for (const key of WEBUI_SETTING_KEYS) {
        if (key in settings) (appSettings as unknown as Record<string, unknown>)[key] = settings[key]
      }
      try { json.applyBoardFilter() } catch { /* board not ready */ }
      try { json.syncFilterSessionFromSettings() } catch { /* tags optional */ }
      applying = false
      if (!sawSnapshot) {
        sawSnapshot = true
        const saved = localStorage.getItem('sn-remote-locale')
        const hostLocale = typeof settings.locale === 'string' ? settings.locale : ''
        if (!saved && hostLocale) void hooks.setLocale(hostLocale)
        const fonts = appSettings.fontsPath
        if (fonts) {
          import('~/composables/customFonts').then((m) => m.loadCustomFonts(fonts)).catch(() => {})
        }
      }
    }
    if (msg.theme) applyThemeMsg(msg.theme)
    hooks.onReady()
    return
  }
  if (msg.type === 'patch') {
    if (typeof msg.seq === 'number' && msg.seq <= seq) return
    if (typeof msg.seq === 'number') seq = msg.seq
    applying = true
    applyOps(msg.ops || [])
    try {
      const json = useJsonHandelingStore()
      json.applyBoardFilter()
      json.syncFilterSessionFromSettings()
    } catch { /* ignore */ }
    applying = false
  }
}

export function bootRemoteSession(hooks: { setLocale: (code: string) => Promise<void> | void; onReady: () => void }) {
  document.documentElement.classList.add('sn-remote')
  const onMsg = (e: Event) => onRemoteMessage((e as CustomEvent).detail || {}, hooks)
  const onStatus = (e: Event) => {
    const state = (e as CustomEvent).detail?.state
    if (state === 'connecting' || state === 'open' || state === 'closed' || state === 'rejected') {
      remoteLink.state = state
    }
    if (state === 'rejected' || state === 'closed') hooks.onReady()
  }
  window.addEventListener('sn-webui', onMsg)
  window.addEventListener('sn-webui-status', onStatus)
  const early = (window as Window & { __SN_WEBUI_STATE__?: string }).__SN_WEBUI_STATE__
  if (early === 'connecting' || early === 'open' || early === 'closed' || early === 'rejected') {
    remoteLink.state = early
    if (early === 'rejected' || early === 'closed') hooks.onReady()
  }
  const buffered = (window as Window & { __SN_WEBUI_LOG__?: unknown[] }).__SN_WEBUI_LOG__ || []
  for (const msg of buffered) onRemoteMessage(msg as Parameters<typeof onRemoteMessage>[0], hooks)
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
      import('~/utils/runtime').then((m) => m.syncWakeLock()).catch(() => {})
    }
  })
  import('~/utils/runtime').then((m) => m.syncWakeLock()).catch(() => {})
}

async function allowGifPaths(result: unknown) {
  if (!Array.isArray(result)) return
  const files: string[] = []
  for (const row of result) {
    if (!row || typeof row !== 'object') continue
    const rec = row as { path?: string; posterPath?: string }
    if (rec.path) files.push(rec.path)
    if (rec.posterPath) files.push(rec.posterPath)
  }
  if (files.length) await publish('allow', { files })
}

function pathAllowed(cmd: string, args: Record<string, unknown>) {
  const appSettings = useAppSettingsStore()
  const json = useJsonHandelingStore()
  const themes = appSettings.themesPath || ''
  if (cmd === 'read_text_file_abs' || cmd === 'read_file_base64_abs') {
    return under(String(args.path || ''), themes)
  }
  if (cmd === 'list_dir_files_abs') return under(String(args.dir || ''), themes)
  if (cmd === 'gif_cache_paths') {
    return normPath(String(args.projectDb || '')) === normPath(json.currentProjectPath || '')
  }
  return true
}

async function runAction(storeName: string, name: string, args: unknown) {
  const store = storeName === 'appSettings' ? useAppSettingsStore() : useJsonHandelingStore()
  const fn = (store as unknown as Record<string, unknown>)[name]
  if (typeof fn !== 'function') return
  const list = Array.isArray(args) ? args : []
  await fn.apply(store, list)
}

async function onHostMessage(msg: {
  type?: string
  store?: string
  name?: string
  args?: unknown
  id?: number
  cmd?: string
  payload?: unknown
}) {
  if (msg.type === 'action' && msg.store && msg.name) {
    try { await runAction(msg.store, msg.name, msg.args) } catch (e) {
      console.error('webui action failed', msg.store, msg.name, e)
    }
    return
  }
  if (msg.type === 'rpc' && msg.cmd && msg.id) {
    const args = (msg.args && typeof msg.args === 'object') ? msg.args as Record<string, unknown> : {}
    if (!pathAllowed(msg.cmd, args)) {
      await invoke('webui_reply', { id: msg.id, result: null, error: 'Path is not available to remote' })
      return
    }
    try {
      const result = await invoke(msg.cmd, args)
      if (msg.cmd === 'gif_cache_paths') await allowGifPaths(result)
      await invoke('webui_reply', { id: msg.id, result: result ?? null, error: null })
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e)
      await invoke('webui_reply', { id: msg.id, result: null, error })
    }
    return
  }
  if (msg.type === 'play' && typeof (msg as { id?: string }).id === 'string') {
    window.dispatchEvent(new CustomEvent('sn:play-sound-id', { detail: (msg as { id: string }).id }))
    return
  }
  if (msg.type === 'stop') {
    const id = (msg as { id?: string }).id
    if (id) window.dispatchEvent(new CustomEvent('sn:stop-sound-id', { detail: id }))
    else {
      invoke('stop_all').catch(() => {})
      window.dispatchEvent(new CustomEvent('sn:stop-all'))
    }
  }
}

const RELAY = ['sound_finished', 'playing_changed']

/** Desktop main window: publish the board and run actions the browsers send. */
export function startWebuiHost(): () => void {
  if (isRemote) return () => {}
  const stops: Array<() => void> = []
  listen('webui_in', (e) => {
    void onHostMessage((e?.payload || {}) as Parameters<typeof onHostMessage>[0])
  }).then((off) => stops.push(off)).catch(() => {})

  for (const name of RELAY) {
    listen(name, (e) => {
      void publish('event', { name, payload: e?.payload ?? null })
    }).then((off) => stops.push(off)).catch(() => {})
  }

  const json = useJsonHandelingStore()
  const appSettings = useAppSettingsStore()
  stops.push(json.$subscribe(() => scheduleDiff(), { detached: true }))
  stops.push(appSettings.$subscribe(() => scheduleDiff(), { detached: true }))
  scheduleDiff()
  void publish('host', { up: true })

  const onLeave = () => { void publish('host', { up: false }) }
  window.addEventListener('beforeunload', onLeave)
  stops.push(() => window.removeEventListener('beforeunload', onLeave))
  return () => {
    if (diffTimer) clearTimeout(diffTimer)
    for (const stop of stops) stop()
  }
}

export type { SettingKey }
