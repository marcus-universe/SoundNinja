import { describe, expect, it } from 'vitest'
import { applySaveProgress, initialSaveProgress } from '~/utils/saveProgress'

describe('applySaveProgress', () => {
  it('follows the backend phase and percent', () => {
    const next = applySaveProgress(initialSaveProgress(), { phase: 'audio', percent: 41.6 })
    expect(next).toEqual({ phase: 'audio', percent: 42 })
  })

  it('never moves backwards when events arrive out of order', () => {
    const at = { phase: 'audio' as const, percent: 60 }
    expect(applySaveProgress(at, { phase: 'board', percent: 20 })).toEqual(at)
  })

  it('keeps the label for an unknown phase and clamps the percent', () => {
    const at = { phase: 'board' as const, percent: 10 }
    expect(applySaveProgress(at, { phase: 'defrag', percent: 250 })).toEqual({ phase: 'board', percent: 100 })
  })

  it('ignores an empty event', () => {
    const at = initialSaveProgress()
    expect(applySaveProgress(at, null)).toBe(at)
  })
})
