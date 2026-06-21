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
      <select v-model="selectedTheme" @change="applyTheme" class="settings-select">
        <option v-for="t in builtinThemes" :key="t.id" :value="t.id">{{ t.label }}</option>
        <option v-for="t in savedThemes" :key="t" :value="'file:' + t">{{ t.replace('.css', '') }}</option>
        <option value="custom">{{ $t('settings.main.customCssOption') }}</option>
      </select>
    </div>

    <Transition name="fade">
      <div v-if="selectedTheme === 'custom'" class="settings-group settings-group--stacked">
        <label class="settings-label">{{ $t('settings.main.customCss') }}</label>
        <p class="settings-hint">{{ $t('settings.main.importCssHint') }}</p>
        <input type="text" class="settings-input" v-model="importThemeName" :placeholder="$t('settings.main.themeNamePlaceholder')" />
        <div class="settings-row" style="margin-top: 0.5rem">
          <button class="settings-btn" @click="importCustomCssFile" :disabled="!importThemeName.trim()">{{ $t('settings.main.importCssFile') }}</button>
        </div>
        <p v-if="importError" class="settings-error" style="margin-top:0.5rem">{{ importError }}</p>
        <p v-if="importSuccess" class="settings-success" style="margin-top:0.5rem">{{ importSuccess }}</p>
      </div>
    </Transition>

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

   
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { readTextFile, writeTextFile, mkdir, readDir, BaseDirectory } from '@tauri-apps/plugin-fs'

const { t, locale, locales: availableLocales, setLocale } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()



const builtinThemes = [
  { id: 'dark-cyan',   label: 'Dark Cyan',   vars: { '--primary_color': 'hsl(189, 100%, 58%)', '--color-bg': '#222831' } },
  { id: 'dark-purple', label: 'Dark Purple',  vars: { '--primary_color': 'hsl(270, 80%, 65%)',  '--color-bg': '#1e1b2e' } },
  { id: 'dark-orange', label: 'Dark Orange',  vars: { '--primary_color': 'hsl(28, 100%, 58%)',  '--color-bg': '#231c15' } },
  { id: 'dark-green',  label: 'Dark Green',   vars: { '--primary_color': 'hsl(145, 80%, 50%)',  '--color-bg': '#162218' } },
  { id: 'dark-pink',   label: 'Dark Pink',    vars: { '--primary_color': 'hsl(330, 80%, 65%)',  '--color-bg': '#231520' } },
]

const selectedTheme = ref('dark-cyan')
const savedThemes = ref<string[]>([])
const importThemeName = ref('')
const importError = ref('')
const importSuccess = ref('')
const stopOnRetrigger = ref(true)
const overlapSounds = ref(false)
const currentLanguage = ref(locale.value)

// ── Language ──────────────────────────────────────────────────────────────────
async function changeLanguage() {
  await setLocale(currentLanguage.value as 'en' | 'de')
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
  if (id === 'custom') { return }
  if (id.startsWith('file:')) {
    const filename = id.slice(5)
    readTextFile(`themes/${filename}`, { baseDir: BaseDirectory.AppData })
      .then(injectCustomCss)
      .catch((e) => console.error('Failed to load theme file', e))
    return
  }
  removeCustomCssTag()
  const theme = builtinThemes.find((t) => t.id === id)
  if (!theme) return
  const root = document.documentElement
  Object.entries(theme.vars).forEach(([k, v]) => root.style.setProperty(k, v))
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
    await mkdir('themes', { baseDir: BaseDirectory.AppData, recursive: true })
    const entries = await readDir('themes', { baseDir: BaseDirectory.AppData })
    savedThemes.value = entries.filter((e) => e.name?.endsWith('.css')).map((e) => e.name as string)
  } catch {
    savedThemes.value = []
  }
}

// ── Import custom CSS ─────────────────────────────────────────────────────────
async function importCustomCssFile() {
  importError.value = ''
  importSuccess.value = ''
  const name = importThemeName.value.trim()
  if (!name) {
    importError.value = t('settings.main.themeNameRequired')
    return
  }
  try {
    const filePath = await open({
      multiple: false,
      title: 'Import Theme CSS',
      filters: [{ name: 'CSS Files', extensions: ['css'] }],
    })
    if (!filePath) return
    const css = await readTextFile(filePath as string)
    if (!css.includes('--')) {
      importError.value = t('settings.main.notCompatibleTheme')
      return
    }
    const safeName = name.replace(/[^a-z0-9_-]/gi, '_')
    await mkdir('themes', { baseDir: BaseDirectory.AppData, recursive: true })
    await writeTextFile(`themes/${safeName}.css`, css, { baseDir: BaseDirectory.AppData })
    importSuccess.value = t('settings.main.importSuccess')
    importThemeName.value = ''
    await loadSavedThemes()
    selectedTheme.value = `file:${safeName}.css`
    applyTheme()
  } catch (e) {
    importError.value = `Failed: ${e}`
  }
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

// ── Refresh when settings panel opens ────────────────────────────────────────
watch(() => appStore.activeOverlay, async (val) => {
  if (val !== 'settings') return
  selectedTheme.value = jsonStore.configFile?.settings?.theme ?? 'dark-cyan'
  stopOnRetrigger.value = jsonStore.configFile?.settings?.stopOnRetrigger ?? true
  overlapSounds.value = jsonStore.configFile?.settings?.overlapSounds ?? false
  applyTheme()
  await loadSavedThemes()
})

onMounted(loadSavedThemes)
</script>
