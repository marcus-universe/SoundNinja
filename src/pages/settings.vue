<template>
  <div class="main settings flex_c_v align_c">
    <h1>Settings</h1>
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
          <option
            v-for="device in OutputDevices[0]"
            :key="device"
            :value="device"
          >
            {{ device }}
          </option>
        </select>
      </div>
    </div>
  </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'

const jsonStore = useJsonHandelingStore()

const OutputDevices = ref([])
const outputSelected = ref('')

const hueState = computed(() => jsonStore.configFile?.settings?.hue)
const OutputState = computed(() => jsonStore.configFile?.settings?.outputSource)

const hue = ref(hueState.value ?? 189)

onUpdated(() => {
  hue.value = hueState.value
})

onMounted(() => {
  const outputVal = OutputState.value
  if (outputVal !== undefined && outputVal !== null && outputVal !== '') {
    outputSelected.value = outputVal
  }
})

watch(hue, (hueval) => {
  jsonStore.setHue(parseInt(hueval))
})

async function getOutputDevices() {
  const devices = [await invoke('get_out_devices')]
  OutputDevices.value.push(devices[0])
  return devices
}

getOutputDevices()

const selectOutputDevice = (event) => {
  const selectedDevice = event.target.value
  jsonStore.setOutSource(selectedDevice)
}
</script>
