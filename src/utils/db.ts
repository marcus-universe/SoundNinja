import Database from '@tauri-apps/plugin-sql'
import { normalizeThemeId } from '~/utils/themePresets'
import { resolveThemeTokens } from '~/utils/themeTokens'

// ── Types (mirrors the shape the components already consume) ───────────────────
export const MAX_GIF_BYTES = 8 * 1024 * 1024

export interface SoundFile {
  name: string
  path: string
  /** Stable 8-char `[a-z0-9]` id. Assigned on create / load migration. */
  id: string
  volume: number
  tabs: string[]
  active: boolean
  index: number
  tabIndexes: Record<string, number>
  color?: string
  /** Content-addressed id into `gif_blobs` (SHA-256 hex). Bytes stay out of Pinia. */
  gifId?: string
  gifPosX?: number
  gifPosY?: number
}

export interface GifBlobRow {
  id: string
  mime: string
  /** Base64 of the animated GIF/WebP bytes. */
  data: string
  /** Base64 of a first-frame PNG poster, if extracted. */
  poster: string | null
  byteLen: number
}

export type ButtonAlign = 'left' | 'center' | 'right'

export type TabTransition = 'slide' | 'fade' | 'stagger' | 'none'

const TAB_TRANSITIONS: TabTransition[] = ['slide', 'fade', 'stagger', 'none']

export function normalizeTabTransition(value: unknown): TabTransition {
  return TAB_TRANSITIONS.includes(value as TabTransition) ? (value as TabTransition) : 'slide'
}

export interface TabEntry {
  name: string
  color?: string
  /** Default button alignment for this tab. Groups may override. */
  buttonAlign?: ButtonAlign
}

export interface Separator {
  id: string
  tab: string
  position: number
  /** Display name. Empty/missing → untitled Group (legacy seps). */
  name?: string
  borderColor?: string
  nameColor?: string
  bgColor?: string
  /** Undefined = inherit Tab.buttonAlign */
  buttonAlign?: ButtonAlign
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
  /** Accent / primary color. */
  primaryColor?: string
  primaryHover?: string
  /** Page background. */
  bg?: string
  /** Secondary surfaces (settings sidebar, tool windows). */
  bg2?: string
  /** Sound button colors. */
  btnBg?: string
  btnBgHover?: string
  btnText?: string
  btnTextHover?: string
  btnBorder?: string
  btnBorderHover?: string
  /** Tab colors. */
  tabBg?: string
  tabBgHover?: string
  tabText?: string
  tabTextHover?: string
  tabBorder?: string
  tabBorderHover?: string
  /** @deprecated Legacy light/dark pairs — read-migrated, never written. */
  themeMode?: 'dark' | 'light'
  bgLight?: string
  bgDark?: string
  btnLight?: string
  btnDark?: string
  textLight?: string
  textDark?: string
  /** Audio driver/host name (e.g. 'WASAPI', 'ASIO'). */
  outputHost?: string
  /** ASIO left-channel index (0-based). Only used when outputHost === 'ASIO'. */
  asioLeftChannel?: number
  /** ASIO right-channel index (0-based). Only used when outputHost === 'ASIO'. */
  asioRightChannel?: number
  /** Enable GPU-accelerated DSP (experimental; only shown when discrete GPU detected). */
  gpuAudioEnabled?: boolean
  /** Show the floating player on the soundboard. */
  showPlayer?: boolean
  /** Enlarge floating player controls / waveform. */
  playerLarge?: boolean
  /** When true, GIF button backgrounds animate only while hovered. Default on. */
  gifPlayOnHover?: boolean
  /** Board animation when switching tabs. Unknown/missing → slide. */
  tabTransition?: TabTransition
  /** User-added sound trigger bindings (travel with the project). */
  soundHotkeys?: { id: string; soundId: string; combo: string }[]
}

export interface ProjectConfig {
  settings: Settings
  tabList: TabEntry[]
  files: SoundFile[]
  separators: Separator[]
}

export function defaultSettings(): Settings {
  return {
    theme: 'soundninja',
    customCss: '',
    outputSource: 'default',
    stopOnRetrigger: true,
    overlapSounds: false,
    uniformButtonHeight: false,
    allowReorder: true,
    primaryColor: '#00d4ff',
    primaryHover: '#33ddff',
    bg: '#222831',
    bg2: '#1a1e25',
    btnBg: '#363f4d',
    btnBgHover: '#434e5f',
    btnText: '#eeeeee',
    btnTextHover: '#00d4ff',
    btnBorder: '#00d4ff',
    btnBorderHover: '#33ddff',
    tabBg: '#00d4ff33',
    tabBgHover: '#00d4ff66',
    tabText: '#eeeeee',
    tabTextHover: '#eeeeee',
    tabBorder: '#00d4ff',
    tabBorderHover: '#33ddff',
    outputHost: 'WASAPI',
    asioLeftChannel: undefined,
    asioRightChannel: undefined,
    showPlayer: true,
    playerLarge: false,
    gifPlayOnHover: true,
    tabTransition: 'slide',
    soundHotkeys: [],
  }
}

export function emptyConfig(): ProjectConfig {
  return { settings: defaultSettings(), tabList: [], files: [], separators: [] }
}

/** Parent folder of a sound path (`C:\board\Lolli\foo.wav` → `Lolli`). */
export function parentFolderName(filePath: string): string {
  const norm = String(filePath || '').replace(/\\/g, '/').replace(/\/+$/, '')
  const parts = norm.split('/').filter(Boolean)
  if (parts.length < 2) return ''
  return parts[parts.length - 2]
}

/**
 * Sounds imported as folders-on-All often have tabs named after those folders
 * but only an `All` row in `sound_tabs`. Assign the matching tab so the board
 * is not empty. Skips sounds that already belong to a non-All tab.
 */
/** Tabs table can be empty while sound_tabs / separators still name tabs. */
export function mergeTabsFromUsage(config: ProjectConfig): number {
  const list = config.tabList ?? (config.tabList = [])
  const have = new Set(list.map((t) => t.name))
  const extra: string[] = []
  const add = (name: string | undefined) => {
    if (!name || name === 'All' || have.has(name)) return
    have.add(name)
    extra.push(name)
  }
  for (const f of config.files ?? []) {
    for (const t of f.tabs ?? []) add(t)
  }
  for (const s of config.separators ?? []) add(s.tab)
  for (const name of extra) list.push({ name })
  dedupeTabList(config)
  return extra.length
}

/** Drop duplicate tab names so SQLite `tabs.name` PRIMARY KEY cannot fail. */
export function dedupeTabList(config: ProjectConfig): number {
  const list = config.tabList ?? []
  const seen = new Set<string>()
  const next = []
  for (const t of list) {
    const n = t?.name
    if (!n || seen.has(n)) continue
    seen.add(n)
    next.push(t)
  }
  const removed = list.length - next.length
  if (removed) config.tabList = next
  return removed
}

export function healFolderTabMembership(config: ProjectConfig): number {
  const names = (config.tabList ?? []).map((t) => t.name).filter(Boolean)
  if (!names.length) return 0
  const exact = new Map(names.map((n) => [n, n]))
  const lower = new Map(names.map((n) => [n.toLowerCase(), n]))
  let healed = 0
  for (const f of config.files ?? []) {
    if (!Array.isArray(f.tabs)) f.tabs = ['All']
    if (f.tabs.some((t) => t && t !== 'All')) continue
    const folder = parentFolderName(f.path)
    if (!folder) continue
    const tab = exact.get(folder) ?? lower.get(folder.toLowerCase())
    if (!tab) continue
    f.tabs = ['All', tab]
    if (!f.tabIndexes) f.tabIndexes = {}
    if (f.tabIndexes[tab] == null) f.tabIndexes[tab] = f.index ?? 0
    healed++
  }
  return healed
}

// ── Connection handling ───────────────────────────────────────────────────────
let db: Database | null = null
let currentUrl: string | null = null
let currentPath: string | null = null
/** Serialises open/persist so a mid-save pool close cannot race. */
let dbChain: Promise<unknown> = Promise.resolve()

function toUrl(dbAbsPath: string): string {
  return 'sqlite:' + dbAbsPath.replace(/\\/g, '/')
}

function withDbLock<T>(fn: () => Promise<T>): Promise<T> {
  const next = dbChain.then(fn, fn)
  // Keep the chain alive even if `fn` rejects.
  dbChain = next.then(() => undefined, () => undefined)
  return next
}

/**
 * Opens (and caches) a project database, initialising its schema.
 *
 * Important: do NOT call `Database.close()` without a path — that shuts down
 * *every* SQL pool (including app-config.db). We also avoid closing when
 * switching projects: tauri-plugin-sql can leave a closed pool in its map and
 * hand it back on the next load ("attempted to acquire a connection on a closed pool").
 */
export async function openDb(dbAbsPath: string): Promise<Database> {
  return withDbLock(async () => openDbUnlocked(dbAbsPath))
}

function isRecoverableDbError(e: unknown): boolean {
  return /closed pool|database is locked|\(code:\s*5\)|SQLITE_BUSY/i.test(String(e))
}

/** WAL + busy_timeout so readers do not instantly fail while a write is in flight.
 *  busy_timeout is per-connection; we set it on every open. */
async function configureConnection(d: Database): Promise<void> {
  await d.execute('PRAGMA journal_mode=WAL')
  await d.execute('PRAGMA busy_timeout=5000')
  await d.execute('PRAGMA synchronous=NORMAL')
}

async function openDbUnlocked(dbAbsPath: string): Promise<Database> {
  const url = toUrl(dbAbsPath)
  if (db && currentUrl === url) return db

  db = await Database.load(url)
  currentUrl = url
  currentPath = dbAbsPath
  await configureConnection(db)
  await initSchema(db)
  return db
}

async function reopenDbUnlocked(dbAbsPath: string): Promise<Database> {
  const url = toUrl(dbAbsPath)
  // Close only this pool (pass the connection string). Never call close() bare —
  // bare close() shuts down *all* pools in the plugin.
  if (db && currentUrl) {
    try { await db.close(currentUrl) } catch { /* ignore */ }
  }
  db = null
  currentUrl = null
  currentPath = null

  db = await Database.load(url)
  currentUrl = url
  currentPath = dbAbsPath
  await configureConnection(db)
  await initSchema(db)
  return db
}

/** Drops the local cache and reloads the pool for `dbAbsPath`. */
export async function reopenDb(dbAbsPath: string): Promise<Database> {
  return withDbLock(async () => reopenDbUnlocked(dbAbsPath))
}

export function getDb(): Database | null {
  return db
}

export function getDbPath(): string | null {
  return currentPath
}

/**
 * Opens the project DB (reloading once on closed-pool / SQLITE_BUSY) and runs
 * `fn` under the connection lock so saves cannot race project switches.
 */
export async function withProjectDb<T>(
  dbAbsPath: string,
  fn: (d: Database) => Promise<T>,
): Promise<T> {
  return withDbLock(async () => {
    let d = await openDbUnlocked(dbAbsPath)
    try {
      return await fn(d)
    } catch (e) {
      if (!isRecoverableDbError(e)) throw e
      d = await reopenDbUnlocked(dbAbsPath)
      return await fn(d)
    }
  })
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
    active INTEGER,
    sound_id TEXT
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
  // Bytes live here, not in Pinia. saveConfig must never DELETE this table.
  await d.execute(`CREATE TABLE IF NOT EXISTS gif_blobs (
    id TEXT PRIMARY KEY,
    mime TEXT NOT NULL,
    data TEXT NOT NULL,
    poster TEXT,
    byte_len INTEGER
  )`)
  await addColumnIfMissing(d, 'sounds', 'gif_id', 'TEXT')
  await addColumnIfMissing(d, 'sounds', 'gif_pos_x', 'REAL')
  await addColumnIfMissing(d, 'sounds', 'gif_pos_y', 'REAL')
  await addColumnIfMissing(d, 'separators', 'name', 'TEXT')
  await addColumnIfMissing(d, 'separators', 'border_color', 'TEXT')
  await addColumnIfMissing(d, 'separators', 'name_color', 'TEXT')
  await addColumnIfMissing(d, 'separators', 'bg_color', 'TEXT')
  await addColumnIfMissing(d, 'separators', 'button_align', 'TEXT')
  await addColumnIfMissing(d, 'tabs', 'button_align', 'TEXT')
  await addColumnIfMissing(d, 'sounds', 'sound_id', 'TEXT')
}

async function addColumnIfMissing(
  d: Database,
  table: string,
  column: string,
  sqlType: string,
): Promise<void> {
  try {
    await d.execute(`ALTER TABLE ${table} ADD COLUMN ${column} ${sqlType}`)
  } catch {
    /* column already exists on upgraded project files */
  }
}

async function tableColumns(d: Database, table: string): Promise<Set<string>> {
  try {
    const rows = await d.select<{ name: string }[]>(`PRAGMA table_info(${table})`)
    return new Set((rows ?? []).map((r) => r.name))
  } catch {
    return new Set()
  }
}

/** Load sounds without requiring a specific id column (avoids empty board on migrate). */
async function loadSoundRows(d: Database): Promise<{
  path: string
  sound_id?: string | null
  name: string
  volume: number
  color: string | null
  global_index: number
  active: number
  gif_id?: string | null
  gif_pos_x?: number | null
  gif_pos_y?: number | null
}[]> {
  const cols = await tableColumns(d, 'sounds')
  const idSql = cols.has('sound_id')
    ? 'sound_id'
    : cols.has('id')
      ? 'id'
      : 'NULL'
  const gifId = cols.has('gif_id') ? 'gif_id' : 'NULL'
  const gifX = cols.has('gif_pos_x') ? 'gif_pos_x' : 'NULL'
  const gifY = cols.has('gif_pos_y') ? 'gif_pos_y' : 'NULL'
  try {
    return await d.select(
      `SELECT path, ${idSql} AS sound_id, name, volume, color, global_index, active, ${gifId} AS gif_id, ${gifX} AS gif_pos_x, ${gifY} AS gif_pos_y FROM sounds ORDER BY global_index ASC`,
    )
  } catch {
    return await d.select(
      'SELECT path, name, volume, color, global_index, active FROM sounds ORDER BY global_index ASC',
    )
  }
}

/** Write missing sound ids with UPDATE — never a full DELETE+INSERT resync. */
export async function persistSoundIds(d: Database, files: SoundFile[]): Promise<void> {
  const cols = await tableColumns(d, 'sounds')
  if (!cols.has('sound_id')) {
    await addColumnIfMissing(d, 'sounds', 'sound_id', 'TEXT')
  }
  for (const f of files) {
    if (!f.id || !f.path) continue
    await d.execute('UPDATE sounds SET sound_id = $1 WHERE path = $2', [f.id, f.path])
  }
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
      case 'showPlayer': settings.showPlayer = value === 'true'; break
      case 'playerLarge': settings.playerLarge = value === 'true'; break
      case 'gifPlayOnHover': settings.gifPlayOnHover = value !== 'false'; break
      case 'tabTransition': settings.tabTransition = normalizeTabTransition(value); break
      case 'soundHotkeys':
        try { settings.soundHotkeys = JSON.parse(value) } catch { settings.soundHotkeys = [] }
        break
      case 'cacheMaxSizeMib': settings.cacheMaxSizeMib = Number(value); break
      case 'cacheMaxEntryMib': settings.cacheMaxEntryMib = Number(value); break
      case 'outputVolume': settings.outputVolume = Number(value); break
      case 'uniformButtonHeight': settings.uniformButtonHeight = value === 'true'; break
      case 'allowReorder': settings.allowReorder = value === 'true'; break
      // Flat theme tokens
      case 'primaryColor': settings.primaryColor = value; break
      case 'primaryHover': settings.primaryHover = value; break
      case 'bg': settings.bg = value; break
      case 'bg2': settings.bg2 = value; break
      case 'btnBg': settings.btnBg = value; break
      case 'btnBgHover': settings.btnBgHover = value; break
      case 'btnText': settings.btnText = value; break
      case 'btnTextHover': settings.btnTextHover = value; break
      case 'btnBorder': settings.btnBorder = value; break
      case 'btnBorderHover': settings.btnBorderHover = value; break
      case 'tabBg': settings.tabBg = value; break
      case 'tabBgHover': settings.tabBgHover = value; break
      case 'tabText': settings.tabText = value; break
      case 'tabTextHover': settings.tabTextHover = value; break
      case 'tabBorder': settings.tabBorder = value; break
      case 'tabBorderHover': settings.tabBorderHover = value; break
      // Legacy pairs (kept in memory for resolveThemeTokens migration)
      case 'themeMode': settings.themeMode = value === 'light' ? 'light' : 'dark'; break
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

  // Migrate legacy dark-* theme ids and pair fields → flat tokens.
  settings.theme = normalizeThemeId(settings.theme)
  if (!settings.bg && !settings.btnBg) {
    const flat = resolveThemeTokens(settings as unknown as Record<string, unknown>)
    Object.assign(settings, flat)
  } else if (!settings.bg2) {
    settings.bg2 = resolveThemeTokens(settings as unknown as Record<string, unknown>).bg2
  }

  const tabRows = await d.select<
    { name: string; color: string | null; position: number; button_align: string | null }[]
  >('SELECT name, color, position, button_align FROM tabs ORDER BY position ASC')
  const tabList: TabEntry[] = tabRows.map((t) => ({
    name: t.name,
    ...(t.color ? { color: t.color } : {}),
    ...(t.button_align === 'left' || t.button_align === 'center' || t.button_align === 'right'
      ? { buttonAlign: t.button_align }
      : {}),
  }))

  const soundRows = await loadSoundRows(d)

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
      id: s.sound_id || '',
      name: s.name,
      volume: s.volume,
      index: s.global_index,
      active: s.active === 1,
      tabs,
      tabIndexes,
      ...(s.color ? { color: s.color } : {}),
      ...(s.gif_id ? { gifId: s.gif_id } : {}),
      ...(s.gif_pos_x != null ? { gifPosX: Number(s.gif_pos_x) } : {}),
      ...(s.gif_pos_y != null ? { gifPosY: Number(s.gif_pos_y) } : {}),
    }
  })

  const sepRows = await d.select<
    {
      id: string
      tab: string
      position: number
      name: string | null
      border_color: string | null
      name_color: string | null
      bg_color: string | null
      button_align: string | null
    }[]
  >(
    'SELECT id, tab, position, name, border_color, name_color, bg_color, button_align FROM separators ORDER BY position ASC',
  )
  const separators: Separator[] = sepRows.map((r) => ({
    id: r.id,
    tab: r.tab,
    position: r.position,
    ...(r.name ? { name: r.name } : {}),
    ...(r.border_color ? { borderColor: r.border_color } : {}),
    ...(r.name_color ? { nameColor: r.name_color } : {}),
    ...(r.bg_color ? { bgColor: r.bg_color } : {}),
    ...(r.button_align === 'left' || r.button_align === 'center' || r.button_align === 'right'
      ? { buttonAlign: r.button_align }
      : {}),
  }))

  const config = { settings, tabList, files, separators }
  mergeTabsFromUsage(config)
  healFolderTabMembership(config)
  return config
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
  chunkRows = 50
): Promise<void> {
  if (rows.length === 0) return
  // SQLite default SQLITE_MAX_VARIABLE_NUMBER is 999. Keep a margin.
  const maxRows = Math.max(1, Math.min(chunkRows, Math.floor(900 / Math.max(cols.length, 1))))
  const colSql = cols.join(', ')
  for (let i = 0; i < rows.length; i += maxRows) {
    const chunk = rows.slice(i, i + maxRows)
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
  // Do NOT wrap this in BEGIN/COMMIT. tauri-plugin-sql uses an sqlx pool;
  // each execute() may land on a different connection. BEGIN on conn A +
  // DELETE on conn B = SQLITE_BUSY ("database is locked"). An abandoned
  // BEGIN also poisons that pooled connection until the app restarts.
  const incoming = config.files?.length ?? 0
  if (incoming === 0) {
    try {
      const rows = await d.select<{ c: number }[]>('SELECT COUNT(*) AS c FROM sounds')
      const existing = Number(rows?.[0]?.c ?? 0)
      if (existing > 0) {
        throw new Error(`Refusing to persist 0 sounds over ${existing} existing rows`)
      }
    } catch (e) {
      if (String(e).includes('Refusing to persist')) throw e
    }
  }
  await d.execute('DELETE FROM settings')
  await d.execute('DELETE FROM tabs')
  await d.execute('DELETE FROM sounds')
  await d.execute('DELETE FROM sound_tabs')
  await d.execute('DELETE FROM separators')

  const s = config.settings
  // Note: outputSource / outputHost / outputVolume / ASIO channels are app-wide
  // (app-config.db) and must not be written into the project file.
  const settingsRows: [string, string][] = [
    ['theme', s.theme ?? 'soundninja'],
    ['customCss', s.customCss ?? ''],
    ['stopOnRetrigger', String(s.stopOnRetrigger ?? true)],
    ['overlapSounds', String(s.overlapSounds ?? false)],
    ['showPlayer', String(s.showPlayer ?? true)],
    ['playerLarge', String(s.playerLarge ?? false)],
    ['cacheMaxSizeMib', String(s.cacheMaxSizeMib ?? 64)],
    ['cacheMaxEntryMib', String(s.cacheMaxEntryMib ?? 16)],
    ['uniformButtonHeight', String(s.uniformButtonHeight ?? false)],
    ['allowReorder', String(s.allowReorder ?? true)],
    ['gifPlayOnHover', String(s.gifPlayOnHover !== false)],
    ['tabTransition', normalizeTabTransition(s.tabTransition)],
    ['soundHotkeys', JSON.stringify(s.soundHotkeys ?? [])],
    ['primaryColor', s.primaryColor ?? '#00d4ff'],
    ['primaryHover', s.primaryHover ?? '#33ddff'],
    ['bg', s.bg ?? '#222831'],
    ['bg2', s.bg2 ?? '#1a1e25'],
    ['btnBg', s.btnBg ?? '#363f4d'],
    ['btnBgHover', s.btnBgHover ?? '#434e5f'],
    ['btnText', s.btnText ?? '#eeeeee'],
    ['btnTextHover', s.btnTextHover ?? '#00d4ff'],
    ['btnBorder', s.btnBorder ?? '#00d4ff'],
    ['btnBorderHover', s.btnBorderHover ?? '#33ddff'],
    ['tabBg', s.tabBg ?? '#00d4ff33'],
    ['tabBgHover', s.tabBgHover ?? '#00d4ff66'],
    ['tabText', s.tabText ?? '#eeeeee'],
    ['tabTextHover', s.tabTextHover ?? '#eeeeee'],
    ['tabBorder', s.tabBorder ?? '#00d4ff'],
    ['tabBorderHover', s.tabBorderHover ?? '#33ddff'],
  ]
  await batchInsert(d, 'settings', ['key', 'value'], settingsRows)

  dedupeTabList(config)
  const tabRows = config.tabList.map((t, i) => [
    t.name,
    t.color ?? null,
    i,
    t.buttonAlign ?? null,
  ])
  await batchInsert(d, 'tabs', ['name', 'color', 'position', 'button_align'], tabRows)

  const soundRows: unknown[][] = []
  const soundTabRows: unknown[][] = []
  for (const f of config.files) {
    soundRows.push([
      f.path,
      f.id || '',
      f.name,
      f.volume ?? 0.4,
      f.color ?? null,
      f.index ?? 0,
      f.active ? 1 : 0,
      f.gifId ?? null,
      f.gifPosX ?? 50,
      f.gifPosY ?? 50,
    ])
    for (const tab of f.tabs ?? ['All']) {
      const tabIdx = tab === 'All' ? f.index ?? 0 : f.tabIndexes?.[tab] ?? 0
      soundTabRows.push([f.path, tab, tabIdx])
    }
  }
  await batchInsert(
    d,
    'sounds',
    ['path', 'sound_id', 'name', 'volume', 'color', 'global_index', 'active', 'gif_id', 'gif_pos_x', 'gif_pos_y'],
    soundRows
  )
  await batchInsert(d, 'sound_tabs', ['sound_path', 'tab', 'tab_index'], soundTabRows)

  const sepRows = (config.separators ?? []).map((sep) => [
    sep.id,
    sep.tab,
    sep.position,
    sep.name ?? null,
    sep.borderColor ?? null,
    sep.nameColor ?? null,
    sep.bgColor ?? null,
    sep.buttonAlign ?? null,
  ])
  await batchInsert(
    d,
    'separators',
    ['id', 'tab', 'position', 'name', 'border_color', 'name_color', 'bg_color', 'button_align'],
    sepRows,
  )
}

// ── GIF blobs (separate from the full-resync save; never wiped by saveConfig) ─
export async function upsertGifBlob(d: Database, row: GifBlobRow): Promise<void> {
  await d.execute(
    `INSERT INTO gif_blobs (id, mime, data, poster, byte_len) VALUES ($1, $2, $3, $4, $5)
     ON CONFLICT(id) DO NOTHING`,
    [row.id, row.mime, row.data, row.poster, row.byteLen],
  )
}

export async function loadGifBlob(d: Database, id: string): Promise<GifBlobRow | null> {
  const rows = await d.select<
    { id: string; mime: string; data: string; poster: string | null; byte_len: number }[]
  >('SELECT id, mime, data, poster, byte_len FROM gif_blobs WHERE id = $1', [id])
  const r = rows[0]
  if (!r) return null
  return { id: r.id, mime: r.mime, data: r.data, poster: r.poster ?? null, byteLen: r.byte_len ?? 0 }
}

export async function loadGifBlobsByIds(d: Database, ids: string[]): Promise<GifBlobRow[]> {
  const unique = [...new Set(ids.filter(Boolean))]
  if (!unique.length) return []
  const out: GifBlobRow[] = []
  for (const id of unique) {
    const row = await loadGifBlob(d, id)
    if (row) out.push(row)
  }
  return out
}

export async function gcOrphanGifs(d: Database, keepIds: string[] = []): Promise<void> {
  const keep = [...new Set(keepIds.filter(Boolean))]
  if (!keep.length) {
    await d.execute('DELETE FROM gif_blobs')
    return
  }
  const placeholders = keep.map((_, i) => `$${i + 1}`).join(', ')
  await d.execute(`DELETE FROM gif_blobs WHERE id NOT IN (${placeholders})`, keep)
}
