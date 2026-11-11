/**
 * Helpers that let a project move between machines and operating systems.
 *
 * Sounds are stored with an absolute path. That path is meaningless on another
 * computer (`C:\Users\me\Music\Board\Lolli\a.wav` on a Steam Deck), so next to
 * it the project keeps the path relative to the project file. On open, sounds
 * whose absolute path is gone are looked up relative to wherever the project
 * file lives now.
 */

/** Split on both separators — a Windows path must parse on Linux and back. */
export function pathSegments(path: string): string[] {
  return String(path || '').split(/[\\/]+/).filter(Boolean)
}

export function osSeparator(base: string): string {
  return base.includes('\\') && !base.startsWith('/') ? '\\' : '/'
}

export function joinOsPath(base: string, ...parts: string[]): string {
  const sep = osSeparator(base)
  const tail = parts.flatMap((p) => pathSegments(p))
  return [base.replace(/[\\/]+$/, ''), ...tail].join(sep)
}

export function dirOf(filePath: string): string {
  return String(filePath || '').replace(/[\\/][^\\/]*$/, '')
}

function isWindowsPath(path: string): boolean {
  return /^[a-zA-Z]:[\\/]/.test(path) || path.startsWith('\\\\')
}

/**
 * `soundPath` relative to `projectDir` with `/` separators, or null when the
 * sound does not live inside the project folder.
 */
export function toRelativePath(soundPath: string, projectDir: string): string | null {
  if (!soundPath || !projectDir) return null
  const sound = pathSegments(soundPath)
  const dir = pathSegments(projectDir)
  if (sound.length <= dir.length) return null
  const fold = isWindowsPath(soundPath) || isWindowsPath(projectDir)
  for (let i = 0; i < dir.length; i++) {
    const a = fold ? sound[i]!.toLowerCase() : sound[i]
    const b = fold ? dir[i]!.toLowerCase() : dir[i]
    if (a !== b) return null
  }
  // A leading "/" on one side and a drive letter on the other never match above.
  if (soundPath.startsWith('/') !== projectDir.startsWith('/')) return null
  return sound.slice(dir.length).join('/')
}

/** Longest trailing part of the old path that is tried under the project dir. */
const MAX_SUFFIX_SEGMENTS = 8

/**
 * Places a missing sound may be found at, best guess first:
 * the stored relative path, then ever shorter tails of the old absolute path
 * (`…/Board/Lolli/a.wav` → `<project>/Board/Lolli/a.wav`, `<project>/Lolli/a.wav`,
 * `<project>/a.wav`).
 */
export function relinkCandidates(
  oldPath: string,
  relPath: string | null | undefined,
  projectDir: string,
): string[] {
  if (!projectDir) return []
  const out: string[] = []
  const add = (parts: string[]) => {
    if (!parts.length || parts.some((p) => p === '..' || p === '.')) return
    const candidate = joinOsPath(projectDir, ...parts)
    if (candidate !== oldPath && !out.includes(candidate)) out.push(candidate)
  }
  if (relPath) add(pathSegments(relPath))
  const segs = pathSegments(oldPath)
  // Skip a drive letter ("C:") — it is never a folder under the project.
  const usable = segs.length && /^[a-zA-Z]:$/.test(segs[0]!) ? segs.slice(1) : segs
  const max = Math.min(usable.length, MAX_SUFFIX_SEGMENTS)
  for (let k = max; k >= 1; k--) add(usable.slice(usable.length - k))
  return out
}

export interface RelinkRequest {
  path: string
  relPath?: string | null
}

/**
 * Resolve missing sounds against the project's current folder.
 * `exists` answers for a batch of absolute paths in one call. A found file is
 * never assigned twice and never to a path another sound already uses.
 */
export async function resolveMovedPaths(
  missing: RelinkRequest[],
  projectDir: string,
  taken: Iterable<string>,
  exists: (paths: string[]) => Promise<boolean[]>,
): Promise<Map<string, string>> {
  const out = new Map<string, string>()
  if (!missing.length || !projectDir) return out
  const perSound = missing.map((m) => relinkCandidates(m.path, m.relPath, projectDir))
  const all = [...new Set(perSound.flat())]
  if (!all.length) return out
  const flags = await exists(all)
  const found = new Set(all.filter((_, i) => flags[i]))
  const used = new Set(taken)
  missing.forEach((m, i) => {
    const hit = perSound[i]!.find((c) => found.has(c) && !used.has(c))
    if (!hit) return
    used.add(hit)
    out.set(m.path, hit)
  })
  return out
}
