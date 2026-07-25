import { invoke } from '@tauri-apps/api/core'

export interface ProjectInfo {
  name: string
  dbPath: string
}

/** Preferred project file extension for new saves / creates. */
export const PROJECT_EXT = 'sninja'
/** Legacy extension still accepted when opening. */
export const PROJECT_EXT_LEGACY = 'db'

export const PROJECT_FILE_FILTER = {
  name: 'Sound Ninja Project',
  extensions: [PROJECT_EXT, PROJECT_EXT_LEGACY],
} as const

export const PROJECT_SAVE_FILTER = {
  name: 'Sound Ninja Project',
  extensions: [PROJECT_EXT],
} as const

const PROJECT_BASENAME_SNINJA = `project.${PROJECT_EXT}`
const PROJECT_BASENAME_DB = `project.${PROJECT_EXT_LEGACY}`

function sep(base: string): string {
  return base.includes('\\') ? '\\' : '/'
}

function join(base: string, ...parts: string[]): string {
  const s = sep(base)
  return [base.replace(/[\\/]+$/, ''), ...parts].join(s)
}

/** Sanitises a user-entered project name into a safe folder name. */
export function safeProjectName(name: string): string {
  return name.replace(/[^a-z0-9_\- ]/gi, '_').replace(/\s+/g, '_')
}

/** Absolute path of a project's file: <projectsPath>/<name>/project.sninja */
export function projectDbPath(projectsPath: string, name: string): string {
  return join(projectsPath, safeProjectName(name), PROJECT_BASENAME_SNINJA)
}

export async function listProjects(projectsPath: string): Promise<ProjectInfo[]> {
  return invoke<ProjectInfo[]>('list_projects', { projectsPath })
}

/** Creates a project folder (the DB itself is created on first open). */
export async function createProjectFolder(projectsPath: string, name: string): Promise<string> {
  const folder = join(projectsPath, safeProjectName(name))
  await invoke('make_dir_abs', { path: folder })
  return join(folder, PROJECT_BASENAME_SNINJA)
}

/** True if path looks like a Sound Ninja project file (.sninja or legacy .db). */
export function isProjectFilePath(path: string): boolean {
  return /\.(sninja|db)$/i.test(path)
}

/**
 * Normalises a user-picked save path to end with `.sninja`.
 * Legacy `.db` is rewritten to `.sninja` for new saves.
 */
export function ensureSaveProjectPath(path: string): string {
  if (/\.sninja$/i.test(path)) return path
  if (/\.db$/i.test(path)) return path.replace(/\.db$/i, `.${PROJECT_EXT}`)
  return `${path}.${PROJECT_EXT}`
}

/** Parent directory of a file path. */
export function parentDir(filePath: string): string {
  return filePath.replace(/[\\/][^\\/]+$/, '')
}

/** Ensures the parent folder of a project file path exists. */
export async function ensureProjectParentDir(absPath: string): Promise<void> {
  const parent = parentDir(absPath)
  if (!parent || parent === absPath) return
  await invoke('make_dir_abs', { path: parent })
}

/**
 * Save dialogs on Windows often create a 0-byte placeholder. SQLite then fails
 * with "file is not a database". Delete that placeholder so open can recreate.
 */
export async function removeInvalidProjectPlaceholder(absPath: string): Promise<void> {
  try {
    const exists = await invoke<boolean>('path_exists_abs', { path: absPath })
    if (!exists) return
    // Tiny probe: base64 of empty file is "" — only delete empties, never real DBs.
    const b64 = await invoke<string>('read_file_base64_abs', { path: absPath })
    if (!b64) {
      await invoke('delete_file_abs', { path: absPath })
    }
  } catch { /* ignore */ }
}

/** Derives a display name from a project file path.
 *  - `.../<name>/project.sninja` or `project.db` → `<name>` (folder layout)
 *  - `.../<name>.sninja` or `.../<name>.db` → `<name>` (standalone file) */
export function projectNameFromDbPath(dbPath: string): string {
  const parts = dbPath.split(/[\\/]/).filter(Boolean)
  const file = parts[parts.length - 1] ?? dbPath
  const lower = file.toLowerCase()
  if (lower === PROJECT_BASENAME_SNINJA || lower === PROJECT_BASENAME_DB) {
    return parts.length >= 2 ? parts[parts.length - 2] : dbPath
  }
  return file.replace(/\.(sninja|db)$/i, '')
}
