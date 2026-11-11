import { invoke } from '@tauri-apps/api/core'
import { normalizeThemeId } from '~/utils/themePresets'
import { resolveThemeTokens } from '~/utils/themeTokens'
import { boardTabNames, groupMembership, layoutFromMembership, applyBoardLayout } from '~/utils/boardLayout'

/**
 * Version of the project file layout this build writes.
 *
 * 1 — implicit: files from builds without format versioning.
 * 2 — atomic saves, single-file journal, explicit group membership
 *     (`sound_tabs.group_id`), `meta` / `history` / `snapshots` / `media`.
 *
 * Stored in the `settings` table on purpose: an older build rewrites that
 * table on save and thereby drops the key, so a file it touched is recognised
 * as "written by an old build" again and its positional layout is trusted
 * over stale `group_id` values.
 */
export const PROJECT_FORMAT_VERSION = 2

// ── Types (mirrors the shape the components already consume) ───────────────────
export const MAX_GIF_BYTES = 8 * 1024 * 1024

export interface SoundTag {
  id: string
  name: string
  color: string
}

export type SortMode = 'user' | 'name' | 'added' | 'duration' | 'size'

const SORT_MODES: SortMode[] = ['user', 'name', 'added', 'duration', 'size']

export function normalizeSortMode(value: unknown): SortMode {
  return SORT_MODES.includes(value as SortMode) ? (value as SortMode) : 'user'
}

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
  /** Project tag ids assigned to this sound. */
  tagIds?: string[]
  /** Unix ms when the sound was added to the project. */
  addedAt?: number
  /** Audio duration in seconds (cached after first probe). */
  durationSecs?: number
  /** File size in bytes. */
  fileSize?: number
  /** Play this sound looping when triggered. Default false. */
  looping?: boolean
  /** Path relative to the project file (`/` separators) when the sound lives
   *  inside the project folder. Used to find it again on another machine. */
  relPath?: string
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
  /** Show the floating player on the soundboard. */
  showPlayer?: boolean
  /** Enlarge floating player controls / waveform. */
  playerLarge?: boolean
  /** Show waveform in the floating player. Off = seekable progress bar. Default on. */
  showWaveform?: boolean
  /** When true, GIF button backgrounds animate only while hovered. Default on. */
  gifPlayOnHover?: boolean
  /** Keep GIF/image blobs in memory across tabs so switching does not reload them. */
  preloadGifs?: boolean
  /** Board animation when switching tabs. Unknown/missing → slide. */
  tabTransition?: TabTransition
  /** User-added sound trigger bindings (travel with the project). */
  soundHotkeys?: { id: string; soundId: string; combo: string }[]
  /**
   * One-shot: unused legacy `volume: 0.4` (never applied in playback) was
   * rewritten to 1.0. After this flag, user-set 40% stays 40%.
   */
  perSoundVolumeV1?: boolean
  /** Board sort. `user` = drag order (default). */
  sortMode?: SortMode
  /** Active tag filter ids (OR). Empty = no tag filter. */
  selectedTagIds?: string[]
}

export interface ProjectConfig {
  settings: Settings
  tabList: TabEntry[]
  files: SoundFile[]
  separators: Separator[]
  tags: SoundTag[]
  /** Settings rows this build does not know (written by a newer version).
   *  Carried through load → save untouched so nothing is lost. */
  extraSettings?: Record<string, string>
  /** Undo/redo snapshot id. Only set on history entries, never on the live config. */
  _hid?: number
}

/** Per-project facts that are not part of the board itself. */
export interface ProjectMeta {
  /** Stable id, kept across Save As. Names the folder embedded audio is unpacked to. */
  projectId: string
  /** Sound files are stored inside the project file. */
  embedAudio: boolean
  /** GIF / image button backgrounds are stored inside the project file. */
  embedGifs: boolean
  /** The user confirmed the two options above at least once. */
  saveOptionsSet: boolean
}

export function defaultMeta(): ProjectMeta {
  return { projectId: '', embedAudio: false, embedGifs: true, saveOptionsSet: false }
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
    showWaveform: true,
    gifPlayOnHover: true,
    preloadGifs: false,
    cacheMaxSizeMib: 256,
    cacheMaxEntryMib: 128,
    tabTransition: 'slide',
    soundHotkeys: [],
    sortMode: 'user',
    selectedTagIds: [],
  }
}

export function emptyConfig(): ProjectConfig {
  return { settings: defaultSettings(), tabList: [], files: [], separators: [], tags: [] }
}

/** Parent folder of a sound path (`C:\board\Lolli\foo.wav` → `Lolli`). */
export function parentFolderName(filePath: string): string {
  const norm = String(filePath || '').replace(/\\/g, '/').replace(/\/+$/, '')
  const parts = norm.split('/').filter(Boolean)
  if (parts.length < 2) return ''
  return parts[parts.length - 2] ?? ''
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

/** Copy tabs, tags, and missing media from `extra` onto `keep`. */
export function mergeSoundInto(keep: SoundFile, extra: SoundFile): void {
  const tabs = [...(keep.tabs ?? ['All'])]
  for (const tab of extra.tabs ?? []) {
    if (tab && !tabs.includes(tab)) tabs.push(tab)
  }
  keep.tabs = tabs
  if (!keep.tabIndexes) keep.tabIndexes = {}
  for (const [tab, idx] of Object.entries(extra.tabIndexes ?? {})) {
    if (keep.tabIndexes[tab] == null && Number.isFinite(idx)) keep.tabIndexes[tab] = idx
  }
  const tagIds = [...(keep.tagIds ?? [])]
  for (const id of extra.tagIds ?? []) {
    if (id && !tagIds.includes(id)) tagIds.push(id)
  }
  keep.tagIds = tagIds
  if (!keep.gifId && extra.gifId) {
    keep.gifId = extra.gifId
    if (extra.gifPosX != null) keep.gifPosX = extra.gifPosX
    if (extra.gifPosY != null) keep.gifPosY = extra.gifPosY
  }
  if (!keep.color && extra.color) keep.color = extra.color
  if (!keep.name && extra.name) keep.name = extra.name
  if (keep.durationSecs == null && extra.durationSecs != null) keep.durationSecs = extra.durationSecs
  if (keep.fileSize == null && extra.fileSize != null) keep.fileSize = extra.fileSize
}

/**
 * Collapse sounds that share the exact `path` string.
 * SQLite `sounds.path` is the primary key (BINARY). A second row with the
 * same path fails the save with SQLITE_CONSTRAINT_PRIMARYKEY (1555).
 * Case-different paths stay separate — Linux filesystems treat them as
 * different files.
 */
export function dedupeSoundsByPath(config: ProjectConfig): number {
  const files = config.files ?? []
  const seen = new Map<string, SoundFile>()
  const next: SoundFile[] = []
  let removed = 0
  for (const file of files) {
    const key = file?.path ?? ''
    const prev = seen.get(key)
    if (!prev) {
      seen.set(key, file)
      next.push(file)
      continue
    }
    removed++
    mergeSoundInto(prev, file)
  }
  if (removed) config.files = next
  return removed
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

// ── Project file access ───────────────────────────────────────────────────────
/** What load/save code talks to. Reads run at once; writes are collected and
 *  committed together (see `withProjectDb`). */
export interface Db {
  select<T = unknown>(sql: string, params?: unknown[]): Promise<T>
  execute(sql: string, params?: unknown[]): Promise<unknown>
}

interface Statement {
  sql: string
  params?: unknown[]
  /** A failure of this statement is ignored (idempotent schema upgrades). */
  tolerant?: boolean
}

export interface CommitOptions {
  /** true: store sound files in the project. false: drop stored audio. */
  embedAudio?: boolean
  /** true: keep GIF bytes in the project. false: move them to the app library. */
  embedGifs?: boolean
  /** Copy the file to `<path>.bak` before writing. */
  backup?: boolean
  /** Report progress through `project_save_progress` events (manual saves). */
  progress?: boolean
}

export interface CommitReport {
  embeddedAudio: number
  embeddedBytes: number
  /** Sounds that could not be embedded (file missing, no stored copy). */
  missingAudio: string[]
  backupWritten: boolean
}

export interface ProjectInspect {
  exists: boolean
  sizeBytes: number
  ok: boolean
  error: string | null
  /** No tables yet — a brand-new file. */
  isNew: boolean
  /** null on files last written by a build without format versioning. */
  formatVersion: number | null
  appVersion: string | null
  soundCount: number
  hasBackup: boolean
}

/**
 * Collects writes so they reach the file as ONE transaction.
 *
 * The old path sent every statement separately through a connection pool:
 * a save was `DELETE` everything, then re-insert in chunks, and whatever
 * interrupted it (crash, lock, closed pool) left a half-written board —
 * tabs without sounds, groups without members. Now nothing is visible in the
 * file until the whole batch committed.
 */
class ProjectBatch implements Db {
  readonly statements: Statement[] = []
  constructor(readonly path: string) {}

  select<T = unknown>(sql: string, params?: unknown[]): Promise<T> {
    return invoke<T>('project_query', { path: this.path, sql, params: params ?? [] })
  }

  async execute(sql: string, params?: unknown[]): Promise<void> {
    this.statements.push({ sql, params: params ?? [] })
  }
}

/** Serialises project access so saves, loads and project switches never interleave. */
let dbChain: Promise<unknown> = Promise.resolve()

function withDbLock<T>(fn: () => Promise<T>): Promise<T> {
  const next = dbChain.then(fn, fn)
  // Keep the chain alive even if `fn` rejects.
  dbChain = next.then(() => undefined, () => undefined)
  return next
}

/** Files whose schema was brought up to date in this session. */
const schemaReady = new Set<string>()

function commit(path: string, statements: Statement[], options?: CommitOptions): Promise<CommitReport> {
  return invoke<CommitReport>('project_commit', { path, statements, options: options ?? null })
}

async function ensureSchemaUnlocked(path: string): Promise<void> {
  if (schemaReady.has(path)) return
  await commit(path, schemaStatements())
  schemaReady.add(path)
}

/** Creates missing tables/columns. Safe to call on any project, any number of times. */
export function ensureSchema(dbAbsPath: string): Promise<void> {
  return withDbLock(() => ensureSchemaUnlocked(dbAbsPath))
}

/** Forget cached schema state (the file at `path` was replaced). */
export function forgetProject(dbAbsPath: string): void {
  schemaReady.delete(dbAbsPath)
}

/**
 * Runs `fn` against the project. Reads inside `fn` see the file as it is
 * before the batch; every `execute` is queued and committed atomically after
 * `fn` returns. If `fn` throws, nothing is written.
 */
export async function commitProject<T>(
  dbAbsPath: string,
  fn: (d: Db) => Promise<T>,
  options?: CommitOptions,
): Promise<{ value: T; report: CommitReport | null }> {
  return withDbLock(async () => {
    await ensureSchemaUnlocked(dbAbsPath)
    const batch = new ProjectBatch(dbAbsPath)
    const value = await fn(batch)
    const wantsSync = options?.embedAudio != null || options?.embedGifs != null || options?.backup
    if (!batch.statements.length && !wantsSync) return { value, report: null }
    const report = await commit(dbAbsPath, batch.statements, options)
    return { value, report }
  })
}

/** `commitProject` for callers that only need `fn`'s result. */
export async function withProjectDb<T>(
  dbAbsPath: string,
  fn: (d: Db) => Promise<T>,
  options?: CommitOptions,
): Promise<T> {
  return (await commitProject(dbAbsPath, fn, options)).value
}

/** Looks at a project file without touching it. */
export function inspectProject(dbAbsPath: string): Promise<ProjectInspect> {
  return withDbLock(() => invoke<ProjectInspect>('project_inspect', { path: dbAbsPath }))
}

/**
 * Copies a project as a clean single file. Unlike a byte copy this includes
 * changes still parked in a `-wal` sidecar by an older build.
 */
export function copyProject(src: string, dst: string): Promise<void> {
  return withDbLock(async () => {
    await invoke('project_copy', { src, dst })
    schemaReady.delete(dst)
  })
}

/** Unpacks embedded audio for sounds whose file is missing here. */
export function extractProjectMedia(
  dbAbsPath: string,
  projectId: string,
): Promise<{ soundId: string; path: string }[]> {
  return withDbLock(() => invoke('project_extract_media', { path: dbAbsPath, projectId }))
}

function schemaStatements(): Statement[] {
  const create = (sql: string): Statement => ({ sql })
  const addColumn = (table: string, column: string, sqlType: string): Statement => ({
    sql: `ALTER TABLE ${table} ADD COLUMN ${column} ${sqlType}`,
    // Fails with "duplicate column" on files that already have it.
    tolerant: true,
  })
  return [
    create(`CREATE TABLE IF NOT EXISTS settings (
      key TEXT PRIMARY KEY,
      value TEXT
    )`),
    create(`CREATE TABLE IF NOT EXISTS tabs (
      name TEXT PRIMARY KEY,
      color TEXT,
      position INTEGER
    )`),
    create(`CREATE TABLE IF NOT EXISTS sounds (
      path TEXT PRIMARY KEY,
      name TEXT,
      volume REAL,
      color TEXT,
      global_index INTEGER,
      active INTEGER,
      sound_id TEXT
    )`),
    create(`CREATE TABLE IF NOT EXISTS sound_tabs (
      sound_path TEXT,
      tab TEXT,
      tab_index INTEGER
    )`),
    create(`CREATE TABLE IF NOT EXISTS separators (
      id TEXT PRIMARY KEY,
      tab TEXT,
      position INTEGER
    )`),
    // Bytes live here, not in Pinia. saveConfig must never DELETE this table.
    create(`CREATE TABLE IF NOT EXISTS gif_blobs (
      id TEXT PRIMARY KEY,
      mime TEXT NOT NULL,
      data TEXT NOT NULL,
      poster TEXT,
      byte_len INTEGER
    )`),
    create(`CREATE TABLE IF NOT EXISTS tags (
      id TEXT PRIMARY KEY,
      name TEXT,
      color TEXT
    )`),
    create(`CREATE TABLE IF NOT EXISTS sound_tags (
      sound_path TEXT,
      tag_id TEXT
    )`),
    // Project facts that must survive a save by an older build (which only
    // rewrites the tables it knows).
    create(`CREATE TABLE IF NOT EXISTS meta (
      key TEXT PRIMARY KEY,
      value TEXT
    )`),
    // Undo / redo stacks, so history survives closing the project.
    create(`CREATE TABLE IF NOT EXISTS history (
      id INTEGER PRIMARY KEY,
      stack TEXT NOT NULL,
      pos INTEGER NOT NULL,
      data TEXT NOT NULL
    )`),
    // Whole-board JSON copies kept inside the file for recovery.
    create(`CREATE TABLE IF NOT EXISTS snapshots (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      created_at INTEGER,
      app_version TEXT,
      reason TEXT,
      sound_count INTEGER,
      data TEXT NOT NULL
    )`),
    addColumn('sounds', 'gif_id', 'TEXT'),
    addColumn('sounds', 'gif_pos_x', 'REAL'),
    addColumn('sounds', 'gif_pos_y', 'REAL'),
    addColumn('separators', 'name', 'TEXT'),
    addColumn('separators', 'border_color', 'TEXT'),
    addColumn('separators', 'name_color', 'TEXT'),
    addColumn('separators', 'bg_color', 'TEXT'),
    addColumn('separators', 'button_align', 'TEXT'),
    addColumn('tabs', 'button_align', 'TEXT'),
    addColumn('sounds', 'sound_id', 'TEXT'),
    addColumn('sounds', 'added_at', 'INTEGER'),
    addColumn('sounds', 'file_size', 'INTEGER'),
    addColumn('sounds', 'duration_secs', 'REAL'),
    addColumn('sounds', 'looping', 'INTEGER'),
    // Older builds wrote this column without migrating existing project files,
    // so Save died with "no such column: failed_sounds_path".
    addColumn('sounds', 'failed_sounds_path', 'TEXT'),
    addColumn('sounds', 'rel_path', 'TEXT'),
    // Separator id the sound belongs to on that tab. NULL = not in a group.
    addColumn('sound_tabs', 'group_id', 'TEXT'),
  ]
}

async function tableColumns(d: Db, table: string): Promise<Set<string>> {
  try {
    const rows = await d.select<{ name: string }[]>(`PRAGMA table_info(${table})`)
    return new Set((rows ?? []).map((r) => r.name))
  } catch {
    return new Set()
  }
}

interface SoundRow {
  path: string
  sound_id?: string | null
  name: string | null
  volume: number | null
  color: string | null
  global_index: number | null
  active: number | null
  gif_id?: string | null
  gif_pos_x?: number | null
  gif_pos_y?: number | null
  added_at?: number | null
  file_size?: number | null
  duration_secs?: number | null
  looping?: number | null
  rel_path?: string | null
}

/** Load sounds without requiring a specific column set (old files lack most). */
async function loadSoundRows(d: Db): Promise<SoundRow[]> {
  const cols = await tableColumns(d, 'sounds')
  const pick = (name: string) => (cols.has(name) ? name : 'NULL')
  const idSql = cols.has('sound_id') ? 'sound_id' : cols.has('id') ? 'id' : 'NULL'
  try {
    return await d.select<SoundRow[]>(
      `SELECT path, ${idSql} AS sound_id, name, volume, color, global_index, active,
        ${pick('gif_id')} AS gif_id, ${pick('gif_pos_x')} AS gif_pos_x, ${pick('gif_pos_y')} AS gif_pos_y,
        ${pick('added_at')} AS added_at, ${pick('file_size')} AS file_size,
        ${pick('duration_secs')} AS duration_secs, ${pick('looping')} AS looping,
        ${pick('rel_path')} AS rel_path
       FROM sounds ORDER BY global_index ASC, rowid ASC`,
    )
  } catch {
    return await d.select<SoundRow[]>(
      'SELECT path, name, volume, color, global_index, active FROM sounds ORDER BY global_index ASC',
    )
  }
}

/** Unused default before per-sound volume was applied in playback. */
const LEGACY_UNUSED_VOLUME = 0.4

/** One-shot: unused 0.4 defaults become 1.0. Returns true if config changed. */
export function ensurePerSoundVolume(config: ProjectConfig): boolean {
  if (config.settings.perSoundVolumeV1) return false
  for (const f of config.files) {
    if (f.volume == null || Math.abs(Number(f.volume) - LEGACY_UNUSED_VOLUME) < 1e-6) {
      f.volume = 1
    }
  }
  config.settings.perSoundVolumeV1 = true
  return true
}

/** Write missing sound ids with UPDATE — never a full DELETE+INSERT resync. */
export async function persistSoundIds(d: Db, files: SoundFile[]): Promise<void> {
  for (const f of files) {
    if (!f.id || !f.path) continue
    await d.execute('UPDATE sounds SET sound_id = $1 WHERE path = $2', [f.id, f.path])
  }
}

// ── Sanitise ──────────────────────────────────────────────────────────────────
function fileBaseName(path: string): string {
  const name = String(path || '').split(/[\\/]/).filter(Boolean).pop() ?? ''
  return name.replace(/\.[^.]+$/, '')
}

function num(value: unknown, fallback: number): number {
  const n = Number(value)
  return value != null && value !== '' && Number.isFinite(n) ? n : fallback
}

export interface SanitizeOptions {
  /** `active` is a playback flag. A stored `true` means the app closed while
   *  a sound played; it must not come back as a button stuck "playing". */
  resetActive?: boolean
}

/**
 * Bring a config into a shape the board can always render and the file can
 * always store, whatever produced it (older build, newer build, crashed
 * save, hand-edited file, history entry). Never throws; returns the number
 * of repairs made.
 */
export function sanitizeConfig(config: ProjectConfig, opts: SanitizeOptions = {}): number {
  let fixes = 0
  const fix = () => { fixes++ }

  if (!config.settings || typeof config.settings !== 'object') {
    config.settings = defaultSettings()
    fix()
  }
  if (!Array.isArray(config.tabList)) { config.tabList = []; fix() }
  if (!Array.isArray(config.files)) { config.files = []; fix() }
  if (!Array.isArray(config.separators)) { config.separators = []; fix() }
  if (!Array.isArray(config.tags)) { config.tags = []; fix() }

  // Tags: unique ids.
  const tagIds = new Set<string>()
  const tags: SoundTag[] = []
  for (const t of config.tags) {
    if (!t || typeof t.id !== 'string' || !t.id || tagIds.has(t.id)) { fix(); continue }
    tagIds.add(t.id)
    if (typeof t.name !== 'string') { t.name = ''; fix() }
    if (typeof t.color !== 'string' || !t.color) { t.color = '#00d4ff'; fix() }
    tags.push(t)
  }
  if (tags.length !== config.tags.length) config.tags = tags

  // Tabs: named, unique, never the built-in "All".
  const tabNames = new Set<string>()
  const tabList: TabEntry[] = []
  for (const t of config.tabList) {
    const name = typeof t === 'string' ? t : t?.name
    if (typeof name !== 'string' || !name || name === 'All' || tabNames.has(name)) { fix(); continue }
    tabNames.add(name)
    tabList.push(typeof t === 'string' ? { name } : t)
  }
  if (tabList.length !== config.tabList.length || config.tabList.some((t) => typeof (t as unknown) === 'string')) {
    config.tabList = tabList
  }

  // Sounds.
  const files: SoundFile[] = []
  const soundIds = new Set<string>()
  for (const f of config.files) {
    if (!f || typeof f.path !== 'string' || !f.path) { fix(); continue }
    if (typeof f.name !== 'string' || !f.name) { f.name = fileBaseName(f.path) || 'Sound'; fix() }
    if (typeof f.id !== 'string') { f.id = ''; fix() }
    // Two buttons must not share an id: hotkeys and remote triggers address by id.
    if (f.id) {
      if (soundIds.has(f.id)) { f.id = ''; fix() }
      else soundIds.add(f.id)
    }
    const volume = Math.min(1, Math.max(0, num(f.volume, 1)))
    if (f.volume !== volume) { f.volume = volume; fix() }
    const index = num(f.index, Number.MAX_SAFE_INTEGER)
    if (f.index !== index) { f.index = index; fix() }

    const tabs: string[] = ['All']
    for (const t of Array.isArray(f.tabs) ? f.tabs : []) {
      if (typeof t === 'string' && t && !tabs.includes(t)) tabs.push(t)
    }
    if (!Array.isArray(f.tabs) || f.tabs.length !== tabs.length || f.tabs.some((t, i) => t !== tabs[i])) {
      f.tabs = tabs
      fix()
    }
    if (!f.tabIndexes || typeof f.tabIndexes !== 'object' || Array.isArray(f.tabIndexes)) {
      f.tabIndexes = {}
      fix()
    }
    for (const key of Object.keys(f.tabIndexes)) {
      const v = f.tabIndexes[key]
      if (!f.tabs.includes(key) || typeof v !== 'number' || !Number.isFinite(v)) {
        delete f.tabIndexes[key]
        fix()
      }
    }

    const wantTags = [...new Set((Array.isArray(f.tagIds) ? f.tagIds : []).filter((id) => tagIds.has(id)))]
    if (!Array.isArray(f.tagIds) || f.tagIds.length !== wantTags.length) {
      f.tagIds = wantTags
      fix()
    }
    if (opts.resetActive && f.active) { f.active = false; fix() }
    if (typeof f.active !== 'boolean') f.active = !!f.active
    if (f.looping != null && typeof f.looping !== 'boolean') f.looping = !!f.looping
    files.push(f)
  }
  if (files.length !== config.files.length) config.files = files
  fixes += dedupeSoundsByPath(config)

  // Groups: unique id, a tab, a numeric slot.
  const sepIds = new Set<string>()
  const separators: Separator[] = []
  for (const s of config.separators) {
    if (!s || typeof s.tab !== 'string' || !s.tab) { fix(); continue }
    if (typeof s.id !== 'string' || !s.id || sepIds.has(s.id)) {
      s.id = `sep_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
      fix()
    }
    sepIds.add(s.id)
    const position = num(s.position, Number.MAX_SAFE_INTEGER)
    if (s.position !== position) { s.position = position; fix() }
    separators.push(s)
  }
  if (separators.length !== config.separators.length) config.separators = separators

  // A tab that sounds or groups still use must exist, or its content is unreachable.
  fixes += mergeTabsFromUsage(config)

  // A tag filter naming tags that no longer exist hides every sound on every tab.
  const selected = config.settings.selectedTagIds
  const wantSelected = (Array.isArray(selected) ? selected : []).filter((id) => tagIds.has(id))
  if (!Array.isArray(selected) || selected.length !== wantSelected.length) {
    config.settings.selectedTagIds = wantSelected
    fix()
  }
  if (!Array.isArray(config.settings.soundHotkeys)) {
    config.settings.soundHotkeys = []
    fix()
  }
  return fixes
}

// ── Load ──────────────────────────────────────────────────────────────────────
/** Settings rows that are bookkeeping, not `Settings` fields. */
const BOOKKEEPING_KEYS = new Set(['soundCount', 'formatVersion', 'appVersion'])

/** Applies one stored settings row. False when this build does not know the key. */
function applySettingRow(settings: Settings, key: string, value: string): boolean {
  switch (key) {
    case 'theme': settings.theme = value; break
    case 'customCss': settings.customCss = value; break
    case 'outputSource': settings.outputSource = value; break
    case 'stopOnRetrigger': settings.stopOnRetrigger = value === 'true'; break
    case 'overlapSounds': settings.overlapSounds = value === 'true'; break
    case 'showPlayer': settings.showPlayer = value === 'true'; break
    case 'playerLarge': settings.playerLarge = value === 'true'; break
    case 'showWaveform': settings.showWaveform = value !== 'false'; break
    case 'gifPlayOnHover': settings.gifPlayOnHover = value !== 'false'; break
    case 'preloadGifs': settings.preloadGifs = value === 'true'; break
    case 'tabTransition': settings.tabTransition = normalizeTabTransition(value); break
    case 'soundHotkeys':
      try { settings.soundHotkeys = JSON.parse(value) } catch { settings.soundHotkeys = [] }
      if (!Array.isArray(settings.soundHotkeys)) settings.soundHotkeys = []
      break
    case 'perSoundVolumeV1': settings.perSoundVolumeV1 = value === 'true'; break
    case 'sortMode': settings.sortMode = normalizeSortMode(value); break
    case 'selectedTagIds':
      try { settings.selectedTagIds = JSON.parse(value) } catch { settings.selectedTagIds = [] }
      if (!Array.isArray(settings.selectedTagIds)) settings.selectedTagIds = []
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
    default: return false
  }
  return true
}

function alignOf(value: unknown): ButtonAlign | undefined {
  return value === 'left' || value === 'center' || value === 'right' ? value : undefined
}

export interface LoadedConfig {
  config: ProjectConfig
  /** null when the file was last written by a build without format versioning. */
  formatVersion: number | null
  /** `settings.soundCount`. null if that write never landed (see readSoundCountMarker). */
  marker: number | null
}

export async function loadConfigDetailed(d: Db): Promise<LoadedConfig> {
  const settings = defaultSettings()
  const extraSettings: Record<string, string> = {}
  let formatVersion: number | null = null
  let marker: number | null = null
  const sRows = await d.select<{ key: string; value: string | null }[]>(
    'SELECT key, value FROM settings'
  )
  for (const row of sRows ?? []) {
    const key = row.key
    const value = row.value ?? ''
    if (key === 'formatVersion') {
      const v = Number(value)
      formatVersion = Number.isFinite(v) && v > 0 ? v : null
    } else if (key === 'soundCount') {
      const n = Number(value)
      marker = value !== '' && Number.isFinite(n) ? n : null
    } else if (BOOKKEEPING_KEYS.has(key)) {
      continue
    } else if (!applySettingRow(settings, key, value)) {
      // Written by a newer build. Keep it so saving here does not delete it.
      extraSettings[key] = value
    }
  }

  // Old factory cache (64/16 MiB) forced 1–3 min clips onto the streaming
  // decoder. Bump only that exact pair so a user-chosen small cache stays.
  if (settings.cacheMaxSizeMib === 64 && settings.cacheMaxEntryMib === 16) {
    settings.cacheMaxSizeMib = 256
    settings.cacheMaxEntryMib = 128
  }

  // Migrate legacy dark-* theme ids and pair fields → flat tokens.
  settings.theme = normalizeThemeId(settings.theme)
  if (!settings.bg && !settings.btnBg) {
    const flat = resolveThemeTokens(settings as unknown as Record<string, unknown>)
    Object.assign(settings, flat)
  } else if (!settings.bg2) {
    settings.bg2 = resolveThemeTokens(settings as unknown as Record<string, unknown>).bg2
  }

  type TabRow = { name: string; color: string | null; position: number; button_align?: string | null }
  let tabRows: TabRow[]
  try {
    tabRows = await d.select<TabRow[]>(
      'SELECT name, color, position, button_align FROM tabs ORDER BY position ASC, rowid ASC',
    )
  } catch {
    tabRows = await d.select<TabRow[]>('SELECT name, color, position FROM tabs ORDER BY position ASC')
  }
  const tabList: TabEntry[] = (tabRows ?? []).map((t) => ({
    name: t.name,
    ...(t.color ? { color: t.color } : {}),
    ...(alignOf(t.button_align) ? { buttonAlign: alignOf(t.button_align) } : {}),
  }))

  const soundRows = await loadSoundRows(d)

  type LinkRow = { sound_path: string; tab: string; tab_index: number | null; group_id?: string | null }
  let tabLinks: LinkRow[]
  try {
    tabLinks = await d.select<LinkRow[]>(
      'SELECT sound_path, tab, tab_index, group_id FROM sound_tabs',
    )
  } catch {
    tabLinks = await d.select<LinkRow[]>('SELECT sound_path, tab, tab_index FROM sound_tabs')
  }
  const linksByPath = new Map<string, LinkRow[]>()
  /** tab → sound path → separator id */
  const groupByTab = new Map<string, Map<string, string>>()
  for (const l of tabLinks ?? []) {
    if (!l?.sound_path || !l.tab) continue
    const arr = linksByPath.get(l.sound_path) ?? []
    arr.push(l)
    linksByPath.set(l.sound_path, arr)
    if (l.group_id) {
      let byPath = groupByTab.get(l.tab)
      if (!byPath) groupByTab.set(l.tab, (byPath = new Map()))
      byPath.set(l.sound_path, l.group_id)
    }
  }

  const files: SoundFile[] = (soundRows ?? []).map((s) => {
    const links = linksByPath.get(s.path) ?? []
    const tabs = ['All']
    const tabIndexes: Record<string, number> = {}
    for (const l of links) {
      if (l.tab === 'All') continue
      // sound_tabs has no unique key: a duplicated row must not list a tab twice.
      if (!tabs.includes(l.tab)) tabs.push(l.tab)
      const idx = Number(l.tab_index)
      if (l.tab_index != null && Number.isFinite(idx) && tabIndexes[l.tab] == null) tabIndexes[l.tab] = idx
    }
    return {
      path: s.path,
      id: s.sound_id || '',
      name: s.name ?? '',
      volume: s.volume ?? 1,
      index: s.global_index ?? Number.MAX_SAFE_INTEGER,
      // Playback state never comes from disk.
      active: false,
      tabs,
      tabIndexes,
      tagIds: [],
      ...(s.color ? { color: s.color } : {}),
      ...(s.gif_id ? { gifId: s.gif_id } : {}),
      ...(s.gif_pos_x != null ? { gifPosX: Number(s.gif_pos_x) } : {}),
      ...(s.gif_pos_y != null ? { gifPosY: Number(s.gif_pos_y) } : {}),
      ...(s.added_at != null ? { addedAt: Number(s.added_at) } : {}),
      ...(s.file_size != null ? { fileSize: Number(s.file_size) } : {}),
      ...(s.duration_secs != null ? { durationSecs: Number(s.duration_secs) } : {}),
      ...(s.rel_path ? { relPath: s.rel_path } : {}),
      looping: s.looping === 1,
    }
  })

  type SepRow = {
    id: string
    tab: string
    position: number
    name?: string | null
    border_color?: string | null
    name_color?: string | null
    bg_color?: string | null
    button_align?: string | null
  }
  let sepRows: SepRow[]
  try {
    sepRows = await d.select<SepRow[]>(
      'SELECT id, tab, position, name, border_color, name_color, bg_color, button_align FROM separators ORDER BY position ASC, rowid ASC',
    )
  } catch {
    sepRows = await d.select<SepRow[]>('SELECT id, tab, position FROM separators ORDER BY position ASC')
  }
  const separators: Separator[] = (sepRows ?? []).map((r) => ({
    id: r.id,
    tab: r.tab,
    position: r.position,
    ...(r.name ? { name: r.name } : {}),
    ...(r.border_color ? { borderColor: r.border_color } : {}),
    ...(r.name_color ? { nameColor: r.name_color } : {}),
    ...(r.bg_color ? { bgColor: r.bg_color } : {}),
    ...(alignOf(r.button_align) ? { buttonAlign: alignOf(r.button_align) } : {}),
  }))

  let tags: SoundTag[] = []
  try {
    const tagRows = await d.select<{ id: string; name: string; color: string }[]>(
      'SELECT id, name, color FROM tags',
    )
    tags = (tagRows ?? []).map((t) => ({
      id: t.id,
      name: t.name ?? '',
      color: t.color || '#00d4ff',
    }))
    const soundTagRows = await d.select<{ sound_path: string; tag_id: string }[]>(
      'SELECT sound_path, tag_id FROM sound_tags',
    )
    const byPath = new Map(files.map((f) => [f.path, f]))
    for (const row of soundTagRows ?? []) {
      const file = byPath.get(row.sound_path)
      if (!file || !row.tag_id) continue
      if (!file.tagIds) file.tagIds = []
      if (!file.tagIds.includes(row.tag_id)) file.tagIds.push(row.tag_id)
    }
  } catch {
    tags = []
  }

  const config: ProjectConfig = { settings, tabList, files, separators, tags }
  if (Object.keys(extraSettings).length) config.extraSettings = extraSettings
  sanitizeConfig(config, { resetActive: true })

  if (formatVersion != null && formatVersion >= 2) {
    // Group membership is stored explicitly. Rebuild the shared number line
    // from it instead of trusting positions, so a colliding or stale index
    // can never move a sound into another group.
    for (const tab of boardTabNames(config)) {
      const byPath = groupByTab.get(tab)
      applyBoardLayout(config, tab, layoutFromMembership(config, tab, (p) => byPath?.get(p)))
    }
  } else {
    // Legacy file: folder-imported sounds sometimes only carry an `All` row.
    healFolderTabMembership(config)
  }
  return { config, formatVersion, marker }
}

export async function loadConfig(d: Db): Promise<ProjectConfig> {
  return (await loadConfigDetailed(d)).config
}

// ── Save (full re-sync inside one transaction) ────────────────────────────────
/** Inserts many rows in a few batched multi-VALUES statements.
 *  Chunked so the bound-parameter count stays well under SQLite's limit. */
async function batchInsert(
  d: Db,
  table: string,
  cols: string[],
  rows: unknown[][],
  chunkRows = 50,
  verb: 'INSERT' | 'INSERT OR REPLACE' = 'INSERT',
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
    await d.execute(`${verb} INTO ${table} (${colSql}) VALUES ${placeholders.join(', ')}`, params)
  }
}

export interface SaveConfigOptions {
  /** User cleared the board. Without this, an empty list must not wipe existing rows. */
  allowEmpty?: boolean
  /** App version stamped into the file (shown when a newer build's file is opened). */
  appVersion?: string
}

/**
 * Reopen should recover an empty board only when it looks unfinished.
 * `soundCount === 0` is a completed intentional clear. `null` (marker missing)
 * or any non-zero marker with zero files is a save that never completed —
 * possible in files written by builds whose save was not atomic.
 */
export function shouldRestoreClearedBoard(fileCount: number, marker: number | null): boolean {
  return fileCount === 0 && marker !== 0
}

/** `settings.soundCount`, written only after sound rows. `null` if that write never landed. */
export async function readSoundCountMarker(d: Db): Promise<number | null> {
  try {
    const rows = await d.select<{ value: string }[]>(
      "SELECT value FROM settings WHERE key = 'soundCount'",
    )
    const raw = rows?.[0]?.value
    if (raw == null || raw === '') return null
    const n = Number(raw)
    return Number.isFinite(n) ? n : null
  } catch {
    return null
  }
}

export async function saveConfig(
  d: Db,
  config: ProjectConfig,
  opts?: SaveConfigOptions,
): Promise<void> {
  // Every `execute` below is queued by the caller's batch and committed as a
  // single transaction (see `commitProject`). A failure anywhere leaves the
  // file exactly as it was.
  sanitizeConfig(config)
  const incoming = config.files?.length ?? 0
  if (incoming === 0 && !opts?.allowEmpty) {
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
  await d.execute('DELETE FROM tags')
  await d.execute('DELETE FROM sound_tags')

  const s = config.settings
  // Note: outputSource / outputHost / outputVolume / ASIO channels are app-wide
  // (app-config.db) and must not be written into the project file.
  const settingsRows: [string, string][] = [
    ['formatVersion', String(PROJECT_FORMAT_VERSION)],
    ['appVersion', opts?.appVersion ?? ''],
    ['theme', s.theme ?? 'soundninja'],
    ['customCss', s.customCss ?? ''],
    ['stopOnRetrigger', String(s.stopOnRetrigger ?? true)],
    ['overlapSounds', String(s.overlapSounds ?? false)],
    ['showPlayer', String(s.showPlayer ?? true)],
    ['playerLarge', String(s.playerLarge ?? false)],
    ['showWaveform', String(s.showWaveform !== false)],
    ['cacheMaxSizeMib', String(s.cacheMaxSizeMib ?? 256)],
    ['cacheMaxEntryMib', String(s.cacheMaxEntryMib ?? 128)],
    ['uniformButtonHeight', String(s.uniformButtonHeight ?? false)],
    ['allowReorder', String(s.allowReorder ?? true)],
    ['gifPlayOnHover', String(s.gifPlayOnHover !== false)],
    ['preloadGifs', String(s.preloadGifs === true)],
    ['tabTransition', normalizeTabTransition(s.tabTransition)],
    ['soundHotkeys', JSON.stringify(s.soundHotkeys ?? [])],
    ['perSoundVolumeV1', String(s.perSoundVolumeV1 === true)],
    ['sortMode', normalizeSortMode(s.sortMode)],
    ['selectedTagIds', JSON.stringify(Array.isArray(s.selectedTagIds) ? s.selectedTagIds : [])],
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
  // Settings a newer build wrote: hand them back unchanged.
  const known = new Set(settingsRows.map(([key]) => key))
  for (const [key, value] of Object.entries(config.extraSettings ?? {})) {
    if (!known.has(key) && !BOOKKEEPING_KEYS.has(key)) settingsRows.push([key, String(value ?? '')])
  }
  await batchInsert(d, 'settings', ['key', 'value'], settingsRows)

  const tabRows = config.tabList.map((t, i) => [
    t.name,
    t.color ?? null,
    i,
    t.buttonAlign ?? null,
  ])
  await batchInsert(d, 'tabs', ['name', 'color', 'position', 'button_align'], tabRows)

  // Which group each sound sits in, per tab — stored next to its position so
  // membership survives even if positions are ever damaged.
  const membership = new Map<string, Map<string, string>>()
  const groupOf = (tab: string, path: string): string | null => {
    let byPath = membership.get(tab)
    if (!byPath) membership.set(tab, (byPath = groupMembership(config, tab)))
    return byPath.get(path) ?? null
  }

  const soundRows: unknown[][] = []
  const soundTabRows: unknown[][] = []
  for (const f of config.files) {
    soundRows.push([
      f.path,
      f.id || '',
      f.name,
      f.volume ?? 1,
      f.color ?? null,
      f.index ?? 0,
      // `active` is playback state; it is never stored as set.
      0,
      f.gifId ?? null,
      f.gifPosX ?? 50,
      f.gifPosY ?? 50,
      f.addedAt ?? null,
      f.fileSize ?? null,
      f.durationSecs ?? null,
      f.looping ? 1 : 0,
      f.relPath ?? null,
    ])
    for (const tab of f.tabs ?? ['All']) {
      const tabIdx = tab === 'All' ? f.index ?? 0 : f.tabIndexes?.[tab] ?? 0
      soundTabRows.push([f.path, tab, tabIdx, groupOf(tab, f.path)])
    }
  }
  await batchInsert(
    d,
    'sounds',
    ['path', 'sound_id', 'name', 'volume', 'color', 'global_index', 'active', 'gif_id', 'gif_pos_x', 'gif_pos_y', 'added_at', 'file_size', 'duration_secs', 'looping', 'rel_path'],
    soundRows,
    50,
    'INSERT OR REPLACE',
  )
  await batchInsert(d, 'sound_tabs', ['sound_path', 'tab', 'tab_index', 'group_id'], soundTabRows)

  const tagRows = (config.tags ?? []).map((t) => [t.id, t.name ?? '', t.color || '#00d4ff'])
  await batchInsert(d, 'tags', ['id', 'name', 'color'], tagRows)
  const soundTagRows: unknown[][] = []
  for (const f of config.files) {
    for (const tagId of f.tagIds ?? []) {
      if (tagId) soundTagRows.push([f.path, tagId])
    }
  }
  await batchInsert(d, 'sound_tags', ['sound_path', 'tag_id'], soundTagRows)

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
    50,
    'INSERT OR REPLACE',
  )

  // Kept for builds that predate atomic saves: they read this marker to tell
  // a finished empty board from an interrupted save.
  await d.execute(
    `INSERT INTO settings (key, value) VALUES ('soundCount', $1)`,
    [String(soundRows.length)],
  )
}

// ── Project meta ──────────────────────────────────────────────────────────────
export async function loadMeta(d: Db): Promise<ProjectMeta> {
  const meta = defaultMeta()
  try {
    const rows = await d.select<{ key: string; value: string | null }[]>('SELECT key, value FROM meta')
    for (const { key, value } of rows ?? []) {
      if (key === 'projectId') meta.projectId = value ?? ''
      else if (key === 'embedAudio') meta.embedAudio = value === '1'
      else if (key === 'embedGifs') meta.embedGifs = value !== '0'
      else if (key === 'saveOptionsSet') meta.saveOptionsSet = value === '1'
    }
  } catch { /* table missing on a file no build of this format has touched */ }
  return meta
}

export async function saveMeta(d: Db, meta: ProjectMeta): Promise<void> {
  const rows: [string, string][] = [
    ['projectId', meta.projectId],
    ['embedAudio', meta.embedAudio ? '1' : '0'],
    ['embedGifs', meta.embedGifs ? '1' : '0'],
    ['saveOptionsSet', meta.saveOptionsSet ? '1' : '0'],
    ['savedAt', String(Date.now())],
  ]
  await batchInsert(d, 'meta', ['key', 'value'], rows, 50, 'INSERT OR REPLACE')
}

// ── Undo / redo history ───────────────────────────────────────────────────────
/** Upper bound of history kept in the file, in JSON characters. A snapshot is
 *  a whole board; without a cap a large project would multiply its own size. */
const HISTORY_BUDGET_CHARS = 8_000_000
const HISTORY_MAX_PERSISTED = 60

function snapshotJson(snap: ProjectConfig): string {
  const { _hid: _omit, ...rest } = snap
  return JSON.stringify(rest)
}

function snapshotFromJson(data: string): ProjectConfig | null {
  try {
    const raw = JSON.parse(data)
    if (!raw || typeof raw !== 'object' || !Array.isArray(raw.files)) return null
    const snap: ProjectConfig = {
      settings: { ...defaultSettings(), ...(raw.settings ?? {}) },
      tabList: raw.tabList ?? [],
      files: raw.files,
      separators: raw.separators ?? [],
      tags: raw.tags ?? [],
    }
    if (raw.extraSettings && typeof raw.extraSettings === 'object') snap.extraSettings = raw.extraSettings
    sanitizeConfig(snap, { resetActive: true })
    return snap
  } catch {
    return null
  }
}

export interface LoadedHistory {
  undo: ProjectConfig[]
  redo: ProjectConfig[]
  /** Stored entry id → JSON length, so unchanged entries are not re-sent. */
  sizes: Map<number, number>
}

export async function loadHistory(d: Db): Promise<LoadedHistory> {
  const out: LoadedHistory = { undo: [], redo: [], sizes: new Map() }
  try {
    const rows = await d.select<{ id: number; stack: string; pos: number; data: string }[]>(
      'SELECT id, stack, pos, data FROM history ORDER BY pos ASC, id ASC',
    )
    for (const row of rows ?? []) {
      const snap = snapshotFromJson(row.data)
      if (!snap) continue
      snap._hid = Number(row.id)
      out.sizes.set(snap._hid, row.data.length)
      if (row.stack === 'redo') out.redo.push(snap)
      else out.undo.push(snap)
    }
  } catch { /* no history table yet */ }
  return out
}

/**
 * Queue the undo/redo stacks for the next commit. Entries already in the file
 * (`stored`) only get their stack/position updated; new ones are inserted.
 * The newest steps win when the budget is tight. Every entry needs `_hid`.
 * Returns the id → size map describing the file after the commit.
 */
export async function saveHistory(
  d: Db,
  undo: ProjectConfig[],
  redo: ProjectConfig[],
  stored: Map<number, number>,
): Promise<Map<number, number>> {
  type Pick = { id: number; stack: 'undo' | 'redo'; pos: number; json: string | null; size: number }
  const picks: Pick[] = []
  let used = 0
  const cursors = [
    { stack: 'undo' as const, list: undo, at: undo.length - 1 },
    { stack: 'redo' as const, list: redo, at: redo.length - 1 },
  ]
  // Newest step of each stack first, alternating, until the budget is spent.
  while (picks.length < HISTORY_MAX_PERSISTED && cursors.some((c) => c.at >= 0)) {
    for (const c of cursors) {
      if (c.at < 0) continue
      const snap = c.list[c.at]!
      const id = snap._hid
      if (id == null) { c.at = -1; continue }
      const known = stored.get(id)
      const json = known == null ? snapshotJson(snap) : null
      const size = known ?? json!.length
      if (used + size > HISTORY_BUDGET_CHARS && picks.length > 0) { c.at = -1; continue }
      used += size
      picks.push({ id, stack: c.stack, pos: c.at, json, size })
      c.at -= 1
    }
  }

  const next = new Map<number, number>()
  if (!picks.length) {
    await d.execute('DELETE FROM history')
    return next
  }
  const ids = picks.map((p) => p.id)
  await d.execute(`DELETE FROM history WHERE id NOT IN (${ids.map((_, i) => `$${i + 1}`).join(', ')})`, ids)
  for (const p of picks) {
    next.set(p.id, p.size)
    if (p.json == null) {
      await d.execute('UPDATE history SET stack = $1, pos = $2 WHERE id = $3', [p.stack, p.pos, p.id])
    } else {
      await d.execute(
        'INSERT OR REPLACE INTO history (id, stack, pos, data) VALUES ($1, $2, $3, $4)',
        [p.id, p.stack, p.pos, p.json],
      )
    }
  }
  return next
}

// ── Snapshots (recovery copies inside the file) ───────────────────────────────
const SNAPSHOTS_KEPT = 5

/** Queue a whole-board copy. Old copies beyond `SNAPSHOTS_KEPT` are dropped. */
export async function saveSnapshot(
  d: Db,
  config: ProjectConfig,
  reason: string,
  appVersion = '',
): Promise<void> {
  await d.execute(
    'INSERT INTO snapshots (created_at, app_version, reason, sound_count, data) VALUES ($1, $2, $3, $4, $5)',
    [Date.now(), appVersion, reason, config.files?.length ?? 0, snapshotJson(config)],
  )
  await d.execute(
    `DELETE FROM snapshots WHERE id NOT IN (SELECT id FROM snapshots ORDER BY id DESC LIMIT ${SNAPSHOTS_KEPT})`,
  )
}

/** Newest stored board copy that still has sounds, or null. */
export async function loadLatestSnapshot(d: Db): Promise<ProjectConfig | null> {
  try {
    const rows = await d.select<{ data: string }[]>(
      'SELECT data FROM snapshots WHERE sound_count > 0 ORDER BY id DESC LIMIT 3',
    )
    for (const row of rows ?? []) {
      const snap = snapshotFromJson(row.data)
      if (snap?.files.length) return snap
    }
  } catch { /* no snapshots table */ }
  return null
}

// ── Whole-project load ────────────────────────────────────────────────────────
export interface LoadedProject extends LoadedConfig {
  meta: ProjectMeta
  history: LoadedHistory
}

/** Reads everything a project holds. Throws when the file cannot be read. */
export async function loadProject(dbAbsPath: string): Promise<LoadedProject> {
  return withProjectDb(dbAbsPath, async (d) => {
    const detail = await loadConfigDetailed(d)
    const meta = await loadMeta(d)
    const history = await loadHistory(d)
    return { ...detail, meta, history }
  })
}

// ── GIF blobs (separate from the full-resync save; never wiped by saveConfig) ─
export async function upsertGifBlob(d: Db, row: GifBlobRow): Promise<void> {
  await d.execute(
    `INSERT INTO gif_blobs (id, mime, data, poster, byte_len) VALUES ($1, $2, $3, $4, $5)
     ON CONFLICT(id) DO NOTHING`,
    [row.id, row.mime, row.data, row.poster, row.byteLen],
  )
}

export async function loadGifBlob(d: Db, id: string): Promise<GifBlobRow | null> {
  const rows = await d.select<
    { id: string; mime: string; data: string; poster: string | null; byte_len: number }[]
  >('SELECT id, mime, data, poster, byte_len FROM gif_blobs WHERE id = $1', [id])
  const r = rows[0]
  if (!r) return null
  return { id: r.id, mime: r.mime, data: r.data, poster: r.poster ?? null, byteLen: r.byte_len ?? 0 }
}

export async function loadGifBlobsByIds(d: Db, ids: string[]): Promise<GifBlobRow[]> {
  const unique = [...new Set(ids.filter(Boolean))]
  if (!unique.length) return []
  const out: GifBlobRow[] = []
  for (const id of unique) {
    const row = await loadGifBlob(d, id)
    if (row) out.push(row)
  }
  return out
}

export async function gcOrphanGifs(d: Db, keepIds: string[] = []): Promise<void> {
  const keep = [...new Set(keepIds.filter(Boolean))]
  if (!keep.length) {
    await d.execute('DELETE FROM gif_blobs')
    return
  }
  const placeholders = keep.map((_, i) => `$${i + 1}`).join(', ')
  await d.execute(`DELETE FROM gif_blobs WHERE id NOT IN (${placeholders})`, keep)
}
