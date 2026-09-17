// Paletton-style monochromatic trio from one ink pick.
// Same hue + saturation; lightness follows explicit dark/light scheme until WCAG AA.

import type { ColorOverride } from '~/utils/colorOverride'
import { contrastRatioHex, relativeLuminance } from '~/utils/contrast'
import { hexToHsl, hslToHex } from '~/utils/hue'

export const WCAG_AA = 4.5

export type ColorScheme = 'dark' | 'light'

export type MonoSet = {
  bg: string
  text: string
  border: string
}

function clamp01(n: number): number {
  return Math.min(1, Math.max(0, n))
}

function hex6(hex: string): string {
  const h = (hex || '').trim()
  if (/^#[0-9a-f]{8}$/i.test(h)) return h.slice(0, 7)
  if (/^#[0-9a-f]{6}$/i.test(h)) return h
  if (/^#[0-9a-f]{3}$/i.test(h)) {
    return `#${h[1]}${h[1]}${h[2]}${h[2]}${h[3]}${h[3]}`
  }
  return h
}

function nudgeText(h: number, s: number, startL: number, bg: string, darken: boolean): string {
  let l = startL
  let text = hslToHex(h, s, l)
  for (let i = 0; i < 48; i++) {
    if (contrastRatioHex(bg, text) >= WCAG_AA) return text.slice(0, 7)
    l += darken ? -0.02 : 0.02
    if (l <= 0.04 || l >= 0.96) {
      text = hslToHex(h, s, clamp01(l))
      break
    }
    text = hslToHex(h, s, l)
  }
  return text.slice(0, 7)
}

function nudgeBg(h: number, s: number, text: string, startL: number, darken: boolean): string {
  let l = startL
  let sat = s
  let bg = hslToHex(h, sat, l)
  for (let i = 0; i < 48; i++) {
    if (contrastRatioHex(bg, text) >= WCAG_AA) return bg.slice(0, 7)
    l += darken ? -0.02 : 0.02
    if (l <= 0.02 || l >= 0.98) {
      l = clamp01(l)
      bg = hslToHex(h, sat, l)
      break
    }
    bg = hslToHex(h, sat, l)
  }
  for (let i = 0; i < 40; i++) {
    if (contrastRatioHex(bg, text) >= WCAG_AA) return bg.slice(0, 7)
    sat = Math.max(0, sat - 0.04)
    bg = hslToHex(h, sat, l)
    if (sat <= 0) break
  }
  return bg.slice(0, 7)
}

/** Infer scheme from stored trio. Empty → dark (app default). */
export function inferScheme(o: ColorOverride | null | undefined): ColorScheme {
  if (o?.text && o?.bg) {
    return relativeLuminance(o.bg) < relativeLuminance(o.text) ? 'dark' : 'light'
  }
  if (o?.textHover && o?.bgHover) {
    return relativeLuminance(o.bgHover) < relativeLuminance(o.textHover) ? 'dark' : 'light'
  }
  return 'dark'
}

/** User pick is text (ink). BG moves dark or light. If AA still fails (bright ink on light BG), text L moves opposite. */
export function deriveMonoSet(pick: string, scheme: ColorScheme = 'dark'): MonoSet {
  const ink = hex6(pick)
  const hsl = hexToHsl(ink)
  if (!hsl) return { bg: ink, text: ink, border: ink }
  const dark = scheme === 'dark'
  const startL = dark
    ? Math.min(hsl.l - 0.35, 0.32)
    : Math.max(hsl.l + 0.32, 0.68)
  const bg = nudgeBg(hsl.h, hsl.s, ink, clamp01(startL), dark)
  let text = ink
  if (contrastRatioHex(bg, text) < WCAG_AA) {
    text = nudgeText(hsl.h, hsl.s, hsl.l, bg, !dark)
  }
  return { bg, text, border: ink }
}

/** Idle trio + hover BG lifted toward text so hover stays visible. */
export function deriveBasicOverride(pick: string, scheme: ColorScheme = 'dark'): ColorOverride {
  const idle = deriveMonoSet(pick, scheme)
  const bgHsl = hexToHsl(idle.bg)
  const textHsl = hexToHsl(idle.text)
  let bgHover = idle.bg
  if (bgHsl && textHsl) {
    const hoverL = clamp01(bgHsl.l + (textHsl.l - bgHsl.l) * 0.22)
    bgHover = hslToHex(bgHsl.h, bgHsl.s, hoverL).slice(0, 7)
  }
  return {
    text: idle.text,
    bg: idle.bg,
    border: idle.border,
    textHover: idle.text,
    bgHover,
    borderHover: idle.border,
  }
}

/** Fill all 6 keys from one ink pick. */
export function applyBasicPick(
  current: ColorOverride,
  hex: string,
  scheme: ColorScheme = 'dark',
): ColorOverride {
  return { ...current, ...deriveBasicOverride(hex, scheme) }
}

/** Re-derive the full set from one ink under a scheme. */
export function applyScheme(
  current: ColorOverride,
  scheme: ColorScheme,
  ink: string,
): ColorOverride {
  return applyBasicPick(current, ink, scheme)
}
