<template>
  <section class="theme-creator-section">
    <!-- Sticky title bar with action icons -->
    <div class="theme-creator-toolbar">
      <h2 class="theme-creator-toolbar__title">{{ $t('settings.themeCreator.title') }}</h2>
      <div class="theme-creator-toolbar__actions">
        <button class="tc-toolbar-btn" :title="$t('settings.themeCreator.importCss')" @click="importThemeFromFile">
          <Icons icon="upload" customClass="tc-toolbar-icon" />
        </button>
        <button class="tc-toolbar-btn" :title="$t('settings.themeCreator.exportCss')" @click="exportTheme">
          <Icons icon="folder" customClass="tc-toolbar-icon" />
        </button>
        <button class="tc-toolbar-btn" :title="$t('settings.themeCreator.addToThemes')" @click="saveThemeToFolder">
          <Icons icon="check" customClass="tc-toolbar-icon" />
        </button>
        <button class="tc-toolbar-btn" :title="$t('settings.themeCreator.reset')" @click="resetToDefaults">
          <Icons icon="reset" customClass="tc-toolbar-icon" />
        </button>
      </div>
    </div>

    <div class="theme-creator-layout">
      <!-- Controls column -->
      <div class="theme-creator-controls">

        <div class="settings-group">
          <label class="settings-label">{{ $t('settings.themeCreator.themeName') }}</label>
          <input type="text" class="settings-input" v-model="themeCreator.name" :placeholder="$t('settings.themeCreator.themeNamePlaceholder')" />
        </div>

        <div class="settings-section-divider">{{ $t('settings.themeCreator.colors') }}</div>

        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.previewMode') }}</label>
          <select v-model="previewMode" class="settings-select" @change="onPreviewModeChange">
            <option value="dark">{{ $t('settings.themeCreator.previewDark') }}</option>
            <option value="light">{{ $t('settings.themeCreator.previewLight') }}</option>
          </select>
        </div>

        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.primaryAccent') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.primaryColor" />
            <input type="text" class="settings-color-text" v-model="themeCreator.primaryColor" @change="fixColorInput('primaryColor')" maxlength="9" spellcheck="false" />
          </div>
        </div>
        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.backgroundLight') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.bgLight" />
            <input type="text" class="settings-color-text" v-model="themeCreator.bgLight" @change="fixColorInput('bgLight')" maxlength="9" spellcheck="false" />
          </div>
        </div>
        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.backgroundDark') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.bgDark" />
            <input type="text" class="settings-color-text" v-model="themeCreator.bgDark" @change="fixColorInput('bgDark')" maxlength="9" spellcheck="false" />
          </div>
        </div>
        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.buttonLight') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.btnLight" />
            <input type="text" class="settings-color-text" v-model="themeCreator.btnLight" @change="fixColorInput('btnLight')" maxlength="9" spellcheck="false" />
          </div>
        </div>
        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.buttonDark') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.btnDark" />
            <input type="text" class="settings-color-text" v-model="themeCreator.btnDark" @change="fixColorInput('btnDark')" maxlength="9" spellcheck="false" />
          </div>
        </div>

        <div class="settings-section-divider">{{ $t('settings.themeCreator.textColors') }}</div>

        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.textLight') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.textLight" />
            <input type="text" class="settings-color-text" v-model="themeCreator.textLight" @change="fixColorInput('textLight')" maxlength="9" spellcheck="false" />
          </div>
        </div>
        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.textDark') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.textDark" />
            <input type="text" class="settings-color-text" v-model="themeCreator.textDark" @change="fixColorInput('textDark')" maxlength="9" spellcheck="false" />
          </div>
        </div>

        <div class="settings-section-divider">{{ $t('settings.themeCreator.buttonTypography') }}</div>

        <div class="settings-row" style="margin-bottom: 0.75rem">
          <button class="settings-btn" @click="uploadFont">{{ $t('settings.themeCreator.uploadFont') }}</button>
          <span v-if="fontUploadMsg" class="settings-hint" style="align-self:center">{{ fontUploadMsg }}</span>
        </div>

        <div class="settings-group settings-group--stacked">
          <label class="settings-label">{{ $t('settings.themeCreator.buttonFont') }}</label>
          <div class="font-dropdown" ref="btnFontDropdownRef">
            <div class="font-dropdown__trigger" @click="btnFontOpen = !btnFontOpen">
              <span :style="{ fontFamily: themeCreator.btnFontFamily }">{{ themeCreator.btnFontFamily }}</span>
              <span class="font-dropdown__arrow">▾</span>
            </div>
            <div v-show="btnFontOpen" class="font-dropdown__list">
              <div class="font-dropdown__search">
                <input v-model="btnFontSearch" type="text" class="font-dropdown__search-input" :placeholder="$t('settings.themeCreator.searchFont')" />
              </div>
              <div
                v-for="font in filteredBtnFonts"
                :key="'btn-' + font"
                class="font-dropdown__option"
                :class="{ active: themeCreator.btnFontFamily === font }"
                :style="{ fontFamily: font }"
                @click="themeCreator.btnFontFamily = font; btnFontOpen = false"
              >{{ font }}</div>
            </div>
          </div>
        </div>
        <div class="settings-group settings-group--stacked">
          <div class="settings-slider-header">
            <label class="settings-label">{{ $t('settings.themeCreator.buttonFontSize') }}</label>
            <div class="settings-unit-input">
              <input type="number" class="settings-input" min="0.3" step="0.05" v-model.number="themeCreator.fontSizeBtn" @change="clampMin('fontSizeBtn', 0.3)" />
              <span class="settings-unit-label">rem</span>
            </div>
          </div>
          <input type="range" class="settings-slider" min="0.5" max="3" step="0.05" v-model.number="themeCreator.fontSizeBtn" />
        </div>

        <div class="settings-section-divider">{{ $t('settings.themeCreator.tabTypography') }}</div>

        <div class="settings-group settings-group--stacked">
          <label class="settings-label">{{ $t('settings.themeCreator.tabFont') }}</label>
          <div class="font-dropdown" ref="tabFontDropdownRef">
            <div class="font-dropdown__trigger" @click="tabFontOpen = !tabFontOpen">
              <span :style="{ fontFamily: themeCreator.tabFontFamily }">{{ themeCreator.tabFontFamily }}</span>
              <span class="font-dropdown__arrow">▾</span>
            </div>
            <div v-show="tabFontOpen" class="font-dropdown__list">
              <div class="font-dropdown__search">
                <input v-model="tabFontSearch" type="text" class="font-dropdown__search-input" :placeholder="$t('settings.themeCreator.searchFont')" />
              </div>
              <div
                v-for="font in filteredTabFonts"
                :key="'tab-' + font"
                class="font-dropdown__option"
                :class="{ active: themeCreator.tabFontFamily === font }"
                :style="{ fontFamily: font }"
                @click="themeCreator.tabFontFamily = font; tabFontOpen = false"
              >{{ font }}</div>
            </div>
          </div>
        </div>
        <div class="settings-group settings-group--stacked">
          <div class="settings-slider-header">
            <label class="settings-label">{{ $t('settings.themeCreator.tabFontSize') }}</label>
            <div class="settings-unit-input">
              <input type="number" class="settings-input" min="0.3" step="0.05" v-model.number="themeCreator.fontSizeTab" @change="clampMin('fontSizeTab', 0.3)" />
              <span class="settings-unit-label">rem</span>
            </div>
          </div>
          <input type="range" class="settings-slider" min="0.5" max="3" step="0.05" v-model.number="themeCreator.fontSizeTab" />
        </div>

        <div class="settings-section-divider">{{ $t('settings.themeCreator.layout') }}</div>

        <div class="settings-group settings-group--stacked">
          <div class="settings-slider-header">
            <label class="settings-label">{{ $t('settings.themeCreator.buttonWidth') }}</label>
            <div class="settings-unit-input">
              <input type="number" class="settings-input" min="3" step="0.5" v-model.number="themeCreator.btnWidth" @change="clampMin('btnWidth', 3)" />
              <span class="settings-unit-label">rem</span>
            </div>
          </div>
          <input type="range" class="settings-slider" min="5" max="30" step="0.5" v-model.number="themeCreator.btnWidth" />
        </div>
        <div class="settings-group settings-group--stacked">
          <div class="settings-slider-header">
            <label class="settings-label">{{ $t('settings.themeCreator.borderRadius') }}</label>
            <div class="settings-unit-input">
              <input type="number" class="settings-input" min="0" step="0.05" v-model.number="themeCreator.borderRadius" @change="clampMin('borderRadius', 0)" />
              <span class="settings-unit-label">rem</span>
            </div>
          </div>
          <input type="range" class="settings-slider" min="0" max="4" step="0.05" v-model.number="themeCreator.borderRadius" />
        </div>
        <div class="settings-group settings-group--stacked">
          <div class="settings-slider-header">
            <label class="settings-label">{{ $t('settings.themeCreator.borderWidth') }}</label>
            <div class="settings-unit-input">
              <input type="number" class="settings-input" min="0" step="0.02" v-model.number="themeCreator.borderWidth" @change="clampMin('borderWidth', 0)" />
              <span class="settings-unit-label">rem</span>
            </div>
          </div>
          <input type="range" class="settings-slider" min="0" max="2" step="0.02" v-model.number="themeCreator.borderWidth" />
        </div>
        <div class="settings-group settings-group--stacked">
          <div class="settings-slider-header">
            <label class="settings-label">{{ $t('settings.themeCreator.buttonGap') }}</label>
            <div class="settings-unit-input">
              <input type="number" class="settings-input" min="0" step="0.05" v-model.number="themeCreator.buttonGap" @change="clampMin('buttonGap', 0)" />
              <span class="settings-unit-label">rem</span>
            </div>
          </div>
          <input type="range" class="settings-slider" min="0" max="6" step="0.05" v-model.number="themeCreator.buttonGap" />
        </div>
        <div class="settings-group settings-group--stacked">
          <div class="settings-slider-header">
            <label class="settings-label">{{ $t('settings.themeCreator.buttonPaddingX') }}</label>
            <div class="settings-unit-input">
              <input type="number" class="settings-input" min="0" step="0.05" v-model.number="themeCreator.btnPaddingX" @change="clampMin('btnPaddingX', 0)" />
              <span class="settings-unit-label">rem</span>
            </div>
          </div>
          <input type="range" class="settings-slider" min="0.25" max="5" step="0.05" v-model.number="themeCreator.btnPaddingX" />
        </div>
        <div class="settings-group settings-group--stacked">
          <div class="settings-slider-header">
            <label class="settings-label">{{ $t('settings.themeCreator.buttonPaddingY') }}</label>
            <div class="settings-unit-input">
              <input type="number" class="settings-input" min="0" step="0.05" v-model.number="themeCreator.btnPaddingY" @change="clampMin('btnPaddingY', 0)" />
              <span class="settings-unit-label">rem</span>
            </div>
          </div>
          <input type="range" class="settings-slider" min="0.1" max="4" step="0.05" v-model.number="themeCreator.btnPaddingY" />
        </div>

        <p v-if="importError" class="settings-error" style="margin-top:0.5rem">{{ importError }}</p>
      </div>
    </div>

    <DialogField
      v-if="closePrompt"
      :title="$t('settings.themeCreator.closeTitle')"
      @close="cancelClose"
    >
      <p class="dialog-text">{{ $t('settings.themeCreator.closeMessage') }}</p>
      <div class="flex_c_h gap1 dialog-actions">
        <button class="settings-btn settings-btn--primary" @click="saveAndClose">{{ $t('dialog.save') }}</button>
        <button class="settings-btn" @click="discardAndClose">{{ $t('dialog.discard') }}</button>
        <button class="settings-btn" @click="cancelClose">{{ $t('dialog.cancel') }}</button>
      </div>
    </DialogField>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { emit, listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { readableTextColor } from '~/utils/contrast'

const { t } = useI18n()
const appSettings = useAppSettingsStore()

function joinPath(base: string, ...parts: string[]) {
  const sep = base.includes('\\') ? '\\' : '/'
  return [base.replace(/[\\/]+$/, ''), ...parts].join(sep)
}

const THEME_DEFAULTS = {
  primaryColor: '#00d4ff',
  bgLight: '#eeeeee',
  bgDark: '#222831',
  btnLight: '#7184a2',
  btnDark: '#363f4d',
  textLight: '#eeeeee',
  textDark: '#222831',
  btnFontFamily: 'Nunito-Bold',
  tabFontFamily: 'Nunito-Bold',
}

const themeCreator = reactive({
  name: 'My Theme',
  primaryColor: '#00d4ff',
  bgLight: '#eeeeee',
  bgDark: '#222831',
  btnLight: '#7184a2',
  btnDark: '#363f4d',
  textLight: '#eeeeee',
  textDark: '#222831',
  btnFontFamily: 'Nunito-Bold',
  tabFontFamily: 'Nunito-Bold',
  fontSizeBtn: 1.0,
  fontSizeTab: 1.0,
  fontSizeMd: 1.2,
  btnWidth: 11,
  borderRadius: 0.5,
  borderWidth: 0.2,
  buttonGap: 1.0,
  btnPaddingX: 0.75,
  btnPaddingY: 0.5,
})

// Preview mode (preview-only; does not persist the project's themeMode).
const previewMode = ref<'dark' | 'light'>('dark')
function onPreviewModeChange() {
  emit('theme_preview_mode', previewMode.value).catch(() => {})
  emitPreview()
}

// Restores all color + font fields to the documented defaults.
function resetToDefaults() {
  Object.assign(themeCreator, THEME_DEFAULTS)
  emitPreview()
}

// ── System fonts ──────────────────────────────────────────────────────────────
const systemFonts = ref(['Segoe UI', 'Arial', 'Verdana', 'Georgia', 'Courier New'])
const customFonts = ref<string[]>([])
const fontUploadMsg = ref('')

onMounted(async () => {
  if (!appSettings.loaded) await appSettings.load()
  try {
    const fonts = await invoke<string[]>('get_system_fonts')
    if (Array.isArray(fonts) && fonts.length > 0) systemFonts.value = fonts.map((f) => f.replace(/;+$/, '').trim())
  } catch (e) {
    console.warn('get_system_fonts failed', e)
  }
  try {
    customFonts.value = await loadCustomFonts(appSettings.fontsPath)
  } catch (e) {
    console.warn('loadCustomFonts failed', e)
  }
  // Initialise the draft from the main window's *current* applied theme so the
  // Theme Creator opens showing exactly what is on screen (no jump/change).
  let gotCurrent = false
  const unlisten = await listen<Record<string, string>>('theme_current', (e) => {
    if (gotCurrent) return
    gotCurrent = true
    applyCurrentVars(e.payload)
    emitPreview()
    unlisten()
  })
  await emit('theme_request_current')
  // Fallback: if the main window never answers, push the defaults after a beat.
  setTimeout(() => {
    if (!gotCurrent) emitPreview()
  }, 400)
})

/** Populates the draft from a set of computed CSS variables (best-effort). */
function applyCurrentVars(v: Record<string, string> | undefined) {
  if (!v) return
  const setColor = (key: keyof typeof themeCreator, varName: string) => {
    const raw = v[varName]
    if (!raw) return
    const hex = cssColorToHex(raw)
    if (/^#[0-9a-f]{3,8}$/i.test(hex)) (themeCreator as Record<string, unknown>)[key] = hex
  }
  const setNum = (key: keyof typeof themeCreator, varName: string) => {
    const raw = v[varName]
    if (!raw) return
    const n = parseRem(raw)
    if (isFinite(n)) (themeCreator as Record<string, unknown>)[key] = n
  }
  setColor('primaryColor', '--primary_color')
  setColor('bgLight', '--color-bg-light')
  setColor('bgDark', '--color-bg-dark')
  setColor('btnLight', '--color-btn-light')
  setColor('btnDark', '--color-btn-dark')
  setColor('textLight', '--text-light')
  setColor('textDark', '--text-dark')
  // Backward-compat with single-var themes.
  if (!v['--color-bg-dark'] && v['--color-bg']) setColor('bgDark', '--color-bg')
  if (!v['--color-btn-dark'] && v['--color-btn']) setColor('btnDark', '--color-btn')
  if (v['--font-btn']) themeCreator.btnFontFamily = extractFontFamily(v['--font-btn'])
  if (v['--font-tab']) themeCreator.tabFontFamily = extractFontFamily(v['--font-tab'])
  setNum('fontSizeBtn', '--font-size-btn')
  setNum('fontSizeTab', '--font-size-tab')
  setNum('fontSizeMd', '--font-size-md')
  setNum('btnWidth', '--btn_width')
  setNum('borderRadius', '--border-radius')
  setNum('borderWidth', '--btn-border-width')
  setNum('buttonGap', '--button-gap')
  if (v['--btn_padding']) {
    const [py, px] = parsePadding(v['--btn_padding'])
    if (isFinite(py)) themeCreator.btnPaddingY = py
    if (isFinite(px)) themeCreator.btnPaddingX = px
  }
  if (v['__mode']) previewMode.value = v['__mode'] === 'light' ? 'light' : 'dark'
}

// Uploaded fonts are listed first so they are easy to find.
const allFonts = computed(() => [...customFonts.value, ...systemFonts.value])

async function uploadFont() {
  fontUploadMsg.value = ''
  try {
    const selected = await open({
      multiple: true,
      title: 'Upload Font',
      filters: [{ name: 'Fonts', extensions: ['ttf', 'otf'] }],
    })
    if (!selected) return
    const paths = Array.isArray(selected) ? selected : [selected]
    await invoke('make_dir_abs', { path: appSettings.fontsPath })
    let added = 0
    for (const src of paths) {
      const dst = await invoke<string>('copy_file_abs', { src, dstDir: appSettings.fontsPath })
      const name = (dst.split(/[\\/]/).pop() ?? '').replace(/\.(ttf|otf)$/i, '')
      try {
        await registerFontFace(name, dst)
        if (!customFonts.value.includes(name)) customFonts.value.push(name)
        added++
      } catch (e) {
        console.warn('register font failed', e)
      }
    }
    fontUploadMsg.value = t('settings.themeCreator.fontUploaded', { count: added })
  } catch (e) {
    console.error('uploadFont failed', e)
  }
}

// ── Live preview → main window ─────────────────────────────────────────────────
let previewTimer: ReturnType<typeof setTimeout> | null = null
function emitPreview() {
  emit('theme_preview', buildThemeCss()).catch(() => {})
}
watch(themeCreator, () => {
  if (previewTimer) clearTimeout(previewTimer)
  previewTimer = setTimeout(emitPreview, 80)
}, { deep: true })

// When the window closes without saving, tell the main window to restore its
// persisted theme so the live preview does not stick.
onBeforeUnmount(() => {
  emit('theme_saved').catch(() => {})
})

// ── Close prompt (Save theme before closing the window) ────────────────────
const closePrompt = ref(false)
let allowClose = false

onMounted(() => {
  getCurrentWindow().onCloseRequested((event) => {
    if (allowClose) return
    event.preventDefault()
    closePrompt.value = true
  }).catch(() => {})
})

async function saveAndClose() {
  closePrompt.value = false
  allowClose = true
  await saveThemeToFolder()
}

async function discardAndClose() {
  closePrompt.value = false
  allowClose = true
  await emit('theme_saved').catch(() => {})
  await getCurrentWindow().destroy()
}

function cancelClose() {
  closePrompt.value = false
}

// ── Font dropdown state ───────────────────────────────────────────────────────
const btnFontOpen = ref(false)
const tabFontOpen = ref(false)
const btnFontDropdownRef = ref<HTMLElement | null>(null)
const tabFontDropdownRef = ref<HTMLElement | null>(null)
onClickOutside(btnFontDropdownRef, () => { btnFontOpen.value = false })
onClickOutside(tabFontDropdownRef, () => { tabFontOpen.value = false })

const btnFontSearch = ref('')
const tabFontSearch = ref('')
watch(btnFontOpen, (v) => { if (!v) btnFontSearch.value = '' })
watch(tabFontOpen, (v) => { if (!v) tabFontSearch.value = '' })

const filteredBtnFonts = computed(() => {
  const q = btnFontSearch.value.trim().toLowerCase()
  return q ? allFonts.value.filter((f) => f.toLowerCase().includes(q)) : allFonts.value
})

const filteredTabFonts = computed(() => {
  const q = tabFontSearch.value.trim().toLowerCase()
  return q ? allFonts.value.filter((f) => f.toLowerCase().includes(q)) : allFonts.value
})

// ── Export / save ─────────────────────────────────────────────────────────────
function buildThemeCss() {
  const name = (themeCreator.name || 'theme').trim()
  const soundTextDark = readableTextColor(themeCreator.btnDark, themeCreator.textLight, themeCreator.textDark)
  const soundTextLight = readableTextColor(themeCreator.btnLight, themeCreator.textLight, themeCreator.textDark)
  return `/* SoundNinja Theme: ${name} */
:root {
  --primary_color: ${themeCreator.primaryColor};
  --color-bg-light: ${themeCreator.bgLight};
  --color-bg-dark: ${themeCreator.bgDark};
  --color-btn-light: ${themeCreator.btnLight};
  --color-btn-dark: ${themeCreator.btnDark};
  --text-light: ${themeCreator.textLight};
  --text-dark: ${themeCreator.textDark};
  --font-btn: '${themeCreator.btnFontFamily}', sans-serif;
  --font-tab: '${themeCreator.tabFontFamily}', sans-serif;
  --font-size-btn: ${themeCreator.fontSizeBtn}rem;
  --font-size-tab: ${themeCreator.fontSizeTab}rem;
  --font-size-md: ${themeCreator.fontSizeMd}rem;
  --btn_width: ${themeCreator.btnWidth}rem;
  --border-radius: ${themeCreator.borderRadius}rem;
  --btn-border-width: ${themeCreator.borderWidth}rem;
  --button-gap: ${themeCreator.buttonGap}rem;
  --btn_padding: ${themeCreator.btnPaddingY}rem ${themeCreator.btnPaddingX}rem;
}
html.theme-dark {
  --color-bg: ${themeCreator.bgDark};
  --color-btn: ${themeCreator.btnDark};
  --sound-text: ${soundTextDark};
}
html.theme-light {
  --color-bg: ${themeCreator.bgLight};
  --color-btn: ${themeCreator.btnLight};
  --sound-text: ${soundTextLight};
}
`
}

async function exportTheme() {
  const css = buildThemeCss()
  const safeName = (themeCreator.name || 'theme').replace(/[^a-z0-9_-]/gi, '_')
  try {
    const filePath = await save({
      title: 'Export Theme',
      defaultPath: `${safeName}.css`,
      filters: [{ name: 'CSS', extensions: ['css'] }],
    })
    if (!filePath) return
    await invoke('write_text_file_abs', { path: filePath, contents: css })
  } catch (e) {
    console.error('Export failed', e)
  }
}

// ── Import ───────────────────────────────────────────────────────────────────
const importError = ref('')

function parseCssVars(css: string): Record<string, string> {
  const result: Record<string, string> = {}
  const re = /(--[\w-]+)\s*:\s*([^;]+)/g
  let m
  while ((m = re.exec(css)) !== null) {
    result[m[1].trim()] = m[2].trim()
  }
  return result
}

function cssColorToHex(color: string): string {
  const c = color.trim()
  if (/^#[0-9a-f]{3,8}$/i.test(c)) return c
  try {
    const canvas = document.createElement('canvas')
    canvas.width = canvas.height = 1
    const ctx = canvas.getContext('2d')!
    ctx.fillStyle = c
    ctx.fillRect(0, 0, 1, 1)
    const [r, g, b] = ctx.getImageData(0, 0, 1, 1).data
    return '#' + [r, g, b].map((x) => x.toString(16).padStart(2, '0')).join('')
  } catch {
    return c
  }
}

function fixColorInput(key: 'primaryColor' | 'bgLight' | 'bgDark' | 'btnLight' | 'btnDark' | 'textLight' | 'textDark') {
  const hex = cssColorToHex(themeCreator[key])
  if (/^#[0-9a-f]{6}$/i.test(hex)) themeCreator[key] = hex
}

function clampMin(
  key: 'fontSizeBtn' | 'fontSizeTab' | 'btnWidth' | 'borderRadius' | 'borderWidth' | 'buttonGap' | 'btnPaddingX' | 'btnPaddingY',
  min: number,
) {
  const v = Number(themeCreator[key])
  themeCreator[key] = isNaN(v) ? min : parseFloat(Math.max(min, v).toFixed(2))
}

function extractFontFamily(v: string): string {
  const m = v.match(/['"]([^'"]+)['"]/);
  if (m) return m[1].replace(/;+$/, '').trim()
  return v.split(',')[0].replace(/;+$/, '').trim()
}

function parseRem(v: string): number {
  return parseFloat(v)
}

function parsePadding(v: string): [number, number] {
  const parts = v.trim().split(/\s+/)
  return [parseFloat(parts[0]), parseFloat(parts[1] ?? parts[0])]
}

// Reads the `/* SoundNinja Theme: NAME */` comment (first line) from a theme CSS.
function parseThemeName(css: string): string {
  const m = css.match(/\/\*\s*SoundNinja Theme:\s*(.+?)\s*\*\//i)
  return m ? m[1].trim() : ''
}

async function importThemeFromFile() {
  importError.value = ''
  try {
    const filePath = await open({
      multiple: false,
      title: 'Import Theme CSS',
      filters: [{ name: 'CSS Files', extensions: ['css'] }],
    })
    if (!filePath) return
    const css = await invoke<string>('read_text_file_abs', { path: filePath as string })
    const vars = parseCssVars(css)
    const hasPairs = '--color-bg-dark' in vars || '--color-btn-dark' in vars
    const required = hasPairs
      ? ['--primary_color', '--font-btn', '--font-tab', '--font-size-btn', '--font-size-tab', '--btn_width', '--border-radius', '--btn_padding']
      : ['--primary_color', '--color-bg', '--color-btn', '--font-btn', '--font-tab', '--font-size-btn', '--font-size-tab', '--btn_width', '--border-radius', '--btn_padding']
    const missing = required.filter((v) => !(v in vars))
    if (missing.length > 0) {
      importError.value = `${t('settings.themeCreator.importError')} ${missing.join(', ')}`
      return
    }
    const parsedName = parseThemeName(css)
    if (parsedName) themeCreator.name = parsedName
    themeCreator.primaryColor = cssColorToHex(vars['--primary_color'])
    if (hasPairs) {
      if (vars['--color-bg-light']) themeCreator.bgLight = cssColorToHex(vars['--color-bg-light'])
      if (vars['--color-bg-dark']) themeCreator.bgDark = cssColorToHex(vars['--color-bg-dark'])
      if (vars['--color-btn-light']) themeCreator.btnLight = cssColorToHex(vars['--color-btn-light'])
      if (vars['--color-btn-dark']) themeCreator.btnDark = cssColorToHex(vars['--color-btn-dark'])
      if (vars['--text-light']) themeCreator.textLight = cssColorToHex(vars['--text-light'])
      if (vars['--text-dark']) themeCreator.textDark = cssColorToHex(vars['--text-dark'])
    } else {
      // Single-var theme: map background/button to both light and dark.
      const bg = cssColorToHex(vars['--color-bg'])
      const btn = cssColorToHex(vars['--color-btn'])
      themeCreator.bgLight = bg
      themeCreator.bgDark = bg
      themeCreator.btnLight = btn
      themeCreator.btnDark = btn
    }
    themeCreator.btnFontFamily = extractFontFamily(vars['--font-btn'])
    themeCreator.tabFontFamily = extractFontFamily(vars['--font-tab'])
    themeCreator.fontSizeBtn = parseRem(vars['--font-size-btn'])
    themeCreator.fontSizeTab = parseRem(vars['--font-size-tab'])
    if (vars['--font-size-md']) themeCreator.fontSizeMd = parseRem(vars['--font-size-md'])
    themeCreator.btnWidth = parseRem(vars['--btn_width'])
    themeCreator.borderRadius = parseRem(vars['--border-radius'])
    if (vars['--btn-border-width']) themeCreator.borderWidth = parseRem(vars['--btn-border-width'])
    if (vars['--button-gap']) themeCreator.buttonGap = parseRem(vars['--button-gap'])
    const [py, px] = parsePadding(vars['--btn_padding'])
    themeCreator.btnPaddingY = py
    themeCreator.btnPaddingX = px
  } catch (e) {
    importError.value = `Failed: ${e}`
  }
}

async function saveThemeToFolder() {
  const css = buildThemeCss()
  const safeName = (themeCreator.name || 'theme').replace(/[^a-z0-9_-]/gi, '_')
  try {
    await invoke('make_dir_abs', { path: appSettings.themesPath })
    await invoke('write_text_file_abs', {
      path: joinPath(appSettings.themesPath, `${safeName}.css`),
      contents: css,
    })
    // Tell the main window to select + persist this theme, then close.
    await emit('theme_apply', { theme: `file:${safeName}.css` })
    allowClose = true
    await getCurrentWindow().destroy()
  } catch (e) {
    console.error('Save to themes folder failed', e)
  }
}
</script>
