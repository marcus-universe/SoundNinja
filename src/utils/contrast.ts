// WCAG-based text-contrast helpers. Given the *solid* background color of an
// element (a tab accent or a button background), pick whichever of the two
// project text colors (light/dark) reads best on it.

/** Parses a #rgb / #rrggbb / #rrggbbaa hex string into [r, g, b] (0–255). */
export function hexToRgb(hex: string): [number, number, number] {
  let h = (hex || '').replace('#', '').trim()
  if (h.length === 3) {
    h = h
      .split('')
      .map((c) => c + c)
      .join('')
  }
  if (h.length < 6) return [0, 0, 0]
  const n = parseInt(h.slice(0, 6), 16)
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255]
}

function channelLuminance(c: number): number {
  const s = c / 255
  return s <= 0.03928 ? s / 12.92 : Math.pow((s + 0.055) / 1.055, 2.4)
}

/** Relative luminance per WCAG 2.1 (0 = black, 1 = white). */
export function relativeLuminance(hex: string): number {
  const [r, g, b] = hexToRgb(hex)
  return 0.2126 * channelLuminance(r) + 0.7152 * channelLuminance(g) + 0.0722 * channelLuminance(b)
}

function contrastRatio(a: number, b: number): number {
  return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05)
}

/**
 * Returns whichever of `lightHex` / `darkHex` has the higher contrast ratio
 * against the given solid background color.
 */
export function readableTextColor(
  bgHex: string,
  lightHex = '#eeeeee',
  darkHex = '#222831'
): string {
  try {
    const bg = relativeLuminance(bgHex)
    const cLight = contrastRatio(bg, relativeLuminance(lightHex))
    const cDark = contrastRatio(bg, relativeLuminance(darkHex))
    return cLight >= cDark ? lightHex : darkHex
  } catch {
    return darkHex
  }
}
