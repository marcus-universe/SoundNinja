<template>
  <div v-if="showCustomBar" class="titlebar-wrapper">
    <!-- Top row: app title + window controls -->
    <div class="titlebar" data-tauri-drag-region>
      <span class="titlebar__title" data-tauri-drag-region>Sound Ninja</span>
      <div class="titlebar__controls">
        <button class="titlebar__btn titlebar__btn--min" @click="minimize" aria-label="Minimize">
          <svg viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect x="1" y="5.5" width="10" height="1" rx="0.5" fill="currentColor"/>
          </svg>
        </button>
        <button class="titlebar__btn titlebar__btn--max" @click="toggleMaximize" aria-label="Maximize">
          <svg viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect x="1.5" y="1.5" width="9" height="9" rx="0.5" stroke="currentColor" stroke-width="1.2" fill="none"/>
          </svg>
        </button>
        <button class="titlebar__btn titlebar__btn--close" @click="closeWindow" aria-label="Close">
          <svg viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
            <line x1="1.5" y1="1.5" x2="10.5" y2="10.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
            <line x1="10.5" y1="1.5" x2="1.5" y2="10.5" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    </div>

    <!-- Second row: app menu bar -->
    <div class="menubar" @mouseleave="closeMenu">
      <div
        v-for="menu in menus"
        :key="menu.id"
        class="menubar__item"
        :class="{ active: openMenu === menu.id }"
        @click="toggleMenu(menu.id)"
        @mouseenter="openMenu && openMenu !== menu.id ? openMenu = menu.id : undefined"
      >
        {{ $t(menu.labelKey) }}
        <div v-if="openMenu === menu.id" class="menubar__dropdown">
          <template v-for="item in menu.items" :key="item.id">
            <div v-if="item.sep" class="menubar__sep" />
            <div
              v-else-if="item.id === 'recent_sub'"
              class="menubar__entry menubar__entry--sub"
            >
              <span>{{ $t(item.labelKey) }}</span>
              <span class="menubar__arrow">▶</span>
              <div class="menubar__subdropdown">
                <div
                  v-if="recentProjects.length === 0"
                  class="menubar__entry menubar__entry--disabled"
                >{{ $t('menu.noRecent') }}</div>
                <div
                  v-for="r in recentProjects"
                  :key="r.dbPath"
                  class="menubar__entry"
                  @click.stop="emitEvent('menu_open_recent', r.dbPath); closeMenu()"
                >{{ r.name }}</div>
              </div>
            </div>
            <div
              v-else-if="isActionItem(item)"
              class="menubar__entry"
              @click.stop="emitEvent(item.event); closeMenu()"
            >{{ $t(item.labelKey) }}</div>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { platform } from '@tauri-apps/plugin-os'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { emit } from '@tauri-apps/api/event'

const appSettings = useAppSettingsStore()

const showCustomBar = ref(false)
const openMenu = ref<string | null>(null)

const recentProjects = computed(() => appSettings.recentProjects ?? [])

type MenuSeparator = {
  id: string
  sep: true
}

type MenuRecentSub = {
  id: 'recent_sub'
  labelKey: string
}

type MenuAction = {
  id: string
  labelKey: string
  event: string
}

type MenuItem = MenuSeparator | MenuRecentSub | MenuAction

type MenuGroup = {
  id: string
  labelKey: string
  items: MenuItem[]
}

const menus: MenuGroup[] = [
  {
    id: 'file',
    labelKey: 'menu.file',
    items: [
      { id: 'new_project',    labelKey: 'menu.newProject',    event: 'menu_new_project' },
      { id: 'open_project',   labelKey: 'menu.openProject',   event: 'menu_open_project' },
      { id: 'recent_sub',     labelKey: 'menu.openRecent' },
      { id: 'select_project', labelKey: 'menu.selectProject', event: 'menu_select_project' },
      { id: 'sep1', sep: true },
      { id: 'save',           labelKey: 'menu.save',          event: 'menu_save' },
      { id: 'save_as',        labelKey: 'menu.saveAs',        event: 'menu_save_as' },
      { id: 'sep2', sep: true },
      { id: 'import_audio',   labelKey: 'menu.importAudio',   event: 'menu_import_audio' },
      { id: 'import_folders', labelKey: 'menu.importFolders', event: 'menu_import_folders' },
      { id: 'sep3', sep: true },
      { id: 'quit',           labelKey: 'menu.quit',          event: 'menu_quit' },
    ],
  },
  {
    id: 'edit',
    labelKey: 'menu.edit',
    items: [
      { id: 'open_settings',        labelKey: 'menu.settings',           event: 'menu_open_settings' },
      { id: 'sep4', sep: true },
      { id: 'open_themes_folder',   labelKey: 'menu.openThemesFolder',   event: 'menu_open_themes_folder' },
      { id: 'open_projects_folder', labelKey: 'menu.openProjectsFolder', event: 'menu_open_projects_folder' },
    ],
  },
  {
    id: 'help',
    labelKey: 'menu.help',
    items: [
      { id: 'about', labelKey: 'menu.about', event: 'menu_open_about' },
    ],
  },
]

function isActionItem(item: MenuItem): item is MenuAction {
  return !('sep' in item) && item.id !== 'recent_sub'
}

function toggleMenu(id: string) {
  openMenu.value = openMenu.value === id ? null : id
}

function closeMenu() {
  openMenu.value = null
}

async function emitEvent(event: string, payload?: unknown) {
  if (event === 'menu_quit') {
    const win = getCurrentWindow()
    await win.close()
    return
  }
  await emit(event, payload)
}

onMounted(async () => {
  try {
    const p = await platform()
    showCustomBar.value = p === 'windows' || p === 'linux'
    if (showCustomBar.value) {
      document.documentElement.style.setProperty('--topbar_height', '5.6rem')
    }
  } catch {
    showCustomBar.value = false
  }
})

const win = getCurrentWindow()
function minimize() { win.minimize() }
function toggleMaximize() { win.toggleMaximize() }
function closeWindow() { win.close() }
</script>
