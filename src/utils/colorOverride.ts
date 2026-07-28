// Per-sound / per-tab color override stored in the existing `color` TEXT column.
// Legacy plain hex → { border: hex }. Empty / '{}' → no override.

export type ColorOverride = {
  bg?: string
  bgHover?: string
  text?: string
  textHover?: string
  border?: string
  borderHover?: string
}

const HEX_RE = /^#[0-9a-f]{3,8}$/i

export function isEmptyOverride(o: ColorOverride | null | undefined): boolean {
  if (!o) return true
  return !o.bg && !o.bgHover && !o.text && !o.textHover && !o.border && !o.borderHover
}

/** Parse stored color column: JSON object, legacy #hex, or empty. */
export function parseOverride(raw: string | undefined | null): ColorOverride {
  if (!raw || !String(raw).trim()) return {}
  const s = String(raw).trim()
  if (HEX_RE.test(s)) return { border: s }
  try {
    const obj = JSON.parse(s)
    if (!obj || typeof obj !== 'object' || Array.isArray(obj)) return {}
    const out: ColorOverride = {}
    if (typeof obj.bg === 'string' && HEX_RE.test(obj.bg)) out.bg = obj.bg
    if (typeof obj.bgHover === 'string' && HEX_RE.test(obj.bgHover)) out.bgHover = obj.bgHover
    if (typeof obj.text === 'string' && HEX_RE.test(obj.text)) out.text = obj.text
    if (typeof obj.textHover === 'string' && HEX_RE.test(obj.textHover)) out.textHover = obj.textHover
    if (typeof obj.border === 'string' && HEX_RE.test(obj.border)) out.border = obj.border
    if (typeof obj.borderHover === 'string' && HEX_RE.test(obj.borderHover)) out.borderHover = obj.borderHover
    return out
  } catch {
    return {}
  }
}

/** Serialize override for DB. Empty → ''. */
export function serializeOverride(o: ColorOverride | null | undefined): string {
  if (isEmptyOverride(o)) return ''
  const clean: ColorOverride = {}
  if (o!.bg) clean.bg = o!.bg
  if (o!.bgHover) clean.bgHover = o!.bgHover
  if (o!.text) clean.text = o!.text
  if (o!.textHover) clean.textHover = o!.textHover
  if (o!.border) clean.border = o!.border
  if (o!.borderHover) clean.borderHover = o!.borderHover
  return JSON.stringify(clean)
}

/** Swatch color for UI previews (prefer border, then bg, then text). */
export function overrideSwatch(o: ColorOverride): string {
  return o.border || o.bg || o.text || '#00d4ff'
}
