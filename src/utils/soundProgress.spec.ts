import { describe, expect, it } from 'vitest'
import {
  progressPositionMs,
  reconcileProgress,
  resolveDuration,
  startProgress,
} from '~/utils/soundProgress'

const NOW = 1_700_000_000_000

describe('progressPositionMs', () => {
  it('advances with the clock from the anchor', () => {
    const info = startProgress(10, NOW)
    expect(progressPositionMs(info, NOW + 2500)).toBe(2500)
  })

  it('stops at the end of a sound that does not loop', () => {
    const info = startProgress(2, NOW)
    expect(progressPositionMs(info, NOW + 9000)).toBe(2000)
  })

  it('wraps around for a looping sound', () => {
    const info = startProgress(2, NOW, true)
    expect(progressPositionMs(info, NOW + 4500)).toBe(500)
  })

  it('holds its position while paused', () => {
    const info = { ...startProgress(10, NOW), paused: true, pausedAtMs: 3000 }
    expect(progressPositionMs(info, NOW + 60_000)).toBe(3000)
  })

  it('is zero for an unknown duration', () => {
    expect(progressPositionMs(startProgress(0, NOW), NOW + 500)).toBe(0)
  })
})

describe('resolveDuration', () => {
  it('takes an exact backend value even when it is shorter than remembered', () => {
    expect(resolveDuration({ path: 'a', durationSecs: 3, durationExact: true }, 5, 5)).toBe(3)
  })

  it('keeps the longest value while the backend is still decoding', () => {
    expect(resolveDuration({ path: 'a', durationSecs: 1.2 }, 5, 0)).toBe(5)
    expect(resolveDuration({ path: 'a', durationSecs: 6 }, 5, 0)).toBe(6)
  })
})

describe('reconcileProgress', () => {
  it('anchors on the moment the backend sampled the position, not on arrival', () => {
    // Sampled 120 ms ago at 1.0 s: the sound is at 1.12 s now.
    const info = reconcileProgress(undefined, { path: 'a', positionSecs: 1, durationSecs: 10, asOfMs: NOW - 120 }, 0, NOW)!
    expect(progressPositionMs(info, NOW)).toBe(1120)
  })

  it('falls back to arrival time without a stamp or with a foreign clock', () => {
    const noStamp = reconcileProgress(undefined, { path: 'a', positionSecs: 1, durationSecs: 10 }, 0, NOW)!
    expect(progressPositionMs(noStamp, NOW)).toBe(1000)
    const skewed = reconcileProgress(undefined, { path: 'a', positionSecs: 1, durationSecs: 10, asOfMs: NOW - 3_600_000 }, 0, NOW)!
    expect(progressPositionMs(skewed, NOW)).toBe(1000)
    const mirror = reconcileProgress(undefined, { path: 'a', positionSecs: 1, durationSecs: 10, asOfMs: NOW - 120 }, 0, NOW, false)!
    expect(progressPositionMs(mirror, NOW)).toBe(1000)
  })

  it('returns the same object when the event changes nothing visible', () => {
    const running = startProgress(10, NOW)
    const again = reconcileProgress(running, { path: 'a', positionSecs: 2.01, durationSecs: 10, asOfMs: NOW + 2000 }, 0, NOW + 2000)
    expect(again).toBe(running)
  })

  it('re-seeks when the backend position differs noticeably', () => {
    const running = startProgress(10, NOW)
    const moved = reconcileProgress(running, { path: 'a', positionSecs: 6, durationSecs: 10, asOfMs: NOW + 2000 }, 0, NOW + 2000)!
    expect(moved).not.toBe(running)
    expect(progressPositionMs(moved, NOW + 2000)).toBe(6000)
  })

  it('freezes on pause and resumes from the paused position', () => {
    const running = startProgress(10, NOW)
    const paused = reconcileProgress(running, { path: 'a', positionSecs: 3, durationSecs: 10, paused: true }, 0, NOW + 3000)!
    expect(paused.paused).toBe(true)
    expect(progressPositionMs(paused, NOW + 30_000)).toBe(3000)
    const resumed = reconcileProgress(paused, { path: 'a', positionSecs: 3, durationSecs: 10 }, 0, NOW + 30_000)!
    expect(progressPositionMs(resumed, NOW + 31_000)).toBe(4000)
  })

  it('leaves progress alone while no duration is known', () => {
    expect(reconcileProgress(undefined, { path: 'a', positionSecs: 1 }, 0, NOW)).toBeUndefined()
  })
})
