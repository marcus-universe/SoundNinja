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
export async function saveConfig(d: Database, config: ProjectConfig): Promise<void> {
  await d.execute('BEGIN')
  try {
    await d.execute('DELETE FROM settings')
    await d.execute('DELETE FROM tabs')
    await d.execute('DELETE FROM sounds')
    await d.execute('DELETE FROM sound_tabs')
    await d.execute('DELETE FROM separators')

    const s = config.settings
    const settingsEntries: [string, string][] = [
      ['theme', s.theme ?? 'dark-cyan'],
      ['customCss', s.customCss ?? ''],
      ['outputSource', s.outputSource ?? 'default'],
      ['stopOnRetrigger', String(s.stopOnRetrigger ?? true)],
      ['overlapSounds', String(s.overlapSounds ?? false)],
      ['cacheMaxSizeMib', String(s.cacheMaxSizeMib ?? 256)],
      ['cacheMaxEntryMib', String(s.cacheMaxEntryMib ?? 50)],
      ['outputVolume', String(s.outputVolume ?? 1)],
    ]
    for (const [key, value] of settingsEntries) {
      await d.execute('INSERT INTO settings (key, value) VALUES ($1, $2)', [key, value])
    }

    for (let i = 0; i < config.tabList.length; i++) {
      const t = config.tabList[i]
      await d.execute('INSERT INTO tabs (name, color, position) VALUES ($1, $2, $3)', [
        t.name, t.color ?? null, i,
      ])
    }

    for (const f of config.files) {
      await d.execute(
        'INSERT INTO sounds (path, name, volume, color, global_index, active) VALUES ($1, $2, $3, $4, $5, $6)',
        [f.path, f.name, f.volume ?? 0.4, f.color ?? null, f.index ?? 0, f.active ? 1 : 0]
      )
      for (const tab of f.tabs) {
        const tabIdx = tab === 'All' ? f.index ?? 0 : f.tabIndexes?.[tab] ?? 0
        await d.execute(
          'INSERT INTO sound_tabs (sound_path, tab, tab_index) VALUES ($1, $2, $3)',
          [f.path, tab, tabIdx]
        )
      }
    }

    for (const sep of config.separators ?? []) {
      await d.execute('INSERT INTO separators (id, tab, position) VALUES ($1, $2, $3)', [
        sep.id, sep.tab, sep.position,
      ])
    }

    await d.execute('COMMIT')
  } catch (e) {
    try { await d.execute('ROLLBACK') } catch { /* ignore */ }
    throw e
  }
}
