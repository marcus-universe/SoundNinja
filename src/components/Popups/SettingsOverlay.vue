<template>
  <Transition name="overlay-fade">
    <div v-if="appStore.activeOverlay === 'settings'" class="overlay" @click.self="appStore.setActiveOverlay(null)">
      <div class="overlay-panel flex_c_v align_c">
        <div class="overlay-header flex_c_h flex_space_between w100">
          <h1>Settings</h1>
          <Icons :icon="'exit'" :customClass="'exit overlay-close'" @triggered="appStore.setActiveOverlay(null)" />
        </div>
        <div class="settingsContainer gap1 flex_c_v align_c">
          <div class="flex_c gap1 w100 setting">
            <label for="hue">Primary Hue:</label>
            <input
              type="range"
              id="hue"
              name="hue"
              min="0"
              max="360"
              v-model="hue"
            />
          </div>
          <div class="flex_c gap1 w100 setting">
            <label>Select Output-Device</label>
            <select v-model="outputSelected" @change="selectOutputDevice">
              <option v-for="device in OutputDevices[0]" :key="device" :value="device">
                {{ device }}
              </option>
            </select>
          </div>
        </div>
      </div>
      <BlurBG />
    </div>
  </Transition>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/tauri'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const OutputDevices = ref([])
const outputSelected = ref('')

const hueState = computed(() => jsonStore.configFile?.settings?.hue)
const OutputState = computed(() => jsonStore.configFile?.settings?.outputSource)
const hue = ref(hueState.value ?? 189)

watch(() => appStore.activeOverlay, (val) => {
  if (val === 'settings') {
    hue.value = hueState.value
    const outputVal = OutputState.value
    if (outputVal) outputSelected.value = outputVal
  }
})

watch(hue, (hueval) => {
  jsonStore.setHue(parseInt(hueval))
})

async function getOutputDevices() {
  const devices = [await invoke('get_out_devices')]
  OutputDevices.value = [devices[0]]
}

onMounted(() => {
  getOutputDevices()
})

const selectOutputDevice = (event) => {
  jsonStore.setOutSource(event.target.value)
}
</script>
