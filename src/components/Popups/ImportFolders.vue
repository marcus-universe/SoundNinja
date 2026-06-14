<template>
  <DialogField title="Import Folders" @close="close">
    <div class="flex_c gap1 w100 flex_wrap">
      <UIButton @click="selectFolders">Select Folders</UIButton>
      <UICheckbox v-model="importAsTabs">Import Folders as Tabs</UICheckbox>
    </div>

    <ul v-if="selectedFolders.length" class="ui-list flex_c_v gap1">
      <UIListItem
        v-for="folder in selectedFolders"
        :key="folder"
        :label="folderName(folder)"
        @remove="removeFolder(folder)"
      />
    </ul>

    <UIButton v-if="selectedFolders.length" :full-width="true" @click="doImport">
      Import Folders
    </UIButton>
  </DialogField>
</template>

<script setup>
import { open } from '@tauri-apps/api/dialog'
import { readDir } from '@tauri-apps/api/fs'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const selectedFolders = ref([])
const importAsTabs = ref(false)

const AUDIO_EXTENSIONS = ['mp3', 'wav', 'ogg']

function folderName(path) {
  return path.replace(/^.*[\\\/]/, '')
}

function isAudioFile(name) {
  if (!name) return false
  const ext = name.split('.').pop()?.toLowerCase()
  return AUDIO_EXTENSIONS.includes(ext)
}

function removeFolder(path) {
  selectedFolders.value = selectedFolders.value.filter((f) => f !== path)
}

function close() {
  appStore.setImportFoldersActive(false)
  selectedFolders.value = []
  importAsTabs.value = false
}

async function selectFolders() {
  const result = await open({ directory: true, multiple: true, title: 'Select Folders' })
  if (!result) return
  const folders = Array.isArray(result) ? result : [result]
  selectedFolders.value = [...new Set([...selectedFolders.value, ...folders])]
}

async function doImport() {
  let indexLength = jsonStore.configFile.files.length
  const allFiles = []

  for (const folder of selectedFolders.value) {
    let entries = []
    try {
      entries = await readDir(folder, { recursive: false })
    } catch {
      appStore.setErrorActive(`Failed to read folder: ${folderName(folder)}`)
      return
    }

    const audioEntries = entries.filter((e) => isAudioFile(e.name))
    const name = folderName(folder)

    if (importAsTabs.value && !jsonStore.configFile.tabList.includes(name)) {
      jsonStore.configFile.tabList.push(name)
    }

    for (const entry of audioEntries) {
      const tabs = ['All']
      if (importAsTabs.value) {
        tabs.push(name)
      } else if (appStore.currentTab !== 'All') {
        tabs.push(appStore.currentTab)
      }
      allFiles.push({
        name: (entry.name ?? '')
          .replace(/\.(wav|mp3|ogg)$/i, '')
          .replaceAll('_', ' ')
          .replace(/([A-Z])/g, ' $1')
          .trim(),
        path: entry.path,
        volume: 0.4,
        tabs,
        active: false,
        index: indexLength++,
      })
    }
  }

  if (allFiles.length > 0) {
    jsonStore.addFiles(allFiles)
  }

  close()
}
</script>

