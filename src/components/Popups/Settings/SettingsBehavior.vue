<template>
  <section>
    <h2 class="settings-content__title">{{ $t('settings.tabs.behavior') }}</h2>

    <div class="settings-section-divider">{{ $t('settings.behavior.playback') }}</div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.stopOnRetrigger') }}</span>
        <span class="settings-hint">{{ $t('settings.main.stopOnRetriggerHint') }}</span>
      </div>
      <UICheckbox :modelValue="stopOnRetrigger" @update:modelValue="onStopOnRetrigger" />
    </div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.overlapSounds') }}</span>
        <span class="settings-hint">{{ $t('settings.main.overlapSoundsHint') }}</span>
      </div>
      <UICheckbox :modelValue="overlapSounds" @update:modelValue="onOverlapSounds" />
    </div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.showPlayer') }}</span>
        <span class="settings-hint">{{ $t('settings.main.showPlayerHint') }}</span>
      </div>
      <UICheckbox :modelValue="showPlayer" @update:modelValue="onShowPlayer" />
    </div>

    <div v-if="showPlayer" class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.playerLarge') }}</span>
        <span class="settings-hint">{{ $t('settings.main.playerLargeHint') }}</span>
      </div>
      <UICheckbox :modelValue="playerLarge" @update:modelValue="onPlayerLarge" />
    </div>

    <div class="settings-section-divider">{{ $t('settings.behavior.window') }}</div>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.main.navbarSide') }}</label>
      <select v-model="navbarSide" @change="onNavbarSide" class="settings-select">
        <option value="left">{{ $t('settings.main.navbarLeft') }}</option>
        <option value="right">{{ $t('settings.main.navbarRight') }}</option>
      </select>
    </div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.hideTitlebar') }}</span>
        <span class="settings-hint">{{ $t('settings.main.hideTitlebarHint') }}</span>
      </div>
      <UICheckbox :modelValue="hideTitlebar" @update:modelValue="onHideTitlebarToggle" />
    </div>

    <div v-if="!hideTitlebar" class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.systemTitlebar') }}</span>
        <span class="settings-hint">{{ $t('settings.main.systemTitlebarHint') }}</span>
      </div>
      <UICheckbox :modelValue="systemTitlebar" @update:modelValue="onSystemTitlebar" />
    </div>

    <DialogField
      v-if="hideTitlebarWarnOpen"
      :title="$t('settings.main.hideTitlebarWarnTitle')"
      @close="cancelHideTitlebar"
    >
      <p class="dialog-text">{{ $t('settings.main.hideTitlebarWarnText') }}</p>
      <UICheckbox v-model="hideTitlebarDontRemind" class="hide-titlebar-dont-remind">
        {{ $t('settings.main.hideTitlebarDontRemind') }}
      </UICheckbox>
      <div class="flex_c_h gap1 dialog-actions">
        <UIButton @click="confirmHideTitlebar">{{ $t('settings.main.hideTitlebarWarnConfirm') }}</UIButton>
        <UIButton @click="cancelHideTitlebar">{{ $t('dialog.cancel') }}</UIButton>
      </div>
    </DialogField>

    <div class="settings-section-divider">{{ $t('settings.behavior.controls') }}</div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.allowReorder') }}</span>
        <span class="settings-hint">{{ $t('settings.main.allowReorderHint') }}</span>
      </div>
      <UICheckbox :modelValue="allowReorder" @update:modelValue="onAllowReorder" />
    </div>

    <div class="settings-section-divider">{{ $t('settings.behavior.ui') }}</div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.uniformButtonHeight') }}</span>
        <span class="settings-hint">{{ $t('settings.main.uniformButtonHeightHint') }}</span>
      </div>
      <UICheckbox :modelValue="uniformButtonHeight" @update:modelValue="onUniformButtonHeight" />
    </div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.navbarTooltips') }}</span>
        <span class="settings-hint">{{ $t('settings.main.navbarTooltipsHint') }}</span>
      </div>
      <UICheckbox :modelValue="navbarTooltips" @update:modelValue="onNavbarTooltips" />
    </div>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.main.tabTransition') }}</label>
      <select v-model="tabTransition" @change="onTabTransition" class="settings-select">
        <option value="slide">{{ $t('settings.main.tabTransitionSlide') }}</option>
        <option value="fade">{{ $t('settings.main.tabTransitionFade') }}</option>
        <option value="stagger">{{ $t('settings.main.tabTransitionStagger') }}</option>
        <option value="none">{{ $t('settings.main.tabTransitionNone') }}</option>
      </select>
    </div>

    <div class="settings-group">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.recentLimit') }}</span>
        <span class="settings-hint">{{ $t('settings.main.recentLimitHint') }}</span>
      </div>
      <input type="number" class="settings-input" v-model.number="recentLimit" min="1" max="100" @change="onRecentLimit" />
    </div>

    <div class="settings-section-divider">{{ $t('settings.behavior.gifs') }}</div>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.gifPlayOnHover') }}</span>
        <span class="settings-hint">{{ $t('settings.main.gifPlayOnHoverHint') }}</span>
      </div>
      <UICheckbox :modelValue="gifPlayOnHover" @update:modelValue="onGifPlayOnHover" />
    </div>

    <div class="settings-group settings-group--stacked">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.main.klipyApiKey') }}</span>
        <span class="settings-hint">{{ $t('settings.main.klipyApiKeyHint') }}</span>
      </div>
      <div class="settings-password-row">
        <input
          class="settings-input settings-input--wide"
          :type="klipyKeyVisible ? 'text' : 'password'"
          autocomplete="off"
          spellcheck="false"
          :placeholder="$t('settings.main.klipyApiKeyPlaceholder')"
          :value="klipyApiKey"
          @change="onKlipyApiKeyEvent"
        />
        <button
          type="button"
          class="settings-btn settings-btn--icon"
          :title="klipyKeyVisible ? $t('settings.main.klipyHideKey') : $t('settings.main.klipyShowKey')"
          :aria-label="klipyKeyVisible ? $t('settings.main.klipyHideKey') : $t('settings.main.klipyShowKey')"
          @click="klipyKeyVisible = !klipyKeyVisible"
        >
          <svg v-if="!klipyKeyVisible" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="settings-eye-icon">
            <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
            <circle cx="12" cy="12" r="3"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="settings-eye-icon">
            <path d="M17.94 17.94A10.07 10.07 0 0112 20c-7 0-11-8-11-8a18.45 18.45 0 015.06-5.94M9.9 4.24A9.12 9.12 0 0112 4c7 0 11 8 11 8a18.5 18.5 0 01-2.16 3.19"/>
            <line x1="1" y1="1" x2="23" y2="23"/>
          </svg>
        </button>
      </div>
      <div class="settings-link-row">
        <button type="button" class="settings-link-btn" @click="openKlipyDocs">
          {{ $t('settings.main.klipyDocs') }}
        </button>
        <button type="button" class="settings-link-btn" @click="openKlipyCreateKey">
          {{ $t('settings.main.klipyCreateKey') }}
        </button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { KLIPY_DOCS_URL, KLIPY_PARTNER_URL, openInSystemBrowser } from '~/utils/openExternal'
import { normalizeTabTransition, type TabTransition } from '~/utils/db'

const jsonStore = useJsonHandelingStore()
const appStore = useAppStore()
const appSettings = useAppSettingsStore()

const stopOnRetrigger = ref(true)
const overlapSounds = ref(false)
const showPlayer = ref(true)
const playerLarge = ref(false)
const uniformButtonHeight = ref(false)
const allowReorder = ref(true)
const gifPlayOnHover = ref(true)
const tabTransition = ref<TabTransition>('slide')
const klipyApiKey = ref('')
const klipyKeyVisible = ref(false)
const navbarTooltips = ref(true)
const systemTitlebar = ref(false)
const hideTitlebar = ref(false)
const hideTitlebarWarnOpen = ref(false)
const hideTitlebarDontRemind = ref(false)
const recentLimit = ref(30)
const navbarSide = ref<'left' | 'right'>('left')

async function onNavbarSide() {
  await appSettings.setNavbarSide(navbarSide.value)
}

function onStopOnRetrigger(val: boolean) {
  stopOnRetrigger.value = val
  jsonStore.setStopOnRetrigger(val)
}

function onOverlapSounds(val: boolean) {
  overlapSounds.value = val
  jsonStore.setOverlapSounds(val)
}

function onShowPlayer(val: boolean) {
  showPlayer.value = val
  jsonStore.setSetting('showPlayer', val)
}

function onPlayerLarge(val: boolean) {
  playerLarge.value = val
  jsonStore.setSetting('playerLarge', val)
}

function onUniformButtonHeight(val: boolean) {
  uniformButtonHeight.value = val
  jsonStore.setUniformButtonHeight(val)
}

function onAllowReorder(val: boolean) {
  allowReorder.value = val
  jsonStore.setAllowReorder(val)
}

function onGifPlayOnHover(val: boolean) {
  gifPlayOnHover.value = val
  jsonStore.setGifPlayOnHover(val)
}

function onTabTransition() {
  const val = normalizeTabTransition(tabTransition.value)
  tabTransition.value = val
  jsonStore.setSetting('tabTransition', val)
}

async function onKlipyApiKey(val: string) {
  klipyApiKey.value = val
  await appSettings.setKlipyApiKey(val)
}

function onKlipyApiKeyEvent(e: Event) {
  const el = e.target as HTMLInputElement
  onKlipyApiKey(el.value)
}

function openKlipyDocs() {
  openInSystemBrowser(KLIPY_DOCS_URL)
}

function openKlipyCreateKey() {
  openInSystemBrowser(KLIPY_PARTNER_URL)
}

async function onNavbarTooltips(val: boolean) {
  navbarTooltips.value = val
  await appSettings.setNavbarTooltips(val)
}

async function onSystemTitlebar(val: boolean) {
  systemTitlebar.value = val
  await appSettings.setTitlebarMode(val ? 'system' : 'styled')
}

async function onHideTitlebarToggle(val: boolean) {
  if (val) {
    if (appSettings.hideTitlebarSkipWarn) {
      hideTitlebar.value = true
      await appSettings.setHideTitlebar(true)
      return
    }
    hideTitlebarDontRemind.value = false
    hideTitlebarWarnOpen.value = true
    return
  }
  hideTitlebar.value = false
  await appSettings.setHideTitlebar(false)
}

async function confirmHideTitlebar() {
  hideTitlebarWarnOpen.value = false
  if (hideTitlebarDontRemind.value) {
    await appSettings.setHideTitlebarSkipWarn(true)
  }
  hideTitlebar.value = true
  await appSettings.setHideTitlebar(true)
}

function cancelHideTitlebar() {
  hideTitlebarWarnOpen.value = false
  hideTitlebarDontRemind.value = false
  hideTitlebar.value = false
}

async function onRecentLimit() {
  const n = Math.max(1, Math.min(100, Number(recentLimit.value) || 30))
  recentLimit.value = n
  await appSettings.setRecentLimit(n)
}

async function syncFromStore() {
  if (!appSettings.loaded) await appSettings.load()
  navbarSide.value = appSettings.navbarSide
  stopOnRetrigger.value = jsonStore.configFile?.settings?.stopOnRetrigger ?? true
  overlapSounds.value = jsonStore.configFile?.settings?.overlapSounds ?? false
  showPlayer.value = jsonStore.configFile?.settings?.showPlayer !== false
  playerLarge.value = jsonStore.configFile?.settings?.playerLarge === true
  uniformButtonHeight.value = jsonStore.configFile?.settings?.uniformButtonHeight ?? false
  allowReorder.value = jsonStore.configFile?.settings?.allowReorder ?? true
  gifPlayOnHover.value = jsonStore.configFile?.settings?.gifPlayOnHover !== false
  tabTransition.value = normalizeTabTransition(jsonStore.configFile?.settings?.tabTransition)
  klipyApiKey.value = appSettings.klipyApiKey || ''
  navbarTooltips.value = appSettings.navbarTooltips !== false
  systemTitlebar.value = appSettings.titlebarMode === 'system'
  hideTitlebar.value = !!appSettings.hideTitlebar
  recentLimit.value = appSettings.recentLimit ?? 30
}

watch(() => appStore.activeOverlay, async (val) => {
  if (val !== 'settings') return
  await syncFromStore()
})

onMounted(syncFromStore)
</script>
