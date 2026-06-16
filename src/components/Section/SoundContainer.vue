<template>
    <div class="SoundContainer">
        <Suspense>
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
                    :style="
                        sound.active && playingInfo?.soundFileIndex === sound.index
                            ? { '--sound-progress': progressPercent + '%' }
                            : {}
                    "
                    ref="dragButton"
                    draggable="true"
                    @dragstart="onDragStart($event, sound)"
                    @dragend="onDragEnd"
                    @click="setActiveSound(soundindex)"
                >
                    <span class="sound-label">{{ sound.name }}</span>
                </div>
            </div>

            <template #fallback>
                <div>Is loading...</div>
            </template>
        </Suspense>
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
  return jsonStore.configFile?.files
    ?.filter((sound) => sound.tabs.includes(currentTab.value))
    .sort(sortByIndex)
})

const Settings = computed(() => jsonStore.configFile?.settings)

// ---- Progress bar state ----
const playingInfo = ref(null) // { soundFileIndex, duration, startTime }
const progressPercent = ref(0)
let rafId = null

function tickProgress() {
  if (!playingInfo.value) return
  const elapsed = (Date.now() - playingInfo.value.startTime) / 1000
  progressPercent.value = Math.min(100, (elapsed / playingInfo.value.duration) * 100)
  if (progressPercent.value < 100) {
    rafId = requestAnimationFrame(tickProgress)
  }
}

function stopProgress() {
  if (rafId) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
  playingInfo.value = null
  progressPercent.value = 0
}

// ---- Tauri event listeners ----
let unlistenFinished = null

onMounted(async () => {
  unlistenFinished = await listen('sound_finished', () => {
    stopProgress()
    jsonStore.ReturnStatusAll()
  })
})

onUnmounted(() => {
  stopProgress()
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
  // Resolve the true index in the full files array (not the filtered/sorted view).
  const fileArrayIndex = jsonStore.configFile.files.indexOf(sound)

  if (!sound.active) {
    jsonStore.ReturnStatusAll()
    jsonStore.setActiveSound({ soundindex: fileArrayIndex, status: true })

    // Fetch duration for the progress bar before starting playback.
    let duration = 0
    try {
      duration = await invoke('get_sound_duration', { soundPath: sound.path })
    } catch (e) {
      console.error('Could not get sound duration', e)
    }

    // Guard: another sound may have been activated while we awaited.
    if (!sound.active) return

    stopProgress()
    if (duration > 0) {
      playingInfo.value = { soundFileIndex: sound.index, duration, startTime: Date.now() }
      rafId = requestAnimationFrame(tickProgress)
    }

    // play_sound now returns immediately; completion is signalled via sound_finished event.
    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: Settings.value.outputSource,
      active: false,
    }).catch((e) => console.error('Sound playback error', e))
  } else {
    stopProgress()
    jsonStore.ReturnStatusAll()
    jsonStore.setActiveSound({ soundindex: fileArrayIndex, status: false })
    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: Settings.value.outputSource,
      active: true,
    }).catch((e) => console.error('Stop error', e))
  }
}
</script>
