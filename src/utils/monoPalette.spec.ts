import { describe, expect, it } from 'vitest'
import { contrastRatioHex, relativeLuminance } from '~/utils/contrast'
import { hexToHsl } from '~/utils/hue'
import {
  applyBasicPick,
  deriveMonoSet,
  inferScheme,
  WCAG_AA,
} from '~/utils/monoPalette'

const SATURATED_RED = '#e53935'

function hueDelta(a: string, b: string): number {
  const ha = hexToHsl(a)!.h
  const hb = hexToHsl(b)!.h
  return Math.min(Math.abs(ha - hb), 360 - Math.abs(ha - hb))
}

describe('deriveMonoSet', () => {
  it('dark scheme → darker bg with AA contrast', () => {
    const set = deriveMonoSet(SATURATED_RED, 'dark')
    expect(relativeLuminance(set.bg)).toBeLessThan(relativeLuminance(set.text))
    expect(contrastRatioHex(set.bg, set.text)).toBeGreaterThanOrEqual(WCAG_AA)
  })

  it('light scheme → lighter bg with AA contrast', () => {
    const set = deriveMonoSet(SATURATED_RED, 'light')
    expect(relativeLuminance(set.bg)).toBeGreaterThan(relativeLuminance(set.text))
    expect(contrastRatioHex(set.bg, set.text)).toBeGreaterThanOrEqual(WCAG_AA)
  })

  it('keeps hue across bg / text / border', () => {
    const set = deriveMonoSet(SATURATED_RED, 'dark')
    for (const hex of [set.bg, set.text, set.border]) {
      expect(hueDelta(hex, SATURATED_RED)).toBeLessThan(2)
    }
  })

  it('keeps pick as text and border', () => {
    const set = deriveMonoSet(SATURATED_RED, 'dark')
    expect(set.text.toLowerCase()).toBe(SATURATED_RED)
    expect(set.border.toLowerCase()).toBe(SATURATED_RED)
  })
})

describe('inferScheme', () => {
  it('reads dark from bg darker than text', () => {
    const set = deriveMonoSet(SATURATED_RED, 'dark')
    expect(inferScheme({ text: set.text, bg: set.bg })).toBe('dark')
  })

  it('reads light from bg lighter than text', () => {
    const set = deriveMonoSet(SATURATED_RED, 'light')
    expect(inferScheme({ text: set.text, bg: set.bg })).toBe('light')
  })

  it('defaults empty override to dark', () => {
    expect(inferScheme({})).toBe('dark')
  })
})

describe('applyBasicPick', () => {
  it('fills idle and hover from one pick', () => {
    const next = applyBasicPick({}, SATURATED_RED, 'dark')
    expect(next.text?.toLowerCase()).toBe(SATURATED_RED)
    expect(next.textHover?.toLowerCase()).toBe(SATURATED_RED)
    expect(next.bg).toBeTruthy()
    expect(next.bgHover).toBeTruthy()
    expect(next.border?.toLowerCase()).toBe(SATURATED_RED)
    expect(next.borderHover?.toLowerCase()).toBe(SATURATED_RED)
    expect(next.bgHover).not.toBe(next.bg)
  })
})
