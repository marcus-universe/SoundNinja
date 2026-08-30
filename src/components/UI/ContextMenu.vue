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
        <li
          v-if="appStore.contextMenu.type !== 'separator'"
          class="context-menu__item"
          @click="openRename"
          @mouseenter="hoveredItem = 'rename'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="rename" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.rename') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'rename'" class="context-menu__desc">{{ $t('contextMenu.renameDesc') }}</span>
          </Transition>
        </li>
        <li
          v-if="appStore.contextMenu.type === 'separator'"
          class="context-menu__item"
          @click="openRenameGroup"
          @mouseenter="hoveredItem = 'renameGroup'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="rename" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.renameGroup') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'renameGroup'" class="context-menu__desc">{{ $t('contextMenu.renameGroupDesc') }}</span>
          </Transition>
        </li>
        <li class="context-menu__item context-menu__item--danger" @click="remove" @mouseenter="hoveredItem = 'remove'" @mouseleave="hoveredItem = null">
          <span class="context-menu__icon">
            <Icons icon="delete" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{
            appStore.contextMenu.type === 'separator' ? $t('contextMenu.removeGroup') : $t('contextMenu.remove')
          }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'remove'" class="context-menu__desc">{{
              appStore.contextMenu.type === 'separator' ? $t('contextMenu.removeGroupDesc') : $t('contextMenu.removeDesc')
            }}</span>
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
          @click="addGroup"
          @mouseenter="hoveredItem = 'group'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="page-separator" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.addGroup') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'group'" class="context-menu__desc">{{ $t('contextMenu.addGroupDesc') }}</span>
          </Transition>
        </li>
        <li
          v-if="appStore.contextMenu.type === 'sound'"
          class="context-menu__item"
          @click="copySoundId"
          @mouseenter="hoveredItem = 'copyId'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="rename" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.copyId') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'copyId'" class="context-menu__desc">{{ $t('contextMenu.copyIdDesc') }}</span>
          </Transition>
        </li>
        <li
          v-if="appStore.contextMenu.type === 'sound'"
          class="context-menu__item"
          @click="assignHotkey"
          @mouseenter="hoveredItem = 'assignHotkey'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="settings" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.assignHotkey') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'assignHotkey'" class="context-menu__desc">{{ $t('contextMenu.assignHotkeyDesc') }}</span>
          </Transition>
        </li>

        <!-- Tab button alignment -->
        <li
          v-if="appStore.contextMenu.type === 'tab'"
          class="context-menu__item"
          @click="toggleTabAlign"
          @mouseenter="hoveredItem = 'tabAlign'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="tab" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.tabButtonAlign') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'tabAlign'" class="context-menu__desc">{{ $t('contextMenu.tabButtonAlignDesc') }}</span>
          </Transition>
          <span class="context-menu__chevron">{{ tabAlignOpen ? '▲' : '▼' }}</span>
        </li>
        <template v-if="tabAlignOpen && appStore.contextMenu.type === 'tab'">
          <li
            v-for="opt in alignOptions"
            :key="'tab-' + opt.value"
            class="context-menu__item context-menu__tab-row"
            @click.stop="setTabAlign(opt.value)"
          >
            <span class="context-menu__check">{{ currentTabAlign === opt.value ? '☑' : '☐' }}</span>
            {{ opt.label }}
          </li>
        </template>

        <!-- Group alignment + colors -->
        <li
          v-if="appStore.contextMenu.type === 'separator'"
          class="context-menu__item"
          @click="toggleGroupAlign"
          @mouseenter="hoveredItem = 'groupAlign'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="tab" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.groupAlign') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'groupAlign'" class="context-menu__desc">{{ $t('contextMenu.groupAlignDesc') }}</span>
          </Transition>
          <span class="context-menu__chevron">{{ groupAlignOpen ? '▲' : '▼' }}</span>
        </li>
        <template v-if="groupAlignOpen && appStore.contextMenu.type === 'separator'">
          <li
            v-for="opt in groupAlignOptions"
            :key="'g-' + String(opt.value)"
            class="context-menu__item context-menu__tab-row"
            @click.stop="setGroupAlign(opt.value)"
          >
            <span class="context-menu__check">{{ currentGroupAlign === opt.value ? '☑' : '☐' }}</span>
            {{ opt.label }}
          </li>
        </template>

        <li
          v-if="appStore.contextMenu.type === 'separator'"
          class="context-menu__item context-menu__item--color"
          @click="toggleGroupColors"
          @mouseenter="hoveredItem = 'groupColors'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__swatch" :style="{ background: activeGroup?.borderColor || 'var(--primary_color)' }" />
          <span class="context-menu__label">{{ $t('contextMenu.groupColors') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'groupColors'" class="context-menu__desc">{{ $t('contextMenu.groupColorsDesc') }}</span>
          </Transition>
          <span class="context-menu__chevron">{{ groupColorsOpen ? '▲' : '▼' }}</span>
        </li>
        <li v-if="groupColorsOpen && appStore.contextMenu.type === 'separator'" class="context-menu__color-panel" @click.stop>
          <label class="context-menu__color-row">
            <span>{{ $t('contextMenu.groupBorderColor') }}</span>
            <input type="color" :value="activeGroup?.borderColor || '#888888'" @input="onGroupBorderColor" />
          </label>
          <label class="context-menu__color-row">
            <span>{{ $t('contextMenu.groupNameColor') }}</span>
            <input type="color" :value="activeGroup?.nameColor || '#ffffff'" @input="onGroupNameColor" />
          </label>
          <button type="button" class="context-menu__reset-colors" @click="resetGroupColors">
            {{ $t('contextMenu.resetColor') }}
          </button>
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
import { copyText } from '~/utils/clipboard'

const { t: $t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const colorPickerOpen = ref(false)
const moveToTabOpen = ref(false)
const tabAlignOpen = ref(false)
const groupAlignOpen = ref(false)
const groupColorsOpen = ref(false)
const hoveredItem = ref(null)
const menuEl = ref(null)
const menuSize = ref({ w: 220, h: 180 })

const allTabs = computed(() => jsonStore.configFile.tabList)
const soundTabs = computed(() => {
  const { targetIndex } = appStore.contextMenu
  return jsonStore.configFile.files[targetIndex]?.tabs ?? []
})

const activeGroup = computed(() => {
  if (appStore.contextMenu.type !== 'separator') return null
  return (jsonStore.configFile.separators ?? []).find((s) => s.id === appStore.contextMenu.targetName) ?? null
})

const alignOptions = computed(() => [
  { value: 'left', label: $t('contextMenu.alignLeft') },
  { value: 'center', label: $t('contextMenu.alignCenter') },
  { value: 'right', label: $t('contextMenu.alignRight') },
])

const groupAlignOptions = computed(() => [
  { value: undefined, label: $t('contextMenu.alignInherit') },
  ...alignOptions.value,
])

const currentTabAlign = computed(() => {
  const tab = jsonStore.configFile.tabList.find((t) => t.name === appStore.contextMenu.targetName)
  return tab?.buttonAlign ?? 'left'
})

const currentGroupAlign = computed(() => activeGroup.value?.buttonAlign)

function toggleMoveToTab() {
  moveToTabOpen.value = !moveToTabOpen.value
}

function toggleTabAlign() {
  tabAlignOpen.value = !tabAlignOpen.value
}

function toggleGroupAlign() {
  groupAlignOpen.value = !groupAlignOpen.value
}

function toggleGroupColors() {
  groupColorsOpen.value = !groupColorsOpen.value
}

function setTabAlign(align) {
  jsonStore.setTabButtonAlign(appStore.contextMenu.targetName, align)
}

function setGroupAlign(align) {
  const id = appStore.contextMenu.targetName
  jsonStore.updateSeparator(id, { buttonAlign: align })
}

function onGroupBorderColor(e) {
  const id = appStore.contextMenu.targetName
  jsonStore.updateSeparator(id, { borderColor: e.target.value })
}

function onGroupNameColor(e) {
  const id = appStore.contextMenu.targetName
  jsonStore.updateSeparator(id, { nameColor: e.target.value })
}

function resetGroupColors() {
  const id = appStore.contextMenu.targetName
  jsonStore.updateSeparator(id, { borderColor: undefined, nameColor: undefined })
}

function addGroup() {
  const { targetIndex } = appStore.contextMenu
  const sound = jsonStore.configFile.files[targetIndex]
  appStore.closeContextMenu()
  resetPanels()
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
    tabAlignOpen.value,
    groupAlignOpen.value,
    groupColorsOpen.value,
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
  const { type } = appStore.contextMenu
  appStore.closeContextMenu()
  resetPanels()
  if (type === 'tab') {
    appStore.setPopupActive({ active: true, type: 'renameTab' })
  } else if (type === 'sound') {
    appStore.setPopupActive({ active: true, type: 'renameSound' })
  }
}

function openRenameGroup() {
  // targetName stays as group id (closeContextMenu only hides the menu).
  appStore.closeContextMenu()
  resetPanels()
  appStore.setPopupActive({ active: true, type: 'renameGroup' })
}

function remove() {
  const { type, targetName, targetIndex } = appStore.contextMenu
  appStore.closeContextMenu()
  resetPanels()
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

function resetPanels() {
  colorPickerOpen.value = false
  moveToTabOpen.value = false
  tabAlignOpen.value = false
  groupAlignOpen.value = false
  groupColorsOpen.value = false
}

function close() {
  appStore.closeContextMenu()
  resetPanels()
}

function currentSound() {
  const { targetIndex } = appStore.contextMenu
  return jsonStore.configFile.files[targetIndex] ?? null
}

async function copySoundId() {
  const sound = currentSound()
  close()
  if (sound?.id) await copyText(sound.id)
}

function assignHotkey() {
  const sound = currentSound()
  close()
  if (!sound?.id) return
  appStore.pendingHotkeySoundId = sound.id
  appStore.openSettingsTab('hotkeys')
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
