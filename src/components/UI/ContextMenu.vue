<template>
  <Teleport to="body">
    <div
      v-if="appStore.contextMenu.visible"
      class="context-menu"
      :style="{ top: menuY + 'px', left: menuX + 'px' }"
      @click.stop
    >
      <ul class="context-menu__list">
        <li class="context-menu__item" @click="openRename">
          <span class="context-menu__icon">✏️</span> Rename
        </li>
        <li class="context-menu__item context-menu__item--danger" @click="remove">
          <span class="context-menu__icon">🗑️</span> Remove
        </li>
        <li class="context-menu__item context-menu__item--color" @click="toggleColorPicker">
          <span
            class="context-menu__swatch"
            :style="swatchStyle"
          />
          Accent Color
          <span class="context-menu__chevron">{{ colorPickerOpen ? '▲' : '▼' }}</span>
        </li>
        <li v-if="colorPickerOpen" class="context-menu__color-row">
          <input
            type="color"
            class="context-menu__colorpicker"
            :value="currentColor"
            @input="onColorInput"
          />
          <button class="context-menu__reset-color" @click="resetColor" title="Reset to default">↺</button>
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
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const colorPickerOpen = ref(false)

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
  if (type === 'tab') {
    jsonStore.removeTab(targetName)
    if (appStore.currentTab === targetName) {
      appStore.setCurrentTab('All')
    }
  } else if (type === 'sound') {
    jsonStore.removeSound(targetIndex)
  }
}

function close() {
  appStore.closeContextMenu()
  colorPickerOpen.value = false
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
  overflow: hidden;

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
</style>
