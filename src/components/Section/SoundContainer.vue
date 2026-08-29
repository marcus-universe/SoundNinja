<template>
    <div
        class="SoundContainer"
        :class="{
          'SoundContainer--player-large': showPlayer && playerLarge,
          'SoundContainer--bulk': appStore.multiSelectActive,
        }"
    >
        <div class="SoundContainer__scroll">
        <div
            class="SoundTab flex_c_h flex_start button-gaps flex_wrap"
            ref="soundListRef"
            :style="uniformHeight ? { '--btn-min-height': uniformHeight + 'px' } : {}"
        >
            <template v-for="item in displayItems" :key="item.domKey">
                <div
                    v-if="item.kind === 'sep'"
                    class="tab-separator"
                    :data-sep-id="item.sep.id"
                    @contextmenu.prevent="(e) => openSeparatorMenu(e, item.sep)"
                />
                <SoundButton
                    v-else
                    :sound="item.sound"
                    :btnStyle="getBtnStyle(item.sound)"
                    :loading="loadingPaths.has(item.sound.path)"
                    :selected="appStore.multiSelectActive && appStore.selectedSoundPaths.includes(item.sound.path)"
                    :gifSrc="gifSrcFor(item.sound)"
                    :gifPosX="item.sound.gifPosX ?? 50"
                    :gifPosY="item.sound.gifPosY ?? 50"
                    :hasGif="!!item.sound.gifId"
                    :data-sound-path="item.sound.path"
                    @play="onSoundClick(item.sound)"
                    @contextmenu="(e) => openSoundMenu(e, item.sound)"
                    @gifhover="(on) => onGifHover(item.sound, on)"
                />
            </template>
        </div>
        </div>

        <Transition name="fade">
            <div v-if="appStore.multiSelectActive" class="bulk-bar flex_c_h align_c">
                <span class="bulk-bar__count">{{ $t('bulk.selected', { count: appStore.selectedSoundPaths.length }) }}</span>
                <div class="bulk-bar__color">
                    <ColorGroupPicker
                        :model-value="bulkOverride"
                        :base-colors="bulkBaseColors"
                        :title="$t('bulk.color')"
                        placement="bottom-right"
                        @change="onBulkOverride"
                    />
                </div>
                <select class="bulk-bar__select" v-model="bulkTab" @change="applyBulkTab">
                    <option value="">{{ $t('bulk.moveToTab') }}</option>
                    <option v-for="t in tabOptions" :key="t" :value="t">{{ t }}</option>
                </select>
                <button class="bulk-bar__btn bulk-bar__btn--danger" :disabled="appStore.selectedSoundPaths.length === 0" @click="applyBulkDelete">
                    {{ $t('bulk.delete') }}
                </button>
                <button class="bulk-bar__btn bulk-bar__btn--primary" @click="appStore.setMultiSelectActive(false)">{{ $t('bulk.done') }}</button>
            </div>
        </Transition>

        <PlayerBar v-if="showPlayer" />
    </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import Sortable from 'sortablejs'
import { parseOverride, serializeOverride, resolveEffectiveColors } from '~/utils/colorOverride'
import { getDb } from '~/utils/db'
import { ensureGifUrls, peekGifUrls } from '~/utils/gifCache'

const MAX_ANIM_GIFS = 16

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const soundListRef = ref(null)
let sortable = null

const showPlayer = computed(() => jsonStore.configFile?.settings?.showPlayer !== false)
const playerLarge = computed(() => jsonStore.configFile?.settings?.playerLarge === true)

// P5: sounds currently resolving their duration / starting playback.
const loadingPaths = reactive(new Set())
// P8: enforced uniform button height (0 = natural height).
const uniformHeight = ref(0)
// P7: multi-select bulk-edit controls.
const bulkOverride = ref({})
const bulkTab = ref('')
const bulkBaseColors = computed(() => {
  void appStore.multiSelectActive
  const s = jsonStore.configFile?.settings
  void s?.theme
  void s?.btnBg
  void s?.btnBorder
  void s?.primaryColor
  return resolveEffectiveColors(bulkOverride.value || {}, 'button')
})

onMounted(() => {
  sortable = Sortable.create(soundListRef.value, {
    animation: 180,
    disabled: jsonStore.configFile?.settings?.allowReorder === false,
    draggable: '.Soundbtn, .tab-separator',
    ghostClass: 'drag-over',
    onEnd(evt) {
      const { oldIndex, newIndex, item, from } = evt
      if (oldIndex === newIndex || oldIndex == null || newIndex == null) return
      // Undo SortableJS's DOM mutation so Vue stays the single source of truth.
      from.removeChild(item)
      from.insertBefore(item, from.children[oldIndex] ?? null)

      const items = displayItems.value
      const moved = items[oldIndex]
      if (!moved) return

      // Rebuild the intended order to read the moved item's new neighbors.
      const arr = items.slice()
      arr.splice(oldIndex, 1)
      arr.splice(newIndex, 0, moved)
      const prev = arr[newIndex - 1]
      const next = arr[newIndex + 1]

      if (moved.kind === 'sep') {
        // Reposition the separator between its new neighbors.
        let pos
        if (prev && next) pos = (orderOf(prev) + orderOf(next)) / 2
        else if (prev) pos = orderOf(prev) + 0.5
        else if (next) pos = orderOf(next) - 0.5
        else pos = 0
        jsonStore.setSeparatorPosition(moved.sep.id, pos)
      } else {
        // Sound moved: reorder relative to the nearest sound neighbor so the
        // existing per-tab reorder logic (and drag/drop) is unaffected.
        const neighbor =
          (next && next.kind === 'sound' && next) ||
          (prev && prev.kind === 'sound' && prev) ||
          null
        if (neighbor && neighbor.sound !== moved.sound) {
          jsonStore.reorderSounds(moved.sound.index, neighbor.sound.index, currentTab.value)
        }
      }
    },
  })
})

onUnmounted(() => {
  sortable?.destroy()
  sortable = null
  gifObserver?.disconnect()
  gifObserver = null
})

const currentTab = computed(() => appStore.currentTab)

const JSONFile = computed(() => {
  const tab = currentTab.value
  const sortByIndex =
    tab === 'All'
      ? (a, b) => a.index - b.index
      : (a, b) => (a.tabIndexes?.[tab] ?? 0) - (b.tabIndexes?.[tab] ?? 0)
  const filesToFilter = appStore.Searchbar.SearchbarActive
    ? jsonStore.filteredFiles
    : jsonStore.configFile?.files

  return filesToFilter
    ?.filter((sound) => sound.tabs.includes(tab))
    .sort(sortByIndex)
})

const Settings = computed(() => jsonStore.configFile?.settings)

const gifPlayOnHover = computed(() => Settings.value?.gifPlayOnHover !== false)
const visibleByPath = reactive({})
const hoveredPath = ref(null)
const gifUrls = reactive({})
let gifObserver = null

function onGifHover(sound, on) {
  if (on) hoveredPath.value = sound.path
  else if (hoveredPath.value === sound.path) hoveredPath.value = null
}

function shouldAnimate(sound) {
  if (!sound.gifId) return false
  if (!visibleByPath[sound.path]) return false
  if (gifPlayOnHover.value) return hoveredPath.value === sound.path
  return true
}

const animatingPaths = computed(() => {
  const paths = []
  for (const item of displayItems.value) {
    if (item.kind !== 'sound') continue
    if (!shouldAnimate(item.sound)) continue
    paths.push(item.sound.path)
    if (paths.length >= MAX_ANIM_GIFS) break
  }
  return new Set(paths)
})

function gifSrcFor(sound) {
  if (!sound.gifId) return ''
  const urls = gifUrls[sound.gifId] || peekGifUrls(sound.gifId)
  if (!urls) return ''
  if (animatingPaths.value.has(sound.path)) return urls.animUrl
  return urls.posterUrl
}

async function loadVisibleGifs() {
  const d = getDb()
  if (!d) return
  for (const item of displayItems.value) {
    if (item.kind !== 'sound' || !item.sound.gifId) continue
    if (!visibleByPath[item.sound.path]) continue
    if (gifUrls[item.sound.gifId]) continue
    try {
      const urls = await ensureGifUrls(d, item.sound.gifId)
      if (urls) gifUrls[item.sound.gifId] = urls
    } catch (e) {
      console.error('Failed to load GIF blob', e)
    }
  }
}

function setupGifObserver() {
  gifObserver?.disconnect()
  const root = soundListRef.value?.closest('.SoundContainer__scroll') || null
  gifObserver = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        const path = e.target.getAttribute('data-sound-path')
        if (!path) continue
        if (e.isIntersecting) visibleByPath[path] = true
        else delete visibleByPath[path]
      }
      loadVisibleGifs()
    },
    { root, rootMargin: '80px', threshold: 0.01 },
  )
  nextTick(() => {
    soundListRef.value?.querySelectorAll('.Soundbtn').forEach((el) => gifObserver.observe(el))
  })
}

// P8: enable/disable drag reordering based on the per-project setting.
watch(
  () => Settings.value?.allowReorder,
  (v) => sortable?.option('disabled', v === false),
)

// P8: measure the tallest button and enforce a uniform height when enabled.
function updateUniformHeight() {
  if (!Settings.value?.uniformButtonHeight) {
    uniformHeight.value = 0
    return
  }
  uniformHeight.value = 0
  nextTick(() => {
    const el = soundListRef.value
    if (!el) return
    let max = 0
    el.querySelectorAll('.Soundbtn').forEach((b) => {
      max = Math.max(max, b.offsetHeight)
    })
    uniformHeight.value = max
  })
}

// P7: available tab names for the "move to tab" bulk action.
const tabOptions = computed(() => (jsonStore.configFile?.tabList ?? []).map((t) => t.name).filter(Boolean))

// ---- Separators ----
// Per-tab order key for a sound (global index in "All", per-tab index elsewhere).
function tabOrder(sound) {
  return currentTab.value === 'All' ? sound.index : (sound.tabIndexes?.[currentTab.value] ?? 0)
}

function orderOf(item) {
  return item.kind === 'sep' ? item.sep.position : tabOrder(item.sound)
}

// Sounds + separators for the current tab, interleaved by their order key.
const displayItems = computed(() => {
  const sounds = JSONFile.value ?? []
  const seps = (jsonStore.separators ?? []).filter((s) => s.tab === currentTab.value)
  const items = [
    ...sounds.map((sound) => ({ kind: 'sound', sound, domKey: 's:' + sound.path })),
    ...seps.map((sep) => ({ kind: 'sep', sep, domKey: 'sep:' + sep.id })),
  ]
  items.sort((a, b) => orderOf(a) - orderOf(b))
  return items
})

watch(
  [
    () => displayItems.value.map((i) => i.domKey).join('|'),
    () => jsonStore.configFile.files.map((f) => f.gifId || '').join('|'),
  ],
  () => {
    for (const f of jsonStore.configFile.files) {
      if (!f.gifId || gifUrls[f.gifId]) continue
      const urls = peekGifUrls(f.gifId)
      if (urls) gifUrls[f.gifId] = urls
    }
    setupGifObserver()
    loadVisibleGifs()
  },
  { flush: 'post' },
)

// P8: re-measure uniform button height when the setting or the item list changes.
watch(
  [() => Settings.value?.uniformButtonHeight, () => displayItems.value.length],
  () => updateUniformHeight(),
  { flush: 'post' },
)

function openSeparatorMenu(event, sep) {
  appStore.openContextMenu({
    x: event.clientX,
    y: event.clientY,
    type: 'separator',
    targetName: sep.id,
    targetIndex: -1,
  })
}

// ---- Styling helper ----
function getBtnStyle(sound) {
  const style = {}
  const info = playingSounds.get(sound.index)
  if (sound.active && info) {
    style['--sound-progress'] = info.percent + '%'
  }
  const o = parseOverride(sound.color)
  if (o.bg) style['--color-btn'] = o.bg
  if (o.bgHover) style['--btn-bg-hover'] = o.bgHover
  if (o.text) style['--sound-text'] = o.text
  if (o.textHover) style['--btn-text-hover'] = o.textHover
  if (o.border) style['--btn-border'] = o.border
  if (o.borderHover) style['--btn-border-hover'] = o.borderHover
  return style
}

// ---- Context menu ----
function openSoundMenu(event, sound) {
  const fileArrayIndex = jsonStore.configFile.files.indexOf(sound)
  appStore.openContextMenu({
    x: event.clientX,
    y: event.clientY,
    type: 'sound',
    targetName: sound.name,
    targetIndex: fileArrayIndex,
  })
}

// ---- Progress bar state ----
// Map<soundFileIndex, { duration, startTime, percent, paused, elapsedMs }>
const playingSounds = reactive(new Map())
let rafId = null

function anyProgressRunning() {
  for (const info of playingSounds.values()) {
    if (!info.paused) return true
  }
  return false
}

function tickProgress() {
  if (!anyProgressRunning()) {
    rafId = null
    return
  }
  const now = Date.now()
  for (const info of playingSounds.values()) {
    if (info.paused || !(info.duration > 0)) continue
    info.percent = Math.min(
      100,
      ((now - info.startTime) / 1000 / info.duration) * 100,
    )
  }
  rafId = requestAnimationFrame(tickProgress)
}

function ensureProgressLoop() {
  if (rafId === null && anyProgressRunning()) {
    rafId = requestAnimationFrame(tickProgress)
  }
}

function startProgress(soundFileIndex, duration) {
  playingSounds.set(soundFileIndex, {
    duration,
    startTime: Date.now(),
    percent: 0,
    paused: false,
    elapsedMs: 0,
  })
  ensureProgressLoop()
}

function stopProgress(soundFileIndex) {
  playingSounds.delete(soundFileIndex)
  if (!anyProgressRunning() && rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
}

function stopAllProgress() {
  playingSounds.clear()
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
}

/** Freeze / unfreeze button progress when backend pause/resume/seek/loop changes. */
function syncProgressPauseState(list) {
  const files = jsonStore.configFile?.files || []
  const byPath = new Map((list || []).map((p) => [p.path, p]))

  for (const [fileIndex, info] of playingSounds) {
    const file = files.find((f) => f.index === fileIndex)
    if (!file || !byPath.has(file.path)) continue
    const snap = byPath.get(file.path)
    const shouldPause = !!snap.paused
    const posSec = Number(snap.positionSecs)
    if (Number.isFinite(posSec) && info.duration > 0) {
      info.elapsedMs = Math.max(0, posSec * 1000)
      info.percent = Math.min(100, (info.elapsedMs / 1000 / info.duration) * 100)
      info.startTime = Date.now() - info.elapsedMs
    }

    if (shouldPause && !info.paused) {
      if (!Number.isFinite(posSec)) {
        info.elapsedMs = Math.max(0, Date.now() - info.startTime)
        info.percent = Math.min(100, (info.elapsedMs / 1000 / info.duration) * 100)
      }
      info.paused = true
    } else if (!shouldPause && info.paused) {
      info.startTime = Date.now() - (info.elapsedMs || 0)
      info.paused = false
    } else if (!shouldPause) {
      info.paused = false
    }
  }

  if (!anyProgressRunning() && rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  } else {
    ensureProgressLoop()
  }
}

// ---- Tauri event listeners ----
let unlistenFinished = null
let unlistenPlaying = null

onMounted(async () => {
  setupGifObserver()
  unlistenFinished = await listen('sound_finished', (event) => {
    const path = event.payload
    const idx = jsonStore.configFile.files.findIndex((f) => f.path === path)
    if (idx !== -1) {
      const fileIndex = jsonStore.configFile.files[idx].index
      stopProgress(fileIndex)
      jsonStore.setActiveSound({ soundindex: idx, status: false })
    } else {
      // Fallback: path not matched, clear all active state
      stopAllProgress()
      jsonStore.ReturnStatusAll()
    }
  })
  unlistenPlaying = await listen('playing_changed', (event) => {
    syncProgressPauseState(event.payload ?? [])
  })
})

onUnmounted(() => {
  stopAllProgress()
  if (unlistenFinished) unlistenFinished()
  if (unlistenPlaying) unlistenPlaying()
})

// ---- Drag & drop ----
// (Handled per-button in SoundButton.vue)

// ---- Sound playback ----
// P7: in multi-select mode a click toggles selection instead of playing.
function onSoundClick(sound) {
  if (appStore.multiSelectActive) {
    appStore.toggleSoundSelection(sound.path)
    return
  }
  setActiveSound(sound)
}

// ---- P7 bulk actions ----
function onBulkOverride(override) {
  bulkOverride.value = override
  const paths = appStore.selectedSoundPaths
  if (paths.length) jsonStore.setSoundColorMany(paths, serializeOverride(override))
}

function applyBulkTab() {
  const tab = bulkTab.value
  const paths = appStore.selectedSoundPaths
  if (tab && paths.length) jsonStore.setSoundTabsMany(paths, tab)
  bulkTab.value = ''
}

function applyBulkDelete() {
  const paths = appStore.selectedSoundPaths
  if (!paths.length) return
  jsonStore.removeSoundsMany(paths)
  appStore.clearSoundSelection()
}

async function setActiveSound(sound) {
  const fileArrayIndex = jsonStore.configFile.files.indexOf(sound)
  const overlapSounds = Settings.value.overlapSounds ?? false
  const stopOnRetrigger = Settings.value.stopOnRetrigger ?? true

  if (!sound.active) {
    if (!overlapSounds) {
      jsonStore.ReturnStatusAll()
      stopAllProgress()
    }
    jsonStore.setActiveSound({ soundindex: fileArrayIndex, status: true })

    loadingPaths.add(sound.path)
    // Start audio immediately — do not wait on duration probe (slow on long files).
    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: appSettings.outputSource,
      hostName: appSettings.outputHost || null,
      active: false,
      overlap: overlapSounds,
    })
      .catch((e) => console.error('Sound playback error', e))
      .finally(() => loadingPaths.delete(sound.path))

    invoke('get_sound_duration', { soundPath: sound.path })
      .then((duration) => {
        if (!sound.active) return
        if (typeof duration === 'number' && duration > 0) {
          startProgress(sound.index, duration)
        }
      })
      .catch((e) => console.error('Could not get sound duration', e))
  } else {
    if (!stopOnRetrigger) return
    stopProgress(sound.index)
    loadingPaths.delete(sound.path)
    jsonStore.setActiveSound({ soundindex: fileArrayIndex, status: false })
    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: appSettings.outputSource,
      hostName: appSettings.outputHost || null,
      active: true,
      overlap: overlapSounds,
    }).catch((e) => console.error('Stop error', e))
  }
}
</script>
