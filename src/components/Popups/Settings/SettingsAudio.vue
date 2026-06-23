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

    <div class="settings-group settings-group--stacked">
      <div class="settings-slider-header">
        <div class="settings-label-info">
          <label class="settings-label">{{ $t('settings.audio.outputVolume') }}</label>
          <span class="settings-info-icon">
            <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/></svg>
            <span class="settings-info-icon__tip">{{ $t('settings.audio.outputVolumeHint') }}</span>
          </span>
        </div>
        <div class="settings-unit-input">
          <input
            type="number"
            class="settings-input"
            min="0"
            max="100"
            step="1"
            v-model.number="outputVolumePct"
            @change="onVolumeChange"
          />
          <span class="settings-unit-label">%</span>
        </div>
      </div>
      <input
        type="range"
        class="settings-slider"
        min="0"
        max="100"
        step="1"
        v-model.number="outputVolumePct"
        @change="onVolumeChange"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'

const jsonStore = useJsonHandelingStore()
const appStore = useAppStore()

const outputDevices = ref<string[]>([])
const outputSelected = ref('')
const outputVolumePct = ref(100)

async function loadOutputDevices() {
  const devices = await invoke<string[]>('get_out_devices')
  outputDevices.value = devices ?? []
}

function selectOutputDevice(event: Event) {
  jsonStore.setOutSource((event.target as HTMLSelectElement).value)
}

async function onVolumeChange() {
  const pct = Math.max(0, Math.min(100, Math.round(outputVolumePct.value)))
  outputVolumePct.value = pct
  const vol = pct / 100
  jsonStore.setOutputVolume(vol)
  try {
    await invoke('set_output_volume', { volume: vol })
  } catch (e) {
    console.error('set_output_volume failed', e)
  }
}

async function syncFromStore() {
  await loadOutputDevices()
  const saved = jsonStore.configFile?.settings?.outputSource
  if (saved) outputSelected.value = saved
  const savedVol = jsonStore.configFile?.settings?.outputVolume
  outputVolumePct.value = Math.round((savedVol ?? 1) * 100)
  try {
    await invoke('set_output_volume', { volume: savedVol ?? 1 })
  } catch (e) { /* ignore */ }
}

watch(() => appStore.activeOverlay, (val) => {
  if (val !== 'settings') return
  syncFromStore()
})

onMounted(syncFromStore)
</script>
