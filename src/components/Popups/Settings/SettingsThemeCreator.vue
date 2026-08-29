<template>
  <section class="theme-creator-section">
    <!-- Sticky title bar with action icons -->
    <div class="theme-creator-toolbar">
      <h2 class="theme-creator-toolbar__title">{{ $t('settings.themeCreator.title') }}</h2>
      <select
        v-model="loadThemeId"
        class="settings-select theme-creator-toolbar__theme"
        :title="$t('settings.themeCreator.loadThemeHint')"
        @change="onLoadTheme"
      >
        <option value="">{{ $t('settings.themeCreator.loadThemePlaceholder') }}</option>
        <optgroup :label="$t('settings.themeCreator.presets')">
          <option v-for="p in presets" :key="p.id" :value="'preset:' + p.id">{{ p.label }}</option>
        </optgroup>
        <optgroup v-if="savedThemes.length" :label="$t('settings.themeCreator.savedThemes')">
          <option v-for="f in savedThemes" :key="f" :value="'file:' + f">{{ f.replace('.css', '') }}</option>
        </optgroup>
      </select>
      <div class="theme-creator-toolbar__actions">
        <button class="tc-toolbar-btn" :title="$t('settings.themeCreator.importCssHint')" @click="importThemeFromFile">
          <Icons icon="upload" customClass="tc-toolbar-icon" />
        </button>
        <button class="tc-toolbar-btn" :title="$t('settings.themeCreator.exportCssHint')" @click="exportTheme">
          <Icons icon="folder" customClass="tc-toolbar-icon" />
        </button>
        <button class="tc-toolbar-btn" :title="$t('settings.themeCreator.addToThemesHint')" @click="saveThemeToFolder">
          <Icons icon="check" customClass="tc-toolbar-icon" />
        </button>
        <button class="tc-toolbar-btn" :title="$t('settings.themeCreator.resetHint')" @click="resetToDefaults">
          <Icons icon="reset" customClass="tc-toolbar-icon" />
        </button>
      </div>
    </div>

    <nav class="theme-creator-tabs" aria-label="Theme Creator sections">
      <button
        v-for="tab in creatorTabs"
        :key="tab.id"
        type="button"
        class="theme-creator-tabs__item"
        :class="{ active: activeTab === tab.id }"
        :title="$t(tab.labelKey + 'Hint')"
        @click="activeTab = tab.id"
      >{{ $t(tab.labelKey) }}</button>
    </nav>

    <div class="theme-creator-layout">
      <div class="theme-creator-controls">

        <!-- ── General ───────────────────────────────────────────────────── -->
        <template v-if="activeTab === 'general'">
          <div class="settings-group settings-group--stacked">
            <SettingsTipLabel fluid :tip="$t('settings.themeCreator.themeNameHint')">{{ $t('settings.themeCreator.themeName') }}</SettingsTipLabel>
            <input type="text" class="settings-input" v-model="themeCreator.name" :placeholder="$t('settings.themeCreator.themeNamePlaceholder')" />
          </div>

          <div class="settings-section-divider">{{ $t('settings.themeCreator.colors') }}</div>

          <div
            v-for="row in generalColorRows"
            :key="row.key"
            class="settings-group settings-group--inline"
          >
            <SettingsTipLabel fluid :tip="$t(row.labelKey + 'Hint')">{{ $t(row.labelKey) }}</SettingsTipLabel>
            <div class="settings-color-row">
              <input type="color" class="settings-color" :value="colorPickerValue(row.key)" @input="onColorWheel(row.key, $event)" />
              <input type="text" class="settings-color-text" v-model="themeCreator[row.key]" @change="fixColorInput(row.key)" maxlength="9" spellcheck="false" />
            </div>
          </div>

          <div class="settings-row theme-creator-font-upload">
            <button class="settings-btn" :title="$t('settings.themeCreator.uploadFontHint')" @click="uploadFont">{{ $t('settings.themeCreator.uploadFont') }}</button>
            <span class="settings-info-icon theme-creator-font-upload__tip">
              <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
              <span class="settings-info-icon__tip">{{ $t('settings.themeCreator.uploadFontHint') }}</span>
            </span>
            <span v-if="fontUploadMsg" class="settings-hint">{{ fontUploadMsg }}</span>
          </div>
        </template>

        <!-- ── Buttons ───────────────────────────────────────────────────── -->
        <template v-else-if="activeTab === 'buttons'">
          <div class="settings-section-divider">{{ $t('settings.themeCreator.colors') }}</div>

          <div
            v-for="row in buttonColorRows"
            :key="row.key"
            class="settings-group settings-group--inline"
          >
            <SettingsTipLabel fluid :tip="$t(row.labelKey + 'Hint')">{{ $t(row.labelKey) }}</SettingsTipLabel>
            <div class="settings-color-row">
              <input type="color" class="settings-color" :value="colorPickerValue(row.key)" @input="onColorWheel(row.key, $event)" />
              <input type="text" class="settings-color-text" v-model="themeCreator[row.key]" @change="fixColorInput(row.key)" maxlength="9" spellcheck="false" />
            </div>
          </div>

          <div class="settings-section-divider">{{ $t('settings.themeCreator.buttonTypography') }}</div>

          <div class="settings-group settings-group--stacked">
            <SettingsTipLabel fluid :tip="$t('settings.themeCreator.buttonFontHint')">{{ $t('settings.themeCreator.buttonFont') }}</SettingsTipLabel>
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
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.buttonFontSizeHint')">{{ $t('settings.themeCreator.buttonFontSize') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0.3" step="0.05" v-model.number="themeCreator.fontSizeBtn" @change="clampMin('fontSizeBtn', 0.3)" />
                <span class="settings-unit-label">rem</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0.5" max="3" step="0.05" v-model.number="themeCreator.fontSizeBtn" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('fontSizeBtn')" />
          </div>

          <div class="settings-section-divider">{{ $t('settings.themeCreator.layout') }}</div>

          <div class="settings-group settings-group--stacked">
            <div class="settings-slider-header">
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.buttonWidthHint')">{{ $t('settings.themeCreator.buttonWidth') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="3" step="0.5" v-model.number="themeCreator.btnWidth" @change="clampMin('btnWidth', 3)" />
                <span class="settings-unit-label">rem</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="5" max="30" step="0.5" v-model.number="themeCreator.btnWidth" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('btnWidth')" />
          </div>
          <div class="settings-group settings-group--stacked">
            <div class="settings-slider-header">
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.borderRadiusHint')">{{ $t('settings.themeCreator.borderRadius') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0" step="0.05" v-model.number="themeCreator.borderRadius" @change="clampMin('borderRadius', 0)" />
                <span class="settings-unit-label">rem</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0" max="4" step="0.05" v-model.number="themeCreator.borderRadius" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('borderRadius')" />
          </div>
          <div class="settings-group settings-group--stacked">
            <div class="settings-slider-header">
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.borderWidthHint')">{{ $t('settings.themeCreator.borderWidth') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0" step="0.02" v-model.number="themeCreator.borderWidth" @change="clampMin('borderWidth', 0)" />
                <span class="settings-unit-label">rem</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0" max="2" step="0.02" v-model.number="themeCreator.borderWidth" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('borderWidth')" />
          </div>
          <div class="settings-group settings-group--stacked">
            <div class="settings-slider-header">
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.buttonGapHint')">{{ $t('settings.themeCreator.buttonGap') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0" step="0.05" v-model.number="themeCreator.buttonGap" @change="clampMin('buttonGap', 0)" />
                <span class="settings-unit-label">rem</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0" max="6" step="0.05" v-model.number="themeCreator.buttonGap" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('buttonGap')" />
          </div>
          <div class="settings-group settings-group--stacked">
            <div class="settings-slider-header">
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.buttonPaddingXHint')">{{ $t('settings.themeCreator.buttonPaddingX') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0" step="0.05" v-model.number="themeCreator.btnPaddingX" @change="clampMin('btnPaddingX', 0)" />
                <span class="settings-unit-label">rem</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0.25" max="5" step="0.05" v-model.number="themeCreator.btnPaddingX" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('btnPaddingX')" />
          </div>
          <div class="settings-group settings-group--stacked">
            <div class="settings-slider-header">
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.buttonPaddingYHint')">{{ $t('settings.themeCreator.buttonPaddingY') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0" step="0.05" v-model.number="themeCreator.btnPaddingY" @change="clampMin('btnPaddingY', 0)" />
                <span class="settings-unit-label">rem</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0.1" max="4" step="0.05" v-model.number="themeCreator.btnPaddingY" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('btnPaddingY')" />
          </div>

          <div class="settings-section-divider">{{ $t('settings.themeCreator.gifOverlaySection') }}</div>

          <div class="settings-group settings-group--stacked">
            <div class="settings-slider-header">
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.gifOverlayHint')">{{ $t('settings.themeCreator.gifOverlay') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0" max="100" step="1" v-model.number="themeCreator.gifOverlay" @change="clampRange('gifOverlay', 0, 100)" />
                <span class="settings-unit-label">%</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0" max="100" step="1" v-model.number="themeCreator.gifOverlay" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('gifOverlay')" />
          </div>
          <div class="settings-group settings-group--stacked">
            <div class="settings-slider-header">
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.gifOverlayHoverHint')">{{ $t('settings.themeCreator.gifOverlayHover') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0" max="100" step="1" v-model.number="themeCreator.gifOverlayHover" @change="clampRange('gifOverlayHover', 0, 100)" />
                <span class="settings-unit-label">%</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0" max="100" step="1" v-model.number="themeCreator.gifOverlayHover" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('gifOverlayHover')" />
          </div>
        </template>

        <!-- ── Tabs ──────────────────────────────────────────────────────── -->
        <template v-else-if="activeTab === 'tabs'">
          <div class="settings-section-divider">{{ $t('settings.themeCreator.colors') }}</div>

          <div
            v-for="row in tabColorRows"
            :key="row.key"
            class="settings-group settings-group--inline"
          >
            <SettingsTipLabel fluid :tip="$t(row.labelKey + 'Hint')">{{ $t(row.labelKey) }}</SettingsTipLabel>
            <div class="settings-color-row">
              <input type="color" class="settings-color" :value="colorPickerValue(row.key)" @input="onColorWheel(row.key, $event)" />
              <input type="text" class="settings-color-text" v-model="themeCreator[row.key]" @change="fixColorInput(row.key)" maxlength="9" spellcheck="false" />
            </div>
          </div>

          <div class="settings-section-divider">{{ $t('settings.themeCreator.tabTypography') }}</div>

          <div class="settings-group settings-group--stacked">
            <SettingsTipLabel fluid :tip="$t('settings.themeCreator.tabFontHint')">{{ $t('settings.themeCreator.tabFont') }}</SettingsTipLabel>
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
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.tabFontSizeHint')">{{ $t('settings.themeCreator.tabFontSize') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0.3" step="0.05" v-model.number="themeCreator.fontSizeTab" @change="clampMin('fontSizeTab', 0.3)" />
                <span class="settings-unit-label">rem</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0.5" max="3" step="0.05" v-model.number="themeCreator.fontSizeTab" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('fontSizeTab')" />
          </div>

          <div class="settings-section-divider">{{ $t('settings.themeCreator.layout') }}</div>

          <div class="settings-group settings-group--stacked">
            <div class="settings-slider-header">
              <SettingsTipLabel fluid :tip="$t('settings.themeCreator.tabBorderWidthHint')">{{ $t('settings.themeCreator.tabBorderWidth') }}</SettingsTipLabel>
              <div class="settings-unit-input">
                <input type="number" class="settings-input" min="0" step="0.02" v-model.number="themeCreator.tabBorderWidth" @change="clampMin('tabBorderWidth', 0)" />
                <span class="settings-unit-label">rem</span>
              </div>
            </div>
            <input type="range" class="settings-slider" min="0" max="2" step="0.02" v-model.number="themeCreator.tabBorderWidth" :title="$t('settings.themeCreator.sliderResetHint')" @dblclick.prevent="resetSlider('tabBorderWidth')" />
          </div>
        </template>

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
import { THEME_PRESETS } from '~/utils/themePresets'
import {
  THEME_TOKEN_DEFAULTS,
  TOKEN_CSS_VARS,
  buildThemeCss as buildCssFromTokens,
  parseThemeCss,
  parseThemeName,
  cssColorToHex,
  type ThemeTokenKey,
  type ThemeTokens,
} from '~/utils/themeTokens'

const { t } = useI18n()
const appSettings = useAppSettingsStore()

function joinPath(base: string, ...parts: string[]) {
  const sep = base.includes('\\') ? '\\' : '/'
  return [base.replace(/[\\/]+$/, ''), ...parts].join(sep)
}

const LAYOUT_DEFAULTS = {
  btnFontFamily: 'Nunito-Bold',
  tabFontFamily: 'Nunito-Bold',
  fontSizeBtn: 1.0,
  fontSizeTab: 1.0,
  fontSizeMd: 1.2,
  btnWidth: 11,
  borderRadius: 0.5,
  borderWidth: 0.2,
  tabBorderWidth: 0.3,
  buttonGap: 1.0,
  btnPaddingX: 0.75,
  btnPaddingY: 0.5,
  gifOverlay: 72,
  gifOverlayHover: 38,
} as const

const THEME_DEFAULTS = {
  ...THEME_TOKEN_DEFAULTS,
  ...LAYOUT_DEFAULTS,
} as const

type ThemeSliderKey =
  | 'fontSizeBtn'
  | 'fontSizeTab'
  | 'btnWidth'
  | 'borderRadius'
  | 'borderWidth'
  | 'tabBorderWidth'
  | 'buttonGap'
  | 'btnPaddingX'
  | 'btnPaddingY'
  | 'gifOverlay'
  | 'gifOverlayHover'

type CreatorTabId = 'general' | 'buttons' | 'tabs'

const creatorTabs: { id: CreatorTabId; labelKey: string }[] = [
  { id: 'general', labelKey: 'settings.themeCreator.tabGeneral' },
  { id: 'buttons', labelKey: 'settings.themeCreator.tabButtons' },
  { id: 'tabs', labelKey: 'settings.themeCreator.tabTabs' },
]
const activeTab = ref<CreatorTabId>('general')

const generalColorRows: { key: ThemeTokenKey; labelKey: string }[] = [
  { key: 'primaryColor', labelKey: 'settings.themeCreator.primaryAccent' },
  { key: 'primaryHover', labelKey: 'settings.themeCreator.primaryHover' },
  { key: 'bg', labelKey: 'settings.themeCreator.background' },
  { key: 'bg2', labelKey: 'settings.themeCreator.background2' },
]

const buttonColorRows: { key: ThemeTokenKey; labelKey: string }[] = [
  { key: 'btnBg', labelKey: 'settings.themeCreator.buttonBg' },
  { key: 'btnBgHover', labelKey: 'settings.themeCreator.buttonBgHover' },
  { key: 'btnText', labelKey: 'settings.themeCreator.buttonText' },
  { key: 'btnTextHover', labelKey: 'settings.themeCreator.buttonTextHover' },
  { key: 'btnBorder', labelKey: 'settings.themeCreator.buttonBorder' },
  { key: 'btnBorderHover', labelKey: 'settings.themeCreator.buttonBorderHover' },
]

const tabColorRows: { key: ThemeTokenKey; labelKey: string }[] = [
  { key: 'tabBg', labelKey: 'settings.themeCreator.tabBg' },
  { key: 'tabBgHover', labelKey: 'settings.themeCreator.tabBgHover' },
  { key: 'tabText', labelKey: 'settings.themeCreator.tabText' },
  { key: 'tabTextHover', labelKey: 'settings.themeCreator.tabTextHover' },
  { key: 'tabBorder', labelKey: 'settings.themeCreator.tabBorder' },
  { key: 'tabBorderHover', labelKey: 'settings.themeCreator.tabBorderHover' },
]

const themeCreator = reactive({
  name: 'My Theme',
  ...THEME_DEFAULTS,
})

const presets = THEME_PRESETS
const loadThemeId = ref('')
const savedThemes = ref<string[]>([])

/** Native color input only accepts #rrggbb — strip alpha for the wheel. */
function colorPickerValue(key: ThemeTokenKey): string {
  const v = themeCreator[key] || '#000000'
  if (/^#[0-9a-f]{8}$/i.test(v)) return v.slice(0, 7)
  if (/^#[0-9a-f]{6}$/i.test(v)) return v
  return cssColorToHex(v)
}

function onColorWheel(key: ThemeTokenKey, e: Event) {
  const hex = (e.target as HTMLInputElement).value
  // Preserve alpha if the previous value had one (tab tints).
  const prev = themeCreator[key]
  if (/^#[0-9a-f]{8}$/i.test(prev)) {
    themeCreator[key] = hex + prev.slice(7)
  } else {
    themeCreator[key] = hex
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

async function onLoadTheme() {
  const id = loadThemeId.value
  if (!id) return
  if (id.startsWith('preset:')) {
    const preset = presets.find((p) => p.id === id.slice(7))
    if (!preset) return
    Object.assign(themeCreator, preset.tokens)
    themeCreator.name = preset.label
    themeCreator.gifOverlay = Math.round(preset.extras.gifOverlay * 100)
    themeCreator.gifOverlayHover = Math.round(preset.extras.gifOverlayHover * 100)
    emitPreview()
    return
  }
  if (id.startsWith('file:')) {
    const filename = id.slice(5)
    try {
      const css = await invoke<string>('read_text_file_abs', {
        path: joinPath(appSettings.themesPath, filename),
      })
      applyParsedTheme(css)
    } catch (e) {
      importError.value = `Failed: ${e}`
    }
  }
}

function applyParsedTheme(css: string) {
  const parsed = parseThemeCss(css)
  const name = parseThemeName(css)
  if (name) themeCreator.name = name
  Object.assign(themeCreator, parsed)
  // Layout vars from raw CSS
  const re = /(--[\w-]+)\s*:\s*([^;]+)/g
  const vars: Record<string, string> = {}
  let m: RegExpExecArray | null
  while ((m = re.exec(css)) !== null) vars[m[1].trim()] = m[2].trim()
  if (vars['--font-btn']) themeCreator.btnFontFamily = extractFontFamily(vars['--font-btn'])
  if (vars['--font-tab']) themeCreator.tabFontFamily = extractFontFamily(vars['--font-tab'])
  if (vars['--font-size-btn']) themeCreator.fontSizeBtn = parseRem(vars['--font-size-btn'])
  if (vars['--font-size-tab']) themeCreator.fontSizeTab = parseRem(vars['--font-size-tab'])
  if (vars['--font-size-md']) themeCreator.fontSizeMd = parseRem(vars['--font-size-md'])
  if (vars['--btn_width']) themeCreator.btnWidth = parseRem(vars['--btn_width'])
  if (vars['--border-radius']) themeCreator.borderRadius = parseRem(vars['--border-radius'])
  if (vars['--btn-border-width']) themeCreator.borderWidth = parseRem(vars['--btn-border-width'])
  if (vars['--tab-border-width']) themeCreator.tabBorderWidth = parseRem(vars['--tab-border-width'])
  if (vars['--button-gap']) themeCreator.buttonGap = parseRem(vars['--button-gap'])
  if (vars['--btn_padding']) {
    const [py, px] = parsePadding(vars['--btn_padding'])
    if (isFinite(py)) themeCreator.btnPaddingY = py
    if (isFinite(px)) themeCreator.btnPaddingX = px
  }
  if (vars['--gif-overlay']) themeCreator.gifOverlay = opacityToPct(vars['--gif-overlay'])
  if (vars['--gif-overlay-hover']) themeCreator.gifOverlayHover = opacityToPct(vars['--gif-overlay-hover'])
  emitPreview()
}

function resetToDefaults() {
  Object.assign(themeCreator, THEME_DEFAULTS)
  loadThemeId.value = ''
  emitPreview()
}

function resetSlider(key: ThemeSliderKey) {
  themeCreator[key] = THEME_DEFAULTS[key]
  emitPreview()
}

// ── System fonts ──────────────────────────────────────────────────────────────
const systemFonts = ref(['Segoe UI', 'Arial', 'Verdana', 'Georgia', 'Courier New'])
const customFonts = ref<string[]>([])
const fontUploadMsg = ref('')

onMounted(async () => {
  if (!appSettings.loaded) await appSettings.load()
  await loadSavedThemes()
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
  let gotCurrent = false
  const unlisten = await listen<Record<string, string>>('theme_current', (e) => {
    if (gotCurrent) return
    gotCurrent = true
    applyCurrentVars(e.payload)
    emitPreview()
    unlisten()
  })
  await emit('theme_request_current')
  setTimeout(() => {
    if (!gotCurrent) emitPreview()
  }, 400)
})

/** Populates the draft from a set of computed CSS variables (best-effort). */
function applyCurrentVars(v: Record<string, string> | undefined) {
  if (!v) return
  const setColor = (key: ThemeTokenKey, varName: string) => {
    const raw = v[varName]
    if (!raw) return
    if (/^#[0-9a-f]{8}$/i.test(raw.trim())) {
      themeCreator[key] = raw.trim()
      return
    }
    const hex = cssColorToHex(raw)
    if (/^#[0-9a-f]{3,8}$/i.test(hex)) themeCreator[key] = hex
  }
  const setNum = (key: keyof typeof themeCreator, varName: string) => {
    const raw = v[varName]
    if (!raw) return
    const n = parseRem(raw)
    if (isFinite(n)) (themeCreator as Record<string, unknown>)[key] = n
  }
  for (const [key, cssVar] of Object.entries(TOKEN_CSS_VARS) as [ThemeTokenKey, string][]) {
    setColor(key, cssVar)
  }
  // Legacy compat when main still has old pair vars.
  if (!v['--color-bg'] && v['--color-bg-dark']) setColor('bg', '--color-bg-dark')
  if (!v['--color-btn'] && v['--color-btn-dark']) setColor('btnBg', '--color-btn-dark')
  if (v['--font-btn']) themeCreator.btnFontFamily = extractFontFamily(v['--font-btn'])
  if (v['--font-tab']) themeCreator.tabFontFamily = extractFontFamily(v['--font-tab'])
  setNum('fontSizeBtn', '--font-size-btn')
  setNum('fontSizeTab', '--font-size-tab')
  setNum('fontSizeMd', '--font-size-md')
  setNum('btnWidth', '--btn_width')
  setNum('borderRadius', '--border-radius')
  setNum('borderWidth', '--btn-border-width')
  setNum('tabBorderWidth', '--tab-border-width')
  setNum('buttonGap', '--button-gap')
  if (v['--btn_padding']) {
    const [py, px] = parsePadding(v['--btn_padding'])
    if (isFinite(py)) themeCreator.btnPaddingY = py
    if (isFinite(px)) themeCreator.btnPaddingX = px
  }
  if (v['--gif-overlay']) themeCreator.gifOverlay = opacityToPct(v['--gif-overlay'])
  if (v['--gif-overlay-hover']) themeCreator.gifOverlayHover = opacityToPct(v['--gif-overlay-hover'])
}

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

onBeforeUnmount(() => {
  emit('theme_saved').catch(() => {})
})

const closePrompt = ref(false)

async function destroyThemeWindow() {
  closePrompt.value = false
  try {
    await getCurrentWindow().destroy()
  } catch (e) {
    console.warn('destroy theme-creator failed', e)
  }
}

onMounted(() => {
  getCurrentWindow().onCloseRequested((event) => {
    event.preventDefault()
    closePrompt.value = true
  }).catch(() => {})
})

async function saveAndClose() {
  closePrompt.value = false
  await saveThemeToFolder()
}

async function discardAndClose() {
  await emit('theme_saved').catch(() => {})
  await destroyThemeWindow()
}

function cancelClose() {
  closePrompt.value = false
}

// ── Font dropdown state ───────────────────────────────────────────────────────
const btnFontOpen = ref(false)
const tabFontOpen = ref(false)
const btnFontDropdownRef = ref<HTMLElement | null>(null)
const tabFontDropdownRef = ref<HTMLElement | null>(null)

function onFontDropdownPointerDown(e: PointerEvent) {
  const t = e.target
  if (!(t instanceof Node)) return
  if (btnFontOpen.value && btnFontDropdownRef.value && !btnFontDropdownRef.value.contains(t)) {
    btnFontOpen.value = false
  }
  if (tabFontOpen.value && tabFontDropdownRef.value && !tabFontDropdownRef.value.contains(t)) {
    tabFontOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('pointerdown', onFontDropdownPointerDown, true)
})
onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onFontDropdownPointerDown, true)
})

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
function currentTokens(): ThemeTokens {
  const tokens = {} as ThemeTokens
  for (const key of Object.keys(TOKEN_CSS_VARS) as ThemeTokenKey[]) {
    tokens[key] = themeCreator[key]
  }
  return tokens
}

function buildThemeCss() {
  const name = (themeCreator.name || 'theme').trim()
  return buildCssFromTokens(name, currentTokens(), {
    '--font-btn': `'${themeCreator.btnFontFamily}', sans-serif`,
    '--font-tab': `'${themeCreator.tabFontFamily}', sans-serif`,
    '--font-size-btn': `${themeCreator.fontSizeBtn}rem`,
    '--font-size-tab': `${themeCreator.fontSizeTab}rem`,
    '--font-size-md': `${themeCreator.fontSizeMd}rem`,
    '--btn_width': `${themeCreator.btnWidth}rem`,
    '--border-radius': `${themeCreator.borderRadius}rem`,
    '--btn-border-width': `${themeCreator.borderWidth}rem`,
    '--tab-border-width': `${themeCreator.tabBorderWidth}rem`,
    '--button-gap': `${themeCreator.buttonGap}rem`,
    '--btn_padding': `${themeCreator.btnPaddingY}rem ${themeCreator.btnPaddingX}rem`,
    '--gif-overlay': String(themeCreator.gifOverlay / 100),
    '--gif-overlay-hover': String(themeCreator.gifOverlayHover / 100),
  })
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

const importError = ref('')

function fixColorInput(key: ThemeTokenKey) {
  const raw = themeCreator[key]
  if (/^#[0-9a-f]{8}$/i.test(raw)) return
  const hex = cssColorToHex(raw)
  if (/^#[0-9a-f]{6}$/i.test(hex)) themeCreator[key] = hex
}

function clampMin(key: ThemeSliderKey, min: number) {
  const v = Number(themeCreator[key])
  themeCreator[key] = isNaN(v) ? min : parseFloat(Math.max(min, v).toFixed(2))
}

function clampRange(key: ThemeSliderKey, min: number, max: number) {
  const v = Number(themeCreator[key])
  const n = isNaN(v) ? min : v
  themeCreator[key] = parseFloat(Math.min(max, Math.max(min, n)).toFixed(0))
}

/** CSS opacity 0–1 (or leftover 0–100 / 55%) → Theme Creator percent. */
function opacityToPct(v: string): number {
  const n = parseFloat(v)
  if (!isFinite(n)) return LAYOUT_DEFAULTS.gifOverlay
  if (n <= 1) return Math.round(n * 100)
  return Math.round(Math.min(100, Math.max(0, n)))
}

function extractFontFamily(v: string): string {
  const m = v.match(/['"]([^'"]+)['"]/)
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
    const parsed = parseThemeCss(css)
    if (!parsed.primaryColor) {
      importError.value = t('settings.themeCreator.importError') + ' --primary_color'
      return
    }
    applyParsedTheme(css)
    loadThemeId.value = ''
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
    await emit('theme_apply', { theme: `file:${safeName}.css` })
    await destroyThemeWindow()
  } catch (e) {
    console.error('Save to themes folder failed', e)
  }
}
</script>
