<template>
  <DialogField v-if="selectedFolders.length" title="Import Folders" @close="close">
    <div v-if="selectedFolders.length" class="import-folders-toolbar flex_c gap1 w100 flex_wrap">
      <UIButton class="import-folders-select" @click="selectFolders()">Add Folders</UIButton>
    </div>

    <ul
      v-if="selectedFolders.length"
      ref="folderListRef"
      class="ui-list import-folders-list flex_c_v gap1"
    >
      <li
        v-for="folder in selectedFolders"
        :key="folder"
        class="ui-listitem import-folder-item flex_c flex_space_between gap1"
      >
        <span
          class="import-folder-item__drag"
          title="Drag to reorder"
          aria-label="Drag to reorder"
        >
          <Icons :icon="'drag'" :customClass="'icon import-folder-item__drag-icon'" />
        </span>

        <span class="ui-listitem-label">{{ folderName(folder) }}</span>

        <Icons
          :icon="'exit'"
          :customClass="'ui-listitem-remove icon import-folder-item__remove'"
          @triggered="removeFolder(folder)"
        />
      </li>
    </ul>

    <div v-if="selectedFolders.length" class="import-folders-actions flex_c gap1 w100">
      <UIButton class="import-folders-submit" :full-width="true" @click="doImport('tabs')">
        Import as Tabs
      </UIButton>
      <UIButton class="import-folders-submit" :full-width="true" @click="doImport('current')">
        Import to current Tab
      </UIButton>
    </div>
  </DialogField>
</template>

<script setup>
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import Sortable from 'sortablejs'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const selectedFolders = ref([])
const folderListRef = ref(null)
let sortable = null

const AUDIO_EXTENSIONS = ['mp3', 'wav', 'ogg']
const MAX_SUBFOLDER_DEPTH = 2

function folderName(path) {
  return path.replace(/^.*[\\\/]/, '')
}

function isAudioFile(name) {
  if (!name) return false
  const ext = name.split('.').pop()?.toLowerCase()
  return AUDIO_EXTENSIONS.includes(ext)
}

function normalizeDir(path) {
  return path.replace(/[\\/]+$/, '')
}

function joinPath(base, name) {
  return normalizeDir(base) + '/' + name
}

function removeFolder(path) {
  selectedFolders.value = selectedFolders.value.filter((f) => f !== path)
  if (selectedFolders.value.length === 0) close()
}

function close() {
  appStore.setImportFoldersActive(false)
  selectedFolders.value = []
}

function initSortable() {
  if (sortable || !folderListRef.value) return
  sortable = Sortable.create(folderListRef.value, {
    animation: 180,
    draggable: '.import-folder-item',
    handle: '.import-folder-item__drag',
    ghostClass: 'drag-over',
    onEnd(evt) {
      const { oldIndex, newIndex } = evt
      if (oldIndex == null || newIndex == null || oldIndex === newIndex) return
      const next = [...selectedFolders.value]
      const [moved] = next.splice(oldIndex, 1)
      if (!moved) return
      next.splice(newIndex, 0, moved)
      selectedFolders.value = next
    },
  })
}

function destroySortable() {
  sortable?.destroy()
  sortable = null
}

watch(
  () => selectedFolders.value.length,
  async (len) => {
    await nextTick()
    if (len > 1) initSortable()
    else destroySortable()
  },
  { immediate: true }
)

onUnmounted(() => {
  destroySortable()
})

onMounted(() => {
  selectFolders({ closeIfEmpty: true })
})

async function selectFolders(options = {}) {
  const { closeIfEmpty = false } = options
  const result = await open({ directory: true, multiple: true, title: 'Select Folders' })
  // Restore WebView focus after native dialog closes (Tauri/Windows focus loss bug)
  try { await getCurrentWindow().setFocus() } catch { /* non-tauri */ }
  if (!result) {
    if (closeIfEmpty && selectedFolders.value.length === 0) close()
    return
  }
  const folders = Array.isArray(result) ? result : [result]
  selectedFolders.value = [...new Set([...selectedFolders.value, ...folders])]
}

async function doImport(mode = 'current') {
  const importAsTabs = mode === 'tabs'
  let indexLength = jsonStore.configFile.files.length
  const allFiles = []

  let buckets = []
  try {
    buckets = await invoke('collect_audio_buckets_abs', {
      roots: selectedFolders.value,
      maxDepth: MAX_SUBFOLDER_DEPTH,
      exts: AUDIO_EXTENSIONS,
    })
  } catch (e) {
    appStore.setErrorActive(String(e))
    return
  }

  for (const bucket of buckets) {
    if (importAsTabs && !jsonStore.configFile.tabList.some((t) => t.name === bucket.name)) {
      jsonStore.addTab(bucket.name)
    }

    for (const fileName of bucket.audioFiles) {
      if (!isAudioFile(fileName)) continue
      const tabs = ['All']
      if (importAsTabs) {
        tabs.push(bucket.name)
      } else if (appStore.currentTab !== 'All') {
        tabs.push(appStore.currentTab)
      }
      allFiles.push({
        name: fileName
          .replace(/\.(wav|mp3|ogg)$/i, '')
          .replaceAll('_', ' ')
          .replace(/([A-Z])/g, ' $1')
          .trim(),
        path: joinPath(bucket.dir, fileName),
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

