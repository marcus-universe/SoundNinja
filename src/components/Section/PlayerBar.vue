<template>
  <div
    class="player-float"
    :class="{ 'player-float--large': playerLarge, 'player-float--wave': showWave }"
  >
    <div class="player-float__row">
      <QuickInfo v-if="hasPlaying" :text="anyPaused ? $t('player.play') : $t('player.pause')">
        <button
          class="player-float__btn"
          type="button"
          :class="{ 'player-float__btn--active': hasPlaying }"
          @click="togglePlayPause"
        >
          <Icons :icon="anyPaused ? 'play' : 'pause'" />
        </button>
      </QuickInfo>

      <QuickInfo :text="$t('player.stop')">
        <button class="player-float__btn" type="button" :disabled="!hasPlaying" @click="stopAll">
          <Icons icon="stop" />
        </button>
      </QuickInfo>

      <template v-if="showWave">
        <canvas
          ref="waveCanvas"
          class="player-float__canvas"
          @pointerdown="onWavePointerDown"
          @pointermove="onWavePointerMove"
          @pointerup="onWavePointerUp"
          @pointercancel="onWavePointerUp"
          @pointerleave="onWavePointerLeave"
        />
        <QuickInfo :text="loopOn ? $t('player.loopOn') : $t('player.loopOff')">
          <button
            class="player-float__btn"
            type="button"
            :class="{ 'player-float__btn--active': loopOn }"
            @click="toggleLoop"
          >
            <Icons icon="loop" />
          </button>
        </QuickInfo>
      </template>

      <QuickInfo :text="$t('player.record')">
        <button
          class="player-float__btn player-float__btn--record"
          type="button"
          :aria-label="$t('player.record')"
          @click="openRecordEditor"
        >
          <div class="player-float__rec" aria-hidden="true">
            <div class="player-float__rec-dot" />
          </div>
        </button>
      </QuickInfo>

      <QuickInfo v-if="overlapSounds" :text="$t('player.playlist')">
        <button
          class="player-float__btn"
          type="button"
          @click="openPlayingList"
        >
          <Icons icon="playlist" />
        </button>
      </QuickInfo>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { openSecondaryWindow, PLAYING_LIST, RECORD_EDITOR } from '~/utils/secondaryWindows'
import { PeakLayer, blitPeaks, drawPlayhead, startThrottledClock } from '~/utils/waveformDraw'
import type { PlayingInfo } from './PlayingList.vue'

const jsonStore = useJsonHandelingStore()

const playing = ref<PlayingInfo[]>([])
const loopOn = ref(false)
const peaks = ref<number[]>([])
const durationSec = ref(0)
const playheadSec = ref(0)
/** Hover preview position (null = not hovering). */
const hoverSec = ref<number | null>(null)
const waveCanvas = ref<HTMLCanvasElement | null>(null)
const scrubbing = ref(false)

let unlisten: UnlistenFn | null = null
let waveClock: { stop: () => void } | null = null
let wavePath = ''
let progressAnchorMs = 0
let progressElapsedMs = 0
let progressPaused = true
let seekInFlight = false
/** Coalesce scrub pointermove redraws to one paint per animation frame. */
let scrubDrawRaf: number | null = null
const peakLayer = new PeakLayer()

function scheduleScrubDraw() {
  if (scrubDrawRaf != null) return
  scrubDrawRaf = requestAnimationFrame(() => {
    scrubDrawRaf = null
    drawWave()
  })
}

const overlapSounds = computed(() => jsonStore.configFile.settings.overlapSounds ?? false)
const playerLarge = computed(() => jsonStore.configFile.settings.playerLarge === true)
const hasPlaying = computed(() => playing.value.length > 0)
const anyPaused = computed(() => hasPlaying.value && playing.value.every((p) => p.paused))
const showWave = computed(() => !overlapSounds.value && hasPlaying.value)
const activePath = computed(() => playing.value[0]?.path ?? '')

async function refreshPlaying() {
  try {
    playing.value = (await invoke<PlayingInfo[]>('get_playing_sounds')) ?? []
  } catch {
    playing.value = []
  }
}

async function togglePlayPause() {
  if (!hasPlaying.value) return
  try {
    if (anyPaused.value) {
      await invoke('resume_all')
    } else {
      await invoke('pause_all')
    }
    await refreshPlaying()
  } catch (e) {
    console.error(e)
  }
}

async function stopAll() {
  try {
    await invoke('stop_all')
    jsonStore.ReturnStatusAll()
    loopOn.value = false
    await refreshPlaying()
  } catch (e) {
    console.error(e)
  }
}

async function openRecordEditor() {
  try {
    await openSecondaryWindow(RECORD_EDITOR)
  } catch (e) {
    console.error('Failed to open Record Editor window', e)
  }
}

async function openPlayingList() {
  try {
    await openSecondaryWindow(PLAYING_LIST)
  } catch (e) {
    console.error('Failed to open Playing List window', e)
  }
}

async function toggleLoop() {
  if (!hasPlaying.value) return
  const next = !loopOn.value
  try {
    await invoke('set_playing_loop', {
      looping: next,
      soundPath: activePath.value || null,
    })
    loopOn.value = next
  } catch (e) {
    console.error(e)
  }
}

function applyPlayingSnapshot(list: PlayingInfo[]) {
  // While scrubbing, keep local playhead until pointer-up seek finishes.
  if (scrubbing.value) {
    playing.value = list
    if (list[0]) loopOn.value = !!list[0].looping
    return
  }

  playing.value = list
  const first = list[0]
  if (!first || overlapSounds.value) {
    stopWaveClock()
    peaks.value = []
    wavePath = ''
    durationSec.value = 0
    playheadSec.value = 0
    loopOn.value = false
    return
  }
  loopOn.value = !!first.looping
  progressPaused = !!first.paused
  const backendPos = Math.max(0, first.positionSecs ?? 0)
  // Right after seek (esp. while paused), prefer local playhead so a stale
  // backend 0 cannot yank the cursor back to the start.
  let posSec = backendPos
  if (seekInFlight || (progressPaused && playheadSec.value > 0.05 && backendPos < 0.05)) {
    posSec = Math.max(playheadSec.value, backendPos)
  }
  progressElapsedMs = posSec * 1000
  progressAnchorMs = Date.now() - progressElapsedMs
  playheadSec.value = posSec
  if (first.durationSecs && first.durationSecs > 0) {
    durationSec.value = first.durationSecs
  }
  if (!progressPaused) startWaveClock()
  else stopWaveClock()
  if (first.path !== wavePath) {
    void loadWaveform(first.path, first.durationSecs)
  } else {
    drawWave()
  }
}

async function loadWaveform(path: string, knownDuration?: number) {
  wavePath = path
  if (knownDuration && knownDuration > 0) {
    durationSec.value = knownDuration
  } else {
    try {
      durationSec.value = await invoke<number>('get_sound_duration', { soundPath: path })
    } catch {
      durationSec.value = 0
    }
  }
  try {
    const buckets = playerLarge.value ? 220 : 100
    peaks.value = (await invoke<number[]>('get_file_waveform_peaks', { path, buckets })) ?? []
  } catch (e) {
    console.warn('waveform peaks failed', e)
    peaks.value = []
  }
  await nextTick()
  drawWave()
}

function startWaveClock() {
  if (waveClock) return
  waveClock = startThrottledClock(20, () => {
    if (progressPaused) return false
    if (!scrubbing.value && durationSec.value > 0) {
      let sec = (Date.now() - progressAnchorMs) / 1000
      if (loopOn.value && sec >= durationSec.value) {
        sec = sec % durationSec.value
        progressElapsedMs = sec * 1000
        progressAnchorMs = Date.now() - progressElapsedMs
      } else {
        sec = Math.min(durationSec.value, sec)
      }
      playheadSec.value = sec
      drawWave()
    }
    return !progressPaused
  })
}

function stopWaveClock() {
  waveClock?.stop()
  waveClock = null
}

function drawWave() {
  const canvas = waveCanvas.value
  if (!canvas) return
  const dpr = window.devicePixelRatio || 1
  const cssW = canvas.clientWidth || 200
  const cssH = canvas.clientHeight || 28
  if (canvas.width !== Math.floor(cssW * dpr) || canvas.height !== Math.floor(cssH * dpr)) {
    canvas.width = Math.floor(cssW * dpr)
    canvas.height = Math.floor(cssH * dpr)
    peakLayer.invalidate()
  }
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  const layer = peakLayer.ensure(cssW, cssH, dpr, peaks.value)
  blitPeaks(ctx, layer, cssW, cssH)
  drawPlayhead(
    ctx,
    cssW,
    cssH,
    durationSec.value,
    playheadSec.value,
    hoverSec.value != null && !scrubbing.value ? hoverSec.value : null,
  )
}

function secFromPointer(e: PointerEvent) {
  const canvas = waveCanvas.value
  if (!canvas || durationSec.value <= 0) return 0
  const rect = canvas.getBoundingClientRect()
  const t = Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width))
  // Keep away from exact EOF — backend also clamps.
  const max = Math.max(0, durationSec.value - 0.05)
  return t * max
}

async function seekTo(sec: number) {
  const max = durationSec.value > 0.08 ? durationSec.value - 0.05 : 0
  const clamped = Math.min(max, Math.max(0, sec))
  playheadSec.value = clamped
  progressElapsedMs = clamped * 1000
  progressAnchorMs = Date.now() - progressElapsedMs
  drawWave()
  seekInFlight = true
  try {
    await invoke('seek_playing', {
      positionSecs: clamped,
      soundPath: activePath.value || null,
    })
  } catch (e) {
    console.error(e)
  } finally {
    seekInFlight = false
  }
}

function onWavePointerDown(e: PointerEvent) {
  scrubbing.value = true
  hoverSec.value = null
  ;(e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId)
  playheadSec.value = secFromPointer(e)
  scheduleScrubDraw()
}

function onWavePointerMove(e: PointerEvent) {
  const sec = secFromPointer(e)
  if (scrubbing.value) {
    playheadSec.value = sec
  } else {
    hoverSec.value = sec
  }
  scheduleScrubDraw()
}

function onWavePointerUp(e: PointerEvent) {
  if (!scrubbing.value) return
  scrubbing.value = false
  if (scrubDrawRaf != null) {
    cancelAnimationFrame(scrubDrawRaf)
    scrubDrawRaf = null
  }
  void seekTo(secFromPointer(e))
}

function onWavePointerLeave() {
  if (scrubbing.value) return
  hoverSec.value = null
  scheduleScrubDraw()
}

onMounted(async () => {
  await refreshPlaying()
  applyPlayingSnapshot(playing.value)
  unlisten = await listen<PlayingInfo[]>('playing_changed', (event) => {
    applyPlayingSnapshot(event.payload ?? [])
  })
  window.addEventListener('resize', drawWave)
})

onUnmounted(() => {
  if (unlisten) unlisten()
  stopWaveClock()
  window.removeEventListener('resize', drawWave)
  if (scrubDrawRaf) {
    cancelAnimationFrame(scrubDrawRaf)
    scrubDrawRaf = null
  }
})

watch(overlapSounds, () => {
  applyPlayingSnapshot(playing.value)
})

watch(playerLarge, () => {
  if (wavePath) void loadWaveform(wavePath)
  else drawWave()
})

watch(showWave, async (on) => {
  if (on) {
    await nextTick()
    if (activePath.value) await loadWaveform(activePath.value)
    else drawWave()
  }
})
</script>
