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
              v-if="canClear"
              type="button"
              class="searchBar__clear"
              :title="$t('navbar.clearSearch')"
              @click="clearSearch"
            >
              <Icons icon="delete" custom-class="searchBar__clear-icon" />
            </button>
          </div>
          <div
            v-if="projectTags.length"
            ref="searchChips"
            class="searchBar__tags"
          >
            <button
              v-for="tag in projectTags"
              :key="tag.id"
              type="button"
              class="tag-chip"
              :class="{ 'is-on': appStore.selectedTagIds.includes(tag.id) }"
              :style="{ '--tag-color': tag.color }"
              @click="appStore.toggleSelectedTag(tag.id)"
            >
              <span class="tag-chip__dot" />
              {{ tag.name }}
            </button>
          </div>
        </div>
      </transition>

      <FilterPanel />

      <template v-for="(navelm, index) in appStore.navbar" :key="navelm">
        <QuickInfo
          v-if="tooltipsOn"
          class="nav-tip"
          :text="tipLabel(navelm)"
          :side="tipSide"
        >
          <Icons
            :icon="appStore.navbar[index]"
            :customClass="navIconClass(navelm)"
            @triggered="IconClicked"
          />
        </QuickInfo>
        <Icons
          v-else
          :icon="appStore.navbar[index]"
          :customClass="navIconClass(navelm)"
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
import { pickAudioFilesAndPresent, pickFoldersAndPresent } from '~/utils/importDrop'

const { t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()
const searchInput = ref(null)

const tooltipsOn = computed(() => appSettings.navbarTooltips !== false)
/** Tip opens toward the content area, opposite the sidebar edge. */
const tipSide = computed(() => (appSettings.navbarSide === 'right' ? 'left' : 'right'))

function navIconClass(icon) {
  if (icon === 'filter') return ['icon', 'filter-toggle', { active: appStore.filterPanelOpen }]
  return undefined
}

function tipLabel(icon) {
  const map = {
    add: 'navbar.import',
    upload: 'navbar.import',
    project: 'navbar.selectProject',
    filter: 'navbar.filter',
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

const projectTags = computed(() => jsonStore.configFile.tags ?? [])
const searchChips = ref(null)
const canClear = computed(() =>
  !!appStore.Searchbar.SearchbarContent
  || appStore.selectedTagIds.length > 0
  || (jsonStore.configFile.settings?.sortMode && jsonStore.configFile.settings.sortMode !== 'user'),
)

function syncSearchExtra() {
  if (typeof document === 'undefined') return
  const h = appStore.Searchbar.SearchbarActive && projectTags.value.length
    ? (searchChips.value?.offsetHeight || 0)
    : 0
  document.documentElement.style.setProperty('--search-extra', `${h}px`)
}

watch(
  () => [appStore.Searchbar.SearchbarActive, projectTags.value.length, appStore.selectedTagIds.length],
  async () => {
    await nextTick()
    syncSearchExtra()
  },
)

function clearSearch() {
  appStore.clearBoardFilters()
  focusSearchInput()
}

function OpenSearch() {
  const next = !appStore.Searchbar.SearchbarActive
  appStore.setSearchOpen(next)
  if (next) focusSearchInput()
}

async function onImportChoice(mode) {
  await nextTick()
  if (mode === 'audio') {
    await pickAudioFilesAndPresent()
  } else if (mode === 'folders') {
    await pickFoldersAndPresent()
  }
}

function IconClicked(icon) {
  if (icon === 'add' || icon === 'upload') {
    appStore.setImportChooserActive(true)
  } else if (icon === 'search') {
    OpenSearch()
  } else if (icon === 'project') {
    appStore.setSelectProjectActive(true)
  } else if (icon === 'filter') {
    appStore.toggleFilterPanel()
  } else if (icon === 'folder') {
    pickFoldersAndPresent()
  } else if (icon === 'settings') {
    appStore.setActiveOverlay(appStore.activeOverlay === 'settings' ? null : 'settings')
  } else if (icon === 'about') {
    appStore.openSettingsTab('about')
  } else if (icon === 'check' || icon === 'multiselect-active' || icon === 'multiselect-inactive') {
    appStore.toggleMultiSelectActive()
  }
}

watch(
  () => appStore.Searchbar.SearchbarActive,
  (active) => {
    if (active) focusSearchInput()
  },
)

onMounted(() => {
  window.addEventListener('sn:activate-search', OpenSearch)
})

onUnmounted(() => {
  window.removeEventListener('sn:activate-search', OpenSearch)
})
</script>
