<template>
  <section>
    <h2 class="settings-content__title">{{ $t('settings.audio.title') }}</h2>

    <!-- Audio Driver (host) -->
    <div class="settings-group">
      <div class="settings-label-info">
        <label class="settings-label">{{ $t('settings.audio.audioDriver') }}</label>
        
      </div>
      <div class="settings-row">
        <select v-model="hostSelected" @change="onHostChange" class="settings-select">
          <option v-for="h in audioHosts" :key="h" :value="h">{{ h }}</option>
        </select>
        <Icons icon="question" customClass="settings-question-icon" @triggered="asioInfoOpen = true" />
      </div>
    </div>

    <!-- ASIO info modal -->
    <DialogField v-if="asioInfoOpen" :title="$t('settings.audio.asioInfoTitle')" @close="asioInfoOpen = false">
      <p class="settings-hint">{{ $t('settings.audio.audioDriverHint') }}</p>
    </DialogField>

    <!-- Output Device -->
    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.audio.outputDevice') }}</label>
      <div class="settings-row">
        <select v-model="outputSelected" @change="selectOutputDevice" class="settings-select">
          <option v-for="device in outputDevices" :key="device" :value="device">{{ device }}</option>
        </select>
        <button class="settings-btn settings-btn--icon" @click="loadOutputDevices" :title="$t('settings.audio.refresh')">&#x21BB;</button>
      </div>
    </div>

    <!-- ASIO Channel Matrix (only visible when ASIO host is selected) -->
    <div v-if="hostSelected === 'ASIO' && asioChannels.length > 0" class="settings-group">
      <label class="settings-label">{{ $t('settings.audio.asioChannelMatrix') }}</label>
      <p class="settings-hint">{{ $t('settings.audio.asioChannelHint') }}</p>
      <table class="asio-matrix">
        <thead>
          <tr>
            <th>{{ $t('settings.audio.asioChannel') }}</th>
            <th>L</th>
            <th>R</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(ch, idx) in asioChannels" :key="idx">
            <td>{{ ch }}</td>
            <td>
              <input
                type="radio"
                name="asio-left"
                :value="idx"
                v-model="asioLeft"
                @change="saveAsioChannels"
              />
            </td>
            <td>
              <input
                type="radio"
                name="asio-right"
                :value="idx"
                v-model="asioRight"
                @change="saveAsioChannels"
              />
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Output Volume -->
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

const appSettings = useAppSettingsStore()
const appStore = useAppStore()

const audioHosts = ref<string[]>([])
const hostSelected = ref('WASAPI')
const outputDevices = ref<string[]>([])
const outputSelected = ref('')
const outputVolumePct = ref(100)
const asioChannels = ref<string[]>([])
const asioLeft = ref<number | null>(null)
const asioRight = ref<number | null>(null)
const asioInfoOpen = ref(false)

async function loadAudioHosts() {
  try {
    const hosts = await invoke<string[]>('get_audio_hosts')
    audioHosts.value = hosts?.length ? hosts : ['WASAPI']
  } catch {
    audioHosts.value = ['WASAPI']
  }
}

async function loadOutputDevices() {
  try {
    const devices = hostSelected.value && hostSelected.value !== 'WASAPI'
      ? await invoke<string[]>('get_out_devices_host', { host: hostSelected.value })
      : await invoke<string[]>('get_out_devices')
    outputDevices.value = devices ?? []
  } catch {
    outputDevices.value = []
  }
}

async function loadAsioChannels() {
  if (hostSelected.value !== 'ASIO' || !outputSelected.value) {
    asioChannels.value = []
    return
  }
  try {
    asioChannels.value = await invoke<string[]>('get_asio_device_channels', {
      deviceName: outputSelected.value,
    })
  } catch {
    asioChannels.value = []
  }
}

async function onHostChange() {
  await appSettings.setOutputHost(hostSelected.value)
  await loadOutputDevices()
  // Reset to first device on host switch.
  if (outputDevices.value.length > 0) {
    outputSelected.value = outputDevices.value[0]
    await appSettings.setOutputSource(outputSelected.value)
  }
  await loadAsioChannels()
}

async function selectOutputDevice(event: Event) {
  const val = (event.target as HTMLSelectElement).value
  await appSettings.setOutputSource(val)
  await loadAsioChannels()
}

async function saveAsioChannels() {
  await appSettings.setAsioChannels(asioLeft.value, asioRight.value)
}

async function onVolumeChange() {
  const pct = Math.max(0, Math.min(100, Math.round(outputVolumePct.value)))
  outputVolumePct.value = pct
  await appSettings.setOutputVolume(pct / 100)
}

async function syncFromStore() {
  if (!appSettings.loaded) await appSettings.load()
  await loadAudioHosts()
  hostSelected.value = appSettings.outputHost || 'WASAPI'
  await loadOutputDevices()
  const saved = appSettings.outputSource
  if (saved && saved !== 'default' && outputDevices.value.includes(saved)) {
    outputSelected.value = saved
  } else if (outputDevices.value.length > 0) {
    outputSelected.value = outputDevices.value[0]
    if (saved !== outputDevices.value[0]) {
      await appSettings.setOutputSource(outputDevices.value[0])
    }
  }
  asioLeft.value = appSettings.asioLeftChannel
  asioRight.value = appSettings.asioRightChannel
  await loadAsioChannels()
  outputVolumePct.value = Math.round((appSettings.outputVolume ?? 1) * 100)
  await appSettings.applyAudioVolume()
}

watch(() => appStore.activeOverlay, (val) => {
  if (val !== 'settings') return
  syncFromStore()
})

onMounted(syncFromStore)
</script>
