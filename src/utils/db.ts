import Database from '@tauri-apps/plugin-sql'

// ── Types (mirrors the shape the components already consume) ───────────────────
export interface SoundFile {
  name: string
  path: string
  volume: number
  tabs: string[]
  active: boolean
  index: number
  tabIndexes: Record<string, number>
  color?: string
}

export interface TabEntry {
  name: string
  color?: string
}

export interface Separator {
  id: string
  tab: string
  position: number
}

export interface Settings {
  theme: string
  customCss: string
  outputSource: string
  stopOnRetrigger: boolean
  overlapSounds: boolean
  cacheMaxSizeMib?: number
  cacheMaxEntryMib?: number
  outputVolume?: number
  /** Force every sound button to the height of the tallest one in the tab. */
  uniformButtonHeight?: boolean
  /** Allow drag-and-drop reordering of sounds/tabs (default on). */
  allowReorder?: boolean
  /** Active theme mode for this project. */
  themeMode?: 'dark' | 'light'
  /** Accent color (single, shared by both modes). */
  primaryColor?: string
  /** Background color for light mode. */
  bgLight?: string
  /** Background color for dark mode. */
  bgDark?: string
  /** Button background for light mode. */
  btnLight?: string
  /** Button background for dark mode. */
  btnDark?: string
  /** Text color used on light-mode surfaces. */
  textLight?: string
  /** Text color used on dark-mode surfaces. */
  textDark?: string
  /** Audio driver/host name (e.g. 'WASAPI', 'ASIO'). */
  outputHost?: string
  /** ASIO left-channel index (0-based). Only used when outputHost === 'ASIO'. */
  asioLeftChannel?: number
  /** ASIO right-channel index (0-based). Only used when outputHost === 'ASIO'. */
  asioRightChannel?: number
  /** Enable GPU-accelerated DSP (experimental; only shown when discrete GPU detected). */
  gpuAudioEnabled?: boolean
}

export interface ProjectConfig {
  settings: Settings
  tabList: TabEntry[]
  files: SoundFile[]
  separators: Separator[]
}

export function defaultSettings(): Settings {
  return {
    theme: 'dark-cyan',
    customCss: '',
    outputSource: 'default',
    stopOnRetrigger: true,
    overlapSounds: false,
    uniformButtonHeight: false,
    allowReorder: true,
    themeMode: 'dark',
    primaryColor: '#00d4ff',
    bgLight: '#eeeeee',
    bgDark: '#222831',
    btnLight: '#7184a2',
    btnDark: '#363f4d',
    textLight: '#eeeeee',
    textDark: '#222831',
    outputHost: 'WASAPI',
    asioLeftChannel: undefined,
    asioRightChannel: undefined,
  }
}

export function emptyConfig(): ProjectConfig {
  return { settings: defaultSettings(), tabList: [], files: [], separators: [] }
}

// ── Connection handling ───────────────────────────────────────────────────────
let db: Database | null = null
let currentUrl: string | null = null

function toUrl(dbAbsPath: string): string {
  return 'sqlite:' + dbAbsPath.replace(/\\/g, '/')
}

/** Opens (and caches) a project database, initialising its schema. */
export async function openDb(dbAbsPath: string): Promise<Database> {
  const url = toUrl(dbAbsPath)
  if (db && currentUrl === url) return db
  if (db) {
    try { await db.close() } catch { /* ignore */ }
    db = null
  }
  db = await Database.load(url)
  currentUrl = url
  await initSchema(db)
  return db
}

export function getDb(): Database | null {
  return db
}

async function initSchema(d: Database): Promise<void> {
  await d.execute(`CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT
  )`)
  await d.execute(`CREATE TABLE IF NOT EXISTS tabs (
    name TEXT PRIMARY KEY,
    color TEXT,
    position INTEGER
  )`)
  await d.execute(`CREATE TABLE IF NOT EXISTS sounds (
    path TEXT PRIMARY KEY,
    name TEXT,
    volume REAL,
    color TEXT,
    global_index INTEGER,
    active INTEGER
  )`)
  await d.execute(`CREATE TABLE IF NOT EXISTS sound_tabs (
    sound_path TEXT,
    tab TEXT,
    tab_index INTEGER
  )`)
  await d.execute(`CREATE TABLE IF NOT EXISTS separators (
    id TEXT PRIMARY KEY,
    tab TEXT,
    position INTEGER
  )`)
}

// ── Load ──────────────────────────────────────────────────────────────────────
export async function loadConfig(d: Database): Promise<ProjectConfig> {
  const settings = defaultSettings()
  const sRows = await d.select<{ key: string; value: string }[]>(
    'SELECT key, value FROM settings'
  )
  for (const { key, value } of sRows) {
    switch (key) {
      case 'theme': settings.theme = value; break
      case 'customCss': settings.customCss = value; break
      case 'outputSource': settings.outputSource = value; break
      case 'stopOnRetrigger': settings.stopOnRetrigger = value === 'true'; break
      case 'overlapSounds': settings.overlapSounds = value === 'true'; break
      case 'cacheMaxSizeMib': settings.cacheMaxSizeMib = Number(value); break
      case 'cacheMaxEntryMib': settings.cacheMaxEntryMib = Number(value); break
      case 'outputVolume': settings.outputVolume = Number(value); break
      case 'uniformButtonHeight': settings.uniformButtonHeight = value === 'true'; break
      case 'allowReorder': settings.allowReorder = value === 'true'; break
      case 'themeMode': settings.themeMode = value === 'light' ? 'light' : 'dark'; break
      case 'primaryColor': settings.primaryColor = value; break
      case 'bgLight': settings.bgLight = value; break
      case 'bgDark': settings.bgDark = value; break
      case 'btnLight': settings.btnLight = value; break
      case 'btnDark': settings.btnDark = value; break
      case 'textLight': settings.textLight = value; break
      case 'textDark': settings.textDark = value; break
      case 'outputHost': settings.outputHost = value; break
      case 'asioLeftChannel': settings.asioLeftChannel = Number(value); break
      case 'asioRightChannel': settings.asioRightChannel = Number(value); break
    }
  }

  const tabRows = await d.select<{ name: string; color: string | null; position: number }[]>(
    'SELECT name, color, position FROM tabs ORDER BY position ASC'
  )
  const tabList: TabEntry[] = tabRows.map((t) => ({
    name: t.name,
    ...(t.color ? { color: t.color } : {}),
  }))

  const soundRows = await d.select<
    { path: string; name: string; volume: number; color: string | null; global_index: number; active: number }[]
  >('SELECT path, name, volume, color, global_index, active FROM sounds ORDER BY global_index ASC')

  const tabLinks = await d.select<{ sound_path: string; tab: string; tab_index: number }[]>(
    'SELECT sound_path, tab, tab_index FROM sound_tabs'
  )
  const linksByPath = new Map<string, { tab: string; tab_index: number }[]>()
  for (const l of tabLinks) {
    const arr = linksByPath.get(l.sound_path) ?? []
    arr.push(l)
    linksByPath.set(l.sound_path, arr)
  }

  const files: SoundFile[] = soundRows.map((s) => {
    const links = linksByPath.get(s.path) ?? []
    const tabs = ['All', ...links.map((l) => l.tab).filter((t) => t !== 'All')]
    const tabIndexes: Record<string, number> = {}
    for (const l of links) {
      if (l.tab !== 'All') tabIndexes[l.tab] = l.tab_index
    }
    return {
      path: s.path,
      name: s.name,
      volume: s.volume,
      index: s.global_index,
      active: s.active === 1,
      tabs,
      tabIndexes,
      ...(s.color ? { color: s.color } : {}),
    }
  })

  const sepRows = await d.select<{ id: string; tab: string; position: number }[]>(
    'SELECT id, tab, position FROM separators ORDER BY position ASC'
  )
  const separators: Separator[] = sepRows.map((r) => ({ id: r.id, tab: r.tab, position: r.position }))

  return { settings, tabList, files, separators }
}

// ── Save (full re-sync inside a transaction) ──────────────────────────────────
/** Inserts many rows in a few batched multi-VALUES statements to cut the number
 *  of IPC round-trips (one execute per row is very slow over the Tauri bridge).
 *  Chunked so the bound-parameter count stays well under SQLite's limit. */
async function batchInsert(
  d: Database,
  table: string,
  cols: string[],
  rows: unknown[][],
  chunkRows = 100
): Promise<void> {
  if (rows.length === 0) return
  const colSql = cols.join(', ')
  for (let i = 0; i < rows.length; i += chunkRows) {
    const chunk = rows.slice(i, i + chunkRows)
    const placeholders: string[] = []
    const params: unknown[] = []
    let p = 1
    for (const row of chunk) {
      placeholders.push('(' + cols.map(() => `$${p++}`).join(', ') + ')')
      params.push(...row)
    }
    await d.execute(`INSERT INTO ${table} (${colSql}) VALUES ${placeholders.join(', ')}`, params)
  }
}

export async function saveConfig(d: Database, config: ProjectConfig): Promise<void> {
  await d.execute('BEGIN')
  try {
    await d.execute('DELETE FROM settings')
    await d.execute('DELETE FROM tabs')
    await d.execute('DELETE FROM sounds')
    await d.execute('DELETE FROM sound_tabs')
    await d.execute('DELETE FROM separators')

    const s = config.settings
    const settingsRows: [string, string][] = [
      ['theme', s.theme ?? 'dark-cyan'],
      ['customCss', s.customCss ?? ''],
      ['outputSource', s.outputSource ?? 'default'],
      ['stopOnRetrigger', String(s.stopOnRetrigger ?? true)],
      ['overlapSounds', String(s.overlapSounds ?? false)],
      ['cacheMaxSizeMib', String(s.cacheMaxSizeMib ?? 256)],
      ['cacheMaxEntryMib', String(s.cacheMaxEntryMib ?? 50)],
      ['outputVolume', String(s.outputVolume ?? 1)],
      ['uniformButtonHeight', String(s.uniformButtonHeight ?? false)],
      ['allowReorder', String(s.allowReorder ?? true)],
      ['themeMode', s.themeMode ?? 'dark'],
      ['primaryColor', s.primaryColor ?? '#00d4ff'],
      ['bgLight', s.bgLight ?? '#eeeeee'],
      ['bgDark', s.bgDark ?? '#222831'],
      ['btnLight', s.btnLight ?? '#7184a2'],
      ['btnDark', s.btnDark ?? '#363f4d'],
      ['textLight', s.textLight ?? '#eeeeee'],
      ['textDark', s.textDark ?? '#222831'],
      ['outputHost', s.outputHost ?? 'WASAPI'],
      ...(s.asioLeftChannel != null ? [['asioLeftChannel', String(s.asioLeftChannel)] as [string, string]] : []),
      ...(s.asioRightChannel != null ? [['asioRightChannel', String(s.asioRightChannel)] as [string, string]] : []),
    ]
    await batchInsert(d, 'settings', ['key', 'value'], settingsRows)

    const tabRows = config.tabList.map((t, i) => [t.name, t.color ?? null, i])
    await batchInsert(d, 'tabs', ['name', 'color', 'position'], tabRows)

    const soundRows: unknown[][] = []
    const soundTabRows: unknown[][] = []
    for (const f of config.files) {
      soundRows.push([f.path, f.name, f.volume ?? 0.4, f.color ?? null, f.index ?? 0, f.active ? 1 : 0])
      for (const tab of f.tabs) {
        const tabIdx = tab === 'All' ? f.index ?? 0 : f.tabIndexes?.[tab] ?? 0
        soundTabRows.push([f.path, tab, tabIdx])
      }
    }
    await batchInsert(
      d,
      'sounds',
      ['path', 'name', 'volume', 'color', 'global_index', 'active'],
      soundRows
    )
    await batchInsert(d, 'sound_tabs', ['sound_path', 'tab', 'tab_index'], soundTabRows)

    const sepRows = (config.separators ?? []).map((sep) => [sep.id, sep.tab, sep.position])
    await batchInsert(d, 'separators', ['id', 'tab', 'position'], sepRows)

    await d.execute('COMMIT')
  } catch (e) {
    try { await d.execute('ROLLBACK') } catch { /* ignore */ }
    throw e
  }
}
