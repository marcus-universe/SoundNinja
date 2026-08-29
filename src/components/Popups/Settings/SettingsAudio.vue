<template>
  <section :class="{ 'settings-audio-compact': compact }">
    <h2 v-if="!compact" class="settings-content__title">{{ $t('settings.audio.title') }}</h2>

    <template v-if="!compact">
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
    </template>

    <!-- Input / Capture Device -->
    <div class="settings-group" :class="{ 'settings-group--compact': compact }">
      <div class="settings-label-info">
        <label class="settings-label">{{ $t('settings.audio.inputDevice') }}</label>
        <span v-if="!compact" class="settings-info-icon">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/></svg>
          <span class="settings-info-icon__tip">{{ $t('settings.audio.inputDeviceHint') }}</span>
        </span>
      </div>
      <div class="settings-row">
        <div ref="inputDropdownRef" class="capture-dropdown">
          <button
            type="button"
            class="capture-dropdown__trigger"
            :aria-expanded="inputDropdownOpen"
            @click="inputDropdownOpen = !inputDropdownOpen"
          >
            <span>{{ inputSelectedLabel }}</span>
            <span class="capture-dropdown__arrow" aria-hidden="true">▾</span>
          </button>
          <div v-show="inputDropdownOpen" class="capture-dropdown__list" role="listbox">
            <div v-if="inputCaptureDevices.length" class="capture-dropdown__group">
              <div class="capture-dropdown__sep">{{ $t('settings.audio.inputDevicesGroup') }}</div>
              <button
                v-for="device in inputCaptureDevices"
                :key="device.name"
                type="button"
                class="capture-dropdown__option"
                :class="{ active: inputSelected === device.name }"
                role="option"
                :aria-selected="inputSelected === device.name"
                @click="pickInputDevice(device.name)"
              >{{ device.name }}</button>
            </div>
            <div v-if="outputCaptureDevices.length" class="capture-dropdown__group">
              <div class="capture-dropdown__sep">{{ $t('settings.audio.outputDevicesGroup') }}</div>
              <button
                v-for="device in outputCaptureDevices"
                :key="device.name"
                type="button"
                class="capture-dropdown__option"
                :class="{ active: inputSelected === device.name }"
                role="option"
                :aria-selected="inputSelected === device.name"
                @click="pickInputDevice(device.name)"
              >{{ loopbackLabel(device.name) }}</button>
            </div>
          </div>
        </div>
        <button class="settings-btn settings-btn--icon" @click="loadInputDevices" :title="$t('settings.audio.refresh')">&#x21BB;</button>
      </div>
    </div>

    <template v-if="!compact">
      <!-- ASIO Channel Matrix -->
      <div v-if="isAsioHost(hostSelected) && asioChannels.length > 0" class="settings-group">
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
    </template>

    <div class="settings-audio-volumes" :class="{ 'settings-audio-volumes--compact': compact }">
      <!-- Output Volume -->
      <div class="settings-group settings-group--stacked">
        <div class="settings-slider-header">
          <div class="settings-label-info">
            <label class="settings-label">{{ $t('settings.audio.outputVolume') }}</label>
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
            <div class="settings-spin" aria-hidden="true">
              <button type="button" class="settings-spin__btn" tabindex="-1" @click="nudgeOutputVolume(1)">▴</button>
              <button type="button" class="settings-spin__btn" tabindex="-1" @click="nudgeOutputVolume(-1)">▾</button>
            </div>
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
          :title="$t('settings.audio.sliderResetHint')"
          @change="onVolumeChange"
          @input="onVolumeChange"
          @dblclick.prevent="resetOutputVolume"
        />
      </div>

      <!-- Input / Capture Volume -->
      <div class="settings-group settings-group--stacked">
        <div class="settings-slider-header">
          <div class="settings-label-info">
            <label class="settings-label">{{ $t('settings.audio.inputVolume') }}</label>
          </div>
          <div class="settings-unit-input">
            <input
              type="number"
              class="settings-input"
              min="0"
              max="200"
              step="1"
              v-model.number="inputVolumePct"
              @change="onInputVolumeChange"
            />
            <div class="settings-spin" aria-hidden="true">
              <button type="button" class="settings-spin__btn" tabindex="-1" @click="nudgeInputVolume(1)">▴</button>
              <button type="button" class="settings-spin__btn" tabindex="-1" @click="nudgeInputVolume(-1)">▾</button>
            </div>
            <span class="settings-unit-label">%</span>
          </div>
        </div>
        <input
          type="range"
          class="settings-slider"
          min="0"
          max="200"
          step="1"
          v-model.number="inputVolumePct"
          :title="$t('settings.audio.sliderResetHint')"
          @change="onInputVolumeChange"
          @input="onInputVolumeChange"
          @dblclick.prevent="resetInputVolume"
        />
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'

const props = withDefaults(defineProps<{ compact?: boolean }>(), { compact: false })

const appSettings = useAppSettingsStore()
const appStore = useAppStore()

interface CaptureDeviceInfo {
  name: string
  loopback: boolean
}

const audioHosts = ref<string[]>([])
const hostSelected = ref('')
const outputDevices = ref<string[]>([])
const outputSelected = ref('')
const inputDevices = ref<CaptureDeviceInfo[]>([])
const inputSelected = ref('')
const outputVolumePct = ref(100)
const inputVolumePct = ref(100)
const asioChannels = ref<string[]>([])
const asioLeft = ref<number | null>(null)
const asioRight = ref<number | null>(null)
const asioInfoOpen = ref(false)

const inputCaptureDevices = computed(() => inputDevices.value.filter((d) => !d.loopback))
const outputCaptureDevices = computed(() => inputDevices.value.filter((d) => d.loopback))

const inputDropdownOpen = ref(false)
const inputDropdownRef = ref<HTMLElement | null>(null)

function onCaptureDropdownPointerDown(e: PointerEvent) {
  const el = inputDropdownRef.value
  if (!el || !inputDropdownOpen.value) return
  if (e.target instanceof Node && !el.contains(e.target)) {
    inputDropdownOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('pointerdown', onCaptureDropdownPointerDown, true)
})
onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onCaptureDropdownPointerDown, true)
})

const inputSelectedLabel = computed(() => {
  const d = inputDevices.value.find((x) => x.name === inputSelected.value)
  if (!d) return inputSelected.value || '—'
  return d.loopback ? loopbackLabel(d.name) : d.name
})

/** Drop trailing " (PC Audio)" — group label already says Output Devices. */
function loopbackLabel(name: string) {
  return name.replace(/\s*\(PC Audio\)\s*$/i, '')
}

/** Match saved host labels (WASAPI / PipeWire / Alsa) to cpal's real host id name. */
function resolveHostName(saved: string | null | undefined, hosts: string[]): string {
  // Backend sorts hosts: PipeWire → PulseAudio → Wasapi → … → Alsa
  const fallback = hosts[0] ?? ''
  if (!hosts.length) return saved || ''
  if (!saved) return fallback
  const hit = hosts.find((h) => h.toLowerCase() === saved.toLowerCase())
  // Legacy Windows default on Linux/mac → pick platform-preferred host.
  if (!hit && saved.toLowerCase() === 'wasapi') return fallback
  return hit ?? fallback
}

function isAsioHost(name: string) {
  return name.toLowerCase() === 'asio'
}

async function loadAudioHosts() {
  try {
    const hosts = await invoke<string[]>('get_audio_hosts')
    audioHosts.value = hosts?.length ? hosts : []
  } catch {
    audioHosts.value = []
  }
}

async function loadOutputDevices() {
  try {
    const devices = hostSelected.value
      ? await invoke<string[]>('get_out_devices_host', { host: hostSelected.value })
      : await invoke<string[]>('get_out_devices')
    outputDevices.value = devices ?? []
  } catch {
    outputDevices.value = []
  }
}

async function loadInputDevices() {
  try {
    inputDevices.value = (await invoke<CaptureDeviceInfo[]>('get_loopback_devices', {
      host: hostSelected.value || null,
    })) ?? []
  } catch {
    inputDevices.value = []
  }
}

async function loadAsioChannels() {
  if (!isAsioHost(hostSelected.value) || !outputSelected.value) {
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
  await appSettings.setInputHost(hostSelected.value)
  await loadOutputDevices()
  await loadInputDevices()
  // Reset to first device on host switch.
  const firstOut = outputDevices.value[0]
  if (firstOut) {
    outputSelected.value = firstOut
    await appSettings.setOutputSource(firstOut)
  }
  const firstIn = inputDevices.value[0]
  if (firstIn) {
    inputSelected.value = firstIn.name
    await appSettings.setInputSource(firstIn.name, firstIn.loopback)
  }
  await loadAsioChannels()
}

async function selectOutputDevice(event: Event) {
  const val = (event.target as HTMLSelectElement).value
  await appSettings.setOutputSource(val)
  await loadAsioChannels()
}

async function pickInputDevice(name: string) {
  inputSelected.value = name
  inputDropdownOpen.value = false
  const info = inputDevices.value.find((d) => d.name === name)
  await appSettings.setInputSource(name, info?.loopback ?? false)
}

async function saveAsioChannels() {
  await appSettings.setAsioChannels(asioLeft.value, asioRight.value)
}

async function onVolumeChange() {
  const pct = Math.max(0, Math.min(100, Math.round(outputVolumePct.value)))
  outputVolumePct.value = pct
  await appSettings.setOutputVolume(pct / 100)
}

async function onInputVolumeChange() {
  const pct = Math.max(0, Math.min(200, Math.round(inputVolumePct.value)))
  inputVolumePct.value = pct
  await appSettings.setInputVolume(pct / 100)
}

async function nudgeOutputVolume(delta: number) {
  outputVolumePct.value = Math.max(0, Math.min(100, Math.round((outputVolumePct.value || 0) + delta)))
  await onVolumeChange()
}

async function nudgeInputVolume(delta: number) {
  inputVolumePct.value = Math.max(0, Math.min(200, Math.round((inputVolumePct.value || 0) + delta)))
  await onInputVolumeChange()
}

async function resetOutputVolume() {
  outputVolumePct.value = 100
  await onVolumeChange()
}

async function resetInputVolume() {
  inputVolumePct.value = 100
  await onInputVolumeChange()
}

async function syncFromStore() {
  if (!appSettings.loaded) await appSettings.load()
  await loadAudioHosts()
  hostSelected.value = resolveHostName(appSettings.outputHost || appSettings.inputHost, audioHosts.value)
  // Persist the canonical cpal host name (e.g. "Wasapi") so record/play match.
  if (hostSelected.value !== appSettings.outputHost) {
    await appSettings.setOutputHost(hostSelected.value)
  }
  if (hostSelected.value !== appSettings.inputHost) {
    await appSettings.setInputHost(hostSelected.value)
  }
  await loadOutputDevices()
  await loadInputDevices()
  const saved = appSettings.outputSource
  const firstOut = outputDevices.value[0]
  if (saved && saved !== 'default' && outputDevices.value.includes(saved)) {
    outputSelected.value = saved
  } else if (firstOut) {
    outputSelected.value = firstOut
    if (saved !== firstOut) {
      await appSettings.setOutputSource(firstOut)
    }
  }
  const savedIn = appSettings.inputSource
  const firstIn = inputDevices.value[0]
  if (savedIn && savedIn !== 'default' && inputDevices.value.some((d) => d.name === savedIn)) {
    inputSelected.value = savedIn
  } else if (firstIn) {
    inputSelected.value = firstIn.name
    if (savedIn !== firstIn.name) {
      await appSettings.setInputSource(firstIn.name, firstIn.loopback)
    }
  }
  asioLeft.value = appSettings.asioLeftChannel
  asioRight.value = appSettings.asioRightChannel
  await loadAsioChannels()
  outputVolumePct.value = Math.round((appSettings.outputVolume ?? 1) * 100)
  inputVolumePct.value = Math.round((appSettings.inputVolume ?? 1) * 100)
  await appSettings.applyAudioVolume()
  await appSettings.applyInputVolume()
}

watch(() => appStore.activeOverlay, (val) => {
  if (props.compact) return
  if (val !== 'settings') return
  syncFromStore()
})

// Keep compact strip in sync when Settings (or another window) changes volumes/devices.
watch(
  () => [
    appSettings.inputSource,
    appSettings.inputVolume,
    appSettings.outputVolume,
    appSettings.inputHost,
  ],
  () => {
    if (!props.compact) return
    outputVolumePct.value = Math.round((appSettings.outputVolume ?? 1) * 100)
    inputVolumePct.value = Math.round((appSettings.inputVolume ?? 1) * 100)
    if (appSettings.inputSource && inputDevices.value.some((d) => d.name === appSettings.inputSource)) {
      inputSelected.value = appSettings.inputSource
    }
  },
)

onMounted(syncFromStore)
</script>
