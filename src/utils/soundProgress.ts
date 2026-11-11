/**
 * Playback progress of one sound, as shown on its button.
 *
 * The position is not stored; it is derived from an anchor: the wall-clock
 * time at which the sound would have been at position 0. A button can then
 * compute where the sweep is at any moment — after a re-render, a tab switch
 * or a remount — without restarting it.
 */
export interface SoundProgress {
  /** Length in seconds. */
  duration: number
  /** `Date.now()` value at which the position was 0. */
  anchorMs: number
  paused: boolean
  /** Position while paused, in ms. */
  pausedAtMs: number
  looping: boolean
}

/** One entry of the backend's `playing_changed` list. */
export interface PlayingSnapshot {
  path: string
  paused?: boolean
  positionSecs?: number
  looping?: boolean
  durationSecs?: number
  /** `durationSecs` is the decoded length, not header metadata or an estimate. */
  durationExact?: boolean
  /** Epoch ms at which `positionSecs` was sampled. */
  asOfMs?: number
}

/** An anchor closer than this to the running one is not worth a visible re-seek. */
export const RESYNC_TOLERANCE_MS = 40
/** A backend stamp further than this from the local clock is another machine's clock. */
const MAX_CLOCK_SKEW_MS = 2000

function positive(n: unknown): number {
  const v = Number(n)
  return Number.isFinite(v) && v > 0 ? v : 0
}

/** Current position in ms, inside `[0, duration]`. */
export function progressPositionMs(info: SoundProgress, now: number): number {
  const durMs = info.duration * 1000
  if (!(durMs > 0)) return 0
  let pos = info.paused ? info.pausedAtMs : now - info.anchorMs
  if (pos < 0) pos = 0
  if (info.looping) return pos % durMs
  return Math.min(durMs, pos)
}

/** Progress for a sound whose playback was just requested. */
export function startProgress(duration: number, now: number, looping = false): SoundProgress {
  return { duration, anchorMs: now, paused: false, pausedAtMs: 0, looping }
}

/**
 * Length to show for a snapshot. An exact backend value wins, even when it is
 * shorter than what the project remembers. Until then the larger of the two is
 * used, because a file still being decoded reports only what it has so far.
 */
export function resolveDuration(snap: PlayingSnapshot, remembered: number, current = 0): number {
  const fromSnap = positive(snap.durationSecs)
  if (snap.durationExact && fromSnap > 0) return fromSnap
  return Math.max(fromSnap, positive(remembered), positive(current))
}

/**
 * Fold a backend snapshot into the running progress.
 *
 * Returns the same object when nothing visible changed, so a button watching
 * it is not re-synced by every event.
 *
 * `trustStamp` is false in the browser mirror: its clock is not the clock that
 * wrote `asOfMs`.
 */
export function reconcileProgress(
  current: SoundProgress | undefined,
  snap: PlayingSnapshot,
  remembered: number,
  now: number,
  trustStamp = true,
): SoundProgress | undefined {
  const duration = resolveDuration(snap, remembered, current?.duration)
  if (!(duration > 0)) return current
  const paused = !!snap.paused
  const looping = !!snap.looping
  const posMs = Math.max(0, positive(snap.positionSecs) * 1000)
  const stamp = Number(snap.asOfMs)
  const sampledAt = trustStamp && Number.isFinite(stamp) && Math.abs(now - stamp) <= MAX_CLOCK_SKEW_MS
    ? stamp
    : now
  const next: SoundProgress = {
    duration,
    anchorMs: sampledAt - posMs,
    paused,
    pausedAtMs: paused ? posMs : 0,
    looping,
  }
  if (!current) return next
  const same = current.duration === next.duration
    && current.paused === next.paused
    && current.looping === next.looping
    && (paused
      ? Math.abs(current.pausedAtMs - next.pausedAtMs) < RESYNC_TOLERANCE_MS
      : Math.abs(current.anchorMs - next.anchorMs) < RESYNC_TOLERANCE_MS)
  return same ? current : next
}
