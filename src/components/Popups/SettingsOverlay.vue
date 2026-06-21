<template>
  <Transition name="overlay-fade">
    <div
      v-if="appStore.activeOverlay === 'settings'"
      class="settings-overlay"
      @click.self="appStore.setActiveOverlay(null)"
    >
      <div class="settings-panel">

        <!-- Close button -->
        <button class="settings-close-btn" :title="$t('settings.close')" @click="appStore.setActiveOverlay(null)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round">
            <line x1="18" y1="6" x2="6" y2="18"/>
            <line x1="6" y1="6" x2="18" y2="18"/>
          </svg>
        </button>

        <!-- Sidebar -->
        <aside :class="['settings-sidebar', { 'settings-sidebar--collapsed': sidebarCollapsed }]">
          <button
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

          <div class="settings-sidebar__header">
            <span class="settings-sidebar__header-text">{{ $t('settings.title') }}</span>
          </div>

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

        <!-- Tab content -->
        <main class="settings-content">
          <SettingsMain v-if="activeTab === 'main'" />
          <SettingsAudio v-else-if="activeTab === 'audio'" />
          <SettingsThemeCreator v-else-if="activeTab === 'theme-creator'" />
          <SettingsAbout v-else-if="activeTab === 'about'" />
        </main>

      </div>
      <BlurBG />
    </div>
  </Transition>

  <SelectProjectDialog />
</template>

<script setup lang="ts">
const { t, locale, setLocale } = useI18n()
const appStore = useAppStore()

const activeTab = ref('main')
const sidebarCollapsed = ref(false)

const tabs = computed(() => [
  {
    id: 'main',
    label: t('settings.tabs.main'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z"/></svg>`,
  },
  {
    id: 'audio',
    label: t('settings.tabs.audio'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 010 14.14M15.54 8.46a5 5 0 010 7.07"/></svg>`,
  },
  {
    id: 'theme-creator',
    label: t('settings.tabs.theme'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="13.5" cy="6.5" r=".5" fill="currentColor"/><circle cx="17.5" cy="10.5" r=".5" fill="currentColor"/><circle cx="8.5" cy="7.5" r=".5" fill="currentColor"/><circle cx="6.5" cy="12.5" r=".5" fill="currentColor"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.47-1.125-.29-.289-.47-.688-.47-1.125a1.64 1.64 0 011.648-1.688h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/></svg>`,
  },
  {
    id: 'about',
    label: t('settings.tabs.about'),
    icon: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/></svg>`,
  },
])

onMounted(async () => {
  if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('sn-locale')
    if (saved && saved !== locale.value) {
      await setLocale(saved as 'en' | 'de')
    }
  }
})

watch(() => appStore.activeOverlay, (val) => {
  if (val === 'settings') {
    const pending = appStore.consumePendingSettingsTab()
    if (pending) activeTab.value = pending
  }
})
</script>
