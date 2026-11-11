<template>
  <div
    class="soundninja"
    :class="isMain ? 'flex_c_h flex_space_between' : 'soundninja--tool'"
  >
    <!-- Styled chrome for main + secondary (Record Editor / Theme Creator). -->
    <TitleBar v-if="!isRemote" />
    <RemoteChrome v-if="isRemote" />
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
      <!-- Lazy* keeps these panels out of the first paint; the inner v-if plus
           its transition stays intact because the wrapper latches on first use. -->
      <LazySettingsOverlay v-if="settingsEverOpened" />
      <LazyImportReview v-if="appStore.importReview" />
      <DropOverlay />
      <ContextMenu />
      <LazyGifPickerDialog v-if="appStore.gifPickerIndex != null" />
      <UpdateDialog ref="updateDialogRef" />
      <DialogField
        v-if="appStore.choicePrompt"
        :title="appStore.choicePrompt.title"
        @close="appStore.resolveChoice()"
      >
        <p class="dialog-text">{{ appStore.choicePrompt.message }}</p>
        <ul v-if="choiceHints.length" class="choice-hints">
          <li v-for="choice in choiceHints" :key="choice.id">
            <strong>{{ choice.label }}</strong> — {{ choice.hint }}
          </li>
        </ul>
        <div class="flex_c_h gap1 dialog-actions dialog-actions--wrap">
          <UIButton
            v-for="choice in appStore.choicePrompt.choices"
            :key="choice.id"
            @click="appStore.resolveChoice(choice.id)"
          >{{ choice.label }}</UIButton>
        </div>
      </DialogField>
      <DialogField
        v-if="appStore.saveOptionsPrompt"
        :title="$t('saveOptions.title')"
        @close="appStore.resolveSaveOptions(null)"
      >
        <div class="save-options">
          <p class="save-options__intro">
            {{ appStore.saveOptionsPrompt.firstSave ? $t('saveOptions.firstSaveHint') : $t('saveOptions.hint') }}
          </p>
          <UICheckbox
            v-model="saveOptions.embedAudio"
            class="save-options__option"
            :class="{ 'is-on': saveOptions.embedAudio }"
          >
            <span class="save-options__text">
              <span class="save-options__label">{{ $t('saveOptions.embedAudio') }}</span>
              <span class="save-options__hint">{{ $t('saveOptions.embedAudioHint') }}</span>
            </span>
          </UICheckbox>
          <UICheckbox
            v-model="saveOptions.embedGifs"
            class="save-options__option"
            :class="{ 'is-on': saveOptions.embedGifs }"
          >
            <span class="save-options__text">
              <span class="save-options__label">{{ $t('saveOptions.embedGifs') }}</span>
              <span class="save-options__hint">{{ $t('saveOptions.embedGifsHint') }}</span>
            </span>
          </UICheckbox>
          <div class="save-options__actions">
            <UIButton @click="appStore.resolveSaveOptions(null)">{{ $t('dialog.cancel') }}</UIButton>
            <UIButton @click="appStore.resolveSaveOptions({ ...saveOptions })">{{ $t('dialog.save') }}</UIButton>
          </div>
        </div>
      </DialogField>
      <LazyRelinkDialog v-if="appStore.relinkActive" />
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
      <DialogField
        v-if="transferPopup"
        :title="transferPopup.title"
        @close="() => {}"
      >
        <div class="stems-progress">
          <p class="dialog-text">{{ transferPopup.label }}</p>
          <div class="stems-progress__bar">
            <div class="stems-progress__fill" :style="{ width: transferPopup.percent + '%' }" />
          </div>
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
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog'
import { emit, listen as tauriListen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openPath } from '@tauri-apps/plugin-opener'
import { defaultSettings, defaultMeta, saveConfig, saveMeta, loadGifBlobsByIds, upsertGifBlob, withProjectDb } from '~/utils/db'
import { openProjectFlow, settleUnsavedChanges } from '~/utils/projectOpen'
import { eventToCombo } from '~/utils/hotkeys'
import {
  buildPortableExport,
  resolveImportedPaths,
  uniqueProjectFolderName,
} from '~/utils/soundboardZip'
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
  pickProjectFile,
  pickSaveProjectFile,
  projectNameFromDbPath,
  parentDir,
  safeProjectName,
} from '~/utils/projects'
import { presentImportPaths, pickAudioFilesAndPresent, pickFoldersAndPresent } from '~/utils/importDrop'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { resolveAppLocale } from '~/utils/locales'
import { publishRemoteState } from '~/utils/remote'
import { isRemote, REMOTE_LOCALE_KEY } from '~/utils/runtime'
import { bootRemoteSession, publishThemeFromDom, startWebuiHost } from '~/utils/webuiHost'

const jsonStore = useJsonHandelingStore()
const appStore = useAppStore()
const appSettings = useAppSettingsStore()
const { t, setLocale } = useI18n()

/**
 * Everything registered outside Vue's own bookkeeping (Tauri event handles,
 * watchers created after an `await`, timers). The root component only unmounts
 * when the window goes away, but leaving these dangling kept handlers alive
 * across window recreation.
 */
const teardown = []

/** `listen()` whose handle is released automatically on unmount. */
function listen(event, handler) {
  const p = tauriListen(event, handler)
  p.then((unlisten) => teardown.push(unlisten)).catch(() => {})
  return p
}

const updateDialogRef = ref(null)
// Latches on the first open so the settings chunk is fetched once and the
// overlay's own enter/leave transition keeps working afterwards.
const settingsEverOpened = ref(false)
watch(() => appStore.activeOverlay, (val) => {
  if (val === 'settings') settingsEverOpened.value = true
})
const savePopup = ref(null)
let savePopupTimer = null
const transferPopup = ref(null)

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

function runAppHotkey(action) {
  if (action === 'save') handleMenuSave()
  else if (action === 'saveAs') handleMenuSaveAs()
  else if (action === 'undo') jsonStore.undo()
  else if (action === 'redo') jsonStore.redo()
  else if (action === 'search') window.dispatchEvent(new CustomEvent('sn:activate-search'))
  else if (action === 'stopAll') {
    invoke('stop_all').catch(() => {})
    window.dispatchEvent(new CustomEvent('sn:stop-all'))
  }
}

function playSoundById(soundId) {
  if (!soundId) return
  window.dispatchEvent(new CustomEvent('sn:play-sound-id', { detail: soundId }))
}

function stopSoundById(soundId) {
  if (!soundId) return
  window.dispatchEvent(new CustomEvent('sn:stop-sound-id', { detail: soundId }))
}

function handleRemoteCommand(payload) {
  const action = payload?.action
  const id = typeof payload?.id === 'string' ? payload.id : ''
  if (action === 'trigger') {
    playSoundById(id)
    return
  }
  if (action === 'stop') {
    if (id) stopSoundById(id)
    else {
      invoke('stop_all').catch(() => {})
      window.dispatchEvent(new CustomEvent('sn:stop-all'))
    }
  }
}

let remotePublishTimer = null
function scheduleRemotePublish() {
  if (remotePublishTimer) clearTimeout(remotePublishTimer)
  remotePublishTimer = setTimeout(() => {
    remotePublishTimer = null
    if (!appSettings.remoteEnabled) return
    publishRemoteState(jsonStore.configFile?.files).catch(() => {})
  }, 150)
}

function onHotkeyKeydown(e) {
  const combo = eventToCombo(e)
  if (!combo) return
  const map = appSettings.hotkeys || {}
  const searchCombo = map.search
  if (searchCombo && searchCombo === combo) {
    e.preventDefault()
    e.stopPropagation()
    runHistoryAction(() => runAppHotkey('search'))
    return
  }
  if (isEditableTarget(e.target) || isEditableTarget(document.activeElement)) return
  for (const action of Object.keys(map)) {
    if (map[action] && map[action] === combo) {
      e.preventDefault()
      runHistoryAction(() => runAppHotkey(action))
      return
    }
  }
  if (appSettings.soundTriggersGlobal) return
  const triggers = jsonStore.configFile?.settings?.soundHotkeys || []
  const hit = triggers.find((row) => row.combo && row.combo === combo && row.soundId)
  if (hit) {
    e.preventDefault()
    playSoundById(hit.soundId)
  }
}

async function syncGlobalSoundHotkeys() {
  const bindings = (jsonStore.configFile?.settings?.soundHotkeys || [])
    .filter((row) => row.combo && row.soundId)
    .map((row) => ({ combo: row.combo, soundId: row.soundId }))
  try {
    await invoke('set_global_sound_hotkeys', {
      enabled: !!appSettings.soundTriggersGlobal,
      bindings,
    })
  } catch (e) {
    console.error('Failed to sync global sound hotkeys', e)
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
    tags: obj.tags ?? [],
  }
}

function isValidConfig(obj) {
  return !!obj && typeof obj === 'object' && Array.isArray(obj.tabList)
    && typeof obj.settings === 'object' && Array.isArray(obj.files)
}

// ── Project bootstrap ─────────────────────────────────────────────────────────
// Opens a project (converting old formats on request) and records it as recent.
// Resolves false when the user cancelled.
function openProjectPath(dbPath, opts) {
  return openProjectFlow(dbPath, opts)
}

async function createFreshProject() {
  const existing = await listProjects(appSettings.projectsPath)
  const names = new Set(existing.map((p) => safeProjectName(p.name).toLowerCase()))
  let name = t('common.newProject')
  let n = 2
  while (names.has(safeProjectName(name).toLowerCase())) name = `${t('common.newProject')} ${n++}`
  const dbPath = await createProjectFolder(appSettings.projectsPath, name)
  await openProjectPath(dbPath, { settleUnsaved: false })
}

async function bootstrapProject() {
  // 1. Re-open the last project when it still exists.
  const last = appSettings.lastProjectPath
  if (last) {
    const exists = await invoke('path_exists_abs', { path: last })
    if (exists && await openProjectPath(last, { settleUnsaved: false })) return
  }
  // 2. First launch after upgrade — migrate the old AppData config.json.
  if (!last && await tryMigrateOldConfig()) return
  // 3. Otherwise open/create the Default project.
  const dbPath = await createProjectFolder(appSettings.projectsPath, 'Default')
  if (dbPath !== last && await openProjectPath(dbPath, { settleUnsaved: false })) return
  // 4. The user declined to convert what was found: start with an empty board.
  await createFreshProject()
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
// Dialog state for "what goes into the project file".
const saveOptions = reactive({ embedAudio: false, embedGifs: true })
watch(() => appStore.saveOptionsPrompt, (prompt) => {
  if (!prompt) return
  saveOptions.embedAudio = !!prompt.embedAudio
  saveOptions.embedGifs = !!prompt.embedGifs
})

const choiceHints = computed(() => (appStore.choicePrompt?.choices ?? []).filter((c) => c.hint))

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

/** After a manual save: say so if audio meant to be stored in the project was missing. */
function reportSaveIssues() {
  const missing = jsonStore.lastSaveReport?.missingAudio ?? []
  if (!missing.length) return
  const shown = missing.slice(0, 8).join('\n')
  const more = missing.length > 8 ? '\n…' : ''
  appStore.setErrorActive(`${t('saveOptions.embedMissing', { count: missing.length })}\n\n${shown}${more}`)
}

function currentSaveOptions() {
  const meta = jsonStore.projectMeta ?? defaultMeta()
  return { embedAudio: !!meta.embedAudio, embedGifs: meta.embedGifs !== false }
}

async function handleMenuNewProject() {
  if (!(await settleUnsavedChanges())) return
  try {
    await createFreshProject()
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
  try {
    if (await openProjectPath(dbPath)) await applyPersistedTheme(jsonStore.configFile)
  } catch (e) {
    console.error('Failed to open project', e)
    appStore.setErrorActive(`${t('common.failedOpenProject')}\n\n${formatError(e)}`)
  }
}

async function handleMenuSave() {
  // First save of a project: choose what the file should contain.
  if (!jsonStore.projectMeta?.saveOptionsSet) {
    const picked = await appStore.askSaveOptions(currentSaveOptions(), true)
    if (!picked) return
    jsonStore.setSaveOptions(picked)
  }
  if (await persistOrReport()) reportSaveIssues()
}

async function handleMenuSaveAs() {
  const picked = await appStore.askSaveOptions(currentSaveOptions(), false)
  if (!picked) return
  const dbPath = await pickSaveProjectFile()
  if (!dbPath) return
  const saved = await withSavePopup(async () => {
    await jsonStore.saveAs(dbPath, picked)
    await appSettings.touchRecent(dbPath, projectNameFromDbPath(dbPath))
  })
  if (saved) reportSaveIssues()
}

async function handleMenuImportAudio() {
  await pickAudioFilesAndPresent()
}

function joinOsPath(base, ...parts) {
  const sep = base.includes('\\') ? '\\' : '/'
  return [base.replace(/[\\/]+$/, ''), ...parts].join(sep)
}

async function handleMenuExportSoundboard() {
  const zipPath = await saveDialog({
    title: t('soundboard.exportTitle'),
    filters: [{ name: 'ZIP', extensions: ['zip'] }],
    defaultPath: `${projectNameFromDbPath(jsonStore.currentProjectPath || 'soundboard')}.zip`,
  })
  if (!zipPath) return
  transferPopup.value = { title: t('soundboard.exportTitle'), label: t('soundboard.exporting'), percent: 0 }
  const unlisten = await listen('soundboard_export_progress', (e) => {
    const pct = Number(e?.payload?.percent)
    if (Number.isFinite(pct)) transferPopup.value = { ...transferPopup.value, percent: pct }
  })
  let tempDir = null
  const livePath = jsonStore.currentProjectPath
  try {
    // The export is built from the board in memory; nothing has to be saved
    // first (with autosave off that would silently commit unsaved changes).
    const { portable, entries } = buildPortableExport(jsonStore.configFile)
    tempDir = await invoke('make_temp_dir')
    const tempDb = joinOsPath(tempDir, 'project.sninja')
    const gifIds = [...new Set(portable.files.map((f) => f.gifId).filter(Boolean))]
    let blobs = []
    if (livePath && gifIds.length) {
      blobs = await withProjectDb(livePath, (d) => loadGifBlobsByIds(d, gifIds))
    }
    // An exported board always carries its images, wherever this project keeps them.
    await withProjectDb(tempDb, async (d) => {
      await saveConfig(d, portable, portable.files.length ? undefined : { allowEmpty: true })
      await saveMeta(d, { ...defaultMeta(), embedGifs: true, saveOptionsSet: true })
      for (const row of blobs) await upsertGifBlob(d, row)
    }, { embedGifs: true })
    const allEntries = [{ src: tempDb, destRel: 'project.sninja' }, ...entries]
    await invoke('export_soundboard_zip', { zipPath, entries: allEntries })
  } catch (e) {
    console.error('Export soundboard failed', e)
    appStore.setErrorActive(`${t('soundboard.exportFailed')}\n\n${formatError(e)}`)
  } finally {
    unlisten()
    transferPopup.value = null
    if (tempDir) {
      try { await invoke('delete_dir_abs', { path: tempDir }) } catch { /* ignore */ }
    }
  }
}

async function handleMenuImportSoundboard() {
  if (!(await settleUnsavedChanges())) return

  const selected = await openDialog({
    title: t('soundboard.importTitle'),
    filters: [{ name: 'ZIP', extensions: ['zip'] }],
    multiple: false,
  })
  const zipPath = Array.isArray(selected) ? selected[0] : selected
  if (!zipPath) return

  transferPopup.value = { title: t('soundboard.importTitle'), label: t('soundboard.importing'), percent: 0 }
  const unlisten = await listen('soundboard_import_progress', (e) => {
    const pct = Number(e?.payload?.percent)
    if (Number.isFinite(pct)) transferPopup.value = { ...transferPopup.value, percent: pct }
  })
  try {
    const zipBase = String(zipPath).replace(/^.*[\\/]/, '').replace(/\.zip$/i, '')
    const existing = await listProjects(appSettings.projectsPath)
    const names = new Set(existing.map((p) => safeProjectName(p.name).toLowerCase()))
    const folder = uniqueProjectFolderName(names, zipBase || t('common.newProject'))
    const destDir = joinOsPath(appSettings.projectsPath, folder)
    const dbPath = await invoke('import_soundboard_zip', { zipPath, destDir })
    // Sounds in the archive are stored relative to project.sninja; opening
    // resolves them against the folder the ZIP was unpacked to.
    if (!(await openProjectPath(dbPath, { settleUnsaved: false }))) return
    const opened = jsonStore.currentProjectPath || dbPath
    resolveImportedPaths(jsonStore.configFile, parentDir(opened))
    await jsonStore.persistNow()
    await jsonStore.validateSoundPaths()
    if (jsonStore.missingPaths.length) appStore.setRelinkActive(true)
    await applyPersistedTheme(jsonStore.configFile)
    await syncGlobalSoundHotkeys()
  } catch (e) {
    console.error('Import soundboard failed', e)
    appStore.setErrorActive(`${t('soundboard.importFailed')}\n\n${formatError(e)}`)
  } finally {
    unlisten()
    transferPopup.value = null
  }
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
    const layoutRe = /(--font-btn|--font-tab|--font-size-btn|--font-size-tab|--font-size-md|--btn_width|--border-radius|--btn-border-width|--tab-border-width|--button-gap|--btn_padding|--gif-overlay-hover|--gif-overlay|--title-chip-pad-x|--title-chip-pad-y)\s*:\s*([^;]+);/g
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
    publishThemeFromDom()
    return
  }
  // Builtin / default: the per-project color model is authoritative.
  document.getElementById('sn-custom-theme')?.remove()
  applyThemeTokens(s, getPreset(theme)?.extras)
  publishThemeFromDom()
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
  if (isRemote) {
    splashLabel.value = t('remoteUi.connecting')
    const savedLocale = localStorage.getItem(REMOTE_LOCALE_KEY)
    if (savedLocale) {
      try { await setLocale(savedLocale) } catch { /* unknown code */ }
    }
    bootRemoteSession({
      setLocale: (code) => setLocale(code),
      onReady: () => { booting.value = false },
    })
    settingsEverOpened.value = true
    return
  }
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
    // Theme Creator / Record Editor: apply locale + CSS chrome only.
    // Do not call set_window_chrome — decorations on all windows recreate HWND on Windows.
    // Tab list for Record Editor comes via `record_context` events from main.
    if (appSettings.locale) setLocale(appSettings.locale)
    appSettings.applyNavbarSide()
    await appSettings.applyChromeCss()
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
  jsonStore.setAutosave(appSettings.autosave, appSettings.autosaveIntervalSec)
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
  splashLabel.value = t('splash.ready')
  splashPercent.value = 100
  setTimeout(() => { booting.value = false }, 280)

  // Fetch the settings chunk once the board is up. Opening settings is then
  // instant, and first paint never waited for it.
  const whenIdle = window.requestIdleCallback ?? ((cb) => setTimeout(cb, 1500))
  whenIdle(() => { settingsEverOpened.value = true })

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
    const stillExists = await invoke('path_exists_abs', { path })
    if (!stillExists) {
      await appSettings.removeRecentProject(path)
      appStore.setErrorActive(t('common.projectGone'))
      return
    }
    try {
      if (await openProjectPath(path)) await applyPersistedTheme(jsonStore.configFile)
    } catch (err) {
      console.error('Failed to open project', err)
      appStore.setErrorActive(`${t('common.failedOpenProject')}\n\n${formatError(err)}`)
    }
  })
  listen('menu_save', handleMenuSave)
  listen('menu_save_as', handleMenuSaveAs)
  listen('menu_import_audio', handleMenuImportAudio)
  listen('menu_import_folders', () => { pickFoldersAndPresent() })
  listen('menu_export_soundboard', handleMenuExportSoundboard)
  listen('menu_import_soundboard', handleMenuImportSoundboard)
  listen('menu_select_project', () => appStore.setSelectProjectActive(true))
  listen('global_sound_hotkey', (e) => {
    const id = e?.payload
    if (typeof id === 'string') playSoundById(id)
  })
  listen('remote_command', (e) => {
    handleRemoteCommand(e?.payload)
  })
  listen('menu_open_themes_folder', () => openPath(appSettings.themesPath).catch(() => {}))
  listen('menu_open_projects_folder', () => openPath(appSettings.projectsPath).catch(() => {}))

  getCurrentWebview().onDragDropEvent((event) => {
    const kind = event?.payload?.type
    if (kind === 'enter' || kind === 'over') {
      appStore.setDropActive(true)
    } else if (kind === 'leave') {
      appStore.setDropActive(false)
    } else if (kind === 'drop') {
      appStore.setDropActive(false)
      const paths = event?.payload?.paths
      if (Array.isArray(paths) && paths.length) presentImportPaths(paths)
    }
  }).then((unlisten) => teardown.push(unlisten)).catch(() => {})

  window.addEventListener('keydown', onHotkeyKeydown, true)
  await syncGlobalSoundHotkeys()
  teardown.push(watch(
    () => [appSettings.autosave, appSettings.autosaveIntervalSec],
    () => { jsonStore.setAutosave(appSettings.autosave, appSettings.autosaveIntervalSec) },
  ))
  // A background save that failed must not go unnoticed.
  teardown.push(watch(
    () => jsonStore.saveError,
    (message) => {
      if (message) appStore.setErrorActive(`${t('common.failedSaveProject')}\n\n${message}`)
    },
  ))
  // Created after an await, so Vue no longer binds them to this instance —
  // their stop handles have to be collected by hand.
  teardown.push(watch(
    () => [
      appSettings.soundTriggersGlobal,
      jsonStore.currentProjectPath,
      JSON.stringify(jsonStore.configFile?.settings?.soundHotkeys || []),
    ],
    () => { syncGlobalSoundHotkeys() },
  ))

  if (appSettings.remoteEnabled) {
    try { await appSettings.applyRemote() } catch { /* remoteError set in store */ }
    scheduleRemotePublish()
  }
  teardown.push(watch(
    () => jsonStore.configFile?.files,
    () => { scheduleRemotePublish() },
    { deep: true },
  ))
  teardown.push(watch(
    () => appSettings.remoteEnabled,
    (on) => { if (on) scheduleRemotePublish() },
  ))

  teardown.push(startWebuiHost())
  if (appSettings.webuiEnabled) {
    try { await appSettings.applyWebui() } catch { /* webuiError set in store */ }
  }

  // Prompt to save unsaved changes before the window closes.
  const mainWindow = getCurrentWindow()
  let allowClose = false
  mainWindow.onCloseRequested(async (event) => {
    if (allowClose || !jsonStore.dirty) return
    event.preventDefault()
    // Autosave on: flush quietly. Off: Save / Discard / Cancel.
    if (!(await settleUnsavedChanges())) return
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
    publishThemeFromDom()
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
      '--title-chip-pad-x', '--title-chip-pad-y',
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
      volume: 1,
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
    window.removeEventListener('keydown', onHotkeyKeydown, true)
  }
  for (const release of teardown.splice(0)) {
    try { release() } catch { /* already gone */ }
  }
  clearTimeout(savePopupTimer)
  savePopupTimer = null
  clearTimeout(remotePublishTimer)
  remotePublishTimer = null
})
</script>

