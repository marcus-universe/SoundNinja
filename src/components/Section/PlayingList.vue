<template>
  <div
    class="playing-list"
    :class="{
      'playing-list--large': playerLarge,
      'playing-list--window': windowMode,
    }"
    @click.stop
  >
    <div v-if="items.length === 0" class="playing-list__empty">
      {{ $t('player.playlistEmpty') }}
    </div>
    <div v-for="item in items" :key="item.path" class="playing-list__row">
      <div class="playing-list__head">
        <span class="playing-list__name">{{ displayName(item.path) }}</span>
        <span class="playing-list__state">
          {{ item.paused ? $t('player.paused') : $t('player.playing') }}
        </span>
        <div class="playing-list__actions">
          <QuickInfo :text="item.paused ? $t('player.play') : $t('player.pause')">
            <button class="playing-list__btn" type="button" @click="toggleOne(item)">
              <Icons :icon="item.paused ? 'play' : 'pause'" />
            </button>
          </QuickInfo>
          <QuickInfo :text="$t('player.stop')">
            <button class="playing-list__btn" type="button" @click="stopOne(item.path)">
              <Icons icon="stop" />
            </button>
          </QuickInfo>
        </div>
      </div>

      <div class="playing-list__wave-row">
        <canvas
          :ref="(el) => setCanvasRef(item.path, el)"
          class="playing-list__canvas"
          @pointerdown="(e) => onWavePointerDown(item, e)"
          @pointermove="(e) => onWavePointerMove(item, e)"
          @pointerup="(e) => onWavePointerUp(item, e)"
          @pointercancel="(e) => onWavePointerUp(item, e)"
        />
        <QuickInfo :text="item.looping ? $t('player.loopOn') : $t('player.loopOff')">
          <button
            class="playing-list__btn"
            type="button"
            :class="{ 'playing-list__btn--active': !!item.looping }"
            @click="toggleLoop(item)"
          >
            <Icons icon="loop" />
          </button>
        </QuickInfo>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'

export interface PlayingInfo {
  path: string
  paused: boolean
  positionSecs?: number
  looping?: boolean
}

const props = withDefaults(defineProps<{
  items: PlayingInfo[]
  windowMode?: boolean
}>(), {
  windowMode: false,
})

const emit = defineEmits<{
  changed: []
}>()

const jsonStore = useJsonHandelingStore()
const playerLarge = computed(() => jsonStore.configFile?.settings?.playerLarge === true)

const peaksByPath = reactive(new Map<string, number[]>())
const durationByPath = reactive(new Map<string, number>())
const playheadByPath = reactive(new Map<string, number>())
const canvasByPath = new Map<string, HTMLCanvasElement>()

const scrubPath = ref<string | null>(null)
const seekInFlight = ref<string | null>(null)

let rafId: number | null = null
const anchorByPath = new Map<string, { originMs: number; elapsedMs: number; paused: boolean }>()

function displayName(path: string) {
  return path.replace(/^.*[\\/]/, '').replace(/\.(wav|mp3|ogg|flac)$/i, '')
}

function setCanvasRef(path: string, el: unknown) {
  if (el instanceof HTMLCanvasElement) {
    canvasByPath.set(path, el)
    drawWave(path)
  } else {
    canvasByPath.delete(path)
  }
}

async function ensureWaveform(path: string) {
  if (peaksByPath.has(path) && durationByPath.has(path)) return
  try {
    const dur = await invoke<number>('get_sound_duration', { soundPath: path })
    durationByPath.set(path, dur || 0)
  } catch {
    durationByPath.set(path, 0)
  }
  try {
    const buckets = playerLarge.value ? 160 : 100
    const peaks = (await invoke<number[]>('get_file_waveform_peaks', { path, buckets })) ?? []
    peaksByPath.set(path, peaks)
  } catch (e) {
    console.warn('playlist waveform failed', e)
    peaksByPath.set(path, [])
  }
  await nextTick()
  drawWave(path)
}

function syncFromItems() {
  const live = new Set(props.items.map((i) => i.path))
  for (const key of [...peaksByPath.keys()]) {
    if (!live.has(key)) {
      peaksByPath.delete(key)
      durationByPath.delete(key)
      playheadByPath.delete(key)
      anchorByPath.delete(key)
      canvasByPath.delete(key)
    }
  }

  for (const item of props.items) {
    void ensureWaveform(item.path)
    if (scrubPath.value === item.path || seekInFlight.value === item.path) {
      continue
    }
    const pos = Math.max(0, item.positionSecs ?? 0)
    playheadByPath.set(item.path, pos)
    anchorByPath.set(item.path, {
      originMs: Date.now() - pos * 1000,
      elapsedMs: pos * 1000,
      paused: !!item.paused,
    })
    drawWave(item.path)
  }
  ensureClock()
}

function ensureClock() {
  const anyRunning = props.items.some((i) => !i.paused)
  if (!anyRunning) {
    if (rafId != null) {
      cancelAnimationFrame(rafId)
      rafId = null
    }
    return
  }
  if (rafId != null) return
  const tick = () => {
    const now = Date.now()
    for (const item of props.items) {
      if (item.paused || scrubPath.value === item.path) continue
      const anchor = anchorByPath.get(item.path)
      const dur = durationByPath.get(item.path) || 0
      if (!anchor || dur <= 0) continue
      let sec = (now - anchor.originMs) / 1000
      if (item.looping && sec >= dur) {
        sec = sec % dur
        anchor.originMs = now - sec * 1000
        anchor.elapsedMs = sec * 1000
      } else {
        sec = Math.min(dur, sec)
      }
      playheadByPath.set(item.path, sec)
      drawWave(item.path)
    }
    rafId = requestAnimationFrame(tick)
  }
  rafId = requestAnimationFrame(tick)
}

function drawWave(path: string) {
  const canvas = canvasByPath.get(path)
  if (!canvas) return
  const dpr = window.devicePixelRatio || 1
  const cssW = canvas.clientWidth || 180
  const cssH = canvas.clientHeight || 28
  if (canvas.width !== Math.floor(cssW * dpr) || canvas.height !== Math.floor(cssH * dpr)) {
    canvas.width = Math.floor(cssW * dpr)
    canvas.height = Math.floor(cssH * dpr)
  }
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, cssW, cssH)

  const mid = cssH / 2
  const data = peaksByPath.get(path) || []
  const pairs = Math.floor(data.length / 2)
  const accent = getComputedStyle(document.documentElement)
    .getPropertyValue('--primary_color')
    .trim() || '#00d4ff'
  const muted = 'rgba(255,255,255,0.22)'

  if (pairs > 0) {
    const step = cssW / pairs
    ctx.fillStyle = muted
    for (let i = 0; i < pairs; i++) {
      const minV = data[i * 2]
      const maxV = data[i * 2 + 1]
      const y1 = mid + minV * mid
      const y2 = mid + maxV * mid
      ctx.fillRect(i * step, y1, Math.max(1, step * 0.85), Math.max(1, y2 - y1))
    }
  } else {
    ctx.strokeStyle = muted
    ctx.beginPath()
    ctx.moveTo(0, mid)
    ctx.lineTo(cssW, mid)
    ctx.stroke()
  }

  const dur = durationByPath.get(path) || 0
  const head = playheadByPath.get(path) || 0
  if (dur > 0) {
    const x = (head / dur) * cssW
    ctx.strokeStyle = accent
    ctx.lineWidth = 2
    ctx.beginPath()
    ctx.moveTo(x, 0)
    ctx.lineTo(x, cssH)
    ctx.stroke()
  }
}

function secFromPointer(path: string, e: PointerEvent) {
  const canvas = canvasByPath.get(path)
  const dur = durationByPath.get(path) || 0
  if (!canvas || dur <= 0) return 0
  const rect = canvas.getBoundingClientRect()
  const t = Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width))
  const max = Math.max(0, dur - 0.05)
  return t * max
}

async function seekItem(path: string, sec: number) {
  const dur = durationByPath.get(path) || 0
  const max = dur > 0.08 ? dur - 0.05 : 0
  const clamped = Math.min(max, Math.max(0, sec))
  playheadByPath.set(path, clamped)
  anchorByPath.set(path, {
    originMs: Date.now() - clamped * 1000,
    elapsedMs: clamped * 1000,
    paused: props.items.find((i) => i.path === path)?.paused ?? false,
  })
  drawWave(path)
  seekInFlight.value = path
  try {
    await invoke('seek_playing', {
      positionSecs: clamped,
      soundPath: path,
    })
    emit('changed')
  } catch (e) {
    console.error(e)
  } finally {
    seekInFlight.value = null
  }
}

function onWavePointerDown(item: PlayingInfo, e: PointerEvent) {
  scrubPath.value = item.path
  ;(e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId)
  const sec = secFromPointer(item.path, e)
  playheadByPath.set(item.path, sec)
  drawWave(item.path)
}

function onWavePointerMove(item: PlayingInfo, e: PointerEvent) {
  if (scrubPath.value !== item.path) return
  const sec = secFromPointer(item.path, e)
  playheadByPath.set(item.path, sec)
  drawWave(item.path)
}

function onWavePointerUp(item: PlayingInfo, e: PointerEvent) {
  if (scrubPath.value !== item.path) return
  scrubPath.value = null
  void seekItem(item.path, secFromPointer(item.path, e))
}

async function toggleLoop(item: PlayingInfo) {
  try {
    await invoke('set_playing_loop', {
      looping: !item.looping,
      soundPath: item.path,
    })
    emit('changed')
  } catch (e) {
    console.error(e)
  }
}

async function toggleOne(item: PlayingInfo) {
  try {
    if (item.paused) {
      await invoke('resume_sound', { soundPath: item.path })
    } else {
      await invoke('pause_sound', { soundPath: item.path })
    }
    emit('changed')
  } catch (e) {
    console.error(e)
  }
}

async function stopOne(path: string) {
  try {
    await invoke('play_sound', {
      soundPath: path,
      deviceName: 'default',
      hostName: null,
      active: true,
      overlap: true,
    })
    emit('changed')
  } catch (e) {
    console.error(e)
  }
}

watch(
  () => props.items,
  () => syncFromItems(),
  { deep: true, immediate: true },
)

watch(playerLarge, () => {
  for (const path of [...peaksByPath.keys()]) {
    peaksByPath.delete(path)
    void ensureWaveform(path)
  }
})

onMounted(() => {
  window.addEventListener('resize', () => {
    for (const path of canvasByPath.keys()) drawWave(path)
  })
})

onUnmounted(() => {
  if (rafId != null) cancelAnimationFrame(rafId)
})
</script>
