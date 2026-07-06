import { invoke } from '@tauri-apps/api/core'

export interface ProjectInfo {
  name: string
  dbPath: string
}

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

/** Absolute path of a project's DB file: <projectsPath>/<name>/project.db */
export function projectDbPath(projectsPath: string, name: string): string {
  return join(projectsPath, safeProjectName(name), 'project.db')
}

export async function listProjects(projectsPath: string): Promise<ProjectInfo[]> {
  return invoke<ProjectInfo[]>('list_projects', { projectsPath })
}

/** Creates a project folder (the DB itself is created on first open). */
export async function createProjectFolder(projectsPath: string, name: string): Promise<string> {
  const folder = join(projectsPath, safeProjectName(name))
  await invoke('make_dir_abs', { path: folder })
  return join(folder, 'project.db')
}

/** Derives a display name from a project DB path. */
export function projectNameFromDbPath(dbPath: string): string {
  const parts = dbPath.split(/[\\/]/).filter(Boolean)
  // .../<name>/project.db  → <name>
  return parts.length >= 2 ? parts[parts.length - 2] : dbPath
}
