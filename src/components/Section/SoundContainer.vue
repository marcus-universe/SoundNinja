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
                    ref="dragButton"
                    draggable="true"
                    @dragstart="onDragStart($event, sound)"
                    @dragend="onDragEnd"
                    @click="setActiveSound(soundindex)"
                    style=""
                >
                    {{ sound.name }}
                </div>
            </div>

            <template #fallback>
                <div>Is loading...</div>
            </template>
        </Suspense>
    </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'

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

function setActiveSound(soundindex) {
  if (!JSONFile.value[soundindex].active) {
    jsonStore.ReturnStatusAll()
    jsonStore.setActiveSound({ soundindex, status: true })
    invoke('play_sound', {
      soundPath: JSONFile.value[soundindex].path,
      deviceName: Settings.value.outputSource,
      active: false,
    })
  } else {
    jsonStore.ReturnStatusAll()
    jsonStore.setActiveSound({ soundindex, status: false })
    invoke('play_sound', {
      soundPath: JSONFile.value[soundindex].path,
      deviceName: Settings.value.outputSource,
      active: true,
    })
  }
}
</script>
