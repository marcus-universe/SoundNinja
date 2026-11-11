/**
 * Telling a click from a drag when the pointer is not steady.
 *
 * A VR controller or a touchpad moves a few pixels between press and release.
 * With drag-to-reorder on, SortableJS takes that movement for a drag, and then
 * swallows the click that would have played the sound or switched the tab.
 * Two measures: a larger dead zone before a drag starts, and treating a drag
 * that ended where it began after very little movement as the click it was.
 */

/** Pointer travel (px) before a press becomes a drag. SortableJS `fallbackTolerance`. */
export const DRAG_TOLERANCE_PX = 10

/** A drag that never left this radius was a click, however long it was held. */
const CLICK_TRAVEL_PX = 24
/** A short press may wander a little further and still be a click. */
const QUICK_PRESS_MS = 250
const QUICK_PRESS_TRAVEL_PX = 60

/** Did a drag that ended in place look like a click? */
export function wasClickLikeDrag(travelPx: number, heldMs: number): boolean {
  if (travelPx <= CLICK_TRAVEL_PX) return true
  return heldMs <= QUICK_PRESS_MS && travelPx <= QUICK_PRESS_TRAVEL_PX
}

export interface PressTracker {
  /** Start listening on `root`. Returns the stop function. */
  attach(root: HTMLElement): () => void
  /** Largest distance from the press point so far, in px. */
  travel(): number
  /** How long the current / last press has been held, in ms. */
  heldMs(): number
  /** Whether the last press, judged at release, was click-like. */
  wasClick(): boolean
}

/**
 * Follows the primary pointer from press to release. The numbers stay
 * readable after release, so a SortableJS `onEnd` handler can ask about the
 * press that just ended.
 */
export function createPressTracker(): PressTracker {
  let startX = 0
  let startY = 0
  let startAt = 0
  let endAt = 0
  let maxTravel = 0
  let pressed = false

  const onMove = (e: PointerEvent) => {
    if (!pressed) return
    const d = Math.hypot(e.clientX - startX, e.clientY - startY)
    if (d > maxTravel) maxTravel = d
  }
  const onUp = (e: PointerEvent) => {
    if (!pressed) return
    onMove(e)
    pressed = false
    endAt = performance.now()
    window.removeEventListener('pointermove', onMove, true)
    window.removeEventListener('pointerup', onUp, true)
    window.removeEventListener('pointercancel', onUp, true)
  }
  const onDown = (e: PointerEvent) => {
    if (e.button !== 0 || pressed) return
    pressed = true
    startX = e.clientX
    startY = e.clientY
    startAt = performance.now()
    endAt = 0
    maxTravel = 0
    // Capture phase on window: SortableJS stops some of these events.
    window.addEventListener('pointermove', onMove, true)
    window.addEventListener('pointerup', onUp, true)
    window.addEventListener('pointercancel', onUp, true)
  }

  return {
    attach(root) {
      root.addEventListener('pointerdown', onDown, true)
      return () => {
        root.removeEventListener('pointerdown', onDown, true)
        window.removeEventListener('pointermove', onMove, true)
        window.removeEventListener('pointerup', onUp, true)
        window.removeEventListener('pointercancel', onUp, true)
        pressed = false
      }
    },
    travel: () => maxTravel,
    heldMs: () => (pressed || !endAt ? performance.now() : endAt) - startAt,
    wasClick() {
      return wasClickLikeDrag(maxTravel, this.heldMs())
    },
  }
}
