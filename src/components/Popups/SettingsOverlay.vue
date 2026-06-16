<template>
  <Transition name="overlay-fade">
    <div v-if="appStore.activeOverlay === 'settings'" class="overlay" @click.self="appStore.setActiveOverlay(null)">
      <div class="overlay-panel flex_c_v align_c">
        <div class="overlay-header flex_c_h flex_space_between w100">
          <h1>Settings</h1>
          <Icons :icon="'exit'" :customClass="'exit overlay-close'" @triggered="appStore.setActiveOverlay(null)" />
        </div>
        <div class="settingsContainer gap1 flex_c_v align_c">

          <!-- Theme Dropdown -->
          <div class="flex_c gap1 w100 setting">
            <label for="theme">Theme:</label>
            <div class="theme-row">
              <select id="theme" v-model="selectedTheme" @change="applyTheme">
                <option v-for="t in builtinThemes" :key="t.id" :value="t.id">{{ t.label }}</option>
                <option value="custom">Custom CSS…</option>
              </select>
            </div>
          </div>

          <!-- Custom CSS editor (shown only when "custom" is selected) -->
          <Transition name="fade">
            <div v-if="selectedTheme === 'custom'" class="flex_c_v gap1 w100 setting">
              <label>Custom CSS</label>
              <p class="setting-hint">
                Edit the CSS variables below to create your own theme. Changes are applied live.
              </p>
              <div class="code-editor-wrapper">
                <textarea
                  class="code-editor"
                  v-model="customCssValue"
                  spellcheck="false"
                  @input="onCustomCssInput"
                />
              </div>
              <div class="flex_c gap1">
                <button class="theme-action-btn" @click="loadCustomCssFile" title="Load .css file">
                  📂 Load .css File
                </button>
                <button class="theme-action-btn theme-action-btn--reset" @click="resetCustomCss" title="Reset to template">
                  ↺ Reset Template
                </button>
              </div>
            </div>
          </Transition>

          <!-- Output Device -->
          <div class="flex_c gap1 w100 setting">
            <label>Select Output-Device</label>
            <div class="flex_c gap1 output-device-row">
              <select v-model="outputSelected" @change="selectOutputDevice">
                <option v-for="device in OutputDevices[0]" :key="device" :value="device">
                  {{ device }}
                </option>
              </select>
              <button class="refresh-btn" @click="getOutputDevices" title="Refresh devices">&#x21BB;</button>
            </div>
          </div>

        </div>
      </div>
      <BlurBG />
    </div>
  </Transition>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { readTextFile } from '@tauri-apps/plugin-fs'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

// ---- Built-in themes ----
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
  --font_size_1: 1.2rem;
  --font_size_2: 1rem;

  /* ── Layout sizes ────────────────────────────── */
  --icon_size: 2.5rem;
  --tab_width: 5rem;
  --btn_width: 11.5rem;
}
`

const selectedTheme = ref('dark-cyan')
const customCssValue = ref(CSS_TEMPLATE)
const OutputDevices = ref([])
const outputSelected = ref('')

const OutputState = computed(() => jsonStore.configFile?.settings?.outputSource)

watch(() => appStore.activeOverlay, (val) => {
  if (val === 'settings') {
    selectedTheme.value = jsonStore.configFile?.settings?.theme ?? 'dark-cyan'
    customCssValue.value = jsonStore.configFile?.settings?.customCss || CSS_TEMPLATE
    const outputVal = OutputState.value
    if (outputVal) outputSelected.value = outputVal
    // Apply persisted theme on open
    applyTheme()
  }
})

function applyTheme() {
  const id = selectedTheme.value
  jsonStore.setTheme(id)

  if (id === 'custom') {
    injectCustomCss(customCssValue.value)
    return
  }

  // Remove any injected custom CSS style tag
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

onMounted(() => {
  getOutputDevices()
})

const selectOutputDevice = (event) => {
  jsonStore.setOutSource(event.target.value)
}
</script>

<style lang="scss" scoped>
@use '~/assets/scss/variables' as *;

.output-device-row {
  flex: 1;
  display: flex;
  align-items: center;

  select {
    flex: 1;
  }

  .refresh-btn {
    flex-shrink: 0;
    background: transparent;
    border: 0.1rem solid var(--primary_color);
    border-radius: 0.4rem;
    color: var(--primary_color);
    font-size: 1.2rem;
    padding: 0.2rem 0.6rem;
    cursor: pointer;
    transition: background 0.15s;

    &:hover {
      background: var(--primary_color);
      color: var(--color-bg, #1a1a1a);
    }
  }
}

.theme-row {
  flex: 1;
  display: flex;
  align-items: center;

  select {
    flex: 1;
  }
}

.setting-hint {
  font-family: $fontRegular;
  font-size: 0.85rem;
  color: rgba(238, 238, 238, 0.55);
  margin: 0;
}

.code-editor-wrapper {
  width: 100%;
  border: 0.15rem solid var(--primary_color);
  border-radius: 0.5rem;
  overflow: hidden;
  background: #1a1a2e;
}

.code-editor {
  width: 100%;
  min-height: 260px;
  background: #1a1a2e;
  color: #a8e6cf;
  font-family: 'Consolas', 'Fira Code', monospace;
  font-size: 0.85rem;
  line-height: 1.5;
  padding: 1rem;
  border: none;
  resize: vertical;
  outline: none;
  box-sizing: border-box;
  tab-size: 2;
}

.flex_c {
  display: flex;
  justify-content: center;
  align-items: center;
}

.theme-action-btn {
  flex: 1;
  padding: 0.4rem 0.8rem;
  background: transparent;
  border: 0.1rem solid var(--primary_color);
  color: var(--primary_color);
  border-radius: 0.4rem;
  font-family: $fontRegular;
  font-size: 0.9rem;
  cursor: pointer;
  transition: background 0.15s;

  &:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  &--reset {
    border-color: $warn;
    color: $warn;

    &:hover {
      background: rgba($warn, 0.1);
    }
  }
}
</style>

