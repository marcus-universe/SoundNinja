/** Shared waveform paint helpers — cache peak bars, draw playhead cheaply. */

const MUTED = 'rgba(255,255,255,0.22)'
const HOVER = 'rgba(255,255,255,0.45)'

let cachedAccent = '#00d4ff'
let accentAt = 0

export function readAccentColor(): string {
  const now = Date.now()
  if (now - accentAt < 2000) return cachedAccent
  accentAt = now
  cachedAccent =
    getComputedStyle(document.documentElement).getPropertyValue('--primary_color').trim()
    || '#00d4ff'
  return cachedAccent
}

/** Offscreen canvas holding the static peak bars for a given size + data. */
export class PeakLayer {
  canvas: HTMLCanvasElement | null = null
  private cssW = 0
  private cssH = 0
  private dpr = 1
  private dataRef: number[] | null = null

  invalidate() {
    this.canvas = null
    this.dataRef = null
  }

  ensure(cssW: number, cssH: number, dpr: number, data: number[]): HTMLCanvasElement {
    if (
      this.canvas
      && this.cssW === cssW
      && this.cssH === cssH
      && this.dpr === dpr
      && this.dataRef === data
    ) {
      return this.canvas
    }
    const c = document.createElement('canvas')
    c.width = Math.max(1, Math.floor(cssW * dpr))
    c.height = Math.max(1, Math.floor(cssH * dpr))
    const ctx = c.getContext('2d')
    if (ctx) {
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
      paintPeaks(ctx, cssW, cssH, data)
    }
    this.canvas = c
    this.cssW = cssW
    this.cssH = cssH
    this.dpr = dpr
    this.dataRef = data
    return c
  }
}

export function paintPeaks(
  ctx: CanvasRenderingContext2D,
  cssW: number,
  cssH: number,
  data: number[],
) {
  ctx.clearRect(0, 0, cssW, cssH)
  const mid = cssH / 2
  const pairs = Math.floor(data.length / 2)
  if (pairs <= 0) {
    ctx.strokeStyle = MUTED
    ctx.beginPath()
    ctx.moveTo(0, mid)
    ctx.lineTo(cssW, mid)
    ctx.stroke()
    return
  }
  const step = cssW / pairs
  ctx.fillStyle = MUTED
  for (let i = 0; i < pairs; i++) {
    const minV = data[i * 2]
    const maxV = data[i * 2 + 1]
    const y1 = mid + minV * mid
    const y2 = mid + maxV * mid
    ctx.fillRect(i * step, y1, Math.max(1, step * 0.85), Math.max(1, y2 - y1))
  }
}

export function blitPeaks(
  ctx: CanvasRenderingContext2D,
  layer: HTMLCanvasElement,
  cssW: number,
  cssH: number,
) {
  ctx.clearRect(0, 0, cssW, cssH)
  ctx.drawImage(layer, 0, 0, cssW, cssH)
}

export function drawPlayhead(
  ctx: CanvasRenderingContext2D,
  cssW: number,
  cssH: number,
  duration: number,
  playhead: number,
  hover: number | null,
) {
  if (duration <= 0) return
  if (hover != null) {
    const hx = (hover / duration) * cssW
    ctx.strokeStyle = HOVER
    ctx.lineWidth = 1
    ctx.setLineDash([3, 3])
    ctx.beginPath()
    ctx.moveTo(hx, 0)
    ctx.lineTo(hx, cssH)
    ctx.stroke()
    ctx.setLineDash([])
  }
  const x = (playhead / duration) * cssW
  ctx.strokeStyle = readAccentColor()
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(x, 0)
  ctx.lineTo(x, cssH)
  ctx.stroke()
}

/** ~20fps rAF clock that stops when document is hidden. */
export function startThrottledClock(
  fps: number,
  tick: () => boolean,
): { stop: () => void } {
  const minMs = 1000 / Math.max(1, fps)
  let raf: number | null = null
  let last = 0
  let stopped = false

  const loop = (now: number) => {
    if (stopped) return
    if (document.hidden) {
      raf = requestAnimationFrame(loop)
      return
    }
    if (now - last >= minMs) {
      last = now
      if (!tick()) {
        stopped = true
        raf = null
        return
      }
    }
    raf = requestAnimationFrame(loop)
  }
  raf = requestAnimationFrame(loop)
  return {
    stop() {
      stopped = true
      if (raf != null) cancelAnimationFrame(raf)
      raf = null
    },
  }
}
