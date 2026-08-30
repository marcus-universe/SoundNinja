<template>
  <div
    class="soundninja"
    :class="isMain ? 'flex_c_h flex_space_between' : 'soundninja--tool'"
  >
    <!-- Styled chrome for main + secondary (Record Editor / Theme Creator). -->
    <TitleBar />
    <Transition name="splash-fade">
      <Splash v-if="isMain && booting" :percent="splashPercent" :label="splashLabel" />
    </Transition>
    <LanguagePicker
      v-if="isMain && languagePickerOpen"
      :initial="languagePickerInitial"
      @done="onFirstLanguage"
    />
    <template v-if="isMain">
      <NavBar />
      <ErrorAlert />
      <SettingsOverlay />
      <ImportFolders v-if="appStore.importFoldersActive" />
      <ContextMenu />
      <GifPickerDialog v-if="appStore.gifPickerIndex != null" />
      <UpdateDialog ref="updateDialogRef" />
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
      <RelinkDialog v-if="appStore.relinkActive" />
      <DialogField
        v-if="savePopup"
        :title="savePopup === 'saved' ? $t('common.savedTitle') : $t('common.saving')"
        @close="savePopup = null"
      >
        <div class="flex_c_v align_c gap1 save-popup-body">
          <span v-if="savePopup === 'saving'" class="saving-indicator__spinner" aria-hidden="true" />
          <p class="dialog-text">{{ savePopup === 'saved' ? $t('common.saved') : $t('common.saving') }}</p>
        </div>
      </DialogField>
    </template>
    <NuxtPage v-if="isMain" v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </NuxtPage>
    <div v-else class="soundninja__tool-page">
      <div v-if="!toolReady" class="tool-loading flex_c_v align_c">
        <span class="saving-indicator__spinner" aria-hidden="true" />
        <span>{{ $t('common.loading') }}</span>
      </div>
      <NuxtPage v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" @vue:mounted="toolReady = true" />
        </transition>
      </NuxtPage>
    </div>
  </div>
</template>

<script setup>
import { readTextFile, rename, BaseDirectory } from '@tauri-apps/plugin-fs'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { emit, listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openPath } from '@tauri-apps/plugin-opener'
import { defaultSettings } from '~/utils/db'
import { getPreset, normalizeThemeId } from '~/utils/themePresets'
import {
  applyThemeTokens,
  THEME_INLINE_VARS,
  parseThemeCss,
  buildThemeCss,
  parseThemeName,
  THEME_TOKEN_DEFAULTS,
} from '~/utils/themeTokens'
import {
  createProjectFolder,
  listProjects,
  pickOpenPaths,
  pickProjectFile,
  pickSaveProjectFile,
  projectNameFromDbPath,
  safeProjectName,
} from '~/utils/projects'
import { resolveAppLocale } from '~/utils/locales'

const jsonStore = useJsonHandelingStore()
const appStore = useAppStore()
const appSettings = useAppSettingsStore()
const { t, setLocale } = useI18n()

const updateDialogRef = ref(null)
const savePopup = ref(null)
let savePopupTimer = null

// Secondary windows (e.g. the Theme Creator) reuse the same SPA bundle. Only the
// main window owns the project/menu lifecycle; others just render their page.
const isMain = ref(true)
const toolReady = ref(false)
const booting = ref(true)
const splashPercent = ref(8)
const splashLabel = ref('')
const languagePickerOpen = ref(false)
const languagePickerInitial = ref('en')
let languagePickerResolve = null
let unlistenChrome = null
if (import.meta.client) {
  try { isMain.value = getCurrentWindow().label === 'main' } catch { /* non-tauri */ }
  if (!isMain.value) {
    // Fallback if page mount event is missed.
    setTimeout(() => { toolReady.value = true }, 800)
  }
}

function joinPath(base, ...parts) {
  const sep = base.includes('\\') ? '\\' : '/'
  return [base.replace(/[\\/]+$/, ''), ...parts].join(sep)
}

/** True when native text undo should win over soundboard undo. */
function isEditableTarget(el) {
  if (!el || typeof el !== 'object') return false
  const tag = el.tagName
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true
  if (el.isContentEditable) return true
  return typeof el.closest === 'function' && !!el.closest('[contenteditable="true"]')
}

/** Coalesce menu-accelerator + window keydown double-fires. */
let lastHistoryActionAt = 0
function runHistoryAction(fn) {
  const now = Date.now()
  if (now - lastHistoryActionAt < 80) return
  lastHistoryActionAt = now
  fn()
}

function onUndoRedoKeydown(e) {
  if (!(e.ctrlKey || e.metaKey) || e.altKey) return
  if (isEditableTarget(e.target) || isEditableTarget(document.activeElement)) return
  const key = e.key.toLowerCase()
  if (key === 'z' && e.shiftKey) {
    e.preventDefault()
    runHistoryAction(() => jsonStore.redo())
  } else if (key === 'z') {
    e.preventDefault()
    runHistoryAction(() => jsonStore.undo())
  } else if (key === 'y' && !e.shiftKey) {
    e.preventDefault()
    runHistoryAction(() => jsonStore.redo())
  } else if (key === 's' && e.shiftKey) {
    e.preventDefault()
    runHistoryAction(() => { handleMenuSaveAs() })
  } else if (key === 's') {
    e.preventDefault()
    runHistoryAction(() => { handleMenuSave() })
  }
}

/** Migrate old JSON config shape to the current schema. */
function migrateConfig(obj) {
  if (Array.isArray(obj.tabList) && obj.tabList.length > 0 && typeof obj.tabList[0] === 'string') {
    obj.tabList = obj.tabList.map((name) => ({ name }))
  }
  if (obj.settings && typeof obj.settings.hue === 'number' && !obj.settings.theme) {
    obj.settings.theme = 'soundninja'
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
  await jsonStore.validateSoundPaths()
  if (jsonStore.missingPaths.length) appStore.setRelinkActive(true)
  if (jsonStore.configFile?.settings?.gpuAudioEnabled) {
    invoke('set_gpu_audio', { enabled: true }).catch(() => {})
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

async function withSavePopup(work) {
  if (savePopupTimer) {
    clearTimeout(savePopupTimer)
    savePopupTimer = null
  }
  savePopup.value = 'saving'
  try {
    await work()
    savePopup.value = 'saved'
    savePopupTimer = setTimeout(() => { savePopup.value = null }, 1000)
    return true
  } catch (e) {
    savePopup.value = null
    console.error('Failed to save project', e)
    appStore.setErrorActive(`${t('common.failedSaveProject')}\n\n${formatError(e)}`)
    return false
  }
}

async function persistOrReport() {
  return withSavePopup(() => jsonStore.persistNow())
}

async function handleMenuNewProject() {
  const choice = await confirmUnsaved()
  if (choice === 'cancel') return
  if (choice === 'save' && !(await persistOrReport())) return
  try {
    const existing = await listProjects(appSettings.projectsPath)
    const names = new Set(existing.map((p) => safeProjectName(p.name).toLowerCase()))
    let name = t('common.newProject')
    let n = 2
    while (names.has(safeProjectName(name).toLowerCase())) name = `${t('common.newProject')} ${n++}`
    const dbPath = await createProjectFolder(appSettings.projectsPath, name)
    await openProjectPath(dbPath)
  } catch (e) {
    console.error('Failed to create project', e)
    appStore.setErrorActive(`${t('common.failedCreateProject')}\n\n${formatError(e)}`)
  }
}

function formatError(e) {
  if (e == null) return t('common.unknownError')
  if (typeof e === 'string') return e
  if (e instanceof Error) return e.message || String(e)
  try { return JSON.stringify(e) } catch { return String(e) }
}

async function handleMenuOpenProject() {
  const dbPath = await pickProjectFile()
  if (!dbPath) return
  const choice = await confirmUnsaved()
  if (choice === 'cancel') return
  if (choice === 'save' && !(await persistOrReport())) return
  try {
    await openProjectPath(dbPath)
    await applyPersistedTheme(jsonStore.configFile)
  } catch (e) {
    console.error('Failed to open project', e)
    appStore.setErrorActive(`${t('common.failedOpenProject')}\n\n${formatError(e)}`)
  }
}

async function handleMenuSave() {
  await persistOrReport()
}

async function handleMenuSaveAs() {
  const dbPath = await pickSaveProjectFile()
  if (!dbPath) return
  await withSavePopup(async () => {
    await jsonStore.saveAs(dbPath)
    await appSettings.touchRecent(dbPath, projectNameFromDbPath(dbPath))
  })
}

async function handleMenuImportAudio() {
  const selected = await openDialog({
    multiple: true,
    title: t('common.importAudioFiles'),
    filters: [{ name: t('common.audioFiles'), extensions: ['mp3', 'wav', 'ogg'] }],
  })
  const files = pickOpenPaths(selected)
  if (!files.length) return
  const indexLength = jsonStore.configFile.files.length
  const soundlist = files.map((file, index) => {
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
function injectThemeCss(css) {
  let tag = document.getElementById('sn-custom-theme')
  if (!tag) { tag = document.createElement('style'); tag.id = 'sn-custom-theme'; document.head.appendChild(tag) }
  // Normalize legacy light/dark pair themes into flat :root tokens.
  const parsed = parseThemeCss(css)
  if (parsed.primaryColor || parsed.bg || parsed.btnBg) {
    const tokens = { ...THEME_TOKEN_DEFAULTS, ...parsed }
    const name = parseThemeName(css) || 'theme'
    // Keep layout extras from the original file by appending after flat rebuild.
    const flat = buildThemeCss(name, tokens)
    const layoutRe = /(--font-btn|--font-tab|--font-size-btn|--font-size-tab|--font-size-md|--btn_width|--border-radius|--btn-border-width|--tab-border-width|--button-gap|--btn_padding|--gif-overlay-hover|--gif-overlay)\s*:\s*([^;]+);/g
    const extras = []
    let m
    while ((m = layoutRe.exec(css)) !== null) extras.push(`  ${m[1]}: ${m[2]};`)
    tag.textContent = extras.length
      ? flat.replace(/\n}\s*$/, `\n${extras.join('\n')}\n}`)
      : flat
  } else {
    tag.textContent = css
  }
}

// Removes any inline theme CSS variables so an injected <style> theme (custom /
// file) can take effect (inline styles otherwise outrank :root rules).
function clearInlineThemeVars() {
  const root = document.documentElement
  THEME_INLINE_VARS.forEach((v) => root.style.removeProperty(v))
}

watch(
  () => jsonStore.currentProjectPath,
  async (path, prev) => {
    if (!isMain.value || !path || path === prev) return
    await applyPersistedTheme(jsonStore.configFile)
  },
)

async function applyPersistedTheme(config) {
  const s = config?.settings
  const theme = normalizeThemeId(s?.theme)
  if (theme === 'custom' || theme.startsWith('file:')) {
    // Injected-CSS themes define their own vars; clear inline overrides.
    clearInlineThemeVars()
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
  applyThemeTokens(s, getPreset(theme)?.extras)
}

async function applyBootLocale(code) {
  const next = resolveAppLocale(code)
  await setLocale(next)
  await appSettings.setLocale(next)
  invoke('rebuild_menu', { lang: next }).catch(() => {})
}

function onFirstLanguage(code) {
  languagePickerOpen.value = false
  const resolve = languagePickerResolve
  languagePickerResolve = null
  if (resolve) resolve(code)
}

function waitForLanguagePick(initial) {
  languagePickerInitial.value = initial
  languagePickerOpen.value = true
  return new Promise((resolve) => {
    languagePickerResolve = resolve
  })
}

onMounted(async () => {
  splashLabel.value = t('splash.settings')
  splashPercent.value = 12
  await appSettings.load()
  splashPercent.value = 28

  unlistenChrome = await listen('sn:chrome-changed', (e) => {
    const payload = e?.payload
    if (!payload || typeof payload !== 'object') return
    appSettings.applyChromeFromEvent(payload)
  })

  if (!isMain.value) {
    // Theme Creator / Record Editor: apply locale + chrome, skip project/menu ownership.
    // Tab list for Record Editor comes via `record_context` events from main.
    if (appSettings.locale) setLocale(appSettings.locale)
    appSettings.applyNavbarSide()
    await appSettings.applyWindowChrome()
    return
  }

  if (appSettings.locale) {
    await applyBootLocale(appSettings.locale)
  } else {
    let installed = null
    try { installed = await invoke('read_install_language') } catch { /* ignore */ }
    const guessed = resolveAppLocale(installed || (typeof navigator !== 'undefined' ? navigator.language : 'en'))
    if (installed) {
      await applyBootLocale(guessed)
    } else {
      const picked = await waitForLanguagePick(guessed)
      await applyBootLocale(picked)
    }
  }
  splashLabel.value = t('splash.fonts')
  splashPercent.value = 48
  // Register any uploaded custom fonts so themed fonts render everywhere.
  loadCustomFonts(appSettings.fontsPath).catch(() => {})
  splashLabel.value = t('splash.project')
  splashPercent.value = 68
  try {
    await bootstrapProject()
  } catch (e) {
    console.error('Failed to bootstrap project', e)
    appStore.setErrorActive(`${t('common.failedOpenProject')}\n\n${formatError(e)}`)
  }
  // One-time: pull audio device/volume prefs out of the project into app-config.db.
  await appSettings.migrateAudioFromProject(jsonStore.configFile?.settings)
  splashLabel.value = t('splash.theme')
  splashPercent.value = 88
  await applyPersistedTheme(jsonStore.configFile)
  if (jsonStore.configFile?.settings?.gpuAudioEnabled) {
    invoke('set_gpu_audio', { enabled: true }).catch(() => {})
  }
  splashLabel.value = t('splash.ready')
  splashPercent.value = 100
  setTimeout(() => { booting.value = false }, 280)

  listen('menu_open_settings', () => appStore.setActiveOverlay('settings'))
  listen('menu_open_about', () => appStore.openSettingsTab('about'))
  listen('menu_check_updates', () => {
    updateDialogRef.value?.checkManual?.()
  })
  listen('menu_undo', () => runHistoryAction(() => jsonStore.undo()))
  listen('menu_redo', () => runHistoryAction(() => jsonStore.redo()))
  listen('menu_new_project', handleMenuNewProject)
  listen('menu_open_project', handleMenuOpenProject)
  listen('menu_open_recent', async (e) => {
    const path = e?.payload
    if (!path || typeof path !== 'string') return
    const choice = await confirmUnsaved()
    if (choice === 'cancel') return
    if (choice === 'save' && !(await persistOrReport())) return
    const stillExists = await invoke('path_exists_abs', { path })
    if (!stillExists) {
      await appSettings.removeRecentProject(path)
      appStore.setErrorActive(t('common.projectGone'))
      return
    }
    try {
      await openProjectPath(path)
      await applyPersistedTheme(jsonStore.configFile)
    } catch (err) {
      console.error('Failed to open project', err)
      appStore.setErrorActive(`${t('common.failedOpenProject')}\n\n${formatError(err)}`)
    }
  })
  listen('menu_save', handleMenuSave)
  listen('menu_save_as', handleMenuSaveAs)
  listen('menu_import_audio', handleMenuImportAudio)
  listen('menu_import_folders', () => appStore.setImportFoldersActive(true))
  listen('menu_select_project', () => appStore.setSelectProjectActive(true))
  listen('menu_open_themes_folder', () => openPath(appSettings.themesPath).catch(() => {}))
  listen('menu_open_projects_folder', () => openPath(appSettings.projectsPath).catch(() => {}))

  window.addEventListener('keydown', onUndoRedoKeydown)

  // Prompt to save unsaved changes before the window closes.
  const mainWindow = getCurrentWindow()
  let allowClose = false
  mainWindow.onCloseRequested(async (event) => {
    if (allowClose || !jsonStore.dirty) return
    event.preventDefault()
    const choice = await confirmUnsaved()
    if (choice === 'cancel') return
    if (choice === 'save' && !(await persistOrReport())) return
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
  listen('theme_saved', () => applyPersistedTheme(jsonStore.configFile))
  // Theme Creator asks for the currently applied theme so it can open showing
  // exactly what is on screen. Reply with the computed CSS variables.
  listen('theme_request_current', () => {
    const cs = getComputedStyle(document.documentElement)
    const get = (n) => cs.getPropertyValue(n).trim()
    const payload = {}
    for (const cssVar of THEME_INLINE_VARS) {
      payload[cssVar] = get(cssVar)
    }
    // Layout extras used by Theme Creator draft.
    ;[
      '--font-btn', '--font-tab', '--font-size-btn', '--font-size-tab', '--font-size-md',
      '--btn_width', '--border-radius', '--btn-border-width', '--tab-border-width', '--button-gap', '--btn_padding',
      '--gif-overlay', '--gif-overlay-hover',
    ].forEach((n) => { payload[n] = get(n) })
    emit('theme_current', payload).catch(() => {})
  })
  // Theme Creator "Save & Apply": persist the selected theme, then apply it.
  listen('theme_apply', async (e) => {
    const theme = e?.payload?.theme
    if (!theme) return
    jsonStore.setTheme(theme)
    document.getElementById('sn-custom-theme')?.remove()
    await applyPersistedTheme(jsonStore.configFile)
  })

  // Record Editor: share current tab list / selection with the secondary window.
  listen('record_request_context', () => {
    emit('record_context', {
      currentTab: appStore.currentTab,
      tabList: (jsonStore.configFile?.tabList || []).map((t) => t.name),
    }).catch(() => {})
  })

  // Optional silent update check after startup settles (popup only if update exists).
  if (appSettings.checkUpdatesOnStart !== false) {
    setTimeout(() => {
      updateDialogRef.value?.checkOnStart?.()
    }, 2500)
  }

  listen('record_import_sound', (e) => {
    const path = e?.payload?.path
    const tabs = Array.isArray(e?.payload?.tabs) ? e.payload.tabs : ['All']
    if (!path || typeof path !== 'string') return
    const indexLength = jsonStore.configFile.files.length
    const rawName = typeof e?.payload?.name === 'string' && e.payload.name.trim()
      ? e.payload.name.trim()
      : path
          .replace(/^.*[\\/]/, '')
          .replace(/\.(wav|mp3|ogg|flac)$/i, '')
          .replaceAll('_', ' ')
          .replace(/([A-Z])/g, ' $1')
          .trim()
    const name = rawName || `Recording ${indexLength + 1}`
    jsonStore.addFiles([{
      name,
      path,
      volume: 0.4,
      tabs,
      active: false,
      index: indexLength,
      tabIndexes: {},
    }])
  })
})

onUnmounted(() => {
  if (unlistenChrome) {
    unlistenChrome()
    unlistenChrome = null
  }
  if (isMain.value) {
    window.removeEventListener('keydown', onUndoRedoKeydown)
  }
})
</script>

