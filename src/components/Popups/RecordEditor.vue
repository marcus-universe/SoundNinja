<template>
  <section class="record-editor">
    <div class="record-editor__toolbar">
      <h2 class="record-editor__title">{{ $t('recordEditor.title') }}</h2>
      <div class="record-editor__meta">
        <span>{{ deviceLabel }}</span>
      </div>
    </div>

    <SettingsAudio compact />

    <div class="record-editor__body">
      <div class="record-editor__meta">
        <QuickInfo :text="recording ? $t('recordEditor.stopRecord') : $t('recordEditor.record')">
          <button
            class="record-editor__rec"
            type="button"
            :class="{ recording }"
            :disabled="busy"
            :aria-label="recording ? $t('recordEditor.stopRecord') : $t('recordEditor.record')"
            @click="toggleRecord"
          >
            <span class="record-editor__rec-dot" />
          </button>
        </QuickInfo>
        <div class="record-editor__level" :title="$t('recordEditor.level')">
          <div class="record-editor__level-fill" :style="{ width: `${Math.round(level * 100)}%` }" />
        </div>
        <span class="record-editor__progress">
          {{ formatDuration(playheadSec) }} / {{ formatDuration(displayDuration) }}
        </span>
        <span v-if="statusText" class="record-editor__progress">{{ statusText }}</span>
      </div>

      <div class="record-editor__track">
        <div class="record-editor__ruler" aria-hidden="true">
          <span
            v-for="tick in rulerTicks"
            :key="`${tick.sec}-${tick.pct}`"
            class="record-editor__ruler-tick"
            :style="{ left: `${tick.pct}%` }"
          >
            {{ tick.label }}
          </span>
        </div>
        <div
          ref="waveWrap"
          class="record-editor__waveform"
          :class="{ panning: dragMode === 'pan' }"
          @pointerdown="onPointerDown"
          @pointermove="onPointerMove"
          @pointerup="onPointerUp"
          @pointercancel="onPointerUp"
          @pointerleave="onPointerUp"
          @auxclick.prevent
          @wheel.prevent="onWheel"
        >
          <canvas ref="canvasRef" />
          <div
            v-if="hasSelection"
            class="record-editor__waveform-sel"
            :style="selectionStyle"
          >
            <div
              class="record-editor__sel-handle record-editor__sel-handle--in"
              @pointerdown.stop="onSelHandleDown('start', $event)"
              @pointermove="onPointerMove"
              @pointerup="onPointerUp"
              @pointercancel="onPointerUp"
            />
            <div
              class="record-editor__sel-handle record-editor__sel-handle--out"
              @pointerdown.stop="onSelHandleDown('end', $event)"
              @pointermove="onPointerMove"
              @pointerup="onPointerUp"
              @pointercancel="onPointerUp"
            />
          </div>
          <div
            v-if="showPlayhead"
            class="record-editor__playhead"
            :class="{ dragging: dragMode === 'playhead' }"
            :style="{ left: `${playheadPct}%` }"
          >
            <div
              class="record-editor__playhead-handle"
              @pointerdown.stop="onPlayheadHandleDown($event)"
              @pointermove="onPointerMove"
              @pointerup="onPointerUp"
              @pointercancel="onPointerUp"
            />
          </div>
        </div>
        <div v-if="zoom > 1.01" class="record-editor__scroll">
          <input
            class="record-editor__scroll-range"
            type="range"
            min="0"
            :max="scrollMax"
            step="0.001"
            :value="viewStart"
            @input="onScrollRange"
          >
        </div>
      </div>

      <div class="record-editor__actions">
        <QuickInfo :text="$t('player.play')">
          <button class="record-editor__btn" type="button" :disabled="!session || busy" @click="previewPlay">
            <Icons icon="play" />
          </button>
        </QuickInfo>
        <QuickInfo :text="$t('player.pause')">
          <button class="record-editor__btn" type="button" :disabled="!session || busy || (!isPlaying && !isPaused)" @click="previewPause">
            <Icons icon="pause" />
          </button>
        </QuickInfo>
        <QuickInfo :text="$t('player.stop')">
          <button class="record-editor__btn" type="button" :disabled="!session || busy" @click="previewStop">
            <Icons icon="stop" />
          </button>
        </QuickInfo>

        <QuickInfo :text="$t('recordEditor.zoomIn')">
          <button class="record-editor__btn" type="button" :disabled="!canZoom || zoom >= ZOOM_MAX" @click="zoomIn">
            <Icons icon="zoom-in" />
          </button>
        </QuickInfo>
        <QuickInfo :text="$t('recordEditor.zoomOut')">
          <button class="record-editor__btn" type="button" :disabled="!canZoom || zoom <= 1" @click="zoomOut">
            <Icons icon="zoom-out" />
          </button>
        </QuickInfo>

        <QuickInfo :text="$t('recordEditor.normalize')">
          <button class="record-editor__btn" type="button" :disabled="!session || busy" @click="doNormalize">
            <Icons icon="normalize" />
          </button>
        </QuickInfo>
        <QuickInfo :text="$t('recordEditor.noiseCancel')">
          <button class="record-editor__btn" type="button" :disabled="!session || busy" @click="doDenoise">
            <Icons icon="noise-cancel" />
          </button>
        </QuickInfo>

        <div v-if="stemsVisible" class="record-editor__stems-menu">
          <QuickInfo :text="$t('recordEditor.stems')">
            <button class="record-editor__btn" type="button" :disabled="!session || busy" @click="stemsOpen = !stemsOpen">
              <Icons icon="stems" />
            </button>
          </QuickInfo>
          <div v-if="stemsOpen" class="record-editor__stems-drop">
            <button type="button" @click="doStems('vocals')">{{ $t('recordEditor.stemsVocals') }}</button>
            <button type="button" @click="doStems('music')">{{ $t('recordEditor.stemsMusic') }}</button>
          </div>
        </div>

        <QuickInfo :text="$t('recordEditor.deleteSelection')">
          <button class="record-editor__btn" type="button" :disabled="!session || !hasSelection || busy" @click="doDeleteSelection">
            <Icons icon="delete" />
          </button>
        </QuickInfo>

        <QuickInfo :text="$t('recordEditor.trimToSelection')">
          <button class="record-editor__btn" type="button" :disabled="!session || !hasSelection || busy" @click="doTrim">
            {{ $t('recordEditor.trim') }}
          </button>
        </QuickInfo>

        <button class="record-editor__btn" type="button" :disabled="!session || busy" @click="doUndo">
          {{ $t('recordEditor.undo') }}
        </button>
        <button class="record-editor__btn" type="button" :disabled="!session || busy" @click="doRedo">
          {{ $t('recordEditor.redo') }}
        </button>
      </div>

      <div class="record-editor__stage-bar">
        <QuickInfo :text="$t('recordEditor.addSelectionHint')">
          <button
            class="record-editor__btn"
            type="button"
            :disabled="!session || !hasSelection || busy"
            @click="addSelectionToQueue"
          >
            {{ $t('recordEditor.addSelection') }}
          </button>
        </QuickInfo>
        <QuickInfo :text="$t('recordEditor.addTrackHint')">
          <button
            class="record-editor__btn"
            type="button"
            :disabled="!session || busy"
            @click="addTrackToQueue"
          >
            {{ $t('recordEditor.addTrack') }}
          </button>
        </QuickInfo>
      </div>

      <div class="record-editor__queue">
        <div class="record-editor__queue-head">
          <h3 class="record-editor__queue-title">
            {{ $t('recordEditor.queueTitle') }}
            <span v-if="staged.length">({{ staged.length }})</span>
          </h3>
          <div class="record-editor__queue-tools">
            <button class="record-editor__btn record-editor__btn--sm" type="button" :disabled="!staged.length" @click="selectAllStaged">
              {{ $t('recordEditor.selectAll') }}
            </button>
            <button class="record-editor__btn record-editor__btn--sm" type="button" :disabled="!staged.length" @click="selectNoneStaged">
              {{ $t('recordEditor.selectNone') }}
            </button>
            <button class="record-editor__btn record-editor__btn--sm" type="button" :disabled="!canImportAll" @click="importStaged(true)">
              {{ $t('recordEditor.importAll') }}
            </button>
          </div>
        </div>

        <p v-if="staged.length && hasDuplicateNames" class="record-editor__queue-warn">{{ $t('recordEditor.duplicateNames') }}</p>
        <p v-else-if="staged.length && !selectedStaged.length" class="record-editor__queue-warn">{{ $t('recordEditor.noneSelected') }}</p>

        <p v-if="!staged.length" class="record-editor__queue-empty">{{ $t('recordEditor.queueEmpty') }}</p>
        <ul v-else class="record-editor__queue-list">
          <li
            v-for="clip in staged"
            :key="clip.id"
            class="record-editor__queue-item"
            :class="{ selected: clip.selected, duplicate: isDuplicateName(clip.name, clip.id) }"
          >
            <label class="record-editor__queue-check">
              <input v-model="clip.selected" type="checkbox">
            </label>
            <input
              v-model="clip.name"
              class="record-editor__queue-name"
              type="text"
              :placeholder="$t('recordEditor.clipName')"
              :aria-label="$t('recordEditor.clipName')"
              :title="isDuplicateName(clip.name, clip.id) ? $t('recordEditor.duplicateNames') : clip.name"
            >
            <span class="record-editor__queue-dur">{{ formatDuration(clip.duration) }}</span>
            <QuickInfo :text="$t('player.play')">
              <button class="record-editor__btn record-editor__btn--sm" type="button" :disabled="busy" @click="previewStaged(clip)">
                <Icons icon="play" />
              </button>
            </QuickInfo>
            <QuickInfo :text="$t('recordEditor.removeFromQueue')">
              <button class="record-editor__btn record-editor__btn--sm" type="button" :disabled="busy" @click="removeStaged(clip.id)">
                <Icons icon="delete" />
              </button>
            </QuickInfo>
          </li>
        </ul>
      </div>
    </div>

    <div class="record-editor__footer">
      <span class="record-editor__import-label">{{ $t('recordEditor.importTarget') }}</span>
      <select v-model="importTab" class="settings-select record-editor__footer-select">
        <option value="">{{ $t('recordEditor.importCurrentTab') }}</option>
        <option v-for="t in tabNames" :key="t" :value="t">{{ t }}</option>
      </select>
      <button
        class="record-editor__btn"
        type="button"
        :disabled="!canImportSelected"
        @click="importStaged(false)"
      >
        {{ $t('recordEditor.import') }}
      </button>
    </div>

    <DialogField v-if="closePrompt" :title="$t('recordEditor.unsavedTitle')" @close="closePrompt = false">
      <p class="settings-hint">{{ $t('recordEditor.unsavedHint') }}</p>
      <div class="settings-row" style="margin-top: 1rem; gap: 0.6rem">
        <button class="settings-btn" type="button" @click="discardAndClose">{{ $t('recordEditor.discard') }}</button>
        <button class="settings-btn" type="button" @click="closePrompt = false">{{ $t('recordEditor.cancel') }}</button>
      </div>
    </DialogField>

    <DialogField
      v-if="stemsPrompt"
      :title="$t('recordEditor.stemsDownloadTitle')"
      @close="cancelStemsPrompt"
    >
      <p class="settings-hint">{{ $t('recordEditor.stemsDownloadIntro') }}</p>
      <p class="settings-hint">
        <strong>{{ stemsStatus?.modelLabel || 'BS-RoFormer' }}</strong>
        — {{ $t('recordEditor.stemsModelDesc') }}
        ({{ stemsStatus?.modelSizeHint || '~158 MB' }})
      </p>
      <p class="settings-hint">{{ $t('recordEditor.stemsThirdParty') }}</p>
      <p class="settings-hint">
        <a
          class="record-editor__link"
          href="#"
          @click.prevent="openStemsModelPage"
        >{{ stemsStatus?.modelPageUrl || 'https://huggingface.co/xycld/BS-RoFormer-ONNX' }}</a>
      </p>
      <p v-if="!stemsStatus?.available" class="record-editor__queue-warn">
        {{ $t('recordEditor.stemsEngineMissing') }}
      </p>
      <p v-if="stemsDownloadPct != null" class="settings-hint">
        {{ $t('recordEditor.stemsDownloading') }}
        {{ Math.round(stemsDownloadPct) }}%
      </p>
      <div class="settings-row" style="margin-top: 1rem; gap: 0.6rem">
        <button
          v-if="stemsStatus?.available"
          class="settings-btn"
          type="button"
          :disabled="busy || stemsDownloading"
          @click="confirmStemsDownload"
        >
          {{ $t('recordEditor.stemsYes') }}
        </button>
        <button
          v-else
          class="settings-btn"
          type="button"
          @click="openStemsModelPage"
        >
          {{ $t('recordEditor.stemsOpenModel') }}
        </button>
        <button
          class="settings-btn"
          type="button"
          :disabled="stemsDownloading"
          @click="cancelStemsPrompt"
        >
          {{ stemsStatus?.available ? $t('recordEditor.stemsNo') : $t('recordEditor.cancel') }}
        </button>
      </div>
    </DialogField>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { emit, listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openUrl } from '@tauri-apps/plugin-opener'
import { open } from '@tauri-apps/plugin-dialog'
import { platform } from '@tauri-apps/plugin-os'

interface SessionInfo {
  session_id: string
  sample_rate: number
  channels: number
  duration_secs: number
}

interface StemsProgress {
  stage: string
  percent: number
  message: string
}

interface StagedClipInfo {
  path: string
  duration_secs: number
}

interface StagedClip {
  id: string
  name: string
  path: string
  duration: number
  selected: boolean
}

interface StemsStatus {
  available: boolean
  modelReady: boolean
  installIntent: boolean
  canPrompt: boolean
  hasBeenAsked: boolean
  modelName: string
  modelLabel: string
  modelPageUrl: string
  modelSizeHint: string
}

type DragMode = 'none' | 'select' | 'playhead' | 'pan' | 'sel-start' | 'sel-end'

const ZOOM_MAX = 64
const CLICK_DRAG_PX = 5

const appSettings = useAppSettingsStore()

const recording = ref(false)
const busy = ref(false)
const level = ref(0)
const session = ref<SessionInfo | null>(null)
const liveDuration = ref(0)
const statusText = ref('')
const stemsOpen = ref(false)
const stemsPrompt = ref(false)
const stemsPendingMode = ref<string | null>(null)
const stemsStatus = ref<StemsStatus | null>(null)
const stemsDownloading = ref(false)
const stemsDownloadPct = ref<number | null>(null)
const osPlatform = ref('')
const importTab = ref('')
const closePrompt = ref(false)
const dirty = ref(false)
const currentTab = ref('All')
const tabNames = ref<string[]>([])
const staged = ref<StagedClip[]>([])
let clipCounter = 0

const canvasRef = ref<HTMLCanvasElement | null>(null)
const waveWrap = ref<HTMLElement | null>(null)
const peaks = ref<number[]>([])
const selStart = ref(0)
const selEnd = ref(0)

const zoom = ref(1)
const viewStart = ref(0)

const cueSec = ref(0)
const playheadSec = ref(0)
const isPlaying = ref(false)
const isPaused = ref(false)
const dragMode = ref<DragMode>('none')
let pointerDownX = 0
let pointerMoved = false
let panOriginX = 0
let panOriginViewStart = 0
let playStartedAt = 0
let playOffset = 0
let rafId = 0
let livePollTimer: ReturnType<typeof setInterval> | null = null

let unlistenLevel: UnlistenFn | null = null
let unlistenStems: UnlistenFn | null = null
let unlistenStemsPct: UnlistenFn | null = null
let unlistenCtx: UnlistenFn | null = null
let unlistenFinished: UnlistenFn | null = null

const deviceLabel = computed(() => {
  const name = appSettings.inputSource && appSettings.inputSource !== 'default'
    ? appSettings.inputSource
    : '—'
  return `${name}`
})

const displayDuration = computed(() => {
  if (recording.value) return liveDuration.value
  return session.value?.duration_secs ?? 0
})

const canZoom = computed(() => displayDuration.value > 0.05)

/** Win/Linux: hide Stems until model installed. macOS: always show when engine present (in-app download). */
const stemsVisible = computed(() => {
  const s = stemsStatus.value
  if (!s?.available) return false
  if (s.modelReady) return true
  return osPlatform.value === 'macos'
})

const viewDuration = computed(() => {
  const dur = displayDuration.value
  if (dur <= 0) return 0.001
  return Math.max(0.001, dur / zoom.value)
})

const viewEnd = computed(() => Math.min(displayDuration.value, viewStart.value + viewDuration.value))

const scrollMax = computed(() => Math.max(0, displayDuration.value - viewDuration.value))

const hasSelection = computed(() => Math.abs(selEnd.value - selStart.value) > 0.01)

const selectedStaged = computed(() => staged.value.filter(c => c.selected))

function normalizeClipName(name: string) {
  return name.trim().toLowerCase()
}

function isDuplicateName(name: string, id: string) {
  const n = normalizeClipName(name)
  if (!n) return true
  return staged.value.some(c => c.id !== id && normalizeClipName(c.name) === n)
}

function hasNameIssues(clips: StagedClip[]) {
  const seen = new Set<string>()
  for (const c of clips) {
    const n = normalizeClipName(c.name)
    if (!n || seen.has(n)) return true
    seen.add(n)
  }
  return false
}

const hasDuplicateNames = computed(() => hasNameIssues(staged.value))

const canImportSelected = computed(() =>
  !busy.value
  && selectedStaged.value.length > 0
  && !hasNameIssues(selectedStaged.value),
)

const canImportAll = computed(() =>
  !busy.value
  && staged.value.length > 0
  && !hasDuplicateNames.value,
)

const selectionStyle = computed(() => {
  const a = Math.min(selStart.value, selEnd.value)
  const b = Math.max(selStart.value, selEnd.value)
  return {
    left: `${secToPct(a)}%`,
    width: `${Math.max(0, secToPct(b) - secToPct(a))}%`,
  }
})

const showPlayhead = computed(() => !!session.value || recording.value || isPlaying.value || isPaused.value)

const playheadPct = computed(() => {
  if (recording.value) return 100
  return secToPct(playheadSec.value)
})

const rulerTicks = computed(() => {
  const start = viewStart.value
  const end = Math.max(viewEnd.value, start + 0.001)
  const span = end - start
  const target = 8
  const rawStep = span / target
  const nice = [0.05, 0.1, 0.2, 0.5, 1, 2, 5, 10, 15, 30, 60, 120]
  const step = nice.find(n => n >= rawStep) ?? Math.ceil(rawStep)
  const first = Math.ceil(start / step) * step
  const ticks: { sec: number; pct: number; label: string }[] = []
  for (let t = first; t <= end + 1e-9; t += step) {
    ticks.push({
      sec: t,
      pct: secToPct(t),
      label: formatDuration(t),
    })
  }
  if (ticks.length === 0) {
    ticks.push({ sec: start, pct: 0, label: formatDuration(start) })
  }
  return ticks
})

function secToPct(sec: number) {
  const span = viewDuration.value || 0.001
  return ((sec - viewStart.value) / span) * 100
}

function formatDuration(sec: number) {
  const s = Math.max(0, sec)
  if (s < 1) {
    return `${Math.round(s * 1000)} ms`
  }
  if (s < 60) {
    const rounded = Math.round(s * 10) / 10
    return Number.isInteger(rounded) ? `${rounded} s` : `${rounded.toFixed(1)} s`
  }
  const mins = Math.floor(s / 60)
  const rem = s - mins * 60
  if (rem < 0.05) {
    return `${mins} min`
  }
  const remRounded = Math.round(rem * 10) / 10
  const remStr = Number.isInteger(remRounded) ? `${remRounded}` : remRounded.toFixed(1)
  return `${mins} min ${remStr} s`
}

function nextUniqueName(base: string) {
  const taken = new Set(staged.value.map(c => normalizeClipName(c.name)))
  if (!taken.has(normalizeClipName(base))) return base
  let i = 2
  while (taken.has(normalizeClipName(`${base} ${i}`))) i += 1
  return `${base} ${i}`
}

function clampView() {
  const maxStart = Math.max(0, displayDuration.value - viewDuration.value)
  viewStart.value = Math.min(Math.max(0, viewStart.value), maxStart)
}

function setCue(sec: number) {
  const dur = displayDuration.value
  const t = Math.min(Math.max(0, sec), Math.max(0, dur))
  cueSec.value = t
  if (!isPlaying.value || isPaused.value) {
    playheadSec.value = t
  }
}

async function ensureSettings() {
  if (!appSettings.loaded) await appSettings.load()
  await appSettings.applyAudioVolume()
  await appSettings.applyInputVolume()
}

function stopPlayheadAnim() {
  if (rafId) {
    cancelAnimationFrame(rafId)
    rafId = 0
  }
}

let lastPlayheadPaint = 0
function tickPlayhead() {
  if (!isPlaying.value || isPaused.value || !session.value) return
  if (document.hidden) {
    rafId = requestAnimationFrame(tickPlayhead)
    return
  }
  const now = performance.now()
  if (now - lastPlayheadPaint < 50) {
    rafId = requestAnimationFrame(tickPlayhead)
    return
  }
  lastPlayheadPaint = now
  playheadSec.value = Math.min(
    session.value.duration_secs,
    playOffset + (now - playStartedAt) / 1000,
  )
  // Keep playhead in view while playing
  if (playheadSec.value > viewEnd.value || playheadSec.value < viewStart.value) {
    viewStart.value = Math.max(0, playheadSec.value - viewDuration.value * 0.2)
    clampView()
    drawWave()
  }
  if (playheadSec.value >= session.value.duration_secs - 0.02) {
    isPlaying.value = false
    isPaused.value = false
    resetPlayheadToStart()
    return
  }
  rafId = requestAnimationFrame(tickPlayhead)
}

function startPlayheadFrom(offset: number) {
  playOffset = offset
  playStartedAt = performance.now()
  playheadSec.value = offset
  isPlaying.value = true
  isPaused.value = false
  stopPlayheadAnim()
  rafId = requestAnimationFrame(tickPlayhead)
}

function resetPlayheadToStart() {
  stopPlayheadAnim()
  isPlaying.value = false
  isPaused.value = false
  cueSec.value = 0
  playheadSec.value = 0
  playOffset = 0
}

function resetPlayheadToCue() {
  stopPlayheadAnim()
  isPlaying.value = false
  isPaused.value = false
  playheadSec.value = cueSec.value
  playOffset = cueSec.value
}

async function toggleRecord() {
  if (recording.value) {
    await stopRecord()
  } else {
    await startRecord()
  }
}

function startLivePoll() {
  stopLivePoll()
  livePollTimer = setInterval(async () => {
    if (!recording.value) return
    try {
      const buckets = Math.min(
        8192,
        Math.max(128, Math.floor(((waveWrap.value?.clientWidth || 600) * Math.min(zoom.value, 16)) / 2)),
      )
      const [dur, livePeaks] = await invoke<[number, number[]]>('get_live_record_peaks', {
        buckets,
        startSec: 0,
        endSec: null,
      })
      liveDuration.value = dur
      clampView()
      peaks.value = livePeaks
      drawWave()
    } catch {
      /* ignore transient poll errors */
    }
  }, 80)
}

function stopLivePoll() {
  if (livePollTimer) {
    clearInterval(livePollTimer)
    livePollTimer = null
  }
}

async function startRecord() {
  await ensureSettings()
  busy.value = true
  statusText.value = ''
  resetPlayheadToStart()
  try {
    await invoke('start_recording', {
      deviceName: appSettings.inputSource || 'default',
      hostName: appSettings.inputHost || null,
      loopback: !!appSettings.inputLoopback,
    })
    recording.value = true
    session.value = null
    liveDuration.value = 0
    peaks.value = []
    selStart.value = 0
    selEnd.value = 0
    zoom.value = 1
    viewStart.value = 0
    startLivePoll()
  } catch (e) {
    statusText.value = String(e)
    console.error(e)
  } finally {
    busy.value = false
  }
}

async function stopRecord() {
  busy.value = true
  stopLivePoll()
  try {
    const path = await invoke<string>('stop_recording')
    recording.value = false
    level.value = 0
    liveDuration.value = 0
    if (path) {
      await loadSession(path)
      dirty.value = true
    }
  } catch (e) {
    statusText.value = String(e)
    console.error(e)
  } finally {
    busy.value = false
  }
}

async function loadSession(path: string) {
  const info = await invoke<SessionInfo>('load_edit_session', { path })
  session.value = info
  selStart.value = 0
  selEnd.value = 0
  cueSec.value = 0
  playheadSec.value = 0
  zoom.value = 1
  viewStart.value = 0
  await refreshPeaks()
}

async function refreshPeaks() {
  if (!session.value) {
    if (!recording.value) {
      peaks.value = []
      drawWave()
    }
    return
  }
  clampView()
  const width = waveWrap.value?.clientWidth || 600
  // Full-track cache dense enough for current zoom — pan then only re-slices locally.
  const buckets = Math.min(
    8192,
    Math.max(128, Math.floor((width * Math.min(zoom.value, 16)) / 2)),
  )
  peaks.value = await invoke<number[]>('get_waveform_peaks', {
    sessionId: session.value.session_id,
    buckets,
    startSec: 0,
    endSec: session.value.duration_secs,
  })
  drawWave()
}

function drawWave() {
  const canvas = canvasRef.value
  const wrap = waveWrap.value
  if (!canvas || !wrap) return
  const dpr = window.devicePixelRatio || 1
  const w = wrap.clientWidth
  const h = wrap.clientHeight
  canvas.width = Math.floor(w * dpr)
  canvas.height = Math.floor(h * dpr)
  canvas.style.width = `${w}px`
  canvas.style.height = `${h}px`
  const ctx = canvas.getContext('2d')
  if (!ctx) return
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, w, h)
  ctx.fillStyle = 'rgba(0,0,0,0.15)'
  ctx.fillRect(0, 0, w, h)
  const mid = h / 2
  ctx.strokeStyle = getComputedStyle(document.documentElement).getPropertyValue('--primary_color').trim() || '#00d4ff'
  ctx.lineWidth = 1
  const data = peaks.value
  const totalBuckets = Math.floor(data.length / 2)
  if (totalBuckets === 0) return

  const dur = Math.max(displayDuration.value, 1e-6)
  const startB = Math.max(0, Math.floor((viewStart.value / dur) * totalBuckets))
  const endB = Math.min(
    totalBuckets,
    Math.max(startB + 1, Math.ceil((viewEnd.value / dur) * totalBuckets)),
  )
  const span = endB - startB
  for (let i = 0; i < span; i++) {
    const bi = startB + i
    const minV = data[bi * 2]
    const maxV = data[bi * 2 + 1]
    const x = (i / span) * w
    const y1 = mid + minV * mid * 0.9
    const y2 = mid + maxV * mid * 0.9
    ctx.beginPath()
    ctx.moveTo(x, y1)
    ctx.lineTo(x, y2)
    ctx.stroke()
  }
}

function secFromPointer(e: PointerEvent | WheelEvent) {
  const wrap = waveWrap.value
  if (!wrap) return 0
  const span = viewDuration.value
  if (span <= 0) return 0
  const rect = wrap.getBoundingClientRect()
  const x = Math.min(Math.max(0, e.clientX - rect.left), rect.width)
  return viewStart.value + (x / rect.width) * span
}

function onPlayheadHandleDown(e: PointerEvent) {
  if (!session.value || recording.value) return
  e.preventDefault()
  pointerDownX = e.clientX
  pointerMoved = false
  dragMode.value = 'playhead'
  if (isPlaying.value && !isPaused.value) {
    previewPause()
  }
  setCue(secFromPointer(e))
  ;(e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId)
}

function onSelHandleDown(which: 'start' | 'end', e: PointerEvent) {
  if (!session.value || recording.value) return
  e.preventDefault()
  // Normalize so start <= end before edge drag.
  const a = Math.min(selStart.value, selEnd.value)
  const b = Math.max(selStart.value, selEnd.value)
  selStart.value = a
  selEnd.value = b
  pointerDownX = e.clientX
  pointerMoved = false
  dragMode.value = which === 'start' ? 'sel-start' : 'sel-end'
  ;(e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId)
}

function onPointerDown(e: PointerEvent) {
  if (!session.value && !recording.value) return

  // Middle mouse: pan zoomed track
  if (e.button === 1) {
    e.preventDefault()
    if (zoom.value <= 1.01) return
    pointerDownX = e.clientX
    pointerMoved = false
    panOriginX = e.clientX
    panOriginViewStart = viewStart.value
    dragMode.value = 'pan'
    ;(e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId)
    return
  }

  if (e.button !== 0) return

  pointerDownX = e.clientX
  pointerMoved = false
  ;(e.currentTarget as HTMLElement).setPointerCapture?.(e.pointerId)

  // Drag in track = selection only (playhead via click or top handle).
  dragMode.value = 'select'
  const t = secFromPointer(e)
  selStart.value = t
  selEnd.value = t
}

function onPointerMove(e: PointerEvent) {
  if (dragMode.value === 'none') return
  if (Math.abs(e.clientX - pointerDownX) > CLICK_DRAG_PX) pointerMoved = true

  if (dragMode.value === 'pan') {
    const wrap = waveWrap.value
    const w = wrap?.clientWidth || 1
    const dx = e.clientX - panOriginX
    viewStart.value = panOriginViewStart - (dx / w) * viewDuration.value
    clampView()
    drawWave()
    return
  }

  if (dragMode.value === 'playhead') {
    setCue(secFromPointer(e))
    return
  }

  if (dragMode.value === 'sel-start') {
    const t = secFromPointer(e)
    selStart.value = Math.min(t, selEnd.value)
    return
  }

  if (dragMode.value === 'sel-end') {
    const t = secFromPointer(e)
    selEnd.value = Math.max(t, selStart.value)
    return
  }

  if (dragMode.value === 'select') {
    selEnd.value = secFromPointer(e)
  }
}

function onPointerUp(e: PointerEvent) {
  if (dragMode.value === 'none') return

  if (dragMode.value === 'pan') {
    dragMode.value = 'none'
    return
  }

  if (dragMode.value === 'playhead') {
    setCue(secFromPointer(e))
    dragMode.value = 'none'
    return
  }

  if (dragMode.value === 'sel-start' || dragMode.value === 'sel-end') {
    dragMode.value = 'none'
    return
  }

  // Click without drag → set playhead / cue only (no selection stretch).
  if (!pointerMoved && !recording.value) {
    setCue(secFromPointer(e))
    selStart.value = 0
    selEnd.value = 0
  } else {
    selEnd.value = secFromPointer(e)
  }
  dragMode.value = 'none'
}

function zoomAt(factor: number, anchorSec?: number) {
  if (!canZoom.value) return
  const anchor = anchorSec ?? (viewStart.value + viewDuration.value / 2)
  const prevDur = viewDuration.value
  const next = Math.min(ZOOM_MAX, Math.max(1, zoom.value * factor))
  if (Math.abs(next - zoom.value) < 1e-6) return
  zoom.value = next
  const newDur = displayDuration.value / zoom.value
  const rel = prevDur > 0 ? (anchor - viewStart.value) / prevDur : 0.5
  viewStart.value = anchor - rel * newDur
  clampView()
  refreshPeaks()
}

function zoomIn() {
  zoomAt(1.25, playheadSec.value || undefined)
}

function zoomOut() {
  zoomAt(1 / 1.25, playheadSec.value || undefined)
}

function onWheel(e: WheelEvent) {
  if (!canZoom.value) return
  if (e.shiftKey) {
    const wrap = waveWrap.value
    const w = wrap?.clientWidth || 1
    const delta = (e.deltaY / w) * viewDuration.value
    viewStart.value += delta
    clampView()
    drawWave()
    return
  }
  const anchor = secFromPointer(e)
  zoomAt(e.deltaY < 0 ? 1.15 : 1 / 1.15, anchor)
}

function onScrollRange(e: Event) {
  viewStart.value = Number((e.target as HTMLInputElement).value) || 0
  clampView()
  drawWave()
}

async function withBusy(fn: () => Promise<void>) {
  busy.value = true
  statusText.value = ''
  try {
    await fn()
    dirty.value = true
  } catch (e) {
    statusText.value = String(e)
    console.error(e)
  } finally {
    busy.value = false
    stemsOpen.value = false
  }
}

async function previewPlay() {
  if (!session.value) return
  await ensureSettings()
  const start = cueSec.value
  await invoke('preview_session', {
    sessionId: session.value.session_id,
    deviceName: appSettings.outputSource || 'default',
    hostName: appSettings.outputHost || null,
    startSec: start,
  })
  startPlayheadFrom(start)
}

async function previewPause() {
  if (isPaused.value) {
    await invoke('resume_preview')
    startPlayheadFrom(playheadSec.value)
  } else {
    await invoke('pause_preview')
    isPaused.value = true
    isPlaying.value = true
    playOffset = playheadSec.value
    stopPlayheadAnim()
  }
}

async function previewStop() {
  await invoke('stop_preview').catch(() => {})
  resetPlayheadToStart()
}

async function togglePlayStop() {
  if (!session.value || busy.value || recording.value) return
  if (isPlaying.value || isPaused.value) {
    await previewStop()
  } else {
    await previewPlay()
  }
}

/** Selection range for DSP ops; omitted keys = process full session. */
function effectRange(): { startSec?: number; endSec?: number } {
  if (!hasSelection.value) return {}
  return {
    startSec: Math.min(selStart.value, selEnd.value),
    endSec: Math.max(selStart.value, selEnd.value),
  }
}

async function doNormalize() {
  if (!session.value) return
  await previewStop().catch(() => {})
  await withBusy(async () => {
    session.value = await invoke<SessionInfo>('normalize_session', {
      sessionId: session.value!.session_id,
      targetPeakDb: -1,
      ...effectRange(),
    })
    await refreshPeaks()
  })
}

async function doDenoise() {
  if (!session.value) return
  statusText.value = 'Denoising…'
  await previewStop().catch(() => {})
  await withBusy(async () => {
    session.value = await invoke<SessionInfo>('denoise_session', {
      sessionId: session.value!.session_id,
      ...effectRange(),
    })
    await refreshPeaks()
  })
}

async function doStems(mode: string) {
  if (!session.value) return
  stemsOpen.value = false
  await previewStop().catch(() => {})

  try {
    stemsStatus.value = await invoke<StemsStatus>('get_stems_status')
  } catch (e) {
    statusText.value = String(e)
    return
  }

  // Model already cached and engine present → run directly.
  if (stemsStatus.value.available && stemsStatus.value.modelReady) {
    await runStemsSplit(mode)
    return
  }

  // Ask before downloading third-party model (or explain missing engine).
  stemsPendingMode.value = mode
  stemsDownloadPct.value = null
  stemsPrompt.value = true
}

async function cancelStemsPrompt() {
  if (stemsDownloading.value) return
  stemsPrompt.value = false
  stemsPendingMode.value = null
  stemsDownloadPct.value = null
  try {
    await invoke('dismiss_stems_intent')
    stemsStatus.value = await invoke<StemsStatus>('get_stems_status')
  } catch {
    /* ignore */
  }
}

async function openStemsModelPage() {
  const url = stemsStatus.value?.modelPageUrl || 'https://huggingface.co/xycld/BS-RoFormer-ONNX'
  try {
    await invoke('open_external_url', { url })
  } catch (e1) {
    try {
      await openUrl(url)
    } catch (e2) {
      console.error('open_external_url failed', e1, e2)
      statusText.value = String(e1)
    }
  }
}

async function confirmStemsDownload() {
  const mode = stemsPendingMode.value

  if (!stemsStatus.value?.available) {
    statusText.value = ''
    await openStemsModelPage()
    return
  }

  stemsDownloading.value = true
  stemsDownloadPct.value = 0
  statusText.value = ''
  try {
    await invoke('ensure_stems_model')
    stemsStatus.value = await invoke<StemsStatus>('get_stems_status')
    stemsPrompt.value = false
    stemsPendingMode.value = null
    stemsDownloadPct.value = null
    // First-run / Settings install: no pending mode — just finish download.
    if (mode) await runStemsSplit(mode)
  } catch (e) {
    const msg = String(e)
    statusText.value = msg.includes('STEMS_ENGINE_UNAVAILABLE')
      ? msg.replace(/^STEMS_ENGINE_UNAVAILABLE:\s*/, '')
      : msg
    console.error(e)
  } finally {
    stemsDownloading.value = false
  }
}

async function runStemsSplit(mode: string) {
  if (!session.value) return
  statusText.value = 'Preparing stems…'
  await withBusy(async () => {
    session.value = await invoke<SessionInfo>('split_session', {
      sessionId: session.value!.session_id,
      mode,
      ...effectRange(),
    })
    await refreshPeaks()
  })
}

async function doDeleteSelection() {
  if (!session.value || !hasSelection.value) return
  const a = Math.min(selStart.value, selEnd.value)
  const b = Math.max(selStart.value, selEnd.value)
  await previewStop().catch(() => {})
  await withBusy(async () => {
    session.value = await invoke<SessionInfo>('delete_range', {
      sessionId: session.value!.session_id,
      startSec: a,
      endSec: b,
    })
    selStart.value = 0
    selEnd.value = 0
    if (cueSec.value > (session.value?.duration_secs ?? 0)) {
      setCue(session.value?.duration_secs ?? 0)
    }
    clampView()
    await refreshPeaks()
  })
}

async function doTrim() {
  if (!session.value || !hasSelection.value) return
  const a = Math.min(selStart.value, selEnd.value)
  const b = Math.max(selStart.value, selEnd.value)
  await previewStop().catch(() => {})
  await withBusy(async () => {
    session.value = await invoke<SessionInfo>('trim_session', {
      sessionId: session.value!.session_id,
      startSec: a,
      endSec: b,
    })
    selStart.value = 0
    selEnd.value = 0
    setCue(0)
    zoom.value = 1
    viewStart.value = 0
    await refreshPeaks()
  })
}

async function doUndo() {
  if (!session.value) return
  await previewStop().catch(() => {})
  await withBusy(async () => {
    session.value = await invoke<SessionInfo>('undo_session', {
      sessionId: session.value!.session_id,
    })
    clampView()
    await refreshPeaks()
  })
}

async function doRedo() {
  if (!session.value) return
  await previewStop().catch(() => {})
  await withBusy(async () => {
    session.value = await invoke<SessionInfo>('redo_session', {
      sessionId: session.value!.session_id,
    })
    clampView()
    await refreshPeaks()
  })
}

function buildTabs(specific?: string) {
  const tabs = ['All']
  if (specific) {
    if (specific !== 'All') tabs.push(specific)
  } else if (importTab.value) {
    if (importTab.value !== 'All') tabs.push(importTab.value)
  } else if (currentTab.value && currentTab.value !== 'All') {
    tabs.push(currentTab.value)
  }
  return tabs
}

function safeFileName(name: string) {
  const cleaned = name.replace(/[<>:"/\\|?*\u0000-\u001f]/g, '_').trim() || 'recording'
  return cleaned.endsWith('.wav') ? cleaned : `${cleaned}.wav`
}

async function stageClip(startSec: number | null, endSec: number | null, label: string) {
  if (!session.value) return
  busy.value = true
  statusText.value = ''
  try {
    const info = await invoke<StagedClipInfo>('stage_session_clip', {
      sessionId: session.value.session_id,
      startSec,
      endSec,
    })
    clipCounter += 1
    staged.value.push({
      id: `clip_${Date.now()}_${clipCounter}`,
      name: label,
      path: info.path,
      duration: info.duration_secs,
      selected: true,
    })
    dirty.value = true
    statusText.value = ''
  } catch (e) {
    statusText.value = String(e)
    console.error(e)
  } finally {
    busy.value = false
  }
}

async function addSelectionToQueue() {
  if (!session.value || !hasSelection.value) return
  const a = Math.min(selStart.value, selEnd.value)
  const b = Math.max(selStart.value, selEnd.value)
  const label = nextUniqueName(`Clip ${clipCounter + 1}`)
  await stageClip(a, b, label)
}

async function addTrackToQueue() {
  if (!session.value) return
  const label = nextUniqueName(`Recording ${clipCounter + 1}`)
  await stageClip(null, null, label)
}

function selectAllStaged() {
  for (const c of staged.value) c.selected = true
}

function selectNoneStaged() {
  for (const c of staged.value) c.selected = false
}

async function removeStaged(id: string) {
  const idx = staged.value.findIndex(c => c.id === id)
  if (idx < 0) return
  const [removed] = staged.value.splice(idx, 1)
  if (removed?.path) {
    await invoke('delete_file_abs', { path: removed.path }).catch(() => {})
  }
}

async function previewStaged(clip: StagedClip) {
  await ensureSettings()
  await invoke('stop_preview').catch(() => {})
  try {
    await invoke('play_sound', {
      soundPath: clip.path,
      deviceName: appSettings.outputSource || 'default',
      hostName: appSettings.outputHost || null,
      active: false,
      overlap: true,
    })
  } catch (e) {
    statusText.value = String(e)
    console.error(e)
  }
}

async function importStaged(all: boolean) {
  if (all && !canImportAll.value) return
  if (!all && !canImportSelected.value) return
  const clips = all ? [...staged.value] : selectedStaged.value
  if (!clips.length) return

  const dir = await open({
    directory: true,
    multiple: false,
    title: 'Import folder',
  })
  if (!dir || typeof dir !== 'string') return

  busy.value = true
  statusText.value = ''
  const tabs = buildTabs()
  let ok = 0
  try {
    for (const clip of clips) {
      const sep = dir.includes('\\') && !dir.includes('/') ? '\\' : '/'
      const dest = `${dir.replace(/[\\/]+$/, '')}${sep}${safeFileName(clip.name)}`
      const path = await invoke<string>('copy_file_to_abs', {
        src: clip.path,
        dst: dest,
      })
      await emit('record_import_sound', { path, tabs, name: clip.name })
      ok += 1
    }
    // Remove imported clips from queue
    const ids = new Set(clips.map(c => c.id))
    const keep: StagedClip[] = []
    for (const c of staged.value) {
      if (ids.has(c.id)) {
        await invoke('delete_file_abs', { path: c.path }).catch(() => {})
      } else {
        keep.push(c)
      }
    }
    staged.value = keep
    if (!staged.value.length && !session.value) dirty.value = false
    statusText.value = `${ok} imported`
  } catch (e) {
    statusText.value = String(e)
    console.error(e)
  } finally {
    busy.value = false
  }
}

async function destroyEditorWindow() {
  closePrompt.value = false
  try {
    await getCurrentWindow().destroy()
  } catch (e) {
    console.warn('destroy record-editor failed', e)
  }
}

async function discardAndClose() {
  try {
    if (recording.value) await invoke('stop_recording').catch(() => {})
    await invoke('stop_preview').catch(() => {})
    for (const c of staged.value) {
      await invoke('delete_file_abs', { path: c.path }).catch(() => {})
    }
    staged.value = []
  } catch { /* ignore */ }
  await destroyEditorWindow()
}

function onKeyDown(e: KeyboardEvent) {
  const tag = (e.target as HTMLElement)?.tagName
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return

  const ctrl = e.ctrlKey || e.metaKey
  const key = e.key.toLowerCase()

  if (ctrl && e.shiftKey && key === 'z') {
    e.preventDefault()
    doRedo()
    return
  }
  if (ctrl && key === 'z') {
    e.preventDefault()
    doUndo()
    return
  }
  if (ctrl && key === 'a') {
    e.preventDefault()
    const dur = displayDuration.value
    if (dur > 0 && (session.value || recording.value)) {
      selStart.value = 0
      selEnd.value = dur
    }
    return
  }
  if (e.code === 'Space' || key === ' ') {
    e.preventDefault()
    togglePlayStop()
    return
  }
  if (key === 'delete' || key === 'backspace') {
    if (hasSelection.value && session.value) {
      e.preventDefault()
      doDeleteSelection()
    }
  }
}

onMounted(async () => {
  await ensureSettings()
  try {
    osPlatform.value = await platform()
  } catch {
    osPlatform.value = ''
  }
  try {
    stemsStatus.value = await invoke<StemsStatus>('get_stems_status')
    if (stemsStatus.value?.canPrompt) {
      stemsPendingMode.value = null
      stemsDownloadPct.value = null
      stemsPrompt.value = true
    }
  } catch {
    /* stems optional */
  }

  unlistenLevel = await listen<number>('record_level', (e) => {
    level.value = Number(e.payload) || 0
  })
  unlistenStems = await listen<StemsProgress>('stems_progress', (e) => {
    const p = e.payload
    if (!p) return
    statusText.value = p.message || p.stage
    if (p.stage === 'download' && stemsDownloading.value) {
      stemsDownloadPct.value = Number(p.percent) || 0
    }
  })
  unlistenStemsPct = await listen<number>('stems_model_progress', (e) => {
    if (!stemsDownloading.value) return
    stemsDownloadPct.value = Number(e.payload) || 0
  })
  unlistenCtx = await listen<{ currentTab: string; tabList: string[] }>('record_context', (e) => {
    if (!e?.payload) return
    currentTab.value = e.payload.currentTab || 'All'
    tabNames.value = e.payload.tabList || []
  })
  unlistenFinished = await listen('sound_finished', () => {
    if (isPlaying.value || isPaused.value) resetPlayheadToStart()
  })
  await emit('record_request_context')

  getCurrentWindow()
    .onCloseRequested((event) => {
      if (!dirty.value && !recording.value && !staged.value.length) return
      event.preventDefault()
      closePrompt.value = true
    })
    .catch(() => {})

  getCurrentWindow()
    .onFocusChanged((e) => {
      if (e.payload) emit('record_request_context').catch(() => {})
    })
    .catch(() => {})

  await nextTick()
  drawWave()
  window.addEventListener('resize', refreshPeaks)
  window.addEventListener('keydown', onKeyDown)
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', refreshPeaks)
  window.removeEventListener('keydown', onKeyDown)
  stopLivePoll()
  stopPlayheadAnim()
  if (unlistenLevel) unlistenLevel()
  if (unlistenStems) unlistenStems()
  if (unlistenStemsPct) unlistenStemsPct()
  if (unlistenCtx) unlistenCtx()
  if (unlistenFinished) unlistenFinished()
  if (recording.value) invoke('stop_recording').catch(() => {})
  invoke('stop_preview').catch(() => {})
})
</script>
