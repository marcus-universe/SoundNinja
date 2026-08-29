// Per-sound / per-tab color override stored in the existing `color` TEXT column.
// Legacy plain hex → { border: hex }. Empty / '{}' → no override.

import { cssColorToHex, withAlpha } from '~/utils/themeTokens'

export type ColorOverride = {
  bg?: string
  bgHover?: string
  text?: string
  textHover?: string
  border?: string
  borderHover?: string
}

export type ColorTargetKind = 'button' | 'tab'

const HEX_RE = /^#[0-9a-f]{3,8}$/i
const OVERRIDE_KEYS: (keyof ColorOverride)[] = [
  'bg', 'bgHover', 'text', 'textHover', 'border', 'borderHover',
]

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
export function overrideSwatch(o: ColorOverride, fallback = '#00d4ff'): string {
  return o.border || o.bg || o.text || fallback
}

/** Paint cssColor over underlay → opaque #rrggbb (handles #rgba / color-mix). */
export function resolveOpaqueColor(cssColor: string, underlay = '#222831'): string {
  const c = (cssColor || '').trim()
  if (!c) return underlay.slice(0, 7)
  if (typeof document === 'undefined') {
    const hex = cssColorToHex(c)
    return /^#[0-9a-f]{6,8}$/i.test(hex) ? hex.slice(0, 7) : underlay.slice(0, 7)
  }
  try {
    const canvas = document.createElement('canvas')
    canvas.width = canvas.height = 1
    const ctx = canvas.getContext('2d')
    if (!ctx) return underlay.slice(0, 7)
    ctx.fillStyle = underlay
    ctx.fillRect(0, 0, 1, 1)
    ctx.fillStyle = c
    ctx.fillRect(0, 0, 1, 1)
    const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data
    return (
      '#' +
      [r, g, b].map((x) => x.toString(16).padStart(2, '0')).join('')
    )
  } catch {
    const hex = cssColorToHex(c)
    return /^#[0-9a-f]{6,8}$/i.test(hex) ? hex.slice(0, 7) : underlay.slice(0, 7)
  }
}

function pageBgHex(): string {
  if (typeof document === 'undefined') return '#222831'
  const raw = getComputedStyle(document.documentElement).getPropertyValue('--color-bg').trim()
  return resolveOpaqueColor(raw || '#222831', '#000000')
}

function readCssOpaque(varName: string, fallback: string, underlay?: string): string {
  if (typeof document === 'undefined') return fallback
  const raw = getComputedStyle(document.documentElement).getPropertyValue(varName).trim()
  if (!raw) return fallback
  return resolveOpaqueColor(raw, underlay || pageBgHex())
}

/** Currently applied sound-button theme colors (CSS vars on :root). */
export function themeButtonColors(): Required<ColorOverride> {
  const under = pageBgHex()
  return {
    bg: readCssOpaque('--color-btn', '#363f4d', under),
    bgHover: readCssOpaque('--btn-bg-hover', '#434e5f', under),
    text: readCssOpaque('--sound-text', '#eeeeee', under),
    textHover: readCssOpaque('--btn-text-hover', '#00d4ff', under),
    border: readCssOpaque('--btn-border', '#00d4ff', under),
    borderHover: readCssOpaque('--btn-border-hover', '#33ddff', under),
  }
}

/** Currently applied tab theme colors (CSS vars on :root). */
export function themeTabColors(): Required<ColorOverride> {
  const under = pageBgHex()
  return {
    bg: readCssOpaque('--tab-bg', '#00d4ff', under),
    bgHover: readCssOpaque('--tab-bg-hover', '#00d4ff', under),
    text: readCssOpaque('--tab-text', '#eeeeee', under),
    textHover: readCssOpaque('--tab-text-hover', '#eeeeee', under),
    border: readCssOpaque('--tab-border', '#00d4ff', under),
    borderHover: readCssOpaque('--tab-border-hover', '#33ddff', under),
  }
}

/**
 * Resolve the colors a user actually sees for a target.
 * Matches TabItem legacy path: border-only → tinted bg/hover from border.
 * Prefer live DOM computed vars when `el` is provided.
 */
export function resolveEffectiveColors(
  override: ColorOverride,
  kind: ColorTargetKind,
  el?: Element | null,
): Required<ColorOverride> {
  const under = pageBgHex()
  const base = kind === 'tab' ? themeTabColors() : themeButtonColors()
  const out: Required<ColorOverride> = { ...base }

  if (el && typeof getComputedStyle !== 'undefined') {
    const cs = getComputedStyle(el)
    if (kind === 'tab') {
      const bg = cs.getPropertyValue('--tab-bg').trim() || cs.backgroundColor
      const bgHover = cs.getPropertyValue('--tab-bg-hover').trim()
      const text = cs.getPropertyValue('--tab-text').trim() || cs.color
      const textHover = cs.getPropertyValue('--tab-text-hover').trim()
      const border = cs.getPropertyValue('--tab-border').trim() || cs.borderTopColor
      const borderHover = cs.getPropertyValue('--tab-border-hover').trim()
      if (bg) out.bg = resolveOpaqueColor(bg, under)
      if (bgHover) out.bgHover = resolveOpaqueColor(bgHover, under)
      if (text) out.text = resolveOpaqueColor(text, under)
      if (textHover) out.textHover = resolveOpaqueColor(textHover, under)
      if (border) out.border = resolveOpaqueColor(border, under)
      if (borderHover) out.borderHover = resolveOpaqueColor(borderHover, under)
    } else {
      const bg = cs.getPropertyValue('--color-btn').trim() || cs.backgroundColor
      const bgHover = cs.getPropertyValue('--btn-bg-hover').trim()
      const text = cs.getPropertyValue('--sound-text').trim() || cs.color
      const textHover = cs.getPropertyValue('--btn-text-hover').trim()
      const border = cs.getPropertyValue('--btn-border').trim() || cs.borderTopColor
      const borderHover = cs.getPropertyValue('--btn-border-hover').trim()
      if (bg) out.bg = resolveOpaqueColor(bg, under)
      if (bgHover) out.bgHover = resolveOpaqueColor(bgHover, under)
      if (text) out.text = resolveOpaqueColor(text, under)
      if (textHover) out.textHover = resolveOpaqueColor(textHover, under)
      if (border) out.border = resolveOpaqueColor(border, under)
      if (borderHover) out.borderHover = resolveOpaqueColor(borderHover, under)
    }
  } else if (kind === 'tab' && override.border && !override.bg && !override.text) {
    // Same derivation as TabItem.vue legacy single-hex path.
    out.border = resolveOpaqueColor(override.border, under)
    out.borderHover = resolveOpaqueColor(override.borderHover || override.border, under)
    out.bg = resolveOpaqueColor(withAlpha(override.border, 0.2), under)
    out.bgHover = resolveOpaqueColor(withAlpha(override.border, 0.4), under)
  }

  // Explicit override keys always win (opaque for the native color input).
  for (const key of OVERRIDE_KEYS) {
    const v = override[key]
    if (v) out[key] = resolveOpaqueColor(v, under)
  }

  return out
}
