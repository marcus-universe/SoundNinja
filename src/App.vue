<template>
  <div class="soundninja flex_c_h flex_space_between">
    <NavBar />
    <ErrorAlert />
    <SettingsOverlay />
    <ImportFolders v-if="appStore.importFoldersActive" />
    <ContextMenu />
    <NuxtPage v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </NuxtPage>
  </div>
</template>

<script setup>
import { readTextFile, writeTextFile, mkdir, BaseDirectory } from '@tauri-apps/plugin-fs'
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'

const jsonStore = useJsonHandelingStore()
const appStore = useAppStore()
const { setLocale } = useI18n()

const defaultConfig = {
  settings: {
    theme: 'dark-cyan',
    customCss: '',
    outputSource: 'default',
  },
  tabList: [],
  files: [],
}

async function readConfigFile() {
  try {
    const contents = JSON.parse(
      await readTextFile('config.json', { baseDir: BaseDirectory.AppData })
    )
    const migrated = migrateConfig(contents)
    jsonStore.updateConfigFile(migrated)
    return migrated
  } catch {
    await mkdir('', { baseDir: BaseDirectory.AppData, recursive: true })
    await writeTextFile(
      'config.json',
      JSON.stringify(defaultConfig, null, 2),
      { baseDir: BaseDirectory.AppData }
    )
    jsonStore.updateConfigFile(defaultConfig)
    return defaultConfig
  }
}

/** Migrate old config formats to the current schema */
function migrateConfig(obj) {
  // Migrate tabList: string[] → TabEntry[]
  if (Array.isArray(obj.tabList) && obj.tabList.length > 0 && typeof obj.tabList[0] === 'string') {
    obj.tabList = obj.tabList.map((name) => ({ name }))
  }
  // Migrate settings.hue → settings.theme
  if (obj.settings && typeof obj.settings.hue === 'number' && !obj.settings.theme) {
    obj.settings.theme = 'dark-cyan'
    delete obj.settings.hue
  }
  // Ensure new settings fields exist
  if (obj.settings) {
    obj.settings.theme = obj.settings.theme ?? 'dark-cyan'
    obj.settings.customCss = obj.settings.customCss ?? ''
  }
  return obj
}

function isValidConfig(obj) {
  if (!obj || typeof obj !== 'object') return false
  return Array.isArray(obj.tabList) && typeof obj.settings === 'object' && Array.isArray(obj.files)
}

async function handleMenuOpenProject() {
  const selected = await openDialog({
    title: 'Open Project',
    filters: [{ name: 'Sound Ninja Config', extensions: ['json'] }],
    multiple: false,
  })
  if (!selected || Array.isArray(selected)) return
  try {
    const contents = JSON.parse(await readTextFile(selected))
    if (!isValidConfig(contents)) {
      appStore.setErrorActive('Invalid project file: missing required fields (tabList, settings, files).')
      return
    }
    jsonStore.updateConfigFile(contents)
    jsonStore.setCurrentProjectPath(selected)
  } catch {
    appStore.setErrorActive('Failed to open project file. The file may be corrupted or not valid JSON.')
  }
}

async function handleMenuSave() {
  if (jsonStore.currentProjectPath) {
    try {
      await jsonStore.saveToPath(jsonStore.currentProjectPath)
    } catch {
      appStore.setErrorActive('Failed to save project.')
    }
  } else {
    await handleMenuSaveAs()
  }
}

async function handleMenuSaveAs() {
  const path = await saveDialog({
    title: 'Save Project As',
    filters: [{ name: 'Sound Ninja Config', extensions: ['json'] }],
    defaultPath: 'soundninja-project.json',
  })
  if (!path) return
  try {
    const savePath = path.endsWith('.json') ? path : path + '.json'
    await jsonStore.saveToPath(savePath)
    jsonStore.setCurrentProjectPath(savePath)
  } catch {
    appStore.setErrorActive('Failed to save project.')
  }
}

async function handleMenuImportAudio() {
  const selected = await openDialog({
    multiple: true,
    title: 'Import Audio Files',
    filters: [{ name: 'Audio Files', extensions: ['mp3', 'wav', 'ogg'] }],
  })
  if (!Array.isArray(selected)) return
  const indexLength = jsonStore.configFile.files.length
  const soundlist = selected.map((file, index) => {
    const tabs = ['All']
    if (appStore.currentTab !== 'All') tabs.push(appStore.currentTab)
    return {
      name: file
        .replace(/^.*[\\\/]/, '')
        .replace(/\.(wav|mp3|ogg)$/i, '')
        .replaceAll('_', ' ')
        .replace(/([A-Z])/g, ' $1')
        .trim(),
      path: file,
      volume: 0.4,
      tabs,
      active: false,
      index: index + indexLength,
    }
  })
  jsonStore.addFiles(soundlist)
}

// Built-in themes — must match SettingsOverlay.vue
const builtinThemes = {
  'dark-cyan':   { '--primary_color': 'hsl(189, 100%, 58%)', '--color-bg': '#222831' },
  'dark-purple': { '--primary_color': 'hsl(270, 80%, 65%)',  '--color-bg': '#1e1b2e' },
  'dark-orange': { '--primary_color': 'hsl(28, 100%, 58%)',  '--color-bg': '#231c15' },
  'dark-green':  { '--primary_color': 'hsl(145, 80%, 50%)',  '--color-bg': '#162218' },
  'dark-pink':   { '--primary_color': 'hsl(330, 80%, 65%)',  '--color-bg': '#231520' },
}

function applyPersistedTheme(config) {
  const theme = config?.settings?.theme ?? 'dark-cyan'
  if (theme === 'custom') {
    const css = config?.settings?.customCss
    if (css) {
      let tag = document.getElementById('sn-custom-theme')
      if (!tag) { tag = document.createElement('style'); tag.id = 'sn-custom-theme'; document.head.appendChild(tag) }
      tag.textContent = css
    }
    return
  }
  const vars = builtinThemes[theme] ?? builtinThemes['dark-cyan']
  const root = document.documentElement
  Object.entries(vars).forEach(([k, v]) => root.style.setProperty(k, v))
}

onMounted(() => {
  // Restore locale from localStorage
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('sn-locale')
    if (saved) {
      setLocale(saved)
      invoke('rebuild_menu', { lang: saved }).catch(() => {})
    }
  }
  readConfigFile().then(applyPersistedTheme)
  listen('menu_open_settings', () => appStore.setActiveOverlay('settings'))
  listen('menu_open_about', () => appStore.openSettingsTab('about'))
  listen('menu_open_project', handleMenuOpenProject)
  listen('menu_save', handleMenuSave)
  listen('menu_save_as', handleMenuSaveAs)
  listen('menu_import_audio', handleMenuImportAudio)
  listen('menu_import_folders', () => appStore.setImportFoldersActive(true))
  listen('menu_select_project', () => appStore.setSelectProjectActive(true))
})
</script>

