<script setup lang="ts">
const VIEW_W = 1440
const VIEW_H = 900
const MID_Y = VIEW_H / 2
const STEP = 10
const BULGE_SIGMA = 280
// Total vertical drift of the frontmost layer between page top and bottom.
const PARALLAX_RANGE = 120

interface WaveConfig {
  amp: number
  yOff: number
  parallax: number
  tilt: number
  harmonics: { freq: number; speed: number; weight: number; offset: number }[]
}

const WAVES: WaveConfig[] = [
  {
    amp: 82,
    yOff: -96,
    parallax: 0.58,
    tilt: 0.62,
    harmonics: [
      { freq: 0.0042, speed: 0.42, weight: 0.62, offset: 0 },
      { freq: 0.0091, speed: -0.29, weight: 0.26, offset: 1.7 },
      { freq: 0.0175, speed: 0.61, weight: 0.12, offset: 3.1 },
    ],
  },
  {
    amp: 64,
    yOff: -18,
    parallax: 0.42,
    tilt: 0.46,
    harmonics: [
      { freq: 0.0052, speed: -0.34, weight: 0.58, offset: 2.4 },
      { freq: 0.0108, speed: 0.47, weight: 0.29, offset: 0.6 },
      { freq: 0.0211, speed: -0.68, weight: 0.13, offset: 4.2 },
    ],
  },
  {
    amp: 48,
    yOff: 58,
    parallax: 0.28,
    tilt: 0.3,
    harmonics: [
      { freq: 0.0064, speed: 0.53, weight: 0.6, offset: 1.1 },
      { freq: 0.0129, speed: -0.4, weight: 0.27, offset: 3.8 },
      { freq: 0.0248, speed: 0.77, weight: 0.13, offset: 2.2 },
    ],
  },
  {
    amp: 34,
    yOff: 126,
    parallax: 0.16,
    tilt: 0.18,
    harmonics: [
      { freq: 0.0079, speed: -0.6, weight: 0.63, offset: 5.1 },
      { freq: 0.0163, speed: 0.44, weight: 0.24, offset: 2.9 },
      { freq: 0.0302, speed: -0.85, weight: 0.13, offset: 0.9 },
    ],
  },
]

interface Motion {
  time: number
  mouseX: number
  mouseY: number
  progress: number
  energy: number
}

function buildPath(cfg: WaveConfig, m: Motion) {
  const focusX = m.mouseX * VIEW_W
  const tilt = (m.mouseY - 0.5) * 70 * cfg.tilt
  const baseY = MID_Y + cfg.yOff + tilt + m.progress * PARALLAX_RANGE * cfg.parallax
  let d = ''

  for (let x = 0; x <= VIEW_W; x += STEP) {
    // Taper both ends to zero so lines dissolve instead of getting cut off.
    const edge = Math.sin((Math.PI * x) / VIEW_W) ** 0.75
    const dx = x - focusX
    const bulge = 1 + m.energy * Math.exp(-(dx * dx) / (2 * BULGE_SIGMA ** 2))

    let sum = 0
    for (const h of cfg.harmonics) {
      sum += Math.sin(x * h.freq + m.time * h.speed + h.offset) * h.weight
    }

    const y = baseY + sum * cfg.amp * edge * bulge
    d += `${x === 0 ? 'M' : 'L'}${x} ${y.toFixed(2)}`
  }
  return d
}

const REST: Motion = { time: 0, mouseX: 0.5, mouseY: 0.5, progress: 0, energy: 0 }

const wrap = ref<HTMLElement | null>(null)
const paths = ref<string[]>(WAVES.map((cfg) => buildPath(cfg, REST)))

onMounted(() => {
  const el = wrap.value
  if (!el) return

  const reducedQuery = window.matchMedia('(prefers-reduced-motion: reduce)')
  const fine = window.matchMedia('(pointer: fine)').matches
  let motionScale = reducedQuery.matches ? 0.3 : 1
  const onReducedChange = (e: MediaQueryListEvent) => {
    motionScale = e.matches ? 0.3 : 1
  }
  reducedQuery.addEventListener('change', onReducedChange)

  let raf = 0
  let running = true
  let time = 0
  let last = performance.now()

  const cur = { x: 0.5, y: 0.5, progress: 0, energy: 0 }
  const target = { x: 0.5, y: 0.5, progress: 0, energy: 0 }

  function frame(now: number) {
    const dt = Math.min(50, now - last)
    last = now
    time += (dt / 1000) * motionScale

    // Frame-rate independent easing gives the trailing "delay" feel.
    const kFast = 1 - Math.exp(-dt * 0.006)
    const kSlow = 1 - Math.exp(-dt * 0.0035)
    cur.x += (target.x - cur.x) * kFast
    cur.y += (target.y - cur.y) * kFast
    cur.energy += (target.energy - cur.energy) * kFast
    cur.progress += (target.progress - cur.progress) * kSlow

    const m: Motion = {
      time,
      mouseX: cur.x,
      mouseY: cur.y,
      progress: cur.progress,
      energy: cur.energy,
    }
    paths.value = WAVES.map((cfg) => buildPath(cfg, m))

    el!.style.setProperty('--halo-x', `${(cur.x * 100).toFixed(2)}%`)
    el!.style.setProperty('--halo-y', `${(cur.y * 100).toFixed(2)}%`)
    el!.style.setProperty('--halo-a', (0.1 + cur.energy * 0.06).toFixed(3))

    if (running) raf = requestAnimationFrame(frame)
  }

  function onPointerMove(e: PointerEvent) {
    target.x = Math.min(1, Math.max(0, e.clientX / window.innerWidth))
    target.y = Math.min(1, Math.max(0, e.clientY / window.innerHeight))
    target.energy = 0.9
  }

  function onPointerLeave() {
    target.energy = 0
  }

  function onScroll() {
    const max = document.documentElement.scrollHeight - window.innerHeight
    target.progress = max > 0 ? Math.min(1, window.scrollY / max) : 0
  }

  function onVisibility() {
    if (document.hidden) {
      running = false
      cancelAnimationFrame(raf)
    } else if (!running) {
      running = true
      last = performance.now()
      raf = requestAnimationFrame(frame)
    }
  }

  if (fine) {
    window.addEventListener('pointermove', onPointerMove, { passive: true })
    document.addEventListener('pointerleave', onPointerLeave)
  }
  window.addEventListener('scroll', onScroll, { passive: true })
  window.addEventListener('resize', onScroll, { passive: true })
  document.addEventListener('visibilitychange', onVisibility)
  onScroll()
  cur.progress = target.progress

  raf = requestAnimationFrame(frame)

  onBeforeUnmount(() => {
    running = false
    cancelAnimationFrame(raf)
    reducedQuery.removeEventListener('change', onReducedChange)
    window.removeEventListener('pointermove', onPointerMove)
    document.removeEventListener('pointerleave', onPointerLeave)
    window.removeEventListener('scroll', onScroll)
    window.removeEventListener('resize', onScroll)
    document.removeEventListener('visibilitychange', onVisibility)
  })
})
</script>

<template>
  <div
    ref="wrap"
    class="site-backdrop pointer-events-none fixed inset-0 z-0 overflow-hidden"
    aria-hidden="true"
  >
    <div class="site-backdrop__halo" />
    <svg
      class="site-backdrop__svg"
      :viewBox="`0 0 ${VIEW_W} ${VIEW_H}`"
      preserveAspectRatio="none"
    >
      <defs>
        <linearGradient id="site-wave-fade" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0" stop-color="hsl(189 100% 58%)" stop-opacity="0" />
          <stop offset="0.28" stop-color="hsl(189 100% 58%)" stop-opacity="1" />
          <stop offset="0.72" stop-color="hsl(189 100% 66%)" stop-opacity="1" />
          <stop offset="1" stop-color="hsl(189 100% 66%)" stop-opacity="0" />
        </linearGradient>
      </defs>
      <path
        v-for="(d, i) in paths"
        :key="i"
        :d="d"
        :class="`site-backdrop__line site-backdrop__line--${i}`"
        fill="none"
        stroke="url(#site-wave-fade)"
      />
    </svg>
  </div>
</template>

<style scoped>
.site-backdrop {
  --halo-x: 50%;
  --halo-y: 40%;
  --halo-a: 0.1;
  mask-image: radial-gradient(
    120% 90% at 50% 45%,
    #000 45%,
    rgb(0 0 0 / 0.35) 78%,
    transparent 100%
  );
}

.site-backdrop__halo {
  position: absolute;
  top: var(--halo-y);
  left: var(--halo-x);
  width: 30rem;
  height: 30rem;
  translate: -50% -50%;
  border-radius: 9999px;
  background: hsl(189 100% 58% / var(--halo-a));
  filter: blur(90px);
}

.site-backdrop__svg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
}

.site-backdrop__line {
  stroke-linecap: round;
  stroke-linejoin: round;
  vector-effect: non-scaling-stroke;
}

.site-backdrop__line--0 {
  stroke-width: 1.6;
  opacity: 0.32;
}

.site-backdrop__line--1 {
  stroke-width: 1.3;
  opacity: 0.24;
}

.site-backdrop__line--2 {
  stroke-width: 1.1;
  opacity: 0.17;
}

.site-backdrop__line--3 {
  stroke-width: 0.9;
  opacity: 0.12;
}
</style>
