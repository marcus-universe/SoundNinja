/** Hex ↔ HSL helpers for shifting a color set by hue while keeping S/L/alpha. */

const HEX_RE = /^#([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i

export type Hsla = { h: number; s: number; l: number; a: number }

function toHex2(n: number): string {
  return Math.max(0, Math.min(255, Math.round(n))).toString(16).padStart(2, '0')
}

export function parseHexColor(
  hex: string | undefined | null,
): { r: number; g: number; b: number; a: number } | null {
  if (!hex) return null
  let raw = hex.trim()
  if (!raw.startsWith('#')) raw = `#${raw}`
  if (!HEX_RE.test(raw)) return null
  let body = raw.slice(1)
  if (body.length === 3 || body.length === 4) {
    body = body
      .split('')
      .map((c) => c + c)
      .join('')
  }
  const r = parseInt(body.slice(0, 2), 16)
  const g = parseInt(body.slice(2, 4), 16)
  const b = parseInt(body.slice(4, 6), 16)
  const a = body.length >= 8 ? parseInt(body.slice(6, 8), 16) / 255 : 1
  return { r, g, b, a }
}

export function rgbToHsl(r: number, g: number, b: number): { h: number; s: number; l: number } {
  const rr = r / 255
  const gg = g / 255
  const bb = b / 255
  const max = Math.max(rr, gg, bb)
  const min = Math.min(rr, gg, bb)
  const l = (max + min) / 2
  if (max === min) return { h: 0, s: 0, l }
  const d = max - min
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min)
  let h = 0
  if (max === rr) h = ((gg - bb) / d + (gg < bb ? 6 : 0)) / 6
  else if (max === gg) h = ((bb - rr) / d + 2) / 6
  else h = ((rr - gg) / d + 4) / 6
  return { h: h * 360, s, l }
}

function hue2rgb(p: number, q: number, t: number): number {
  let tt = t
  if (tt < 0) tt += 1
  if (tt > 1) tt -= 1
  if (tt < 1 / 6) return p + (q - p) * 6 * tt
  if (tt < 1 / 2) return q
  if (tt < 2 / 3) return p + (q - p) * (2 / 3 - tt) * 6
  return p
}

export function hslToRgb(h: number, s: number, l: number): { r: number; g: number; b: number } {
  const hh = (((h % 360) + 360) % 360) / 360
  if (s <= 0) {
    const v = Math.round(l * 255)
    return { r: v, g: v, b: v }
  }
  const q = l < 0.5 ? l * (1 + s) : l + s - l * s
  const p = 2 * l - q
  return {
    r: Math.round(hue2rgb(p, q, hh + 1 / 3) * 255),
    g: Math.round(hue2rgb(p, q, hh) * 255),
    b: Math.round(hue2rgb(p, q, hh - 1 / 3) * 255),
  }
}

export function hexToHsl(hex: string): Hsla | null {
  const rgb = parseHexColor(hex)
  if (!rgb) return null
  return { ...rgbToHsl(rgb.r, rgb.g, rgb.b), a: rgb.a }
}

export function hslToHex(h: number, s: number, l: number, a = 1): string {
  const { r, g, b } = hslToRgb(h, s, l)
  const rgb = `#${toHex2(r)}${toHex2(g)}${toHex2(b)}`
  if (a >= 254.5 / 255) return rgb
  return `${rgb}${toHex2(a * 255)}`
}

/** Rotate hue. Grayscale (near-zero S) stays put. Alpha kept. */
export function shiftHexHue(hex: string, delta: number): string {
  const hsl = hexToHsl(hex)
  if (!hsl || hsl.s < 0.01) return hex
  return hslToHex(hsl.h + delta, hsl.s, hsl.l, hsl.a)
}

/** First chromatic color wins. Gray-only set → 0. */
export function leadHue(hexes: Array<string | undefined | null>): number {
  for (const hex of hexes) {
    const hsl = hexToHsl(hex || '')
    if (hsl && hsl.s > 0.08) return Math.round(hsl.h)
  }
  return 0
}

export function shiftColorRecord<T extends Record<string, string | undefined>>(
  colors: T,
  delta: number,
): T {
  const out = { ...colors }
  for (const key of Object.keys(out) as (keyof T)[]) {
    const v = out[key]
    if (typeof v === 'string' && v) out[key] = shiftHexHue(v, delta) as T[keyof T]
  }
  return out
}
