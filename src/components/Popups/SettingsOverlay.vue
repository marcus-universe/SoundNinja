<template>
  <Transition name="overlay-fade">
    <div
      v-if="appStore.activeOverlay === 'settings'"
      class="settings-overlay"
      @click.self="appStore.setActiveOverlay(null)"
    >
      <div class="settings-panel" :class="{ 'is-detail': phoneDetail }">

        <header v-if="phone" class="settings-phone-head">
          <button
            v-if="phoneDetail"
            type="button"
            class="settings-phone-head__back"
            :aria-label="$t('remoteUi.settingsBack')"
            @click="detail = false"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <polyline points="15 18 9 12 15 6" />
            </svg>
          </button>
          <span v-else class="settings-phone-head__side" />
          <h2 class="settings-phone-head__title">{{ phoneDetail ? activeTabLabel : $t('settings.title') }}</h2>
          <button
            type="button"
            class="settings-phone-head__close"
            :aria-label="$t('settings.close')"
            @click="appStore.setActiveOverlay(null)"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </header>
        <DialogCloseButton v-if="!phone" :title="$t('settings.close')" @close="appStore.setActiveOverlay(null)" />

        <!-- Sidebar -->
        <aside :class="['settings-sidebar', { 'settings-sidebar--collapsed': sidebarCollapsed && !phone }]">
          <button
            v-if="!phone"
            class="settings-collapse-btn"
            :title="sidebarCollapsed ? $t('settings.expand') : $t('settings.collapse')"
            @click="sidebarCollapsed = !sidebarCollapsed"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="3" y1="6" x2="21" y2="6"/>
              <line x1="3" y1="12" x2="21" y2="12"/>
              <line x1="3" y1="18" x2="21" y2="18"/>
            </svg>
          </button>

          <div v-if="!phone" class="settings-sidebar__header">
            <span class="settings-sidebar__header-text">{{ $t('settings.title') }}</span>
          </div>

          <nav class="settings-nav">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              :class="['settings-nav__item', { active: !phone && activeTab === tab.id }]"
              :title="tab.label"
              @click="selectTab(tab.id)"
            >
              <span class="settings-nav__icon" v-html="tab.icon" />
              <span class="settings-nav__label">{{ tab.label }}</span>
              <svg v-if="phone" class="settings-nav__chevron" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <polyline points="9 18 15 12 9 6" />
              </svg>
            </button>
          </nav>
        </aside>

        <!-- Tab content -->
        <main ref="contentEl" class="settings-content" tabindex="-1">
          <SettingsMain v-if="activeTab === 'main'" />
          <SettingsBehavior v-else-if="activeTab === 'behavior'" />
          <SettingsHotkeys v-else-if="activeTab === 'hotkeys'" />
          <SettingsAudio v-else-if="activeTab === 'audio'" />
          <SettingsPerformance v-else-if="activeTab === 'performance'" />
          <SettingsRemote v-else-if="activeTab === 'remote'" />
          <SettingsAbout v-else-if="activeTab === 'about'" />
        </main>

      </div>
    </div>
  </Transition>

  <SelectProjectDialog />
</template>

<script setup lang="ts">
import { isRemote } from '~/utils/runtime'

const { t } = useI18n()
const appStore = useAppStore()

const contentEl = ref<HTMLElement | null>(null)
const activeTab = ref('main')

function focusContent() {
  nextTick(() => {
    if (appStore.activeOverlay !== 'settings') return
    contentEl.value?.focus({ preventScroll: true })
  })
}
const sidebarCollapsed = ref(false)
const narrow = ref(false)
const detail = ref(false)
const phone = computed(() => isRemote && narrow.value)
const phoneDetail = computed(() => phone.value && detail.value)
const activeTabLabel = computed(() => tabs.value.find((tab) => tab.id === activeTab.value)?.label || '')
let mq: MediaQueryList | null = null

function onMq() {
  narrow.value = !!mq?.matches
  if (!narrow.value) detail.value = false
}

function selectTab(id: string) {
  activeTab.value = id
  if (isRemote && narrow.value) detail.value = true
  focusContent()
}

onMounted(() => {
  mq = window.matchMedia('(max-width: 639px)')
  onMq()
  mq.addEventListener('change', onMq)
})
onUnmounted(() => mq?.removeEventListener('change', onMq))

const tabs = computed(() => {
  const all = [
  {
    id: 'main',
    label: t('settings.tabs.main'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>`,
  },
  {
    id: 'behavior',
    label: t('settings.tabs.behavior'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 013 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>`,
  },
  {
    id: 'hotkeys',
    label: t('settings.tabs.hotkeys'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h.01M12 12h.01M16 12h.01M6 16h12"/></svg>`,
  },
  {
    id: 'audio',
    label: t('settings.tabs.audio'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 010 14.14M15.54 8.46a5 5 0 010 7.07"/></svg>`,
  },
  {
    id: 'performance',
    label: t('settings.tabs.performance'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>`,
  },
  {
    id: 'remote',
    label: t('settings.tabs.remote'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="2"/><path d="M16.24 7.76a6 6 0 010 8.49M7.76 16.24a6 6 0 010-8.49M19.07 4.93a10 10 0 010 14.14M4.93 19.07a10 10 0 010-14.14"/></svg>`,
  },
  {
    id: 'about',
    label: t('settings.tabs.about'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>`,
  },
  ]
  return isRemote ? all.filter((tab) => tab.id !== 'remote') : all
})

watch(() => appStore.activeOverlay, (val) => {
  if (val !== 'settings') {
    detail.value = false
    return
  }
  const pending = appStore.consumePendingSettingsTab()
  if (pending) {
    activeTab.value = pending
    if (narrow.value) detail.value = true
  } else if (narrow.value) {
    detail.value = false
  }
  focusContent()
}, { immediate: true })
</script>
