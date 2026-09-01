const ID_CHARS = 'abcdefghijklmnopqrstuvwxyz0123456789'
const ID_LEN = 8

/** Short unique sound id: 8 chars `[a-z0-9]`. */
export function newSoundId(existing: Iterable<string> = []): string {
  const used = new Set(existing)
  for (let attempt = 0; attempt < 64; attempt++) {
    const buf = new Uint8Array(ID_LEN)
    crypto.getRandomValues(buf)
    let id = ''
    for (let i = 0; i < ID_LEN; i++) id += ID_CHARS[buf[i]! % ID_CHARS.length]
    if (!used.has(id)) return id
  }
  return `${Date.now().toString(36).slice(-ID_LEN).padStart(ID_LEN, '0')}`
}

export function isSoundId(value: unknown): value is string {
  return typeof value === 'string' && /^[a-z0-9]{8}$/.test(value)
}

/** Assigns missing ids. Returns true when any file changed. */
export function ensureSoundIds(files: { id?: string }[]): boolean {
  const used = new Set<string>()
  for (const f of files) {
    if (f.id) used.add(f.id)
  }
  let changed = false
  for (const f of files) {
    if (!f.id) {
      const id = newSoundId(used)
      f.id = id
      used.add(id)
      changed = true
    }
  }
  return changed
}
