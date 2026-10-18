<template>
  <section>
    <h2 class="settings-content__title">{{ $t('settings.main.title') }}</h2>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.main.language') }}</label>
      <UISelect
        :model-value="currentLanguage"
        :options="languageOptions"
        :aria-label="$t('settings.main.language')"
        @update:model-value="pickLanguage"
      />
    </div>

    <div v-if="isRemote" class="settings-group settings-group--stacked">
      <NumberSlider
        :label="$t('settings.themeCreator.uiScale')"
        :model-value="remoteScale"
        :min="0.5"
        :max="2"
        :step="0.01"
        :reset-value="1"
        @update:model-value="onRemoteScale"
      />
      <p class="settings-hint">{{ $t('settings.themeCreator.uiScaleHint') }}</p>
    </div>

    <div v-if="isRemote" class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('remoteUi.keepAwake') }}</span>
        <span class="settings-hint">{{ $t('remoteUi.keepAwakeHint') }}</span>
      </div>
      <UICheckbox :modelValue="keepAwake" @update:modelValue="onWake" />
    </div>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.main.theme') }}</label>
      <div class="settings-row settings-row--nowrap">
        <UISelect
          :model-value="selectedTheme"
          :options="themeOptions"
          :aria-label="$t('settings.main.theme')"
          @update:model-value="pickTheme"
        />
        <button v-if="!isRemote" class="settings-btn settings-btn--primary settings-btn--icon-only" :title="$t('settings.main.openThemeCreator')" @click="openThemeCreator">
          <Icons icon="palette" custom-class="settings-btn-icon" />
        </button>
      </div>
    </div>

    <Transition name="fade">
      <div v-if="selectedTheme === 'custom'" class="settings-group settings-group--stacked">
        <label class="settings-label">{{ $t('settings.main.customCss') }}</label>
        <p class="settings-hint">{{ $t('settings.main.importCssHint') }}</p>
        <div class="settings-row" style="margin-top: 0.5rem">
          <button v-if="!isRemote" class="settings-btn" @click="importCustomCssFile">{{ $t('settings.main.importCssFile') }}</button>
        </div>
        <p v-if="importError" class="settings-error" style="margin-top:0.5rem">{{ importError }}</p>
        <p v-if="importSuccess" class="settings-success" style="margin-top:0.5rem">{{ importSuccess }}</p>
      </div>
    </Transition>

    <DialogField
      v-if="nameDialogOpen"
      :title="$t('settings.main.themeNoNameTitle')"
      :errorMessage="nameDialogError"
      @close="closeNameDialog"
    >
      <p class="settings-hint">{{ $t('settings.main.themeNoNameHint') }}</p>
      <input
        type="text"
        class="settings-input"
        v-model="nameDialogInput"
        :placeholder="$t('settings.main.themeNamePlaceholder')"
        @keyup.enter="confirmNameDialog"
      />
      <div class="settings-row" style="margin-top: 0.75rem">
        <button class="settings-btn settings-btn--primary" @click="confirmNameDialog" :disabled="!nameDialogInput.trim()">{{ $t('settings.main.themeNoNameConfirm') }}</button>
      </div>
    </DialogField>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.checkUpdatesOnStart') }}</span>
        <span class="settings-hint">{{ $t('settings.main.checkUpdatesOnStartHint') }}</span>
      </div>
      <UICheckbox :modelValue="checkUpdatesOnStart" @update:modelValue="onCheckUpdatesOnStart" />
    </div>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { openSecondaryWindow, THEME_CREATOR } from '~/utils/secondaryWindows'
import { THEME_PRESETS, DEFAULT_THEME_ID, normalizeThemeId } from '~/utils/themePresets'
import {
  THEME_INLINE_VARS,
  applyThemeTokens,
  parseThemeCss,
  buildThemeCss,
  parseThemeName,
  THEME_TOKEN_DEFAULTS,
} from '~/utils/themeTokens'
import { isAppLocale, type AppLocale } from '~/utils/locales'
import { applyUiScale, clampUiScale, isRemote, REMOTE_LOCALE_KEY, REMOTE_SCALE_KEY, REMOTE_WAKE_KEY, syncWakeLock } from '~/utils/runtime'

const { t, locale, locales: availableLocales, setLocale } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

function joinPath(base: string, ...parts: string[]) {
  const sep = base.includes('\\') ? '\\' : '/'
  return [base.replace(/[\\/]+$/, ''), ...parts].join(sep)
}

async function openThemeCreator() {
  try {
    await openSecondaryWindow(THEME_CREATOR)
  } catch (e) {
    console.error('Failed to open Theme Creator window', e)
  }
  appStore.setActiveOverlay(null)
}

const builtinThemes = THEME_PRESETS

const selectedTheme = ref(DEFAULT_THEME_ID)
const savedThemes = ref<string[]>([])
const importError = ref('')
const importSuccess = ref('')
const checkUpdatesOnStart = ref(true)
const currentLanguage = ref(locale.value)
const remoteScale = ref(1)
const keepAwake = ref(false)

onMounted(() => {
  if (!isRemote) return
  const stored = localStorage.getItem(REMOTE_SCALE_KEY)
  const raw = stored != null ? Number(stored) : 1
  remoteScale.value = clampUiScale(raw)
  applyUiScale(remoteScale.value)
  keepAwake.value = localStorage.getItem(REMOTE_WAKE_KEY) === '1'
  syncWakeLock().catch(() => {})
})

function onRemoteScale(value: number) {
  remoteScale.value = applyUiScale(value)
  localStorage.setItem(REMOTE_SCALE_KEY, String(remoteScale.value))
}

function onWake(value: boolean) {
  keepAwake.value = value
  localStorage.setItem(REMOTE_WAKE_KEY, value ? '1' : '0')
  syncWakeLock().catch(() => {})
}

const nameDialogOpen = ref(false)
const nameDialogInput = ref('')
const nameDialogError = ref('')
const pendingCss = ref('')

const languageOptions = computed(() =>
  availableLocales.value.map((loc) => ({ value: String(loc.code), label: String(loc.name || loc.code) })),
)

const themeOptions = computed(() => [
  ...builtinThemes.map((theme) => ({ value: theme.id, label: theme.label })),
  ...savedThemes.value.map((name) => ({ value: 'file:' + name, label: name.replace('.css', '') })),
  { value: 'custom', label: t('settings.main.customCssOption') },
])

async function pickLanguage(value: string) {
  currentLanguage.value = value
  await changeLanguage()
}

function pickTheme(value: string) {
  selectedTheme.value = value
  applyTheme()
}

async function changeLanguage() {
  const next = (isAppLocale(currentLanguage.value) ? currentLanguage.value : 'en') as AppLocale
  await setLocale(next)
  if (isRemote) {
    localStorage.setItem(REMOTE_LOCALE_KEY, next)
    return
  }
  await appSettings.setLocale(next)
  try {
    await invoke('rebuild_menu', { lang: next })
  } catch { /* not available in all envs */ }
}

function applyTheme() {
  const id = selectedTheme.value
  jsonStore.setTheme(id)
  if (id === 'custom') {
    clearInlineThemeVars()
    return
  }
  if (id.startsWith('file:')) {
    clearInlineThemeVars()
    const filename = id.slice(5)
    invoke<string>('read_text_file_abs', { path: joinPath(appSettings.themesPath, filename) })
      .then(injectCustomCss)
      .catch((e) => console.error('Failed to load theme file', e))
    return
  }
  removeCustomCssTag()
  const theme = builtinThemes.find((t) => t.id === id)
  if (!theme) return
  jsonStore.setThemeColors(theme.tokens)
  applyThemeTokens(jsonStore.configFile?.settings as unknown as Record<string, unknown>, theme.extras)
}

function clearInlineThemeVars() {
  const root = document.documentElement
  THEME_INLINE_VARS.forEach((k) => root.style.removeProperty(k))
}

function removeCustomCssTag() {
  document.getElementById('sn-custom-theme')?.remove()
}

function injectCustomCss(css: string) {
  let tag = document.getElementById('sn-custom-theme')
  if (!tag) {
    tag = document.createElement('style')
    tag.id = 'sn-custom-theme'
    document.head.appendChild(tag)
  }
  const parsed = parseThemeCss(css)
  if (parsed.primaryColor || parsed.bg || parsed.btnBg) {
    const tokens = { ...THEME_TOKEN_DEFAULTS, ...parsed }
    const name = parseThemeName(css) || 'theme'
    const flat = buildThemeCss(name, tokens)
    const layoutRe = /(--font-btn|--font-tab|--font-size-btn|--font-size-tab|--font-size-md|--btn_width|--border-radius|--btn-border-width|--tab-border-width|--button-gap|--btn_padding|--gif-overlay-hover|--gif-overlay|--title-chip-pad-x|--title-chip-pad-y)\s*:\s*([^;]+);/g
    const extras: string[] = []
    let m: RegExpExecArray | null
    while ((m = layoutRe.exec(css)) !== null) extras.push(`  ${m[1]}: ${m[2]};`)
    tag.textContent = extras.length
      ? flat.replace(/\n}\s*$/, `\n${extras.join('\n')}\n}`)
      : flat
  } else {
    tag.textContent = css
  }
}

async function loadSavedThemes() {
  try {
    await invoke('make_dir_abs', { path: appSettings.themesPath })
    const files = await invoke<string[]>('list_dir_files_abs', {
      dir: appSettings.themesPath,
      exts: ['css'],
    })
    savedThemes.value = files
  } catch {
    savedThemes.value = []
  }
}

async function importCustomCssFile() {
  importError.value = ''
  importSuccess.value = ''
  try {
    const filePath = await open({
      multiple: false,
      title: t('native.importThemeCss'),
      filters: [{ name: 'CSS Files', extensions: ['css'] }],
    })
    if (!filePath) return
    const css = await invoke<string>('read_text_file_abs', { path: filePath as string })
    if (!css.includes('--')) {
      importError.value = t('settings.main.notCompatibleTheme')
      return
    }
    const name = parseThemeName(css)
    if (name) {
      await saveImportedTheme(css, name)
    } else {
      pendingCss.value = css
      nameDialogInput.value = ''
      nameDialogError.value = ''
      nameDialogOpen.value = true
    }
  } catch (e) {
    importError.value = `Failed: ${e}`
  }
}

async function saveImportedTheme(css: string, name: string) {
  const safeName = name.replace(/[^a-z0-9_-]/gi, '_')
  await invoke('make_dir_abs', { path: appSettings.themesPath })
  await invoke('write_text_file_abs', {
    path: joinPath(appSettings.themesPath, `${safeName}.css`),
    contents: css,
  })
  importSuccess.value = t('settings.main.importSuccess')
  await loadSavedThemes()
  selectedTheme.value = `file:${safeName}.css`
  applyTheme()
}

async function confirmNameDialog() {
  const name = nameDialogInput.value.trim()
  if (!name) {
    nameDialogError.value = t('settings.main.themeNameRequired')
    return
  }
  try {
    const css = `/* SoundNinja Theme: ${name} */\n${pendingCss.value}`
    await saveImportedTheme(css, name)
    closeNameDialog()
  } catch (e) {
    nameDialogError.value = `Failed: ${e}`
  }
}

function closeNameDialog() {
  nameDialogOpen.value = false
  pendingCss.value = ''
  nameDialogInput.value = ''
  nameDialogError.value = ''
}

async function onCheckUpdatesOnStart(val: boolean) {
  checkUpdatesOnStart.value = val
  await appSettings.setCheckUpdatesOnStart(val)
}

async function syncFromStore() {
  if (!appSettings.loaded) await appSettings.load()
  await loadSavedThemes()
  selectedTheme.value = normalizeThemeId(jsonStore.configFile?.settings?.theme)
  checkUpdatesOnStart.value = appSettings.checkUpdatesOnStart !== false
  currentLanguage.value = locale.value
  applyTheme()
}

watch(() => appStore.activeOverlay, async (val) => {
  if (val !== 'settings') return
  await syncFromStore()
})

watch(
  () => [jsonStore.configFile?.settings?.theme, appSettings.checkUpdatesOnStart],
  () => {
    if (appStore.activeOverlay !== 'settings') return
    selectedTheme.value = normalizeThemeId(jsonStore.configFile?.settings?.theme)
    checkUpdatesOnStart.value = appSettings.checkUpdatesOnStart !== false
  },
)

onMounted(syncFromStore)
</script>
