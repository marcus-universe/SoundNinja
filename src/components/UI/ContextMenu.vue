<template>
  <Teleport to="body">
    <div
      v-if="appStore.contextMenu.visible"
      ref="menuEl"
      class="context-menu"
      :style="{ top: menuY + 'px', left: menuX + 'px' }"
      @click.stop
    >
      <ul class="context-menu__list">
        <li v-if="appStore.contextMenu.type !== 'separator'" class="context-menu__item" @click="openRename" @mouseenter="hoveredItem = 'rename'" @mouseleave="hoveredItem = null">
          <span class="context-menu__icon">
            <Icons icon="rename" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.rename') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'rename'" class="context-menu__desc">{{ $t('contextMenu.renameDesc') }}</span>
          </Transition>
        </li>
        <li class="context-menu__item context-menu__item--danger" @click="remove" @mouseenter="hoveredItem = 'remove'" @mouseleave="hoveredItem = null">
          <span class="context-menu__icon">
            <Icons icon="delete" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.remove') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'remove'" class="context-menu__desc">{{ $t('contextMenu.removeDesc') }}</span>
          </Transition>
        </li>
        <li
          v-if="appStore.contextMenu.type === 'sound'"
          class="context-menu__item"
          @click="toggleMoveToTab"
          @mouseenter="hoveredItem = 'moveToTab'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="tab" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.moveToTab') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'moveToTab'" class="context-menu__desc">{{ $t('contextMenu.moveToTabDesc') }}</span>
          </Transition>
          <span class="context-menu__chevron">{{ moveToTabOpen ? '▲' : '▼' }}</span>
        </li>
        <template v-if="moveToTabOpen && appStore.contextMenu.type === 'sound'">
          <li
            v-for="tab in allTabs"
            :key="tab.name"
            class="context-menu__item context-menu__tab-row"
            @click.stop="toggleSoundTab(tab.name)"
          >
            <span class="context-menu__check">{{ soundTabs.includes(tab.name) ? '☑' : '☐' }}</span>
            {{ tab.name }}
          </li>
        </template>
        <li
          v-if="appStore.contextMenu.type === 'sound'"
          class="context-menu__item"
          @click="addSeparator"
          @mouseenter="hoveredItem = 'separator'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="page-separator" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.addSeparator') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'separator'" class="context-menu__desc">{{ $t('contextMenu.addSeparatorDesc') }}</span>
          </Transition>
        </li>
        <li v-if="appStore.contextMenu.type !== 'separator'" class="context-menu__item context-menu__item--color" @click="toggleColorPicker" @mouseenter="hoveredItem = 'color'" @mouseleave="hoveredItem = null">
          <span
            class="context-menu__swatch"
            :style="swatchStyle"
          />
          <span class="context-menu__label">{{ $t('contextMenu.colors') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'color'" class="context-menu__desc">{{ $t('contextMenu.colorsDesc') }}</span>
          </Transition>
          <span class="context-menu__chevron">{{ colorPickerOpen ? '▲' : '▼' }}</span>
        </li>
        <li v-if="colorPickerOpen" class="context-menu__color-panel" @click.stop>
          <ColorGroupPicker
            :model-value="currentOverride"
            :base-colors="baseColors"
            inline
            @change="onOverrideChange"
          />
        </li>
        <li
          v-if="appStore.contextMenu.type === 'sound'"
          class="context-menu__item"
          @click="openGifPicker"
          @mouseenter="hoveredItem = 'gifBg'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="gif" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.gifBg') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'gifBg'" class="context-menu__desc">{{ $t('contextMenu.gifBgDesc') }}</span>
          </Transition>
        </li>
      </ul>
    </div>
    <!-- invisible backdrop to close on outside click -->
    <div
      v-if="appStore.contextMenu.visible"
      class="context-menu__backdrop"
      @click="close"
      @contextmenu.prevent="close"
    />
  </Teleport>
</template>

<script setup>
import {
  parseOverride,
  serializeOverride,
  overrideSwatch,
  resolveEffectiveColors,
} from '~/utils/colorOverride'

const { t: $t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const colorPickerOpen = ref(false)
const moveToTabOpen = ref(false)
const hoveredItem = ref(null)
const menuEl = ref(null)
const menuSize = ref({ w: 220, h: 180 })

const allTabs = computed(() => jsonStore.configFile.tabList)
const soundTabs = computed(() => {
  const { targetIndex } = appStore.contextMenu
  return jsonStore.configFile.files[targetIndex]?.tabs ?? []
})

function toggleMoveToTab() {
  moveToTabOpen.value = !moveToTabOpen.value
}

function addSeparator() {
  const { targetIndex } = appStore.contextMenu
  const sound = jsonStore.configFile.files[targetIndex]
  appStore.closeContextMenu()
  colorPickerOpen.value = false
  moveToTabOpen.value = false
  if (!sound) return
  const tab = appStore.currentTab
  const order = tab === 'All' ? (sound.index ?? 0) : (sound.tabIndexes?.[tab] ?? 0)
  jsonStore.addSeparator(tab, order - 0.5)
}

function toggleSoundTab(tabName) {
  const { targetIndex } = appStore.contextMenu
  const sound = jsonStore.configFile.files[targetIndex]
  if (!sound) return
  const tabs = [...sound.tabs]
  const idx = tabs.indexOf(tabName)
  if (idx !== -1) {
    tabs.splice(idx, 1)
  } else {
    tabs.push(tabName)
  }
  jsonStore.setSoundTabs(targetIndex, tabs)
}

async function measureMenu() {
  await nextTick()
  const el = menuEl.value
  if (!el || typeof window === 'undefined') return
  const rect = el.getBoundingClientRect()
  menuSize.value = { w: rect.width || 220, h: rect.height || 180 }
}

watch(
  () => [
    appStore.contextMenu.visible,
    appStore.contextMenu.x,
    appStore.contextMenu.y,
    colorPickerOpen.value,
    moveToTabOpen.value,
  ],
  ([visible]) => {
    if (visible) measureMenu()
  },
  { flush: 'post' },
)

const menuX = computed(() => {
  const x = appStore.contextMenu.x
  if (typeof window === 'undefined') return x
  return Math.min(x, Math.max(4, window.innerWidth - menuSize.value.w - 8))
})
const menuY = computed(() => {
  const y = appStore.contextMenu.y
  if (typeof window === 'undefined') return y
  return Math.min(y, Math.max(4, window.innerHeight - menuSize.value.h - 8))
})

const rawColor = computed(() => {
  const { type, targetName, targetIndex } = appStore.contextMenu
  if (type === 'tab') {
    const tab = jsonStore.configFile.tabList.find((t) => t.name === targetName)
    return tab?.color ?? ''
  }
  if (type === 'sound') {
    return jsonStore.configFile.files[targetIndex]?.color ?? ''
  }
  return ''
})

const currentOverride = computed(() => parseOverride(rawColor.value))

const baseColors = computed(() => {
  // Recompute when menu target / visibility / stored color / theme changes.
  void appStore.contextMenu.visible
  void appStore.contextMenu.type
  void appStore.contextMenu.targetName
  void appStore.contextMenu.targetIndex
  void rawColor.value
  const s = jsonStore.configFile?.settings
  void s?.theme
  void s?.btnBg
  void s?.tabBg
  void s?.primaryColor

  const { type, targetName, targetIndex } = appStore.contextMenu
  let el = null
  if (typeof document !== 'undefined') {
    if (type === 'tab' && targetName) {
      el = document.querySelector(`.tab[data-tab-name="${CSS.escape(targetName)}"]`)
    } else if (type === 'sound' && targetIndex != null) {
      const path = jsonStore.configFile.files[targetIndex]?.path
      if (path) {
        el = document.querySelector(`[data-sound-path="${CSS.escape(path)}"]`)
      }
    }
  }
  const kind = type === 'tab' ? 'tab' : 'button'
  return resolveEffectiveColors(currentOverride.value, kind, el)
})

const swatchStyle = computed(() => ({
  background: overrideSwatch(currentOverride.value, baseColors.value.border),
}))

function toggleColorPicker() {
  colorPickerOpen.value = !colorPickerOpen.value
}

function onOverrideChange(override) {
  const serialized = serializeOverride(override)
  const { type, targetName, targetIndex } = appStore.contextMenu
  if (type === 'tab') {
    jsonStore.setTabColor(targetName, serialized)
  } else if (type === 'sound') {
    jsonStore.setSoundColor(targetIndex, serialized)
  }
}

function openRename() {
  const { type, targetName, targetIndex } = appStore.contextMenu
  appStore.closeContextMenu()
  colorPickerOpen.value = false
  moveToTabOpen.value = false
  if (type === 'tab') {
    appStore.setPopupActive({ active: true, type: 'renameTab' })
  } else if (type === 'sound') {
    appStore.setPopupActive({ active: true, type: 'renameSound' })
  }
}

function remove() {
  const { type, targetName, targetIndex } = appStore.contextMenu
  appStore.closeContextMenu()
  colorPickerOpen.value = false
  moveToTabOpen.value = false
  if (type === 'tab') {
    jsonStore.removeTab(targetName)
    if (appStore.currentTab === targetName) {
      appStore.setCurrentTab('All')
    }
  } else if (type === 'sound') {
    jsonStore.removeSound(targetIndex)
  } else if (type === 'separator') {
    jsonStore.removeSeparator(targetName)
  }
}

function close() {
  appStore.closeContextMenu()
  colorPickerOpen.value = false
  moveToTabOpen.value = false
}

function openGifPicker() {
  const { targetIndex } = appStore.contextMenu
  close()
  if (typeof targetIndex === 'number' && targetIndex >= 0) {
    appStore.openGifPicker(targetIndex)
  }
}

onMounted(() => {
  const handler = (e) => { if (e.key === 'Escape') close() }
  window.addEventListener('keydown', handler)
  onUnmounted(() => window.removeEventListener('keydown', handler))
})
</script>
