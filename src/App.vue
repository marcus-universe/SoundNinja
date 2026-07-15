<template>
  <div class="soundninja flex_c_h flex_space_between">
    <template v-if="isMain">
      <TitleBar />
      <NavBar />
      <ErrorAlert />
      <SettingsOverlay />
      <ImportFolders v-if="appStore.importFoldersActive" />
      <ContextMenu />
      <DialogField
        v-if="unsavedPrompt"
        :title="$t('dialog.unsavedTitle')"
        @close="resolveUnsaved('cancel')"
      >
        <p class="dialog-text">{{ $t('dialog.unsavedMessage') }}</p>
        <div class="flex_c_h gap1 dialog-actions">
          <UIButton @click="resolveUnsaved('save')">{{ $t('dialog.save') }}</UIButton>
          <UIButton @click="resolveUnsaved('discard')">{{ $t('dialog.discard') }}</UIButton>
          <UIButton @click="resolveUnsaved('cancel')">{{ $t('dialog.cancel') }}</UIButton>
        </div>
      </DialogField>
      <Transition name="fade">
        <div v-if="jsonStore.saving" class="saving-indicator flex_c_h align_c gap1">
          <span class="saving-indicator__spinner" aria-hidden="true" />
          <span>{{ $t('common.saving') }}</span>
        </div>
      </Transition>
    </template>
    <NuxtPage v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </NuxtPage>
  </div>
</template>

<script setup>
import { readTextFile, rename, BaseDirectory } from '@tauri-apps/plugin-fs'
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { emit } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openPath } from '@tauri-apps/plugin-opener'
import { defaultSettings } from '~/utils/db'

const jsonStore = useJsonHandelingStore()
const appStore = useAppStore()
const appSettings = useAppSettingsStore()
const { setLocale } = useI18n()

// Secondary windows (e.g. the Theme Creator) reuse the same SPA bundle. Only the
// main window owns the project/menu lifecycle; others just render their page.
const isMain = ref(true)
if (import.meta.client) {
  try { isMain.value = getCurrentWindow().label === 'main' } catch { /* non-tauri */ }
}

function joinPath(base, ...parts) {
  const sep = base.includes('\\') ? '\\' : '/'
  return [base.replace(/[\\/]+$/, ''), ...parts].join(sep)
}

/** Migrate old JSON config shape to the current schema. */
function migrateConfig(obj) {
  if (Array.isArray(obj.tabList) && obj.tabList.length > 0 && typeof obj.tabList[0] === 'string') {
    obj.tabList = obj.tabList.map((name) => ({ name }))
  }
  if (obj.settings && typeof obj.settings.hue === 'number' && !obj.settings.theme) {
    obj.settings.theme = 'dark-cyan'
    delete obj.settings.hue
  }
  return obj
}

function toProjectConfig(obj) {
  return {
    settings: { ...defaultSettings(), ...(obj.settings ?? {}) },
    tabList: obj.tabList ?? [],
    files: obj.files ?? [],
    separators: obj.separators ?? [],
  }
}

function isValidConfig(obj) {
  return !!obj && typeof obj === 'object' && Array.isArray(obj.tabList)
    && typeof obj.settings === 'object' && Array.isArray(obj.files)
}

// ── Project bootstrap ─────────────────────────────────────────────────────────
// Opens a project DB and records it in the recent-projects list.
async function openProjectPath(dbPath) {
  await jsonStore.openProject(dbPath)
  try {
    await appSettings.touchRecent(dbPath, projectNameFromDbPath(dbPath))
  } catch (e) {
    // Project open is primary action; recents update failure should not abort flow.
    console.error('Failed to update recent projects', e)
  }
}

async function bootstrapProject() {
  // 1. Re-open the last project when it still exists.
  if (appSettings.lastProjectPath) {
    const exists = await invoke('path_exists_abs', { path: appSettings.lastProjectPath })
    if (exists) {
      await openProjectPath(appSettings.lastProjectPath)
      return
    }
  }
  // 2. First launch after upgrade — migrate the old AppData config.json.
  if (await tryMigrateOldConfig()) return
  // 3. Otherwise open/create the Default project.
  const dbPath = await createProjectFolder(appSettings.projectsPath, 'Default')
  await openProjectPath(dbPath)
}

async function tryMigrateOldConfig() {
  try {
    const txt = await readTextFile('config.json', { baseDir: BaseDirectory.AppData })
    const old = migrateConfig(JSON.parse(txt))
    const dbPath = await createProjectFolder(appSettings.projectsPath, 'Default')
    await jsonStore.importConfig(toProjectConfig(old), dbPath)
    await appSettings.touchRecent(dbPath, projectNameFromDbPath(dbPath))
    // Prevent re-migration on future launches.
    try {
      await rename('config.json', 'config.migrated.json', {
        oldPathBaseDir: BaseDirectory.AppData,
        newPathBaseDir: BaseDirectory.AppData,
      })
    } catch { /* non-critical */ }
    return true
  } catch {
    return false
  }
}

// ── Menu handlers ─────────────────────────────────────────────────────────────
// Unsaved-changes prompt (Save / Discard / Cancel). Resolves the pending promise
// so callers can await the user's decision before continuing.
const unsavedPrompt = ref(false)
let unsavedResolver = null

function confirmUnsaved() {
  return new Promise((resolve) => {
    if (!jsonStore.dirty) { resolve('discard'); return }
    unsavedResolver = resolve
    unsavedPrompt.value = true
  })
}

function resolveUnsaved(choice) {
  unsavedPrompt.value = false
  const resolve = unsavedResolver
  unsavedResolver = null
  if (resolve) resolve(choice)
}

async function handleMenuNewProject() {
  const choice = await confirmUnsaved()
  if (choice === 'cancel') return
  if (choice === 'save') await jsonStore.persistNow()
  try {
    const existing = await listProjects(appSettings.projectsPath)
    const names = new Set(existing.map((p) => safeProjectName(p.name).toLowerCase()))
    let name = 'New Project'
    let n = 2
    while (names.has(safeProjectName(name).toLowerCase())) name = `New Project ${n++}`
    const dbPath = await createProjectFolder(appSettings.projectsPath, name)
    await openProjectPath(dbPath)
  } catch (e) {
    console.error('Failed to create project', e)
    appStore.setErrorActive('Failed to create project.')
  }
}

async function handleMenuOpenProject() {
  const selected = await openDialog({
    title: 'Open Project',
    filters: [{ name: 'Sound Ninja Project', extensions: ['db'] }],
    multiple: false,
  })
  if (!selected || Array.isArray(selected)) return
  try {
    await openProjectPath(selected)
  } catch {
    appStore.setErrorActive('Failed to open project database.')
  }
}

async function handleMenuSave() {
  try {
    await jsonStore.persistNow()
  } catch {
    appStore.setErrorActive('Failed to save project.')
  }
}

async function handleMenuSaveAs() {
  const path = await saveDialog({
    title: 'Save Project As',
    filters: [{ name: 'Sound Ninja Project', extensions: ['db'] }],
    defaultPath: 'project.db',
  })
  if (!path) return
  try {
    const dbPath = path.endsWith('.db') ? path : path + '.db'
    await jsonStore.importConfig(jsonStore.configFile, dbPath)
    await appSettings.touchRecent(dbPath, projectNameFromDbPath(dbPath))
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
      tabIndexes: {},
    }
  })
  jsonStore.addFiles(soundlist)
}

// ── Theme application ─────────────────────────────────────────────────────────
const builtinThemes = {
  'dark-cyan':   { '--primary_color': 'hsl(189, 100%, 58%)', '--color-bg': '#222831' },
  'dark-purple': { '--primary_color': 'hsl(270, 80%, 65%)',  '--color-bg': '#1e1b2e' },
  'dark-orange': { '--primary_color': 'hsl(28, 100%, 58%)',  '--color-bg': '#231c15' },
  'dark-green':  { '--primary_color': 'hsl(145, 80%, 50%)',  '--color-bg': '#162218' },
  'dark-pink':   { '--primary_color': 'hsl(330, 80%, 65%)',  '--color-bg': '#231520' },
}

function injectThemeCss(css) {
  let tag = document.getElementById('sn-custom-theme')
  if (!tag) { tag = document.createElement('style'); tag.id = 'sn-custom-theme'; document.head.appendChild(tag) }
  tag.textContent = css
}

// Removes any inline theme CSS variables so an injected <style> theme (custom /
// file) can take effect (inline styles otherwise outrank :root rules).
function clearInlineThemeVars() {
  const root = document.documentElement
  ;[
    '--primary_color', '--color-bg', '--color-btn', '--sound-text',
    '--color-bg-light', '--color-bg-dark', '--color-btn-light', '--color-btn-dark',
    '--text-light', '--text-dark',
  ].forEach((v) => root.style.removeProperty(v))
}

async function applyPersistedTheme(config) {
  const s = config?.settings
  const theme = s?.theme ?? 'dark-cyan'
  if (theme === 'custom' || theme.startsWith('file:')) {
    // Injected-CSS themes define their own vars; clear inline overrides + just
    // toggle the light/dark root class.
    clearInlineThemeVars()
    const root = document.documentElement
    root.classList.toggle('theme-light', s?.themeMode === 'light')
    root.classList.toggle('theme-dark', s?.themeMode !== 'light')
    if (theme === 'custom') {
      if (s?.customCss) injectThemeCss(s.customCss)
    } else {
      const filename = theme.slice(5)
      try {
        const css = await invoke('read_text_file_abs', { path: joinPath(appSettings.themesPath, filename) })
        injectThemeCss(css)
      } catch (e) {
        console.error('Failed to load persisted theme file', e)
      }
    }
    return
  }
  // Builtin / default: the per-project color model is authoritative.
  document.getElementById('sn-custom-theme')?.remove()
  applyThemeColors(s)
}

onMounted(async () => {
  await appSettings.load()
  if (!isMain.value) {
    // Theme Creator (or any secondary window): apply the current locale/theme so
    // it matches the main window, but skip project + menu ownership.
    if (appSettings.locale) setLocale(appSettings.locale)
    appSettings.applyNavbarSide()
    return
  }
  if (appSettings.locale) {
    setLocale(appSettings.locale)
    invoke('rebuild_menu', { lang: appSettings.locale }).catch(() => {})
  }
  // Register any uploaded custom fonts so themed fonts render everywhere.
  loadCustomFonts(appSettings.fontsPath).catch(() => {})
  await bootstrapProject()
  await applyPersistedTheme(jsonStore.configFile)

  listen('menu_open_settings', () => appStore.setActiveOverlay('settings'))
  listen('menu_open_about', () => appStore.openSettingsTab('about'))
  listen('menu_new_project', handleMenuNewProject)
  listen('menu_open_project', handleMenuOpenProject)
  listen('menu_open_recent', async (e) => {
    const path = e?.payload
    if (!path || typeof path !== 'string') return
    const choice = await confirmUnsaved()
    if (choice === 'cancel') return
    if (choice === 'save') await jsonStore.persistNow()
    const stillExists = await invoke('path_exists_abs', { path })
    if (!stillExists) {
      await appSettings.removeRecentProject(path)
      appStore.setErrorActive('Project no longer exists.')
      return
    }
    try {
      await openProjectPath(path)
      await applyPersistedTheme(jsonStore.configFile)
    } catch {
      appStore.setErrorActive('Failed to open project database.')
    }
  })
  listen('menu_save', handleMenuSave)
  listen('menu_save_as', handleMenuSaveAs)
  listen('menu_import_audio', handleMenuImportAudio)
  listen('menu_import_folders', () => appStore.setImportFoldersActive(true))
  listen('menu_select_project', () => appStore.setSelectProjectActive(true))
  listen('menu_open_themes_folder', () => openPath(appSettings.themesPath).catch(() => {}))
  listen('menu_open_projects_folder', () => openPath(appSettings.projectsPath).catch(() => {}))

  // Prompt to save unsaved changes before the window closes.
  const mainWindow = getCurrentWindow()
  let allowClose = false
  mainWindow.onCloseRequested(async (event) => {
    if (allowClose || !jsonStore.dirty) return
    event.preventDefault()
    const choice = await confirmUnsaved()
    if (choice === 'cancel') return
    if (choice === 'save') await jsonStore.persistNow()
    allowClose = true
    await mainWindow.destroy()
  })

  // Live theme preview coming from the Theme Creator window.
  listen('theme_preview', (e) => {
    if (!e?.payload) return
    // Inline vars outrank the injected :root rule, so clear them all first, then
    // inject the preview CSS.
    clearInlineThemeVars()
    injectThemeCss(e.payload)
  })
  // Preview-only mode switch from the Theme Creator (does not persist themeMode).
  listen('theme_preview_mode', (e) => {
    const mode = e?.payload === 'light' ? 'light' : 'dark'
    document.documentElement.classList.toggle('theme-light', mode === 'light')
    document.documentElement.classList.toggle('theme-dark', mode === 'dark')
  })
  listen('theme_saved', () => applyPersistedTheme(jsonStore.configFile))
  // Theme Creator asks for the currently applied theme so it can open showing
  // exactly what is on screen. Reply with the computed CSS variables.
  listen('theme_request_current', () => {
    const cs = getComputedStyle(document.documentElement)
    const get = (n) => cs.getPropertyValue(n).trim()
    const mode = document.documentElement.classList.contains('theme-light') ? 'light' : 'dark'
    emit('theme_current', {
      '--primary_color': get('--primary_color'),
      '--color-bg': get('--color-bg'),
      '--color-btn': get('--color-btn'),
      '--color-bg-light': get('--color-bg-light'),
      '--color-bg-dark': get('--color-bg-dark'),
      '--color-btn-light': get('--color-btn-light'),
      '--color-btn-dark': get('--color-btn-dark'),
      '--text-light': get('--text-light'),
      '--text-dark': get('--text-dark'),
      '--font-btn': get('--font-btn'),
      '--font-tab': get('--font-tab'),
      '--font-size-btn': get('--font-size-btn'),
      '--font-size-tab': get('--font-size-tab'),
      '--font-size-md': get('--font-size-md'),
      '--btn_width': get('--btn_width'),
      '--border-radius': get('--border-radius'),
      '--btn-border-width': get('--btn-border-width'),
      '--button-gap': get('--button-gap'),
      '--btn_padding': get('--btn_padding'),
      __mode: mode,
    }).catch(() => {})
  })
  // Theme Creator "Save & Apply": persist the selected theme, then apply it.
  listen('theme_apply', async (e) => {
    const theme = e?.payload?.theme
    if (!theme) return
    jsonStore.setTheme(theme)
    document.getElementById('sn-custom-theme')?.remove()
    await applyPersistedTheme(jsonStore.configFile)
  })
})
</script>

