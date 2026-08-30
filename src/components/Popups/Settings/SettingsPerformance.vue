<template>
  <section>
    <h2 class="settings-content__title">{{ $t('settings.tabs.performance') }}</h2>

    <div class="settings-section-divider">{{ $t('settings.main.performanceCache') }}</div>

    <div class="settings-group">
      <div class="settings-label-info">
        <span class="settings-label">{{ $t('settings.main.cacheMaxSize') }}</span>
        <span class="settings-info-icon">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
          <span class="settings-info-icon__tip">{{ $t('settings.main.cacheMaxSizeHint') }}</span>
        </span>
      </div>
      <div class="settings-unit-input">
        <input type="number" class="settings-input" v-model.number="cacheMaxSizeMib" min="32" max="4096" @change="onCacheConfig" />
        <span class="settings-unit-label">MiB</span>
      </div>
    </div>

    <div class="settings-group">
      <div class="settings-label-info">
        <span class="settings-label">{{ $t('settings.main.cacheMaxEntry') }}</span>
        <span class="settings-info-icon">
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
          <span class="settings-info-icon__tip">{{ $t('settings.main.cacheMaxEntryHint') }}</span>
        </span>
      </div>
      <div class="settings-unit-input">
        <input type="number" class="settings-input" v-model.number="cacheMaxEntryMib" min="1" max="500" @change="onCacheConfig" />
        <span class="settings-unit-label">MiB</span>
      </div>
    </div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.cacheStatus') }}</span>
        <span class="settings-hint">{{ cacheStatsText }}</span>
      </div>
      <button class="settings-btn" style="flex: 0; white-space: nowrap" @click="onClearCache">{{ $t('settings.main.clearCache') }}</button>
    </div>

    <div v-if="hasDedicatedGpu" class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.gpuAudio') }}</span>
        <span class="settings-hint">{{ $t('settings.main.gpuAudioHint') }}</span>
      </div>
      <UICheckbox :modelValue="gpuAudioEnabled" @update:modelValue="onGpuAudio" />
    </div>

    <div class="settings-section-divider">{{ $t('settings.main.stemsSection') }}</div>

    <div class="settings-group settings-group--stacked">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.stemsModel') }}</span>
        <span class="settings-hint">{{ stemsModelHint }}</span>
      </div>
      <div v-if="stemsDownloading" class="stems-progress">
        <div class="stems-progress__bar" role="progressbar" :aria-valuenow="stemsPct" aria-valuemin="0" aria-valuemax="100">
          <div class="stems-progress__fill" :style="{ width: `${stemsPct}%` }" />
        </div>
        <span class="settings-hint">{{ $t('recordEditor.stemsDownloading') }} {{ stemsPct }}%</span>
      </div>
      <div class="settings-row">
        <button
          v-if="stemsAvailable && !stemsModelReady"
          class="settings-btn"
          style="flex: 0; white-space: nowrap"
          @click="stemsDownloading ? onCancelStemsModel() : onInstallStemsModel()"
        >
          {{ stemsDownloading ? $t('settings.main.stemsCancel') : $t('settings.main.stemsInstall') }}
        </button>
        <span v-else-if="stemsModelReady" class="settings-hint" style="white-space: nowrap">
          {{ $t('settings.main.stemsInstalled') }}
        </span>
        <span v-else class="settings-hint" style="white-space: nowrap">
          {{ $t('settings.main.stemsUnavailable') }}
        </span>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const { t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const cacheMaxSizeMib = ref(64)
const cacheMaxEntryMib = ref(16)
const cacheStatsText = ref('')
const hasDedicatedGpu = ref(false)
const gpuAudioEnabled = ref(false)
const stemsAvailable = ref(false)
const stemsModelReady = ref(false)
const stemsModelLabel = ref('BS-RoFormer')
const stemsModelSize = ref('~158 MB')
const stemsDownloading = ref(false)
const stemsDownloadPct = ref<number | null>(null)
let unlistenStemsPct: UnlistenFn | null = null

const stemsPct = computed(() => Math.max(0, Math.min(100, Math.round(stemsDownloadPct.value ?? 0))))

const stemsModelHint = computed(() => {
  if (!stemsAvailable.value) return t('settings.main.stemsUnavailableHint')
  if (stemsModelReady.value) {
    return t('settings.main.stemsInstalledHint', {
      model: stemsModelLabel.value,
      size: stemsModelSize.value,
    })
  }
  return t('settings.main.stemsInstallHint', {
    model: stemsModelLabel.value,
    size: stemsModelSize.value,
  })
})

async function refreshStemsStatus() {
  try {
    const s = await invoke<{
      available: boolean
      modelReady: boolean
      modelLabel: string
      modelSizeHint: string
    }>('get_stems_status')
    stemsAvailable.value = !!s.available
    stemsModelReady.value = !!s.modelReady
    stemsModelLabel.value = s.modelLabel || 'BS-RoFormer'
    stemsModelSize.value = s.modelSizeHint || '~158 MB'
  } catch {
    stemsAvailable.value = false
    stemsModelReady.value = false
  }
}

async function onInstallStemsModel() {
  if (!stemsAvailable.value || stemsDownloading.value) return
  stemsDownloading.value = true
  stemsDownloadPct.value = 0
  try {
    await invoke('ensure_stems_model')
    await refreshStemsStatus()
  } catch (e) {
    const msg = String(e ?? '')
    if (!msg.toLowerCase().includes('cancel')) console.error(e)
  } finally {
    stemsDownloading.value = false
    stemsDownloadPct.value = null
  }
}

async function onCancelStemsModel() {
  try {
    await invoke('cancel_stems_model_download')
  } catch (e) {
    console.error(e)
  }
}

async function refreshCacheStats() {
  try {
    const stats = await invoke<{ cached_count: number; total_size_bytes: number }>('get_cache_stats')
    const usedMib = (stats.total_size_bytes / 1048576).toFixed(1)
    cacheStatsText.value = t('settings.main.cacheStatsText', { count: stats.cached_count, size: usedMib })
  } catch {
    cacheStatsText.value = ''
  }
}

async function onCacheConfig() {
  const maxSize = Math.max(32, Math.min(4096, Number(cacheMaxSizeMib.value) || 64))
  const maxEntry = Math.max(1, Math.min(500, Number(cacheMaxEntryMib.value) || 16))
  cacheMaxSizeMib.value = maxSize
  cacheMaxEntryMib.value = maxEntry
  jsonStore.setCacheConfig(maxSize, maxEntry)
  try {
    await invoke('set_cache_config', { maxSizeMib: maxSize, maxEntryMib: maxEntry })
  } catch (e) {
    console.error('set_cache_config failed', e)
  }
  await refreshCacheStats()
}

async function onClearCache() {
  try {
    await invoke('clear_sound_cache')
    await refreshCacheStats()
  } catch (e) {
    console.error('clear_sound_cache failed', e)
  }
}

async function onGpuAudio(val: boolean) {
  gpuAudioEnabled.value = val
  jsonStore.setSetting('gpuAudioEnabled', val)
  try {
    await invoke('set_gpu_audio', { enabled: val })
  } catch (e) {
    console.error('set_gpu_audio failed', e)
  }
}

async function syncFromStore() {
  if (!appSettings.loaded) await appSettings.load()
  cacheMaxSizeMib.value = jsonStore.configFile?.settings?.cacheMaxSizeMib ?? 64
  cacheMaxEntryMib.value = jsonStore.configFile?.settings?.cacheMaxEntryMib ?? 16
  try {
    await invoke('set_cache_config', {
      maxSizeMib: cacheMaxSizeMib.value,
      maxEntryMib: cacheMaxEntryMib.value,
    })
  } catch { /* not critical */ }
  await refreshCacheStats()
  if (!hasDedicatedGpu.value) {
    try {
      hasDedicatedGpu.value = await invoke<boolean>('has_dedicated_gpu')
    } catch { /* ignore */ }
  }
  if (hasDedicatedGpu.value) {
    gpuAudioEnabled.value = jsonStore.configFile?.settings?.gpuAudioEnabled ?? false
  }
  await refreshStemsStatus()
}

watch(() => appStore.activeOverlay, async (val) => {
  if (val !== 'settings') return
  await syncFromStore()
})

onMounted(async () => {
  await syncFromStore()
  unlistenStemsPct = await listen<number>('stems_model_progress', (e) => {
    if (!stemsDownloading.value) return
    stemsDownloadPct.value = Number(e.payload) || 0
  })
})

onBeforeUnmount(() => {
  if (unlistenStemsPct) unlistenStemsPct()
})
</script>
