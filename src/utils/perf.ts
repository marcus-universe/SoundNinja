import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { isRemote } from '~/utils/runtime'

/**
 * Developer diagnostics, off unless the app is started with `SN_PERF=1`
 * (timing lines in the log) or `SN_PERF_SELFTEST=1` (scripted scroll and
 * playback run). Numbers before guesses: this is how a change is checked on a
 * device that has no dev tools, such as a headset reached over SSH.
 */
interface PerfFlags {
  perf: boolean
  selftest: boolean
}

let flags: PerfFlags = { perf: false, selftest: false }

function log(line: string) {
  invoke('perf_log', { line }).catch(() => {})
}

function percentile(sorted: number[], p: number): number {
  if (!sorted.length) return 0
  return sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * p))]!
}

/** Frame-time statistics between now and `stop()`. */
function frameStats() {
  const deltas: number[] = []
  let last = performance.now()
  let raf = requestAnimationFrame(function tick(now) {
    deltas.push(now - last)
    last = now
    raf = requestAnimationFrame(tick)
  })
  return {
    stop(label: string) {
      cancelAnimationFrame(raf)
      const sorted = deltas.slice().sort((a, b) => a - b)
      const long = deltas.filter((d) => d > 33.4).length
      log(
        `${label}: frames=${deltas.length} p50=${percentile(sorted, 0.5).toFixed(1)}ms `
        + `p95=${percentile(sorted, 0.95).toFixed(1)}ms max=${(sorted[sorted.length - 1] ?? 0).toFixed(1)}ms `
        + `over33ms=${long}`,
      )
    },
  }
}

function nextFrame(): Promise<number> {
  return new Promise((resolve) => requestAnimationFrame(resolve))
}

function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms))
}

/** Scroll the board down and back up at a steady speed, one step per frame. */
async function scrollRun() {
  const el = document.querySelector<HTMLElement>('.SoundContainer__scroll')
  if (!el) return log('selftest scroll: no board')
  const max = el.scrollHeight - el.clientHeight
  if (max <= 0) return log('selftest scroll: board does not scroll')
  const previous = el.style.scrollBehavior
  el.style.scrollBehavior = 'auto'
  const step = Math.max(8, Math.round(el.clientHeight / 30))
  const stats = frameStats()
  for (let pass = 0; pass < 2; pass++) {
    for (let y = 0; y <= max; y += step) { el.scrollTop = y; await nextFrame() }
    for (let y = max; y >= 0; y -= step) { el.scrollTop = y; await nextFrame() }
  }
  stats.stop(`selftest scroll (${max}px, step ${step}px)`)
  el.scrollTop = 0
  el.style.scrollBehavior = previous
}

/** Click the first few sounds and time click → playing, then stop them. */
async function playRun() {
  const buttons = [...document.querySelectorAll<HTMLElement>('.Soundbtn[data-sound-path]')].slice(0, 4)
  if (!buttons.length) return log('selftest play: no sounds')
  let waiting: { path: string; done: (ms: number) => void; at: number } | null = null
  const off = await listen<Array<{ path: string }>>('playing_changed', (e) => {
    if (waiting && (e.payload ?? []).some((p) => p.path === waiting!.path)) {
      waiting.done(performance.now() - waiting.at)
      waiting = null
    }
  })
  const stats = frameStats()
  for (const [i, btn] of buttons.entries()) {
    const path = btn.getAttribute('data-sound-path') || ''
    const ms = await new Promise<number>((resolve) => {
      waiting = { path, done: resolve, at: performance.now() }
      btn.click()
      setTimeout(() => resolve(-1), 5000)
    })
    log(`selftest play #${i + 1}: click→playing ${ms < 0 ? 'timeout' : ms.toFixed(0) + 'ms'}`)
    await wait(1200)
  }
  await invoke('stop_all').catch(() => {})
  window.dispatchEvent(new CustomEvent('sn:stop-all'))
  stats.stop('selftest play frames')
  off()
}

async function selfTest() {
  log('selftest start')
  await wait(1500)
  await scrollRun()
  await wait(500)
  await playRun()
  await wait(500)
  await scrollRun()
  log('selftest done')
}

/**
 * Read the flags once the main window is up. With the self test requested it
 * starts a few seconds later, when the project has had time to load.
 */
export async function initPerf(): Promise<void> {
  if (isRemote || typeof window === 'undefined') return
  try {
    const got = await invoke<Partial<PerfFlags> | null>('perf_flags')
    flags = { perf: got?.perf === true, selftest: got?.selftest === true }
  } catch {
    return
  }
  if (flags.selftest) setTimeout(() => { void selfTest() }, 4000)
}

/** Log a timing line when `SN_PERF=1`. */
export function perfLog(line: string) {
  if (flags.perf || flags.selftest) log(line)
}
