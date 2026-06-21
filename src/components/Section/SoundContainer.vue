<template>
    <div class="SoundContainer">
        <div
            class="SoundTab flex_c_h flex_start gap1 flex_wrap"
            ref="soundListRef"
        >
            <SoundButton
                v-for="(sound, soundindex) in JSONFile"
                :key="sound.path"
                :sound="sound"
                :btnStyle="getBtnStyle(sound, soundindex)"
                @play="setActiveSound(soundindex)"
                @contextmenu="(e) => openSoundMenu(e, sound)"
            />
        </div>
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

onMounted(() => {
  sortable = Sortable.create(soundListRef.value, {
    animation: 180,
    draggable: '.Soundbtn',
    ghostClass: 'drag-over',
    onEnd(evt) {
      const { oldIndex, newIndex, item, from } = evt
      if (oldIndex === newIndex || oldIndex == null || newIndex == null) return
      // Undo SortableJS's DOM mutation so Vue stays the single source of truth.
      from.removeChild(item)
      from.insertBefore(item, from.children[oldIndex] ?? null)
      const list = JSONFile.value
      const fromSound = list[oldIndex]
      const toSound = list[newIndex]
      if (fromSound && toSound) {
        jsonStore.reorderSounds(fromSound.index, toSound.index, currentTab.value)
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

// ---- Styling helper ----
function getBtnStyle(sound, soundindex) {
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
async function setActiveSound(soundindex) {
  const sound = JSONFile.value[soundindex]
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
    try {
      duration = await invoke('get_sound_duration', { soundPath: sound.path })
    } catch (e) {
      console.error('Could not get sound duration', e)
    }

    if (!sound.active) return

    if (duration > 0) {
      startProgress(sound.index, duration)
    }

    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: Settings.value.outputSource,
      active: false,
      overlap: overlapSounds,
    }).catch((e) => console.error('Sound playback error', e))
  } else {
    if (!stopOnRetrigger) return
    stopProgress(sound.index)
    jsonStore.setActiveSound({ soundindex: fileArrayIndex, status: false })
    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: Settings.value.outputSource,
      active: true,
      overlap: overlapSounds,
    }).catch((e) => console.error('Stop error', e))
  }
}
</script>
