<template>
  <section class="theme-creator-section">
    <h2 class="settings-content__title">{{ $t('settings.themeCreator.title') }}</h2>

    <div class="theme-creator-layout">
      <!-- Controls column -->
      <div class="theme-creator-controls">

        <div class="settings-group">
          <label class="settings-label">{{ $t('settings.themeCreator.themeName') }}</label>
          <input type="text" class="settings-input" v-model="themeCreator.name" :placeholder="$t('settings.themeCreator.themeNamePlaceholder')" />
        </div>

        <div class="settings-section-divider">{{ $t('settings.themeCreator.colors') }}</div>

        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.primaryAccent') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.primaryColor" />
            <span class="settings-color-value">{{ themeCreator.primaryColor }}</span>
          </div>
        </div>
        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.background') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.bgColor" />
            <span class="settings-color-value">{{ themeCreator.bgColor }}</span>
          </div>
        </div>
        <div class="settings-group settings-group--inline">
          <label class="settings-label">{{ $t('settings.themeCreator.buttonBackground') }}</label>
          <div class="settings-color-row">
            <input type="color" class="settings-color" v-model="themeCreator.btnColor" />
            <span class="settings-color-value">{{ themeCreator.btnColor }}</span>
          </div>
        </div>

        <div class="settings-section-divider">{{ $t('settings.themeCreator.buttonTypography') }}</div>

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
          <label class="settings-label">{{ $t('settings.themeCreator.buttonFontSize') }} — {{ themeCreator.fontSizeBtn }}rem</label>
          <input type="range" class="settings-slider" min="0.7" max="1.8" step="0.05" v-model.number="themeCreator.fontSizeBtn" />
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
          <label class="settings-label">{{ $t('settings.themeCreator.tabFontSize') }} — {{ themeCreator.fontSizeTab }}rem</label>
          <input type="range" class="settings-slider" min="0.7" max="1.8" step="0.05" v-model.number="themeCreator.fontSizeTab" />
        </div>

        <div class="settings-section-divider">{{ $t('settings.themeCreator.layout') }}</div>

        <div class="settings-group settings-group--stacked">
          <label class="settings-label">{{ $t('settings.themeCreator.buttonWidth') }} — {{ themeCreator.btnWidth }}rem</label>
          <input type="range" class="settings-slider" min="7" max="18" step="0.5" v-model.number="themeCreator.btnWidth" />
        </div>
        <div class="settings-group settings-group--stacked">
          <label class="settings-label">{{ $t('settings.themeCreator.borderRadius') }} — {{ themeCreator.borderRadius }}rem</label>
          <input type="range" class="settings-slider" min="0" max="2" step="0.05" v-model.number="themeCreator.borderRadius" />
        </div>
        <div class="settings-group settings-group--stacked">
          <label class="settings-label">{{ $t('settings.themeCreator.buttonPaddingX') }} — {{ themeCreator.btnPaddingX }}rem</label>
          <input type="range" class="settings-slider" min="0.25" max="2.5" step="0.05" v-model.number="themeCreator.btnPaddingX" />
        </div>
        <div class="settings-group settings-group--stacked">
          <label class="settings-label">{{ $t('settings.themeCreator.buttonPaddingY') }} — {{ themeCreator.btnPaddingY }}rem</label>
          <input type="range" class="settings-slider" min="0.1" max="1.5" step="0.05" v-model.number="themeCreator.btnPaddingY" />
        </div>

        <div class="settings-row" style="margin-top: 1.5rem">
          <button class="settings-btn" @click="importThemeFromFile">{{ $t('settings.themeCreator.importCss') }}</button>
          <button class="settings-btn settings-btn--primary" @click="exportTheme">{{ $t('settings.themeCreator.exportCss') }}</button>
          <button class="settings-btn" @click="saveThemeToFolder">{{ $t('settings.themeCreator.addToThemes') }}</button>
        </div>
        <p v-if="importError" class="settings-error" style="margin-top:0.5rem">{{ importError }}</p>
      </div>

      <!-- Preview column -->
      <div class="theme-creator-preview-col">
        <div class="tc-preview-label">{{ $t('settings.themeCreator.livePreview') }}</div>

        <div class="tc-preview-section">{{ $t('settings.themeCreator.previewTabs') }}</div>
        <div class="tc-tab-bar" :style="previewBgStyle">
          <div class="tc-tab" :style="tabPreviewStyle">{{ $t('settings.themeCreator.previewTabNames[0]') }}</div>
          <div class="tc-tab" :style="tabActivePreviewStyle">{{ $t('settings.themeCreator.previewTabNames[1]') }}</div>
          <div class="tc-tab" :style="tabPreviewStyle">{{ $t('settings.themeCreator.previewTabNames[2]') }}</div>
        </div>

        <div class="tc-preview-section" style="margin-top: 1.25rem">{{ $t('settings.themeCreator.previewButtons') }}</div>
        <div class="tc-btn-grid" :style="previewBgStyle">
          <div class="tc-btn" :style="btnPreviewStyle">{{ $t('settings.themeCreator.previewSounds[0]') }}</div>
          <div class="tc-btn" :style="btnActivePreviewStyle">{{ $t('settings.themeCreator.previewSounds[1]') }}</div>
          <div class="tc-btn" :style="btnPreviewStyle">{{ $t('settings.themeCreator.previewSounds[2]') }}</div>
          <div class="tc-btn" :style="btnPreviewStyle">{{ $t('settings.themeCreator.previewSounds[3]') }}</div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readTextFile, writeTextFile, mkdir, BaseDirectory } from '@tauri-apps/plugin-fs'

const { t } = useI18n()

const themeCreator = reactive({
  name: 'My Theme',
  primaryColor: '#00d4ff',
  bgColor: '#222831',
  btnColor: '#363f4d',
  btnFontFamily: 'Segoe UI',
  tabFontFamily: 'Segoe UI',
  fontSizeBtn: 1.0,
  fontSizeTab: 1.0,
  fontSizeMd: 1.2,
  btnWidth: 11,
  borderRadius: 0.5,
  btnPaddingX: 0.75,
  btnPaddingY: 0.5,
})

// ── System fonts ──────────────────────────────────────────────────────────────
const systemFonts = ref(['Segoe UI', 'Arial', 'Verdana', 'Georgia', 'Courier New'])

onMounted(async () => {
  try {
    const fonts = await invoke<string[]>('get_system_fonts')
    if (Array.isArray(fonts) && fonts.length > 0) systemFonts.value = fonts
  } catch (e) {
    console.warn('get_system_fonts failed', e)
  }
})

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
  return q ? systemFonts.value.filter((f) => f.toLowerCase().includes(q)) : systemFonts.value
})

const filteredTabFonts = computed(() => {
  const q = tabFontSearch.value.trim().toLowerCase()
  return q ? systemFonts.value.filter((f) => f.toLowerCase().includes(q)) : systemFonts.value
})

// ── Preview styles ────────────────────────────────────────────────────────────
const previewBgStyle = computed(() => ({
  background: themeCreator.bgColor,
  borderColor: themeCreator.primaryColor,
}))

const btnPreviewStyle = computed(() => ({
  background: themeCreator.btnColor,
  border: `0.15rem solid ${themeCreator.primaryColor}`,
  borderRadius: themeCreator.borderRadius + 'rem',
  color: '#eee',
  fontFamily: `'${themeCreator.btnFontFamily}', sans-serif`,
  fontSize: themeCreator.fontSizeBtn + 'rem',
  width: themeCreator.btnWidth + 'rem',
  padding: `${themeCreator.btnPaddingY}rem ${themeCreator.btnPaddingX}rem`,
  textAlign: 'center' as const,
}))

const btnActivePreviewStyle = computed(() => ({
  ...btnPreviewStyle.value,
  background: themeCreator.primaryColor,
  color: themeCreator.bgColor,
}))

const tabPreviewStyle = computed(() => ({
  fontFamily: `'${themeCreator.tabFontFamily}', sans-serif`,
  fontSize: themeCreator.fontSizeTab + 'rem',
  color: 'rgba(238,238,238,0.65)',
  padding: '0.4rem 1rem',
  borderBottom: '2px solid transparent',
  cursor: 'default',
  whiteSpace: 'nowrap' as const,
}))

const tabActivePreviewStyle = computed(() => ({
  ...tabPreviewStyle.value,
  color: themeCreator.primaryColor,
  borderBottomColor: themeCreator.primaryColor,
}))

// ── Export / save ─────────────────────────────────────────────────────────────
function buildThemeCss() {
  const name = (themeCreator.name || 'theme').trim()
  return `/* SoundNinja Theme: ${name} */
:root {
  --primary_color: ${themeCreator.primaryColor};
  --color-bg: ${themeCreator.bgColor};
  --color-btn: ${themeCreator.btnColor};
  --font-btn: '${themeCreator.btnFontFamily}', sans-serif;
  --font-tab: '${themeCreator.tabFontFamily}', sans-serif;
  --font-size-btn: ${themeCreator.fontSizeBtn}rem;
  --font-size-tab: ${themeCreator.fontSizeTab}rem;
  --font-size-md: ${themeCreator.fontSizeMd}rem;
  --btn_width: ${themeCreator.btnWidth}rem;
  --border-radius: ${themeCreator.borderRadius}rem;
  --btn_padding: ${themeCreator.btnPaddingY}rem ${themeCreator.btnPaddingX}rem;
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
    await writeTextFile(filePath, css)
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

function extractFontFamily(v: string): string {
  const m = v.match(/['"]([^'"]+)['"]/);
  if (m) return m[1]
  return v.split(',')[0].trim()
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
    const css = await readTextFile(filePath as string)
    const vars = parseCssVars(css)
    const required = [
      '--primary_color', '--color-bg', '--color-btn',
      '--font-btn', '--font-tab',
      '--font-size-btn', '--font-size-tab',
      '--btn_width', '--border-radius', '--btn_padding',
    ]
    const missing = required.filter((v) => !(v in vars))
    if (missing.length > 0) {
      importError.value = `${t('settings.themeCreator.importError')} ${missing.join(', ')}`
      return
    }
    const parsedName = parseThemeName(css)
    if (parsedName) themeCreator.name = parsedName
    themeCreator.primaryColor = cssColorToHex(vars['--primary_color'])
    themeCreator.bgColor = cssColorToHex(vars['--color-bg'])
    themeCreator.btnColor = cssColorToHex(vars['--color-btn'])
    themeCreator.btnFontFamily = extractFontFamily(vars['--font-btn'])
    themeCreator.tabFontFamily = extractFontFamily(vars['--font-tab'])
    themeCreator.fontSizeBtn = parseRem(vars['--font-size-btn'])
    themeCreator.fontSizeTab = parseRem(vars['--font-size-tab'])
    if (vars['--font-size-md']) themeCreator.fontSizeMd = parseRem(vars['--font-size-md'])
    themeCreator.btnWidth = parseRem(vars['--btn_width'])
    themeCreator.borderRadius = parseRem(vars['--border-radius'])
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
    await mkdir('themes', { baseDir: BaseDirectory.AppData, recursive: true })
    await writeTextFile(`themes/${safeName}.css`, css, { baseDir: BaseDirectory.AppData })
  } catch (e) {
    console.error('Save to themes folder failed', e)
  }
}
</script>
