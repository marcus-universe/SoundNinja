<template>
  <Transition name="overlay-fade">
    <div v-if="appStore.activeOverlay === 'settings'" class="settings-overlay" @click.self="appStore.setActiveOverlay(null)">
      <div class="settings-panel">

        <!-- X close button -->
        <button class="settings-close-btn" :title="'Close'" @click="appStore.setActiveOverlay(null)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>

        <!-- Left sidebar -->
        <aside :class="['settings-sidebar', { 'settings-sidebar--collapsed': sidebarCollapsed }]">
          <button class="settings-collapse-btn" :title="sidebarCollapsed ? 'Expand' : 'Collapse'" @click="sidebarCollapsed = !sidebarCollapsed">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="3" y1="6" x2="21" y2="6"/>
              <line x1="3" y1="12" x2="21" y2="12"/>
              <line x1="3" y1="18" x2="21" y2="18"/>
            </svg>
          </button>
          <div class="settings-sidebar__header"><span class="settings-sidebar__header-text">Settings</span></div>
          <nav class="settings-nav">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              :class="['settings-nav__item', { active: activeTab === tab.id }]"
              :title="tab.label"
              @click="activeTab = tab.id"
            >
              <span class="settings-nav__icon" v-html="tab.icon" />
              <span class="settings-nav__label">{{ tab.label }}</span>
            </button>
          </nav>
        </aside>

        <!-- Right content -->
        <main class="settings-content">

          <!-- ── Main Tab ── -->
          <section v-if="activeTab === 'main'">
            <h2 class="settings-content__title">Main</h2>

            <div class="settings-group">
              <label class="settings-label">Theme</label>
              <select v-model="selectedTheme" @change="applyTheme" class="settings-select">
                <option v-for="t in builtinThemes" :key="t.id" :value="t.id">{{ t.label }}</option>
                <option v-for="t in savedThemes" :key="t" :value="'file:' + t">{{ t.replace('.css', '') }}</option>
                <option value="custom">Custom CSS…</option>
              </select>
            </div>

            <Transition name="fade">
              <div v-if="selectedTheme === 'custom'" class="settings-group settings-group--stacked">
                <label class="settings-label">Custom CSS</label>
                <p class="settings-hint">Edit CSS variables to create your own theme. Changes apply live.</p>
                <div class="code-editor-wrapper">
                  <textarea class="code-editor" v-model="customCssValue" spellcheck="false" @input="onCustomCssInput" />
                </div>
                <div class="settings-row">
                  <button class="settings-btn" @click="loadCustomCssFile">📂 Load .css File</button>
                  <button class="settings-btn settings-btn--warn" @click="resetCustomCss">↺ Reset Template</button>
                </div>
              </div>
            </Transition>

            <div class="settings-section-divider">Playback Behavior</div>

            <div class="settings-group settings-group--toggle">
              <div class="settings-toggle-text">
                <span class="settings-label">Stop on retrigger</span>
                <span class="settings-hint">Pressing active sound again stops it</span>
              </div>
              <UICheckbox :modelValue="stopOnRetrigger" @update:modelValue="onStopOnRetrigger" />
            </div>

            <div class="settings-group settings-group--toggle">
              <div class="settings-toggle-text">
                <span class="settings-label">Overlap sounds</span>
                <span class="settings-hint">Multiple sounds play simultaneously</span>
              </div>
              <UICheckbox :modelValue="overlapSounds" @update:modelValue="onOverlapSounds" />
            </div>

          </section>

          <!-- ── Audio Tab ── -->
          <section v-if="activeTab === 'audio'">
            <h2 class="settings-content__title">Audio</h2>

            <div class="settings-group">
              <label class="settings-label">Output Device</label>
              <div class="settings-row">
                <select v-model="outputSelected" @change="selectOutputDevice" class="settings-select">
                  <option v-for="device in OutputDevices[0]" :key="device" :value="device">{{ device }}</option>
                </select>
                <button class="settings-btn settings-btn--icon" @click="getOutputDevices" title="Refresh">&#x21BB;</button>
              </div>
            </div>
          </section>

          <!-- ── Theme Creator Tab ── -->
          <section v-if="activeTab === 'theme-creator'" class="theme-creator-section">
            <h2 class="settings-content__title">Theme Creator</h2>

            <div class="theme-creator-layout">
              <!-- Controls column -->
              <div class="theme-creator-controls">

                <div class="settings-group">
                  <label class="settings-label">Theme Name</label>
                  <input type="text" class="settings-input" v-model="themeCreator.name" placeholder="My Custom Theme" />
                </div>

                <div class="settings-section-divider">Colors</div>

                <div class="settings-group settings-group--inline">
                  <label class="settings-label">Primary / Accent</label>
                  <div class="settings-color-row">
                    <input type="color" class="settings-color" v-model="themeCreator.primaryColor" @input="previewTheme" />
                    <span class="settings-color-value">{{ themeCreator.primaryColor }}</span>
                  </div>
                </div>
                <div class="settings-group settings-group--inline">
                  <label class="settings-label">Background</label>
                  <div class="settings-color-row">
                    <input type="color" class="settings-color" v-model="themeCreator.bgColor" @input="previewTheme" />
                    <span class="settings-color-value">{{ themeCreator.bgColor }}</span>
                  </div>
                </div>
                <div class="settings-group settings-group--inline">
                  <label class="settings-label">Button Background</label>
                  <div class="settings-color-row">
                    <input type="color" class="settings-color" v-model="themeCreator.btnColor" @input="previewTheme" />
                    <span class="settings-color-value">{{ themeCreator.btnColor }}</span>
                  </div>
                </div>

                <div class="settings-section-divider">Button Typography</div>

                <div class="settings-group settings-group--stacked">
                  <label class="settings-label">Button Font</label>
                  <div class="font-dropdown" ref="btnFontDropdownRef">
                    <div class="font-dropdown__trigger" @click="btnFontOpen = !btnFontOpen">
                      <span :style="{ fontFamily: themeCreator.btnFontFamily }">{{ themeCreator.btnFontFamily }}</span>
                      <span class="font-dropdown__arrow">▾</span>
                    </div>
                    <div v-show="btnFontOpen" class="font-dropdown__list">
                      <div class="font-dropdown__search">
                        <input
                          v-model="btnFontSearch"
                          type="text"
                          class="font-dropdown__search-input"
                          placeholder="Search font…"
                        />
                      </div>
                      <div
                        v-for="font in filteredBtnFonts"
                        :key="'btn-' + font"
                        class="font-dropdown__option"
                        :class="{ active: themeCreator.btnFontFamily === font }"
                        :style="{ fontFamily: font }"
                        @click="themeCreator.btnFontFamily = font; btnFontOpen = false; previewTheme()"
                      >{{ font }}</div>
                    </div>
                  </div>
                </div>
                <div class="settings-group settings-group--stacked">
                  <label class="settings-label">Button Font Size — {{ themeCreator.fontSizeBtn }}rem</label>
                  <input type="range" class="settings-slider" min="0.7" max="1.8" step="0.05" v-model.number="themeCreator.fontSizeBtn" @input="previewTheme" />
                </div>

                <div class="settings-section-divider">Tab Typography</div>

                <div class="settings-group settings-group--stacked">
                  <label class="settings-label">Tab Font</label>
                  <div class="font-dropdown" ref="tabFontDropdownRef">
                    <div class="font-dropdown__trigger" @click="tabFontOpen = !tabFontOpen">
                      <span :style="{ fontFamily: themeCreator.tabFontFamily }">{{ themeCreator.tabFontFamily }}</span>
                      <span class="font-dropdown__arrow">▾</span>
                    </div>
                    <div v-show="tabFontOpen" class="font-dropdown__list">
                      <div class="font-dropdown__search">
                        <input
                          v-model="tabFontSearch"
                          type="text"
                          class="font-dropdown__search-input"
                          placeholder="Search font…"
                        />
                      </div>
                      <div
                        v-for="font in filteredTabFonts"
                        :key="'tab-' + font"
                        class="font-dropdown__option"
                        :class="{ active: themeCreator.tabFontFamily === font }"
                        :style="{ fontFamily: font }"
                        @click="themeCreator.tabFontFamily = font; tabFontOpen = false; previewTheme()"
                      >{{ font }}</div>
                    </div>
                  </div>
                </div>
                <div class="settings-group settings-group--stacked">
                  <label class="settings-label">Tab Font Size — {{ themeCreator.fontSizeTab }}rem</label>
                  <input type="range" class="settings-slider" min="0.7" max="1.8" step="0.05" v-model.number="themeCreator.fontSizeTab" @input="previewTheme" />
                </div>

                <div class="settings-section-divider">Layout</div>

                <div class="settings-group settings-group--stacked">
                  <label class="settings-label">Button Width — {{ themeCreator.btnWidth }}rem</label>
                  <input type="range" class="settings-slider" min="7" max="18" step="0.5" v-model.number="themeCreator.btnWidth" @input="previewTheme" />
                </div>
                <div class="settings-group settings-group--stacked">
                  <label class="settings-label">Border Radius — {{ themeCreator.borderRadius }}rem</label>
                  <input type="range" class="settings-slider" min="0" max="2" step="0.05" v-model.number="themeCreator.borderRadius" @input="previewTheme" />
                </div>
                <div class="settings-group settings-group--stacked">
                  <label class="settings-label">Button Padding X — {{ themeCreator.btnPaddingX }}rem</label>
                  <input type="range" class="settings-slider" min="0.25" max="2.5" step="0.05" v-model.number="themeCreator.btnPaddingX" @input="previewTheme" />
                </div>
                <div class="settings-group settings-group--stacked">
                  <label class="settings-label">Button Padding Y — {{ themeCreator.btnPaddingY }}rem</label>
                  <input type="range" class="settings-slider" min="0.1" max="1.5" step="0.05" v-model.number="themeCreator.btnPaddingY" @input="previewTheme" />
                </div>

                <div class="settings-row" style="margin-top: 1.5rem">
                  <button class="settings-btn settings-btn--primary" @click="exportTheme">⬇ Export .css</button>
                  <button class="settings-btn" @click="saveThemeToFolder">＋ Add to Themes</button>
                </div>
              </div>

              <!-- Preview column -->
              <div class="theme-creator-preview-col">
                <div class="tc-preview-label">Live Preview</div>

                <div class="tc-preview-section">Tabs</div>
                <div class="tc-tab-bar" :style="previewBgStyle">
                  <div class="tc-tab" :style="tabPreviewStyle">Sounds</div>
                  <div class="tc-tab" :style="tabActivePreviewStyle">Music</div>
                  <div class="tc-tab" :style="tabPreviewStyle">SFX</div>
                </div>

                <div class="tc-preview-section" style="margin-top: 1.25rem">Buttons</div>
                <div class="tc-btn-grid" :style="previewBgStyle">
                  <div class="tc-btn" :style="btnPreviewStyle">Sample Sound</div>
                  <div class="tc-btn" :style="btnActivePreviewStyle">Playing…</div>
                  <div class="tc-btn" :style="btnPreviewStyle">Ambient</div>
                  <div class="tc-btn" :style="btnPreviewStyle">Effect</div>
                </div>
              </div>
            </div>
          </section>

        </main>
      </div>
      <BlurBG />
    </div>
  </Transition>

  <!-- ── Select Project Dialog ─────────────────────────────────────────────── -->
  <Transition name="overlay-fade">
    <div
      v-if="appStore.selectProjectActive"
      class="project-overlay"
      @click.self="appStore.setSelectProjectActive(false)"
    >
      <div class="project-dialog">
        <div class="project-dialog__header">
          <h2>Select Project</h2>
          <Icons :icon="'exit'" :customClass="'exit icon'" @triggered="appStore.setSelectProjectActive(false)" />
        </div>
        <div class="project-dialog__body">
          <div v-if="projectsLoading" class="project-dialog__empty">Loading…</div>
          <div v-else-if="projectsList.length === 0" class="project-dialog__empty">No projects found. Create one below.</div>
          <ul v-else class="project-list">
            <li
              v-for="proj in projectsList"
              :key="proj"
              class="project-list__item"
              :class="{ active: jsonStore.currentProjectPath === proj }"
              @click="selectProject(proj)"
            >
              <span class="project-list__name">{{ proj.replace('.json', '') }}</span>
              <span v-if="jsonStore.currentProjectPath === proj" class="project-list__check">✓</span>
            </li>
          </ul>
        </div>
        <div class="project-dialog__footer">
          <div class="new-project-row">
            <input
              v-model="newProjectName"
              type="text"
              class="new-project-input settings-input"
              placeholder="New project name…"
              @keyup.enter="createProject"
            />
            <button
              class="settings-btn settings-btn--primary"
              :disabled="!newProjectName.trim()"
              @click="createProject"
            >
              Create
            </button>
          </div>
        </div>
      </div>
      <BlurBG />
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readTextFile, writeTextFile, mkdir, readDir, BaseDirectory } from '@tauri-apps/plugin-fs'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

// ── Tabs ─────────────────────────────────────────────────────────────────────
const tabs = [
  {
    id: 'main',
    label: 'Main',
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>`,
  },
  {
    id: 'audio',
    label: 'Audio',
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 010 14.14M15.54 8.46a5 5 0 010 7.07"/></svg>`,
  },
  {
    id: 'theme-creator',
    label: 'Theme',
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="13.5" cy="6.5" r=".5" fill="currentColor"/><circle cx="17.5" cy="10.5" r=".5" fill="currentColor"/><circle cx="8.5" cy="7.5" r=".5" fill="currentColor"/><circle cx="6.5" cy="12.5" r=".5" fill="currentColor"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.47-1.125-.29-.289-.47-.688-.47-1.125a1.64 1.64 0 011.648-1.688h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/></svg>`,
  },
]
const activeTab = ref('main')
const sidebarCollapsed = ref(false)

// ── Built-in themes ───────────────────────────────────────────────────────────
const builtinThemes = [
  { id: 'dark-cyan',   label: 'Dark Cyan',   vars: { '--primary_color': 'hsl(189, 100%, 58%)', '--color-bg': '#222831' } },
  { id: 'dark-purple', label: 'Dark Purple',  vars: { '--primary_color': 'hsl(270, 80%, 65%)',  '--color-bg': '#1e1b2e' } },
  { id: 'dark-orange', label: 'Dark Orange',  vars: { '--primary_color': 'hsl(28, 100%, 58%)',  '--color-bg': '#231c15' } },
  { id: 'dark-green',  label: 'Dark Green',   vars: { '--primary_color': 'hsl(145, 80%, 50%)',  '--color-bg': '#162218' } },
  { id: 'dark-pink',   label: 'Dark Pink',    vars: { '--primary_color': 'hsl(330, 80%, 65%)',  '--color-bg': '#231520' } },
]

const CSS_TEMPLATE = `:root {
  /* ── Primary accent color ────────────────────── */
  --primary_color: hsl(189, 100%, 58%);

  /* ── Background color ────────────────────────── */
  --color-bg: #222831;

  /* ── Font sizes ──────────────────────────────── */
  --font-size-sm: 1.2rem;
  --font-size-md: 1rem;

  /* ── Layout sizes ────────────────────────────── */
  --icon_size: 2.5rem;
  --tab_width: 5rem;
  --btn_width: 11.5rem;
  --btn_padding: 0.5rem 0.75rem;
}
`

const selectedTheme = ref('dark-cyan')
const customCssValue = ref(CSS_TEMPLATE)
const stopOnRetrigger = ref(true)
const overlapSounds = ref(false)

function onStopOnRetrigger(val: boolean) {
  stopOnRetrigger.value = val
  jsonStore.setStopOnRetrigger(val)
}

function onOverlapSounds(val: boolean) {
  overlapSounds.value = val
  jsonStore.setOverlapSounds(val)
}
const savedThemes = ref([])

// ── Audio ─────────────────────────────────────────────────────────────────────
const OutputDevices = ref([])
const outputSelected = ref('')

// ── Theme Creator ─────────────────────────────────────────────────────────────
const systemFonts = ref(['Segoe UI', 'Arial', 'Verdana', 'Georgia', 'Courier New'])
const btnFontOpen = ref(false)
const tabFontOpen = ref(false)
const btnFontDropdownRef = ref(null)
const tabFontDropdownRef = ref(null)
onClickOutside(btnFontDropdownRef, () => { btnFontOpen.value = false })
onClickOutside(tabFontDropdownRef, () => { tabFontOpen.value = false })

const btnFontSearch = ref('')
const tabFontSearch = ref('')
watch(btnFontOpen, (val) => { if (!val) btnFontSearch.value = '' })
watch(tabFontOpen, (val) => { if (!val) tabFontSearch.value = '' })

const filteredBtnFonts = computed(() => {
  const q = btnFontSearch.value.trim().toLowerCase()
  return q ? systemFonts.value.filter(f => f.toLowerCase().includes(q)) : systemFonts.value
})

const filteredTabFonts = computed(() => {
  const q = tabFontSearch.value.trim().toLowerCase()
  return q ? systemFonts.value.filter(f => f.toLowerCase().includes(q)) : systemFonts.value
})

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

function buildThemeCss() {
  return `:root {
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
  textAlign: 'center',
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
  whiteSpace: 'nowrap',
}))

const tabActivePreviewStyle = computed(() => ({
  ...tabPreviewStyle.value,
  color: themeCreator.primaryColor,
  borderBottomColor: themeCreator.primaryColor,
}))

function previewTheme() {
  // live preview – no-op here; computed styles handle it
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

async function saveThemeToFolder() {
  const css = buildThemeCss()
  const safeName = (themeCreator.name || 'theme').replace(/[^a-z0-9_-]/gi, '_')
  try {
    await mkdir('themes', { baseDir: BaseDirectory.AppData, recursive: true })
    await writeTextFile(`themes/${safeName}.css`, css, { baseDir: BaseDirectory.AppData })
    await loadSavedThemes()
  } catch (e) {
    console.error('Save to themes folder failed', e)
  }
}

async function loadSavedThemes() {
  try {
    await mkdir('themes', { baseDir: BaseDirectory.AppData, recursive: true })
    const entries = await readDir('themes', { baseDir: BaseDirectory.AppData })
    savedThemes.value = entries
      .filter((e) => e.name && e.name.endsWith('.css'))
      .map((e) => e.name)
  } catch {
    savedThemes.value = []
  }
}

// ── Project management ──────────────────────────────────────────────────────
const projectsList = ref([])
const projectsLoading = ref(false)
const newProjectName = ref('')

async function refreshProjects() {
  projectsLoading.value = true
  try {
    await mkdir('projects', { baseDir: BaseDirectory.AppData, recursive: true })
    const entries = await readDir('projects', { baseDir: BaseDirectory.AppData })
    projectsList.value = entries
      .filter((e) => e.name && e.name.endsWith('.json'))
      .map((e) => e.name)
      .sort()
  } catch {
    projectsList.value = []
  } finally {
    projectsLoading.value = false
  }
}

async function selectProject(filename) {
  try {
    const content = await readTextFile(`projects/${filename}`, { baseDir: BaseDirectory.AppData })
    const config = JSON.parse(content)
    jsonStore.updateConfigFile(config)
    jsonStore.setCurrentProjectPath(filename)
    appStore.setSelectProjectActive(false)
  } catch (e) {
    console.error('Failed to load project', e)
  }
}

async function createProject() {
  const name = newProjectName.value.trim()
  if (!name) return
  const safeName = name.replace(/[^a-z0-9_\- ]/gi, '_').replace(/\s+/g, '_')
  const filename = `${safeName}.json`
  const emptyConfig = {
    settings: { theme: 'dark-cyan', customCss: '', outputSource: 'default' },
    tabList: [],
    files: [],
  }
  try {
    await mkdir('projects', { baseDir: BaseDirectory.AppData, recursive: true })
    await writeTextFile(`projects/${filename}`, JSON.stringify(emptyConfig, null, 2), {
      baseDir: BaseDirectory.AppData,
    })
    jsonStore.updateConfigFile(emptyConfig)
    jsonStore.setCurrentProjectPath(filename)
    newProjectName.value = ''
    await refreshProjects()
    appStore.setSelectProjectActive(false)
  } catch (e) {
    console.error('Failed to create project', e)
  }
}

watch(
  () => appStore.selectProjectActive,
  (val) => { if (val) refreshProjects() }
)

// ── Main settings ─────────────────────────────────────────────────────────────
const OutputState = computed(() => jsonStore.configFile?.settings?.outputSource)

watch(() => appStore.activeOverlay, async (val) => {
  if (val === 'settings') {
    selectedTheme.value = jsonStore.configFile?.settings?.theme ?? 'dark-cyan'
    customCssValue.value = jsonStore.configFile?.settings?.customCss || CSS_TEMPLATE
    stopOnRetrigger.value = jsonStore.configFile?.settings?.stopOnRetrigger ?? true
    overlapSounds.value = jsonStore.configFile?.settings?.overlapSounds ?? false
    const outputVal = OutputState.value
    if (outputVal) outputSelected.value = outputVal
    applyTheme()
    await loadSavedThemes()
  }
})

function applyTheme() {
  const id = selectedTheme.value
  jsonStore.setTheme(id)

  if (id === 'custom') {
    injectCustomCss(customCssValue.value)
    return
  }

  if (id.startsWith('file:')) {
    const filename = id.slice(5)
    readTextFile(`themes/${filename}`, { baseDir: BaseDirectory.AppData })
      .then((css) => injectCustomCss(css))
      .catch((e) => console.error('Failed to load theme file', e))
    return
  }

  removeCustomCssTag()
  const theme = builtinThemes.find((t) => t.id === id)
  if (!theme) return
  const root = document.documentElement
  Object.entries(theme.vars).forEach(([k, v]) => root.style.setProperty(k, v))
}

function onCustomCssInput() {
  jsonStore.setCustomCss(customCssValue.value)
  injectCustomCss(customCssValue.value)
}

function injectCustomCss(css) {
  let tag = document.getElementById('sn-custom-theme')
  if (!tag) {
    tag = document.createElement('style')
    tag.id = 'sn-custom-theme'
    document.head.appendChild(tag)
  }
  tag.textContent = css
}

function removeCustomCssTag() {
  const tag = document.getElementById('sn-custom-theme')
  if (tag) tag.remove()
}

function resetCustomCss() {
  customCssValue.value = CSS_TEMPLATE
  jsonStore.setCustomCss(CSS_TEMPLATE)
  injectCustomCss(CSS_TEMPLATE)
}

async function loadCustomCssFile() {
  try {
    const filePath = await open({
      multiple: false,
      title: 'Load Custom Theme CSS',
      filters: [{ name: 'CSS Files', extensions: ['css'] }],
    })
    if (!filePath) return
    const contents = await readTextFile(filePath)
    customCssValue.value = contents
    jsonStore.setCustomCss(contents)
    injectCustomCss(contents)
  } catch (e) {
    console.error('Failed to load CSS file', e)
  }
}

async function getOutputDevices() {
  const devices = [await invoke('get_out_devices')]
  OutputDevices.value = [devices[0]]
}

onMounted(async () => {
  getOutputDevices()
  try {
    const fonts = await invoke('get_system_fonts')
    if (Array.isArray(fonts) && fonts.length > 0) systemFonts.value = fonts
  } catch (e) {
    console.warn('get_system_fonts failed', e)
  }
})

function selectOutputDevice(event) {
  jsonStore.setOutSource(event.target.value)
}
</script>

<style lang="scss" scoped>
@use '~/assets/scss/variables' as *;

// ── Overlay backdrop ──────────────────────────────────────────────────────────
.settings-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
}

// ── Panel ─────────────────────────────────────────────────────────────────────
.settings-panel {
  display: flex;
  min-width: 90vw;
  min-height: 90vh;
  max-width: 90vw;
  max-height: 90vh;
  background: $color-bg;
  border: 0.15rem solid var(--primary_color);
  border-radius: 0.75rem;
  overflow: hidden;
  z-index: 3;
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.6);
  position: relative;
}

// ── Close button (top-right of panel) ────────────────────────────────────────
.settings-close-btn {
  all: unset;
  position: absolute;
  top: 0.85rem;
  right: 0.85rem;
  z-index: 10;
  cursor: pointer;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.35rem;
  color: rgba(238, 238, 238, 0.5);
  transition: color 0.15s, background 0.15s;

  svg {
    width: 1.1rem;
    height: 1.1rem;
  }

  &:hover {
    color: $white;
    background: rgba(255, 255, 255, 0.08);
  }
}

// ── Sidebar ───────────────────────────────────────────────────────────────────
.settings-sidebar {
  display: flex;
  flex-direction: column;
  width: 220px;
  flex-shrink: 0;
  background: rgba(0, 0, 0, 0.25);
  border-right: 0.1rem solid rgba(255, 255, 255, 0.07);
  padding: 1.5rem 0.75rem;
  gap: 0.25rem;
  transition: width 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;

  &--collapsed {
    width: 56px;

    .settings-sidebar__header-text {
      opacity: 0;
      pointer-events: none;
    }

    .settings-nav__label {
      opacity: 0;
      max-width: 0;
      overflow: hidden;
    }

    .settings-nav__item {
      justify-content: center;
      padding: 0.55rem;
    }

    .settings-collapse-btn {
      margin-left: auto;
      margin-right: auto;
    }
  }

  &__header {
    font-family: $fontBold;
    font-size: 0.75rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: rgba(238, 238, 238, 0.45);
    padding: 0 0.5rem 0.75rem;
    border-bottom: 0.1rem solid rgba(255, 255, 255, 0.07);
    margin-bottom: 0.5rem;
    white-space: nowrap;
    overflow: hidden;
  }

  &__header-text {
    display: inline-block;
    transition: opacity 0.2s ease;
  }

  &__footer {
    margin-top: auto;
    padding-top: 1rem;
    display: flex;
    justify-content: center;
  }
}

.settings-nav {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
}

.settings-nav__item {
  all: unset;
  cursor: pointer;
  padding: 0.55rem 0.75rem;
  border-radius: 0.4rem;
  font-family: $fontRegular;
  font-size: 0.95rem;
  color: rgba(238, 238, 238, 0.65);
  transition: background 0.15s, color 0.15s;
  display: flex;
  align-items: center;
  gap: 0.5rem;

  &:hover {
    background: rgba(255, 255, 255, 0.07);
    color: $white;
  }

  &.active {
    background: var(--primary_color);
    color: $color-bg;
    font-family: $fontBold;
  }
}

.settings-nav__icon {
  width: 1.1rem;
  height: 1.1rem;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;

  svg {
    width: 100%;
    height: 100%;
    display: block;
  }
}

.settings-nav__label {
  font-size: 0.9rem;
  white-space: nowrap;
  transition: opacity 0.2s ease, max-width 0.25s ease;
  max-width: 160px;
}

// ── Collapse button ───────────────────────────────────────────────────────────
.settings-collapse-btn {
  all: unset;
  cursor: pointer;
  width: 2rem;
  height: 2rem;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 0.35rem;
  color: rgba(238, 238, 238, 0.5);
  margin-bottom: 0.75rem;
  transition: color 0.15s, background 0.15s;
  flex-shrink: 0;

  svg {
    width: 1.1rem;
    height: 1.1rem;
  }

  &:hover {
    color: $white;
    background: rgba(255, 255, 255, 0.07);
  }
}

// ── Content area ──────────────────────────────────────────────────────────────
.settings-content {
  flex: 1;
  padding: 2rem 2.5rem;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0;

  section {
    display: flex;
    flex-direction: column;
    gap: 0;
    max-width: 680px;
  }

  &__title {
    font-family: $fontBold;
    font-size: 1.4rem;
    color: $white;
    margin: 0 0 1.5rem;
    padding-bottom: 0.75rem;
    border-bottom: 0.1rem solid rgba(255, 255, 255, 0.1);
  }
}

// ── Settings groups ───────────────────────────────────────────────────────────
.settings-group {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.75rem 0;
  border-bottom: 0.05rem solid rgba(255, 255, 255, 0.05);

  &--stacked {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.5rem;
  }

  &--inline {
    justify-content: space-between;
  }

  &--toggle {
    justify-content: space-between;
    align-items: center;
  }
}

.settings-toggle-text {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;

  .settings-label {
    min-width: unset;
  }
}

.settings-label {
  font-family: $fontRegular;
  font-size: 0.95rem;
  color: $white;
  flex-shrink: 0;
  min-width: 160px;
}

.settings-hint {
  font-family: $fontRegular;
  font-size: 0.82rem;
  color: rgba(238, 238, 238, 0.5);
  margin: 0;
  line-height: 1.5;
}

.settings-section-divider {
  font-family: $fontBold;
  font-size: 0.72rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: rgba(238, 238, 238, 0.4);
  margin: 1.25rem 0 0.25rem;
}

// ── Form controls ─────────────────────────────────────────────────────────────
.settings-select,
.settings-input {
  flex: 1;
  background: rgba(255, 255, 255, 0.05);
  border: 0.1rem solid rgba(255, 255, 255, 0.15);
  border-radius: 0.4rem;
  color: $white;
  font-family: $fontRegular;
  font-size: 0.9rem;
  padding: 0.45rem 0.75rem;
  transition: border-color 0.15s;
  outline: none;

  &:hover,
  &:focus {
    border-color: var(--primary_color);
  }

  option {
    background: $color-bg;
  }
}

.settings-row {
  display: flex;
  gap: 0.75rem;
  width: 100%;
  align-items: center;

  .settings-select {
    flex: 1;
  }
}

.settings-slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  height: 4px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.15);
  outline: none;

  &::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--primary_color);
    cursor: pointer;
  }
}

.settings-btn {
  flex: 1;
  padding: 0.45rem 1rem;
  background: transparent;
  border: 0.1rem solid rgba(255, 255, 255, 0.2);
  border-radius: 0.4rem;
  color: $white;
  font-family: $fontRegular;
  font-size: 0.9rem;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s;

  &:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: var(--primary_color);
  }

  &--primary {
    border-color: var(--primary_color);
    color: var(--primary_color);

    &:hover {
      background: var(--primary_color);
      color: $color-bg;
    }
  }

  &--warn {
    border-color: $warn;
    color: $warn;

    &:hover {
      background: rgba($warn, 0.12);
    }
  }

  &--icon {
    flex: 0;
    padding: 0.45rem 0.75rem;
    font-size: 1.1rem;
  }
}

// ── Color picker ──────────────────────────────────────────────────────────────
.settings-color-row {
  display: flex;
  align-items: center;
  gap: 0.75rem;
}

.settings-color {
  -webkit-appearance: none;
  appearance: none;
  width: 2.5rem;
  height: 2.5rem;
  border: 0.1rem solid rgba(255, 255, 255, 0.2);
  border-radius: 0.4rem;
  padding: 0;
  cursor: pointer;
  background: transparent;

  &::-webkit-color-swatch-wrapper {
    padding: 0;
    border-radius: 0.35rem;
    overflow: hidden;
  }

  &::-webkit-color-swatch {
    border: none;
  }
}

.settings-color-value {
  font-family: 'Consolas', monospace;
  font-size: 0.85rem;
  color: rgba(238, 238, 238, 0.55);
}

// ── Code editor ───────────────────────────────────────────────────────────────
.code-editor-wrapper {
  width: 100%;
  border: 0.1rem solid rgba(255, 255, 255, 0.15);
  border-radius: 0.5rem;
  overflow: hidden;
  background: rgba(0, 0, 0, 0.3);
}

.code-editor {
  width: 100%;
  min-height: 260px;
  background: transparent;
  color: #a8e6cf;
  font-family: 'Consolas', 'Fira Code', monospace;
  font-size: 0.82rem;
  line-height: 1.55;
  padding: 1rem;
  border: none;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
}

// ── Theme creator 2-column layout ─────────────────────────────────────────────
.theme-creator-section {
  max-width: unset;
}

.theme-creator-layout {
  display: flex;
  gap: 2rem;
  align-items: flex-start;
  flex-wrap: wrap;
}

.theme-creator-controls {
  flex: 1 1 300px;
  min-width: 0;
}

.theme-creator-preview-col {
  flex: 0 1 260px;
  width: 260px;
  position: sticky;
  top: 0;
  padding: 1rem;
  background: rgba(0, 0, 0, 0.2);
  border-radius: 0.5rem;
  border: 0.1rem solid rgba(255, 255, 255, 0.07);
}

// ── Font dropdown ─────────────────────────────────────────────────────────────
.font-dropdown {
  position: relative;
  width: 100%;
}

.font-dropdown__trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.45rem 0.75rem;
  background: rgba(255, 255, 255, 0.05);
  border: 0.1rem solid rgba(255, 255, 255, 0.15);
  border-radius: 0.4rem;
  cursor: pointer;
  color: $white;
  font-size: 0.9rem;
  transition: border-color 0.15s;
  gap: 0.5rem;

  &:hover {
    border-color: var(--primary_color);
  }

  span:first-child {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.font-dropdown__arrow {
  flex-shrink: 0;
  opacity: 0.5;
  font-style: normal;
}

.font-dropdown__list {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  z-index: 200;
  max-height: 220px;
  overflow-y: auto;
  background: $color-bg;
  border: 0.1rem solid rgba(255, 255, 255, 0.15);
  border-radius: 0.4rem;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);

  &::-webkit-scrollbar {
    width: 6px;
  }
  &::-webkit-scrollbar-track {
    background: transparent;
  }
  &::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 3px;
  }
}

.font-dropdown__option {
  padding: 0.45rem 0.75rem;
  font-size: 0.95rem;
  color: rgba(238, 238, 238, 0.75);
  cursor: pointer;
  transition: background 0.1s;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;

  &:hover {
    background: rgba(255, 255, 255, 0.08);
    color: $white;
  }

  &.active {
    background: var(--primary_color);
    color: $color-bg;
  }
}

.font-dropdown__search {
  padding: 0.4rem 0.5rem;
  border-bottom: 0.1rem solid rgba(255, 255, 255, 0.08);
  position: sticky;
  top: 0;
  background: $color-bg;
  z-index: 1;
}

.font-dropdown__search-input {
  width: 100%;
  background: rgba(255, 255, 255, 0.07);
  border: 0.1rem solid rgba(255, 255, 255, 0.15);
  border-radius: 0.3rem;
  color: $white;
  font-family: $fontRegular;
  font-size: 0.85rem;
  padding: 0.3rem 0.5rem;
  outline: none;
  box-sizing: border-box;

  &:focus {
    border-color: var(--primary_color);
  }
}

// ── Preview column elements ───────────────────────────────────────────────────
.tc-preview-label {
  font-family: $fontBold;
  font-size: 0.72rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: rgba(238, 238, 238, 0.4);
  margin-bottom: 0.75rem;
}

.tc-preview-section {
  font-family: $fontRegular;
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: rgba(238, 238, 238, 0.4);
  margin-bottom: 0.4rem;
}

.tc-tab-bar {
  display: flex;
  border-radius: 0.3rem;
  overflow: hidden;
  border: 0.1rem solid;
  padding: 0.25rem 0.5rem 0;
}

.tc-tab {
  padding: 0.4rem 0.75rem;
  font-size: 0.9rem;
  user-select: none;
}

.tc-btn-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
  padding: 0.75rem;
  border-radius: 0.3rem;
  border: 0.1rem solid;
}

.tc-btn {
  padding: 0.5rem 0.75rem;
  text-align: center;
  font-size: 0.9rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 100%;
}

// ── Transitions ───────────────────────────────────────────────────────────────
.overlay-fade-enter-active,
.overlay-fade-leave-active {
  transition: opacity 0.2s ease;
}
.overlay-fade-enter-from,
.overlay-fade-leave-to {
  opacity: 0;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

// ── Project Dialog ────────────────────────────────────────────────────────────
.project-overlay {
  position: fixed;
  inset: 0;
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
}

.project-dialog {
  position: relative;
  z-index: 1;
  width: 480px;
  max-width: 86vw;
  max-height: 80vh;
  background: $color-bg;
  border: 0.15rem solid var(--primary_color);
  border-radius: 0.75rem;
  overflow: hidden;
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.6);
  display: flex;
  flex-direction: column;

  &__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.25rem 1.5rem;
    border-bottom: 0.1rem solid rgba(255, 255, 255, 0.08);

    h2 {
      font-family: $fontBold;
      font-size: 1.1rem;
      color: $white;
      margin: 0;
    }
  }

  &__body {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 0;
    min-height: 100px;
  }

  &__empty {
    padding: 2.5rem;
    text-align: center;
    color: rgba(238, 238, 238, 0.35);
    font-family: $fontRegular;
    font-size: 0.9rem;
  }

  &__footer {
    padding: 1rem 1.5rem;
    border-top: 0.1rem solid rgba(255, 255, 255, 0.08);
  }
}

.project-list {
  list-style: none;
  padding: 0;
  margin: 0;

  &__item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1.5rem;
    cursor: pointer;
    transition: background 0.15s;
    font-family: $fontRegular;
    font-size: 0.95rem;
    color: rgba(238, 238, 238, 0.85);

    &:hover {
      background: rgba(255, 255, 255, 0.06);
    }

    &.active {
      color: var(--primary_color);
    }
  }

  &__check {
    color: var(--primary_color);
  }
}

.new-project-row {
  display: flex;
  gap: 0.5rem;

  .settings-input {
    flex: 1;
  }
}
</style>

