import Database from '@tauri-apps/plugin-sql'
import { invoke } from '@tauri-apps/api/core'

/** Portable-first default paths resolved by the Rust backend. */
export interface DefaultPaths {
  projectsPath: string
  themesPath: string
  appConfigDbPath: string
  appSettingsJsonPath: string
}

export interface RecentProject {
  dbPath: string
  name: string
  lastOpened: number
}

export async function getDefaultPaths(): Promise<DefaultPaths> {
  return invoke<DefaultPaths>('get_default_paths')
}

let db: Database | null = null

function toUrl(dbAbsPath: string): string {
  return 'sqlite:' + dbAbsPath.replace(/\\/g, '/')
}

/** Opens (and caches) the app-config database, initialising its schema. */
export async function openAppConfigDb(dbAbsPath: string): Promise<Database> {
  if (db) return db
  db = await Database.load(toUrl(dbAbsPath))
  await initSchema(db)
  return db
}

async function initSchema(d: Database): Promise<void> {
  await d.execute(`CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT
  )`)
  await d.execute(`CREATE TABLE IF NOT EXISTS recent_projects (
    db_path TEXT PRIMARY KEY,
    name TEXT,
    last_opened INTEGER
  )`)
}

// ── Settings ──────────────────────────────────────────────────────────────────
export async function loadSettings(d: Database): Promise<Record<string, string>> {
  const rows = await d.select<{ key: string; value: string }[]>('SELECT key, value FROM settings')
  const out: Record<string, string> = {}
  for (const r of rows) out[r.key] = r.value
  return out
}

export async function saveSetting(d: Database, key: string, value: string): Promise<void> {
  await d.execute(
    'INSERT INTO settings (key, value) VALUES ($1, $2) ON CONFLICT(key) DO UPDATE SET value = $2',
    [key, value]
  )
}

// ── Recent projects ───────────────────────────────────────────────────────────
export async function addRecent(d: Database, dbPath: string, name: string): Promise<void> {
  await d.execute(
    `INSERT INTO recent_projects (db_path, name, last_opened) VALUES ($1, $2, $3)
     ON CONFLICT(db_path) DO UPDATE SET name = $2, last_opened = $3`,
    [dbPath, name, Date.now()]
  )
}

export async function listRecent(d: Database, limit: number): Promise<RecentProject[]> {
  const rows = await d.select<{ db_path: string; name: string; last_opened: number }[]>(
    'SELECT db_path, name, last_opened FROM recent_projects ORDER BY last_opened DESC LIMIT $1',
    [Math.max(1, limit)]
  )
  return rows.map((r) => ({ dbPath: r.db_path, name: r.name, lastOpened: r.last_opened }))
}

export async function removeRecent(d: Database, dbPath: string): Promise<void> {
  await d.execute('DELETE FROM recent_projects WHERE db_path = $1', [dbPath])
}

/** Trims the recent list to the newest `limit` entries. */
export async function trimRecent(d: Database, limit: number): Promise<void> {
  await d.execute(
    `DELETE FROM recent_projects WHERE db_path NOT IN (
       SELECT db_path FROM recent_projects ORDER BY last_opened DESC LIMIT $1
     )`,
    [Math.max(1, limit)]
  )
}
