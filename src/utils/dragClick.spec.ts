import { describe, expect, it } from 'vitest'
import { wasClickLikeDrag } from '~/utils/dragClick'

describe('wasClickLikeDrag', () => {
  it('treats a drag that barely moved as a click, however long it was held', () => {
    expect(wasClickLikeDrag(12, 80)).toBe(true)
    expect(wasClickLikeDrag(24, 2000)).toBe(true)
  })

  it('allows a quick press to wander a little further', () => {
    expect(wasClickLikeDrag(50, 200)).toBe(true)
  })

  it('keeps a real drag a drag', () => {
    expect(wasClickLikeDrag(50, 600)).toBe(false)
    expect(wasClickLikeDrag(140, 120)).toBe(false)
  })
})
