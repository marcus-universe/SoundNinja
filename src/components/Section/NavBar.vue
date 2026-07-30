<template>
  <div class="navbar flex_c_v">
    <div class="iconContainer flex_c_v flex_space_evenly gap1">
      <QuickInfo
        v-if="tooltipsOn"
        class="nav-tip nav-tip--search"
        :text="$t('navbar.search')"
        :side="tipSide"
      >
        <Icons
          :icon="'search'"
          :customClass="[
            'icon searchButton',
            { active: appStore.Searchbar.SearchbarActive },
          ]"
          @triggered="IconClicked"
        />
      </QuickInfo>
      <Icons
        v-else
        :icon="'search'"
        :customClass="[
          'icon searchButton',
          { active: appStore.Searchbar.SearchbarActive },
        ]"
        @triggered="IconClicked"
      />

      <transition name="slideIn">
        <div
          v-if="appStore.Searchbar.SearchbarActive"
          class="searchBar flex_c_h align_c flex_start gap1"
        >
          <div class="searchBar__field">
            <input
              ref="searchInput"
              type="text"
              :placeholder="$t('navbar.search')"
              v-model="appStore.Searchbar.SearchbarContent"
              @input="jsonStore.filterSounds(appStore.Searchbar.SearchbarContent)"
            />
            <button
              v-if="appStore.Searchbar.SearchbarContent"
              type="button"
              class="searchBar__clear"
              :title="$t('navbar.clearSearch')"
              @click="clearSearch"
            >
              <Icons icon="delete" custom-class="searchBar__clear-icon" />
            </button>
          </div>
        </div>
      </transition>

      <template v-for="(navelm, index) in appStore.navbar" :key="navelm">
        <QuickInfo
          v-if="tooltipsOn"
          class="nav-tip"
          :text="tipLabel(navelm)"
          :side="tipSide"
        >
          <Icons
            :icon="appStore.navbar[index]"
            @triggered="IconClicked"
          />
        </QuickInfo>
        <Icons
          v-else
          :icon="appStore.navbar[index]"
          @triggered="IconClicked"
        />
      </template>

      <QuickInfo
        v-if="tooltipsOn"
        class="nav-tip nav-tip--multiselect"
        :text="$t('navbar.multiSelect')"
        :side="tipSide"
      >
        <Icons
          :icon="appStore.multiSelectActive ? 'multiselect-active' : 'multiselect-inactive'"
          :customClass="[
            'icon multiSelectButton',
            { active: appStore.multiSelectActive },
          ]"
          @triggered="IconClicked"
        />
      </QuickInfo>
      <Icons
        v-else
        :icon="appStore.multiSelectActive ? 'multiselect-active' : 'multiselect-inactive'"
        :customClass="[
          'icon multiSelectButton',
          { active: appStore.multiSelectActive },
        ]"
        @triggered="IconClicked"
      />
    </div>

    <ImportChooser @choose="onImportChoice" />
  </div>
</template>

<script setup>
import { open } from '@tauri-apps/plugin-dialog'

const { t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()
const searchInput = ref(null)

const tooltipsOn = computed(() => appSettings.navbarTooltips !== false)
/** Tip opens toward the content area, opposite the sidebar edge. */
const tipSide = computed(() => (appSettings.navbarSide === 'right' ? 'left' : 'right'))

function tipLabel(icon) {
  const map = {
    add: 'navbar.import',
    upload: 'navbar.import',
    project: 'navbar.selectProject',
    settings: 'navbar.settings',
    about: 'navbar.about',
    folder: 'navbar.importFolders',
  }
  return t(map[icon] || 'navbar.search')
}

async function focusSearchInput() {
  await nextTick()
  // Wait a frame so the slide-in transition has mounted the input.
  requestAnimationFrame(() => {
    searchInput.value?.focus?.()
  })
}

async function uploadFiles() {
  const selected = await open({
    multiple: true,
    title: 'Select files to upload',
    filters: [
      {
        name: 'Add Sounds',
        extensions: ['mp3', 'wav', 'ogg'],
      },
    ],
  })

  if (Array.isArray(selected)) {
    const indexLength = jsonStore.configFile.files.length
    const soundlist = selected.map((file, index) => {
      const tabs = ['All']
      if (appStore.currentTab !== 'All') {
        tabs.push(appStore.currentTab)
      }
      return {
        name: file
          .replace(/^.*[\\]/, '')
          .replace('.wav', '')
          .replace('.mp3', '')
          .replace('.ogg', '')
          .replaceAll('_', ' ')
          .replace(/([A-Z])/g, ' $1')
          .trim(),
        path: file,
        volume: 0.4,
        tabs: tabs,
        active: false,
        index: index + indexLength,
      }
    })

    try {
      jsonStore.addFiles(soundlist)
    } catch (err) {
      console.log(err)
    }
  }
}

function clearSearch() {
  appStore.setSearchContent('')
  jsonStore.filterSounds('')
  focusSearchInput()
}

function OpenSearch() {
  const next = !appStore.Searchbar.SearchbarActive
  appStore.setSearchOpen(next)
  if (next) focusSearchInput()
}

/** Open search (or refocus if already open). Used by Ctrl/Cmd+F. */
function activateSearch() {
  if (!appStore.Searchbar.SearchbarActive) {
    appStore.setSearchOpen(true)
  }
  focusSearchInput()
}

async function onImportChoice(mode) {
  await nextTick()
  if (mode === 'audio') {
    await uploadFiles()
  } else if (mode === 'folders') {
    appStore.setImportFoldersActive(true)
  }
}

function IconClicked(icon) {
  if (icon === 'add' || icon === 'upload') {
    appStore.setImportChooserActive(true)
  } else if (icon === 'search') {
    OpenSearch()
  } else if (icon === 'project') {
    appStore.setSelectProjectActive(true)
  } else if (icon === 'folder') {
    appStore.setImportFoldersActive(true)
  } else if (icon === 'settings') {
    appStore.setActiveOverlay(appStore.activeOverlay === 'settings' ? null : 'settings')
  } else if (icon === 'about') {
    appStore.openSettingsTab('about')
  } else if (icon === 'check' || icon === 'multiselect-active' || icon === 'multiselect-inactive') {
    appStore.toggleMultiSelectActive()
  }
}

function onGlobalKeydown(e) {
  // Ctrl+F (Windows/Linux) or Cmd+F (macOS)
  if ((e.ctrlKey || e.metaKey) && !e.altKey && e.key.toLowerCase() === 'f') {
    e.preventDefault()
    activateSearch()
  }
}

watch(
  () => appStore.Searchbar.SearchbarActive,
  (active) => {
    if (active) focusSearchInput()
  },
)

onMounted(() => {
  window.addEventListener('keydown', onGlobalKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', onGlobalKeydown)
})
</script>
