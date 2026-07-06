import { invoke } from '@tauri-apps/api/core'

function joinPath(base: string, ...parts: string[]) {
  const sep = base.includes('\\') ? '\\' : '/'
  return [base.replace(/[\\/]+$/, ''), ...parts].join(sep)
}

/** Registers a single font file (ttf/otf) as a runtime `@font-face`. */
export async function registerFontFace(family: string, absPath: string): Promise<void> {
  const ext = absPath.split('.').pop()?.toLowerCase()
  const mime = ext === 'otf' ? 'font/otf' : 'font/ttf'
  const b64 = await invoke<string>('read_file_base64_abs', { path: absPath })
  const face = new FontFace(family, `url(data:${mime};base64,${b64})`)
  await face.load()
  ;(document as unknown as { fonts: FontFaceSet }).fonts.add(face)
}

/**
 * Loads every uploaded font from the `fonts` folder and registers it.
 * Returns the list of registered font-family names (file name without extension).
 */
export async function loadCustomFonts(fontsPath: string): Promise<string[]> {
  if (!fontsPath) return []
  let files: string[] = []
  try {
    files = await invoke<string[]>('list_dir_files_abs', { dir: fontsPath, exts: ['ttf', 'otf'] })
  } catch {
    return []
  }
  const families: string[] = []
  for (const f of files) {
    const family = f.replace(/\.(ttf|otf)$/i, '')
    try {
      await registerFontFace(family, joinPath(fontsPath, f))
      families.push(family)
    } catch (e) {
      console.warn('Failed to register font', f, e)
    }
  }
  return families
}
