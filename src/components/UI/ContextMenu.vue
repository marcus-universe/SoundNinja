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

<style lang="scss" scoped>
@use '~/assets/scss/variables' as *;

.context-menu__backdrop {
  position: fixed;
  inset: 0;
  z-index: 999;
}

.context-menu {
  position: fixed;
  z-index: 1000;
  background: $color-bg;
  border: 0.15rem solid var(--primary_color);
  border-radius: 0.5rem;
  min-width: 160px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
  overflow: visible;

  &__list {
    list-style: none;
    margin: 0;
    padding: 0.25rem 0;
  }

  &__item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    font-family: $fontRegular;
    font-size: 0.95rem;
    color: #eee;
    cursor: pointer;
    transition: background 0.15s;
    position: relative;

    &:hover {
      background: rgba(255, 255, 255, 0.08);
    }

    &--danger {
      color: $warn;
      &:hover { background: rgba($warn, 0.12); }
    }

    &--color {
      justify-content: space-between;
    }
  }

  &__label {
    flex: 1;
    white-space: nowrap;
  }

  &__desc {
    position: absolute;
    left: calc(100% + 0.5rem);
    top: 50%;
    transform: translateY(-50%);
    background: rgba(0, 0, 0, 0.85);
    border: 0.1rem solid var(--primary_color);
    border-radius: 0.4rem;
    padding: 0.35rem 0.65rem;
    font-size: 0.78rem;
    color: rgba(255, 255, 255, 0.85);
    white-space: nowrap;
    max-width: 22rem;
    white-space: normal;
    width: max-content;
    pointer-events: none;
    z-index: 1001;
    box-shadow: 0 2px 12px rgba(0, 0, 0, 0.5);
  }

  &__icon {
    font-size: 0.9rem;
  }

  &__swatch {
    width: 1rem;
    height: 1rem;
    border-radius: 50%;
    border: 0.1rem solid rgba(255,255,255,0.3);
    flex-shrink: 0;
  }

  &__tab-row {
    padding-left: 1.5rem;
    font-size: 0.9rem;
    color: rgba(255, 255, 255, 0.8);
  }

  &__check {
    font-size: 1rem;
    min-width: 1rem;
  }

  &__chevron {
    font-size: 0.7rem;
    color: rgba(255,255,255,0.5);
  }

  &__color-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 1rem;
    background: rgba(255,255,255,0.04);
    border-top: 0.1rem solid rgba(255,255,255,0.08);
  }

  &__colorpicker {
    flex: 1;
    height: 2rem;
    border: none;
    background: none;
    cursor: pointer;
    padding: 0;
  }

  &__reset-color {
    background: transparent;
    border: 0.1rem solid var(--primary_color);
    color: var(--primary_color);
    border-radius: 0.3rem;
    padding: 0.1rem 0.4rem;
    cursor: pointer;
    font-size: 1rem;
    transition: background 0.15s;

    &:hover {
      background: var(--primary_color);
      color: $color-bg;
    }
  }
}

.desc-fade-enter-active,
.desc-fade-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}
.desc-fade-enter-from,
.desc-fade-leave-to {
  opacity: 0;
  transform: translateY(-50%) translateX(-4px);
}
</style>
