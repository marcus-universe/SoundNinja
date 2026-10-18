import { isRemote } from '~/utils/runtime'

/** Long-press opens the context menu on a phone. Tap still plays. */
export function useRemoteLongPress() {
  let timer = 0
  let sx = 0
  let sy = 0
  let suppress = false

  function start(e: PointerEvent, fire: (x: number, y: number) => void) {
    if (!isRemote) return
    if (e.button != null && e.button !== 0) return
    suppress = false
    sx = e.clientX
    sy = e.clientY
    window.clearTimeout(timer)
    timer = window.setTimeout(() => {
      suppress = true
      try { navigator.vibrate?.(10) } catch { /* unsupported */ }
      fire(sx, sy)
    }, 450)
  }

  function move(e: PointerEvent) {
    if (!timer) return
    if (Math.hypot(e.clientX - sx, e.clientY - sy) > 12) {
      window.clearTimeout(timer)
      timer = 0
    }
  }

  function end() {
    window.clearTimeout(timer)
    timer = 0
  }

  /** True when the following click came from a long-press and must not play. */
  function eaten(e?: Event) {
    if (!suppress) return false
    suppress = false
    e?.preventDefault?.()
    e?.stopPropagation?.()
    return true
  }

  return { start, move, end, eaten }
}
