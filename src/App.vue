<template>
  <div class="soundninja flex_c_h flex_space_between">
    <NavBar />
    <ErrorAlert />
    <SettingsOverlay />
    <AboutOverlay />
    <ImportFolders v-if="appStore.importFoldersActive" />
    <NuxtPage v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </NuxtPage>
  </div>
</template>

<script setup>
import { readTextFile, writeTextFile, createDir, BaseDirectory } from '@tauri-apps/api/fs'
import { open as openDialog, save as saveDialog } from '@tauri-apps/api/dialog'
import { listen } from '@tauri-apps/api/event'

const jsonStore = useJsonHandelingStore()
const appStore = useAppStore()

const hue = computed(() => jsonStore.configFile?.settings?.hue)

watch(hue, (hueval) => {
  if (hueval !== undefined && hueval !== null) {
    document.documentElement.style.setProperty(
      '--primary_color',
      `hsl(${hueval}, 100%, 58%)`
    )
  }
})

const defaultConfig = {
  tabList: [],
  settings: {
    hue: 189,
    outputSource: 'default',
  },
  files: [],
}

async function readConfigFile() {
  try {
    const contents = JSON.parse(
      await readTextFile('config.json', { dir: BaseDirectory.App })
    )
    jsonStore.updateConfigFile(contents)
    return contents
  } catch {
    await createDir('', { dir: BaseDirectory.App, recursive: true })
    await writeTextFile(
      { path: 'config.json', contents: JSON.stringify(defaultConfig, null, 2) },
      { dir: BaseDirectory.App }
    )
    jsonStore.updateConfigFile(defaultConfig)
    return defaultConfig
  }
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

onMounted(() => {
  readConfigFile()
  listen('menu_open_settings', () => appStore.setActiveOverlay('settings'))
  listen('menu_open_about', () => appStore.setActiveOverlay('about'))
  listen('menu_open_project', handleMenuOpenProject)
  listen('menu_save', handleMenuSave)
  listen('menu_save_as', handleMenuSaveAs)
  listen('menu_import_audio', handleMenuImportAudio)
  listen('menu_import_folders', () => appStore.setImportFoldersActive(true))
})
</script>

