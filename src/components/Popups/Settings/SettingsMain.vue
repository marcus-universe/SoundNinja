<template>
  <section>
    <h2 class="settings-content__title">{{ $t('settings.main.title') }}</h2>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.main.language') }}</label>
      <select v-model="currentLanguage" @change="changeLanguage" class="settings-select">
        <option v-for="loc in availableLocales" :key="loc.code" :value="loc.code">{{ loc.name }}</option>
      </select>
    </div>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.main.theme') }}</label>
      <div class="settings-row settings-row--nowrap">
        <select v-model="selectedTheme" @change="applyTheme" class="settings-select">
          <option v-for="t in builtinThemes" :key="t.id" :value="t.id">{{ t.label }}</option>
          <option v-for="t in savedThemes" :key="t" :value="'file:' + t">{{ t.replace('.css', '') }}</option>
          <option value="custom">{{ $t('settings.main.customCssOption') }}</option>
        </select>
        <button class="settings-btn settings-btn--primary settings-btn--icon-only" :title="$t('settings.main.openThemeCreator')" @click="openThemeCreator">
          <Icons icon="palette" custom-class="settings-btn-icon" />
        </button>
      </div>
    </div>

    <Transition name="fade">
      <div v-if="selectedTheme === 'custom'" class="settings-group settings-group--stacked">
        <label class="settings-label">{{ $t('settings.main.customCss') }}</label>
        <p class="settings-hint">{{ $t('settings.main.importCssHint') }}</p>
        <div class="settings-row" style="margin-top: 0.5rem">
          <button class="settings-btn" @click="importCustomCssFile">{{ $t('settings.main.importCssFile') }}</button>
        </div>
        <p v-if="importError" class="settings-error" style="margin-top:0.5rem">{{ importError }}</p>
        <p v-if="importSuccess" class="settings-success" style="margin-top:0.5rem">{{ importSuccess }}</p>
      </div>
    </Transition>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.main.themeMode') }}</label>
      <select v-model="themeMode" @change="onThemeMode" class="settings-select">
        <option value="dark">{{ $t('settings.main.themeModeDark') }}</option>
        <option value="light">{{ $t('settings.main.themeModeLight') }}</option>
      </select>
    </div>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.main.navbarSide') }}</label>
      <select v-model="navbarSide" @change="onNavbarSide" class="settings-select">
        <option value="left">{{ $t('settings.main.navbarLeft') }}</option>
        <option value="right">{{ $t('settings.main.navbarRight') }}</option>
      </select>
    </div>

    <DialogField
      v-if="nameDialogOpen"
      :title="$t('settings.main.themeNoNameTitle')"
      :errorMessage="nameDialogError"
      @close="closeNameDialog"
    >
      <p class="settings-hint">{{ $t('settings.main.themeNoNameHint') }}</p>
      <input
        type="text"
        class="settings-input"
        v-model="nameDialogInput"
        :placeholder="$t('settings.main.themeNamePlaceholder')"
        @keyup.enter="confirmNameDialog"
      />
      <div class="settings-row" style="margin-top: 0.75rem">
        <button class="settings-btn settings-btn--primary" @click="confirmNameDialog" :disabled="!nameDialogInput.trim()">{{ $t('settings.main.themeNoNameConfirm') }}</button>
      </div>
    </DialogField>

    <div class="settings-section-divider">{{ $t('settings.main.playbackBehavior') }}</div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.stopOnRetrigger') }}</span>
        <span class="settings-hint">{{ $t('settings.main.stopOnRetriggerHint') }}</span>
      </div>
      <UICheckbox :modelValue="stopOnRetrigger" @update:modelValue="onStopOnRetrigger" />
    </div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.overlapSounds') }}</span>
        <span class="settings-hint">{{ $t('settings.main.overlapSoundsHint') }}</span>
      </div>
      <UICheckbox :modelValue="overlapSounds" @update:modelValue="onOverlapSounds" />
    </div>

    <div class="settings-section-divider">{{ $t('settings.main.behavior') }}</div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.uniformButtonHeight') }}</span>
        <span class="settings-hint">{{ $t('settings.main.uniformButtonHeightHint') }}</span>
      </div>
      <UICheckbox :modelValue="uniformButtonHeight" @update:modelValue="onUniformButtonHeight" />
    </div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.allowReorder') }}</span>
        <span class="settings-hint">{{ $t('settings.main.allowReorderHint') }}</span>
      </div>
      <UICheckbox :modelValue="allowReorder" @update:modelValue="onAllowReorder" />
    </div>

    <div class="settings-group">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.recentLimit') }}</span>
        <span class="settings-hint">{{ $t('settings.main.recentLimitHint') }}</span>
      </div>
      <input type="number" class="settings-input" v-model.number="recentLimit" min="1" max="100" @change="onRecentLimit" />
    </div>

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

   
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

const { t, locale, locales: availableLocales, setLocale } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

function joinPath(base: string, ...parts: string[]) {
  const sep = base.includes('\\') ? '\\' : '/'
  return [base.replace(/[\\/]+$/, ''), ...parts].join(sep)
}

// ── Theme Creator window ──────────────────────────────────────────────────────
async function openThemeCreator() {
  try {
    const existing = await WebviewWindow.getByLabel('theme-creator')
    if (existing) {
      await existing.setFocus()
    } else {
      const win = new WebviewWindow('theme-creator', {
        url: '#/theme-creator',
        title: 'Theme Creator',
        width: 940,
        height: 720,
        minWidth: 720,
        minHeight: 560,
        resizable: true,
      })
      win.once('tauri://error', (e) => console.error('Theme Creator window error', e))
    }
  } catch (e) {
    console.error('Failed to open Theme Creator window', e)
  }
  appStore.setActiveOverlay(null)
}



const builtinThemes = [
  { id: 'dark-cyan',   label: 'Dark Cyan',   colors: { primaryColor: '#00d4ff', bgDark: '#222831' } },
  { id: 'dark-purple', label: 'Dark Purple', colors: { primaryColor: '#a855f7', bgDark: '#1e1b2e' } },
  { id: 'dark-orange', label: 'Dark Orange', colors: { primaryColor: '#ff8a29', bgDark: '#231c15' } },
  { id: 'dark-green',  label: 'Dark Green',  colors: { primaryColor: '#22e06a', bgDark: '#162218' } },
  { id: 'dark-pink',   label: 'Dark Pink',   colors: { primaryColor: '#f062a6', bgDark: '#231520' } },
]

const selectedTheme = ref('dark-cyan')
const savedThemes = ref<string[]>([])
const importError = ref('')
const importSuccess = ref('')
const stopOnRetrigger = ref(true)
const overlapSounds = ref(false)
const uniformButtonHeight = ref(false)
const allowReorder = ref(true)
const themeMode = ref<'dark' | 'light'>('dark')
const recentLimit = ref(30)
const currentLanguage = ref(locale.value)
const navbarSide = ref<'left' | 'right'>('left')
const cacheMaxSizeMib = ref(256)
const cacheMaxEntryMib = ref(50)
const cacheStatsText = ref('')
const hasDedicatedGpu = ref(false)
const gpuAudioEnabled = ref(false)

// ── Navbar side ───────────────────────────────────────────────────────────────
async function onNavbarSide() {
  await appSettings.setNavbarSide(navbarSide.value)
}

// Name-prompt dialog state (shown when an imported CSS has no theme-name comment)
const nameDialogOpen = ref(false)
const nameDialogInput = ref('')
const nameDialogError = ref('')
const pendingCss = ref('')

// ── Language ──────────────────────────────────────────────────────────────────
async function changeLanguage() {
  await setLocale(currentLanguage.value as 'en' | 'de')
  await appSettings.setLocale(currentLanguage.value)
  if (typeof window !== 'undefined') {
    localStorage.setItem('sn-locale', currentLanguage.value)
  }
  try {
    await invoke('rebuild_menu', { lang: currentLanguage.value })
  } catch { /* not available in all envs */ }
}

// ── Theme ─────────────────────────────────────────────────────────────────────
function applyTheme() {
  const id = selectedTheme.value
  jsonStore.setTheme(id)
  if (id === 'custom') {
    // Custom/file themes are injected via a <style> tag. Inline vars set by a
    // previously selected builtin theme would override it, so clear them.
    clearInlineThemeVars()
    return
  }
  if (id.startsWith('file:')) {
    clearInlineThemeVars()
    const filename = id.slice(5)
    invoke<string>('read_text_file_abs', { path: joinPath(appSettings.themesPath, filename) })
      .then(injectCustomCss)
      .catch((e) => console.error('Failed to load theme file', e))
    return
  }
  removeCustomCssTag()
  const theme = builtinThemes.find((t) => t.id === id)
  if (!theme) return
  // Builtin themes are presets for the per-project color model.
  jsonStore.setThemeColors(theme.colors)
  applyThemeColors(jsonStore.configFile?.settings)
}

// Removes every theme variable a builtin/model theme may have set inline on
// <html>. Inline styles outrank the injected `:root {}` rule, so they must be
// cleared before a file/custom theme can take effect.
function clearInlineThemeVars() {
  const root = document.documentElement
  ;[
    '--primary_color', '--color-bg', '--color-btn', '--sound-text',
    '--color-bg-light', '--color-bg-dark', '--color-btn-light', '--color-btn-dark',
    '--text-light', '--text-dark',
  ].forEach((k) => root.style.removeProperty(k))
}

function removeCustomCssTag() {
  document.getElementById('sn-custom-theme')?.remove()
}

function injectCustomCss(css: string) {
  let tag = document.getElementById('sn-custom-theme')
  if (!tag) {
    tag = document.createElement('style')
    tag.id = 'sn-custom-theme'
    document.head.appendChild(tag)
  }
  tag.textContent = css
}

async function loadSavedThemes() {
  try {
    await invoke('make_dir_abs', { path: appSettings.themesPath })
    const files = await invoke<string[]>('list_dir_files_abs', {
      dir: appSettings.themesPath,
      exts: ['css'],
    })
    savedThemes.value = files
  } catch {
    savedThemes.value = []
  }
}

// ── Import custom CSS ─────────────────────────────────────────────────────────
async function importCustomCssFile() {
  importError.value = ''
  importSuccess.value = ''
  try {
    const filePath = await open({
      multiple: false,
      title: 'Import Theme CSS',
      filters: [{ name: 'CSS Files', extensions: ['css'] }],
    })
    if (!filePath) return
    const css = await invoke<string>('read_text_file_abs', { path: filePath as string })
    if (!css.includes('--')) {
      importError.value = t('settings.main.notCompatibleTheme')
      return
    }
    const name = parseThemeName(css)
    if (name) {
      await saveImportedTheme(css, name)
    } else {
      // No name comment — ask the user to name the theme.
      pendingCss.value = css
      nameDialogInput.value = ''
      nameDialogError.value = ''
      nameDialogOpen.value = true
    }
  } catch (e) {
    importError.value = `Failed: ${e}`
  }
}

// Reads the `/* SoundNinja Theme: NAME */` comment from a theme CSS file.
function parseThemeName(css: string): string {
  const m = css.match(/\/\*\s*SoundNinja Theme:\s*(.+?)\s*\*\//i)
  return m?.[1]?.trim() ?? ''
}

async function saveImportedTheme(css: string, name: string) {
  const safeName = name.replace(/[^a-z0-9_-]/gi, '_')
  await invoke('make_dir_abs', { path: appSettings.themesPath })
  await invoke('write_text_file_abs', {
    path: joinPath(appSettings.themesPath, `${safeName}.css`),
    contents: css,
  })
  importSuccess.value = t('settings.main.importSuccess')
  await loadSavedThemes()
  selectedTheme.value = `file:${safeName}.css`
  applyTheme()
}

async function confirmNameDialog() {
  const name = nameDialogInput.value.trim()
  if (!name) {
    nameDialogError.value = t('settings.main.themeNameRequired')
    return
  }
  try {
    // Prepend the name comment so the theme keeps its name on future imports.
    const css = `/* SoundNinja Theme: ${name} */\n${pendingCss.value}`
    await saveImportedTheme(css, name)
    closeNameDialog()
  } catch (e) {
    nameDialogError.value = `Failed: ${e}`
  }
}

function closeNameDialog() {
  nameDialogOpen.value = false
  pendingCss.value = ''
  nameDialogInput.value = ''
  nameDialogError.value = ''
}

// ── Playback toggles ──────────────────────────────────────────────────────────
function onStopOnRetrigger(val: boolean) {
  stopOnRetrigger.value = val
  jsonStore.setStopOnRetrigger(val)
}

function onOverlapSounds(val: boolean) {
  overlapSounds.value = val
  jsonStore.setOverlapSounds(val)
}

// ── Behavior (P8/P9) ──────────────────────────────────────────────────────────
function onUniformButtonHeight(val: boolean) {
  uniformButtonHeight.value = val
  jsonStore.setUniformButtonHeight(val)
}

function onAllowReorder(val: boolean) {
  allowReorder.value = val
  jsonStore.setAllowReorder(val)
}

function onThemeMode() {
  jsonStore.setThemeMode(themeMode.value)
  applyThemeMode(themeMode.value, jsonStore.configFile?.settings)
}

async function onRecentLimit() {
  const n = Math.max(1, Math.min(100, Number(recentLimit.value) || 30))
  recentLimit.value = n
  await appSettings.setRecentLimit(n)
}

// ── Cache ─────────────────────────────────────────────────────────────────────
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
  const maxSize = Math.max(32, Math.min(4096, Number(cacheMaxSizeMib.value) || 256))
  const maxEntry = Math.max(1, Math.min(500, Number(cacheMaxEntryMib.value) || 50))
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

// ── Refresh when settings panel opens ────────────────────────────────────────
async function syncFromStore() {
  if (!appSettings.loaded) await appSettings.load()
  navbarSide.value = appSettings.navbarSide
  // Load the saved-theme options first so the dropdown always has a matching
  // <option> for a persisted `file:` theme — otherwise v-model can't select it.
  await loadSavedThemes()
  selectedTheme.value = jsonStore.configFile?.settings?.theme ?? 'dark-cyan'
  stopOnRetrigger.value = jsonStore.configFile?.settings?.stopOnRetrigger ?? true
  overlapSounds.value = jsonStore.configFile?.settings?.overlapSounds ?? false
  uniformButtonHeight.value = jsonStore.configFile?.settings?.uniformButtonHeight ?? false
  allowReorder.value = jsonStore.configFile?.settings?.allowReorder ?? true
  themeMode.value = jsonStore.configFile?.settings?.themeMode ?? 'dark'
  applyThemeMode(themeMode.value, jsonStore.configFile?.settings)
  recentLimit.value = appSettings.recentLimit ?? 30
  cacheMaxSizeMib.value = jsonStore.configFile?.settings?.cacheMaxSizeMib ?? 256
  cacheMaxEntryMib.value = jsonStore.configFile?.settings?.cacheMaxEntryMib ?? 50
  applyTheme()
  try {
    await invoke('set_cache_config', {
      maxSizeMib: cacheMaxSizeMib.value,
      maxEntryMib: cacheMaxEntryMib.value,
    })
  } catch { /* not critical */ }
  await refreshCacheStats()
  // GPU detection — runs once; result cached in ref.
  if (!hasDedicatedGpu.value) {
    try {
      hasDedicatedGpu.value = await invoke<boolean>('has_dedicated_gpu')
    } catch { /* ignore — GPU toggle stays hidden */ }
  }
  if (hasDedicatedGpu.value) {
    gpuAudioEnabled.value = jsonStore.configFile?.settings?.gpuAudioEnabled ?? false
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

watch(() => appStore.activeOverlay, async (val) => {
  if (val !== 'settings') return
  await syncFromStore()
})

onMounted(syncFromStore)
</script>
