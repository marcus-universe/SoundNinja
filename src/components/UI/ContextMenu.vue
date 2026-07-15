<template>
  <Teleport to="body">
    <div
      v-if="appStore.contextMenu.visible"
      class="context-menu"
      :style="{ top: menuY + 'px', left: menuX + 'px' }"
      @click.stop
    >
      <ul class="context-menu__list">
        <li v-if="appStore.contextMenu.type !== 'separator'" class="context-menu__item" @click="openRename" @mouseenter="hoveredItem = 'rename'" @mouseleave="hoveredItem = null">
          <span class="context-menu__icon">✏️</span>
          <span class="context-menu__label">{{ $t('contextMenu.rename') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'rename'" class="context-menu__desc">{{ $t('contextMenu.renameDesc') }}</span>
          </Transition>
        </li>
        <li class="context-menu__item context-menu__item--danger" @click="remove" @mouseenter="hoveredItem = 'remove'" @mouseleave="hoveredItem = null">
          <span class="context-menu__icon">🗑️</span>
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
          <span class="context-menu__icon">📂</span>
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
          <span class="context-menu__icon">➖</span>
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
          <span class="context-menu__label">{{ $t('contextMenu.color') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'color'" class="context-menu__desc">{{ $t('contextMenu.colorDesc') }}</span>
          </Transition>
          <span class="context-menu__chevron">{{ colorPickerOpen ? '▲' : '▼' }}</span>
        </li>
        <li v-if="colorPickerOpen" class="context-menu__color-row">
          <input
            type="color"
            class="context-menu__colorpicker"
            :value="currentColor"
            @input="onColorInput"
          />
          <button class="context-menu__reset-color" @click="resetColor" :title="$t('contextMenu.resetColor')">↺</button>
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
const { t: $t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const colorPickerOpen = ref(false)
const moveToTabOpen = ref(false)
const hoveredItem = ref(null)

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

// Keep menu inside viewport
const menuX = computed(() => {
  const x = appStore.contextMenu.x
  if (typeof window === 'undefined') return x
  return Math.min(x, window.innerWidth - 200)
})
const menuY = computed(() => {
  const y = appStore.contextMenu.y
  if (typeof window === 'undefined') return y
  return Math.min(y, window.innerHeight - 160)
})

const currentColor = computed(() => {
  const { type, targetName, targetIndex } = appStore.contextMenu
  if (type === 'tab') {
    const tab = jsonStore.configFile.tabList.find((t) => t.name === targetName)
      return (tab?.color ?? getComputedStyle(document.documentElement).getPropertyValue('--primary_color').trim()) || '#00e5ff'
  }
  if (type === 'sound') {
    return jsonStore.configFile.files[targetIndex]?.color ?? '#00e5ff'
  }
  return '#00e5ff'
})

const swatchStyle = computed(() => ({
  background: currentColor.value,
}))

function toggleColorPicker() {
  colorPickerOpen.value = !colorPickerOpen.value
}

function onColorInput(e) {
  const color = e.target.value
  const { type, targetName, targetIndex } = appStore.contextMenu
  if (type === 'tab') {
    jsonStore.setTabColor(targetName, color)
  } else if (type === 'sound') {
    jsonStore.setSoundColor(targetIndex, color)
  }
}

function resetColor() {
  const { type, targetName, targetIndex } = appStore.contextMenu
  if (type === 'tab') {
    jsonStore.setTabColor(targetName, '')
  } else if (type === 'sound') {
    jsonStore.setSoundColor(targetIndex, '')
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

// Also close on Escape
onMounted(() => {
  const handler = (e) => { if (e.key === 'Escape') close() }
  window.addEventListener('keydown', handler)
  onUnmounted(() => window.removeEventListener('keydown', handler))
})
</script>
