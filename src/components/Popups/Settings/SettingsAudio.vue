<template>
  <section>
    <h2 class="settings-content__title">{{ $t('settings.audio.title') }}</h2>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.audio.outputDevice') }}</label>
      <div class="settings-row">
        <select v-model="outputSelected" @change="selectOutputDevice" class="settings-select">
          <option v-for="device in outputDevices" :key="device" :value="device">{{ device }}</option>
        </select>
        <button class="settings-btn settings-btn--icon" @click="loadOutputDevices" :title="$t('settings.audio.refresh')">&#x21BB;</button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'

const jsonStore = useJsonHandelingStore()
const appStore = useAppStore()

const outputDevices = ref<string[]>([])
const outputSelected = ref('')

async function loadOutputDevices() {
  const devices = await invoke<string[]>('get_out_devices')
  outputDevices.value = devices ?? []
}

function selectOutputDevice(event: Event) {
  jsonStore.setOutSource((event.target as HTMLSelectElement).value)
}

watch(() => appStore.activeOverlay, async (val) => {
  if (val !== 'settings') return
  await loadOutputDevices()
  const saved = jsonStore.configFile?.settings?.outputSource
  if (saved) outputSelected.value = saved
})

onMounted(async () => {
  await loadOutputDevices()
  const saved = jsonStore.configFile?.settings?.outputSource
  if (saved) outputSelected.value = saved
})
</script>
