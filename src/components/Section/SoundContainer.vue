<template>
    <div class="SoundContainer">
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
                    :data-sound-path="item.sound.path"
                    @play="onSoundClick(item.sound)"
                    @contextmenu="(e) => openSoundMenu(e, item.sound)"
                />
            </template>
        </div>
        </div>

        <Transition name="fade">
            <div v-if="appStore.multiSelectActive" class="bulk-bar flex_c_h align_c gap1">
                <span class="bulk-bar__count">{{ $t('bulk.selected', { count: appStore.selectedSoundPaths.length }) }}</span>
                <label class="bulk-bar__color">
                    {{ $t('bulk.color') }}
                    <input type="color" v-model="bulkColor" @change="applyBulkColor" />
                </label>
                <select class="bulk-bar__select" v-model="bulkTab" @change="applyBulkTab">
                    <option value="">{{ $t('bulk.moveToTab') }}</option>
                    <option v-for="t in tabOptions" :key="t" :value="t">{{ t }}</option>
                </select>
                <button class="bulk-bar__btn bulk-bar__btn--danger" :disabled="appStore.selectedSoundPaths.length === 0" @click="applyBulkDelete">
                    {{ $t('bulk.delete') }}
                </button>
                <button class="bulk-bar__btn" @click="appStore.setMultiSelectActive(false)">{{ $t('bulk.done') }}</button>
            </div>
        </Transition>
    </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import Sortable from 'sortablejs'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const soundListRef = ref(null)
let sortable = null

// P5: sounds currently resolving their duration / starting playback.
const loadingPaths = reactive(new Set())
// P8: enforced uniform button height (0 = natural height).
const uniformHeight = ref(0)
// P7: multi-select bulk-edit controls.
const bulkColor = ref('#7184a2')
const bulkTab = ref('')

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
  if (sound.color) {
    style['--btn-accent'] = sound.color
  }
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
// Map<soundFileIndex, { duration, startTime, percent }> — tracks every playing sound.
const playingSounds = reactive(new Map())
let rafId = null

function tickProgress() {
  if (playingSounds.size === 0) {
    rafId = null
    return
  }
  const now = Date.now()
  for (const info of playingSounds.values()) {
    info.percent = Math.min(100, ((now - info.startTime) / 1000 / info.duration) * 100)
  }
  rafId = requestAnimationFrame(tickProgress)
}

function startProgress(soundFileIndex, duration) {
  playingSounds.set(soundFileIndex, { duration, startTime: Date.now(), percent: 0 })
  if (rafId === null) {
    rafId = requestAnimationFrame(tickProgress)
  }
}

function stopProgress(soundFileIndex) {
  playingSounds.delete(soundFileIndex)
  if (playingSounds.size === 0 && rafId !== null) {
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

// ---- Tauri event listeners ----
let unlistenFinished = null

onMounted(async () => {
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
})

onUnmounted(() => {
  stopAllProgress()
  if (unlistenFinished) unlistenFinished()
})

// ---- Drag & drop ----
// (Handled per-button in SoundButton.vue via useDropZone)

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
function applyBulkColor() {
  const paths = appStore.selectedSoundPaths
  if (paths.length) jsonStore.setSoundColorMany(paths, bulkColor.value)
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

    let duration = 0
    loadingPaths.add(sound.path)
    try {
      duration = await invoke('get_sound_duration', { soundPath: sound.path })
    } catch (e) {
      console.error('Could not get sound duration', e)
    }

    if (!sound.active) {
      loadingPaths.delete(sound.path)
      return
    }

    if (duration > 0) {
      startProgress(sound.index, duration)
    }

    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: Settings.value.outputSource,
      hostName: Settings.value.outputHost ?? null,
      active: false,
      overlap: overlapSounds,
    })
      .catch((e) => console.error('Sound playback error', e))
      .finally(() => loadingPaths.delete(sound.path))
  } else {
    if (!stopOnRetrigger) return
    stopProgress(sound.index)
    loadingPaths.delete(sound.path)
    jsonStore.setActiveSound({ soundindex: fileArrayIndex, status: false })
    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: Settings.value.outputSource,
      hostName: Settings.value.outputHost ?? null,
      active: true,
      overlap: overlapSounds,
    }).catch((e) => console.error('Stop error', e))
  }
}
</script>
