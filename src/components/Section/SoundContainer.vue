<template>
    <div class="SoundContainer">
        <div
            class="SoundTab flex_c_h flex_start gap1 flex_wrap"
            ref="dropZoneRef"
            @drop="onDrop($event)"
            @dragover.prevent
            @dragenter.prevent
        >
            <div
                v-for="(sound, soundindex) in JSONFile"
                class="Soundbtn flex_c_v flex_wrap"
                :class="{ active: sound.active }"
                :key="sound"
                :style="getBtnStyle(sound, soundindex)"
                ref="dragButton"
                draggable="true"
                @dragstart="onDragStart($event, sound)"
                @dragend="onDragEnd"
                @click="setActiveSound(soundindex)"
                @contextmenu.prevent="(e) => openSoundMenu(e, sound)"
            >
                <span class="sound-label">{{ sound.name }}</span>
            </div>
        </div>
    </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const dragButton = ref(null)
const dropZoneRef = ref(null)

const currentTab = computed(() => appStore.currentTab)

const JSONFile = computed(() => {
  const sortByIndex = (a, b) => a.index - b.index
  const filesToFilter = appStore.Searchbar.SearchbarActive
    ? jsonStore.filteredFiles
    : jsonStore.configFile?.files

  return filesToFilter
    ?.filter((sound) => sound.tabs.includes(currentTab.value))
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
function onDragStart(event, sound) {
  event.dataTransfer.setData('SoundID', sound.index)
  event.dataTransfer.effectAllowed = 'move'
  event.dataTransfer.dropEffect = 'move'
  event.target.style.opacity = '0.5'
}

function onDragEnd(event) {
  event.target.style.opacity = '1'
}

function onDrop(event) {
  const itemID = event.dataTransfer.getData('SoundID')
  console.log(itemID)
}

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
