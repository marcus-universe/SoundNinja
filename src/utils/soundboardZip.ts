import type { ProjectConfig, SoundFile } from '~/utils/db'

export interface ZipCopyEntry {
  src: string
  destRel: string
}

function sanitizeFolder(name: string): string {
  const cleaned = String(name || '')
    .replace(/[<>:"/\\|?*\x00-\x1f]/g, '_')
    .replace(/\.+$/g, '')
    .trim()
  return cleaned || 'Tab'
}

function basename(filePath: string): string {
  const norm = String(filePath || '').replace(/\\/g, '/')
  const i = norm.lastIndexOf('/')
  return i >= 0 ? norm.slice(i + 1) : norm
}

function extname(fileName: string): { stem: string; ext: string } {
  const m = fileName.match(/^(.*)(\.[^.]+)$/)
  if (!m) return { stem: fileName, ext: '' }
  return { stem: m[1] || fileName, ext: m[2] || '' }
}

export function primaryTabName(sound: SoundFile): string {
  const extra = (sound.tabs || []).filter((t) => t !== 'All')
  return extra[0] || 'All'
}

/** Build copy list + clone config with relative `sounds/<tab>/<file>` paths. */
export function buildPortableExport(config: ProjectConfig): {
  portable: ProjectConfig
  entries: ZipCopyEntry[]
} {
  const portable: ProjectConfig = JSON.parse(JSON.stringify(config))
  const used = new Set<string>()
  const entries: ZipCopyEntry[] = []

  for (const file of portable.files) {
    const folder = sanitizeFolder(primaryTabName(file))
    const rawName = basename(file.path) || `${file.name || 'sound'}.wav`
    const { stem, ext } = extname(rawName)
    let destRel = `sounds/${folder}/${rawName}`
    let n = 2
    while (used.has(destRel.toLowerCase())) {
      destRel = `sounds/${folder}/${stem}_${n}${ext}`
      n++
    }
    used.add(destRel.toLowerCase())
    if (file.path) entries.push({ src: file.path, destRel })
    file.path = destRel
  }

  return { portable, entries }
}

export function isAbsoluteSoundPath(path: string): boolean {
  const p = String(path || '')
  if (/^[a-zA-Z]:[\\/]/.test(p)) return true
  if (p.startsWith('\\\\')) return true
  if (p.startsWith('/')) return true
  return false
}

function joinOs(base: string, rel: string): string {
  const sep = base.includes('\\') ? '\\' : '/'
  const relNorm = rel.replace(/\\/g, '/').replace(/^\/+/, '')
  return `${base.replace(/[\\/]+$/, '')}${sep}${relNorm.split('/').join(sep)}`
}

/** Rewrite relative sound paths to absolute under `projectDir`. */
export function resolveImportedPaths(config: ProjectConfig, projectDir: string): void {
  for (const file of config.files) {
    if (!file.path || isAbsoluteSoundPath(file.path)) continue
    file.path = joinOs(projectDir, file.path)
  }
}

export function uniqueProjectFolderName(existingLower: Set<string>, base: string): string {
  const safe = base.replace(/[^a-z0-9_\- ]/gi, '_').replace(/\s+/g, '_') || 'Imported'
  if (!existingLower.has(safe.toLowerCase())) return safe
  let n = 2
  while (existingLower.has(`${safe}_${n}`.toLowerCase())) n++
  return `${safe}_${n}`
}
