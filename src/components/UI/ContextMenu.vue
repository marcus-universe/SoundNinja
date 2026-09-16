<template>
  <Teleport to="body">
    <div
      v-if="appStore.contextMenu.visible"
      ref="menuEl"
      class="context-menu"
      :style="{ top: menuY + 'px', left: menuX + 'px' }"
      @click.stop
    >
      <ul class="context-menu__list" role="menu">
        <li
          v-if="isSound"
          class="context-menu__volume"
          @click.stop
          @mousedown.stop
          @mouseenter="hoveredItem = 'volume'"
          @mouseleave="hoveredItem = null"
        >
          <div class="context-menu__volume-head">
            <span class="context-menu__volume-label">{{ $t('contextMenu.volume') }}</span>
            <span class="context-menu__volume-pct">{{ volumePct }}%</span>
            <Transition name="desc-fade">
              <span v-if="hoveredItem === 'volume'" class="context-menu__desc">{{ $t('contextMenu.volumeDesc') }}</span>
            </Transition>
          </div>
          <input
            type="range"
            class="settings-slider"
            min="0"
            max="100"
            step="1"
            :value="volumePct"
            :aria-label="$t('contextMenu.volume')"
            @pointerdown="onVolumePointerDown"
            @input="onVolumeInput"
            @dblclick.prevent="resetVolume"
          />
        </li>
        <li
          v-if="isSound"
          class="context-menu__item"
          :class="{ 'context-menu__item--checked': soundLooping }"
          role="menuitemcheckbox"
          :aria-checked="soundLooping"
          @click="toggleSoundLoop"
          @mouseenter="hoveredItem = 'loop'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="loop" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.loop') }}</span>
          <span v-if="soundLooping" class="context-menu__check" aria-hidden="true">✓</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'loop'" class="context-menu__desc">{{ $t('contextMenu.loopDesc') }}</span>
          </Transition>
        </li>
        <li v-if="isSound" class="context-menu__sep" role="separator" />
        <li
          v-if="isTab || isSound"
          class="context-menu__item"
          role="menuitem"
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
          role="menuitem"
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
        <li v-if="isTab || isSound || isSeparator" class="context-menu__item context-menu__item--danger" role="menuitem" @click="remove" @mouseenter="hoveredItem = 'remove'" @mouseleave="hoveredItem = null">
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
        <li v-if="!isBoard" class="context-menu__sep" role="separator" />
        <li
          v-if="isSound"
          ref="moveToTabRowEl"
          class="context-menu__item"
          :class="{ 'context-menu__item--flyout-open': activeFlyout === 'tabs' }"
          role="menuitem"
          aria-haspopup="menu"
          :aria-expanded="activeFlyout === 'tabs'"
          @click="openFlyout('tabs', true)"
          @mouseenter="onFlyoutRowEnter('tabs', 'moveToTab')"
          @mouseleave="onFlyoutRowLeave"
        >
          <span class="context-menu__icon">
            <Icons icon="tab" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.moveToTab') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'moveToTab' && activeFlyout !== 'tabs'" class="context-menu__desc">{{ $t('contextMenu.moveToTabDesc') }}</span>
          </Transition>
          <span class="context-menu__chevron">{{ flyoutChevron('tabs') }}</span>
        </li>
        <li
          v-if="appStore.contextMenu.type === 'sound'"
          ref="tagsRowEl"
          class="context-menu__item"
          :class="{ 'context-menu__item--flyout-open': activeFlyout === 'tags' }"
          role="menuitem"
          aria-haspopup="menu"
          :aria-expanded="activeFlyout === 'tags'"
          @click="openFlyout('tags', true)"
          @mouseenter="onFlyoutRowEnter('tags', 'tags')"
          @mouseleave="onFlyoutRowLeave"
        >
          <span class="context-menu__icon">
            <Icons icon="filter" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.tags') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'tags' && activeFlyout !== 'tags'" class="context-menu__desc">{{ $t('contextMenu.tagsDesc') }}</span>
          </Transition>
          <span class="context-menu__chevron">{{ flyoutChevron('tags') }}</span>
        </li>
        <li
          v-if="isSound || isBoard"
          class="context-menu__item"
          role="menuitem"
          @click="addGroup"
          @mouseenter="hoveredItem = 'group'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="page-separator" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.addGroup') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'group'" class="context-menu__desc">{{
              isBoard ? $t('contextMenu.addGroupBoardDesc') : $t('contextMenu.addGroupDesc')
            }}</span>
          </Transition>
        </li>
        <li v-if="appStore.contextMenu.type === 'sound'" class="context-menu__sep" role="separator" />
        <li
          v-if="appStore.contextMenu.type === 'sound'"
          class="context-menu__item"
          role="menuitem"
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
          role="menuitem"
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
        <li v-if="isSound" class="context-menu__sep" role="separator" />
        <li
          v-if="isSound"
          class="context-menu__item"
          role="menuitem"
          @click="showInFolder"
          @mouseenter="hoveredItem = 'showInFolder'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="folder" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.showInFolder') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'showInFolder'" class="context-menu__desc">{{ $t('contextMenu.showInFolderDesc') }}</span>
          </Transition>
        </li>
        <li
          v-if="isSound"
          class="context-menu__item"
          role="menuitem"
          @click="copySoundPath"
          @mouseenter="hoveredItem = 'copyPath'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="rename" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.copyPath') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'copyPath'" class="context-menu__desc">{{ $t('contextMenu.copyPathDesc') }}</span>
          </Transition>
        </li>
        <li
          v-if="isSound"
          class="context-menu__item"
          role="menuitem"
          @click="replaceAudio"
          @mouseenter="hoveredItem = 'replaceAudio'"
          @mouseleave="hoveredItem = null"
        >
          <span class="context-menu__icon">
            <Icons icon="audio-file" custom-class="context-menu__icon-svg" />
          </span>
          <span class="context-menu__label">{{ $t('contextMenu.replaceAudio') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'replaceAudio'" class="context-menu__desc">{{ $t('contextMenu.replaceAudioDesc') }}</span>
          </Transition>
        </li>
        <li v-if="isSound" class="context-menu__sep" role="separator" />

        <!-- Tab button alignment -->
        <li
          v-if="appStore.contextMenu.type === 'tab'"
          class="context-menu__item"
          role="menuitem"
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
            role="menuitemradio"
            @click.stop="setTabAlign(opt.value)"
          >
            <span class="context-menu__check">{{ currentTabAlign === opt.value ? '☑' : '☐' }}</span>
            <span class="context-menu__label">{{ opt.label }}</span>
          </li>
        </template>
        <li v-if="appStore.contextMenu.type === 'tab'" class="context-menu__sep" role="separator" />

        <!-- Group alignment + colors -->
        <li
          v-if="appStore.contextMenu.type === 'separator'"
          class="context-menu__item"
          role="menuitem"
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
            role="menuitemradio"
            @click.stop="setGroupAlign(opt.value)"
          >
            <span class="context-menu__check">{{ currentGroupAlign === opt.value ? '☑' : '☐' }}</span>
            <span class="context-menu__label">{{ opt.label }}</span>
          </li>
        </template>
        <li v-if="appStore.contextMenu.type === 'separator'" class="context-menu__sep" role="separator" />

        <li
          v-if="appStore.contextMenu.type === 'separator'"
          class="context-menu__item context-menu__item--color"
          role="menuitem"
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
          <div class="context-menu__hue" :title="$t('contextMenu.colorHueHint')">
            <div class="context-menu__hue-head">
              <span>{{ $t('contextMenu.colorHue') }}</span>
              <span>{{ groupHue }}°</span>
            </div>
            <HueSlider
              :model-value="groupHue"
              :hint="$t('contextMenu.colorHueHint')"
              :aria-label="$t('contextMenu.colorHue')"
              @drag-start="beginGroupHue"
              @update:model-value="onGroupHue"
              @drag-end="endGroupHue"
            />
          </div>
          <label class="context-menu__color-row">
            <span>{{ $t('contextMenu.groupBgColor') }}</span>
            <input type="color" :value="activeGroup?.bgColor || groupFallback.bg" @input="onGroupBgColor" />
          </label>
          <label class="context-menu__color-row">
            <span>{{ $t('contextMenu.groupBorderColor') }}</span>
            <input type="color" :value="activeGroup?.borderColor || groupFallback.border" @input="onGroupBorderColor" />
          </label>
          <label class="context-menu__color-row">
            <span>{{ $t('contextMenu.groupNameColor') }}</span>
            <input type="color" :value="activeGroup?.nameColor || groupFallback.name" @input="onGroupNameColor" />
          </label>
          <button type="button" class="context-menu__reset-colors" @click="resetGroupColors">
            {{ $t('contextMenu.resetColor') }}
          </button>
        </li>

        <li
          v-if="isTab || isSound"
          ref="colorsRowEl"
          class="context-menu__item context-menu__item--color"
          :class="{ 'context-menu__item--flyout-open': activeFlyout === 'colors' }"
          role="menuitem"
          aria-haspopup="menu"
          :aria-expanded="activeFlyout === 'colors'"
          @click="openFlyout('colors', true)"
          @mouseenter="onFlyoutRowEnter('colors', 'color')"
          @mouseleave="onFlyoutRowLeave"
        >
          <span
            class="context-menu__swatch"
            :style="swatchStyle"
          />
          <span class="context-menu__label">{{ $t('contextMenu.colors') }}</span>
          <Transition name="desc-fade">
            <span v-if="hoveredItem === 'color' && activeFlyout !== 'colors'" class="context-menu__desc">{{ $t('contextMenu.colorsDesc') }}</span>
          </Transition>
          <span class="context-menu__chevron">{{ flyoutChevron('colors') }}</span>
        </li>
        <li
          v-if="appStore.contextMenu.type === 'sound'"
          class="context-menu__item"
          role="menuitem"
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
    <div
      v-if="appStore.contextMenu.visible && activeFlyout"
      ref="flyoutEl"
      class="context-menu__flyout"
      :class="{
        'context-menu__flyout--left': flyoutSide === 'left',
        'context-menu__flyout--ready': flyoutReady,
      }"
      :style="{ top: flyoutPos.y + 'px', left: flyoutPos.x + 'px' }"
      role="menu"
      @click.stop
      @contextmenu.prevent.stop
      @mouseenter="onFlyoutEnter"
      @mouseleave="onFlyoutLeave"
    >
      <div class="context-menu__flyout-inner">
        <ul v-if="activeFlyout === 'tabs'" class="context-menu__list">
          <li
            v-for="tab in allTabs"
            :key="tab.name"
            class="context-menu__item context-menu__tab-row"
            role="menuitemcheckbox"
            @click.stop="toggleSoundTab(tab.name)"
          >
            <span class="context-menu__check">{{ soundTabs.includes(tab.name) ? '☑' : '☐' }}</span>
            <span class="context-menu__label">{{ tab.name }}</span>
          </li>
        </ul>
        <ul v-else-if="activeFlyout === 'tags'" class="context-menu__list">
          <li
            v-for="tag in projectTags"
            :key="tag.id"
            class="context-menu__item context-menu__tab-row"
            role="menuitemcheckbox"
            @click.stop="toggleSoundTag(tag.id)"
          >
            <span class="context-menu__check">{{ soundTagIds.includes(tag.id) ? '☑' : '☐' }}</span>
            <span class="filter-panel__swatch" :style="{ background: tag.color }" />
            <span class="context-menu__label">{{ tag.name }}</span>
          </li>
          <li
            v-if="!projectTags.length"
            class="context-menu__item context-menu__tab-row"
            role="menuitem"
            @click.stop="openFilterForTags"
          >
            <span class="context-menu__label">{{ $t('contextMenu.manageTags') }}</span>
          </li>
        </ul>
        <div v-else-if="activeFlyout === 'colors'" class="context-menu__color-panel" @click.stop>
          <ColorGroupPicker
            :model-value="currentOverride"
            :base-colors="baseColors"
            inline
            @change="onOverrideChange"
          />
        </div>
      </div>
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
  themeButtonColors,
} from '~/utils/colorOverride'
import { copyText } from '~/utils/clipboard'
import { leadHue, shiftColorRecord } from '~/utils/hue'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { pickOpenPath } from '~/utils/projects'
import { invoke } from '@tauri-apps/api/core'

const { t: $t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const tabAlignOpen = ref(false)
const groupAlignOpen = ref(false)
const groupColorsOpen = ref(false)
const hoveredItem = ref(null)
const menuEl = ref(null)
const menuSize = ref({ w: 220, h: 180 })

const activeFlyout = ref(null)
const flyoutEl = ref(null)
const moveToTabRowEl = ref(null)
const tagsRowEl = ref(null)
const colorsRowEl = ref(null)
const flyoutPos = ref({ x: 0, y: 0 })
const flyoutSide = ref('right')
const flyoutReady = ref(false)

const FLYOUT_GAP = 4
const FLYOUT_PAD = 8
const FLYOUT_CLOSE_MS = 150
let flyoutCloseTimer = null
let ignoreRowLeaveUntil = 0

const menuType = computed(() => appStore.contextMenu.type)
const isTab = computed(() => menuType.value === 'tab')
const isSound = computed(() => menuType.value === 'sound')
const isSeparator = computed(() => menuType.value === 'separator')
const isBoard = computed(() => menuType.value === 'board')

const allTabs = computed(() => jsonStore.configFile.tabList)
const soundTabs = computed(() => {
  const { targetIndex } = appStore.contextMenu
  return jsonStore.configFile.files[targetIndex]?.tabs ?? []
})

const projectTags = computed(() => jsonStore.configFile.tags ?? [])
const soundTagIds = computed(() => {
  const { targetIndex } = appStore.contextMenu
  return jsonStore.configFile.files[targetIndex]?.tagIds ?? []
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

const volumePct = computed(() => {
  const sound = jsonStore.configFile.files[appStore.contextMenu.targetIndex]
  return Math.round((sound?.volume ?? 1) * 100)
})

const soundLooping = computed(() => {
  const sound = jsonStore.configFile.files[appStore.contextMenu.targetIndex]
  return !!sound?.looping
})

let volumeHistoryPushed = false

function onVolumePointerDown() {
  volumeHistoryPushed = false
}

function applySoundVolume(pct) {
  const { targetIndex } = appStore.contextMenu
  const sound = jsonStore.configFile.files[targetIndex]
  if (!sound) return
  const volume = Math.min(1, Math.max(0, pct / 100))
  jsonStore.setSoundVolume(targetIndex, volume, { history: !volumeHistoryPushed })
  volumeHistoryPushed = true
  invoke('set_sound_volume', { soundPath: sound.path, volume }).catch(() => {})
}

function onVolumeInput(e) {
  applySoundVolume(Number(e.target.value))
}

function resetVolume() {
  volumeHistoryPushed = false
  applySoundVolume(100)
}

function toggleSoundLoop() {
  const { targetIndex } = appStore.contextMenu
  const sound = jsonStore.configFile.files[targetIndex]
  if (!sound) return
  const next = !sound.looping
  jsonStore.setSoundLoop(targetIndex, next)
  if (sound.active) {
    invoke('set_playing_loop', { looping: next, soundPath: sound.path }).catch(() => {})
  }
}

function flyoutAnchor(kind) {
  if (kind === 'tabs') return moveToTabRowEl.value
  if (kind === 'tags') return tagsRowEl.value
  return colorsRowEl.value
}

function flyoutChevron(kind) {
  return flyoutSide.value === 'left' && activeFlyout.value === kind ? '‹' : '›'
}

function clearFlyoutClose() {
  if (flyoutCloseTimer != null) {
    clearTimeout(flyoutCloseTimer)
    flyoutCloseTimer = null
  }
}

function scheduleFlyoutClose() {
  clearFlyoutClose()
  flyoutCloseTimer = setTimeout(() => {
    activeFlyout.value = null
    flyoutReady.value = false
    flyoutCloseTimer = null
  }, FLYOUT_CLOSE_MS)
}

function openFlyout(kind, fromClick = false) {
  clearFlyoutClose()
  if (fromClick) ignoreRowLeaveUntil = Date.now() + 400
  if (activeFlyout.value !== kind) {
    flyoutReady.value = false
    activeFlyout.value = kind
  }
}

function onFlyoutRowEnter(kind, hoverKey) {
  hoveredItem.value = hoverKey
  openFlyout(kind)
}

function onFlyoutRowLeave() {
  hoveredItem.value = null
  if (Date.now() < ignoreRowLeaveUntil) return
  scheduleFlyoutClose()
}

function onFlyoutEnter() {
  clearFlyoutClose()
}

function onFlyoutLeave() {
  scheduleFlyoutClose()
}

function applyFlyoutPosition() {
  const flyout = flyoutEl.value
  const menu = menuEl.value
  const row = flyoutAnchor(activeFlyout.value)
  if (!flyout || !menu || !row || typeof window === 'undefined') return false

  const menuRect = menu.getBoundingClientRect()
  const rowRect = row.getBoundingClientRect()
  const flyoutWidth = flyout.offsetWidth
  const flyoutHeight = flyout.offsetHeight
  if (!flyoutWidth || !flyoutHeight) return false

  let left = menuRect.right + FLYOUT_GAP
  let side = 'right'
  if (left + flyoutWidth > window.innerWidth - FLYOUT_PAD) {
    left = Math.max(4, menuRect.left - flyoutWidth - FLYOUT_GAP)
    side = 'left'
  }

  let top = rowRect.top
  top = Math.min(top, window.innerHeight - flyoutHeight - FLYOUT_PAD)
  top = Math.max(4, top)

  flyoutPos.value = { x: left, y: top }
  flyoutSide.value = side
  flyoutReady.value = true
  return true
}

async function positionFlyout() {
  await nextTick()
  if (applyFlyoutPosition()) return
  requestAnimationFrame(() => applyFlyoutPosition())
}

function toggleSoundTag(tagId) {
  const sound = currentSound()
  if (!sound?.path) return
  jsonStore.toggleSoundTag(sound.path, tagId)
}

function openFilterForTags() {
  close()
  appStore.setFilterPanelOpen(true)
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

const groupFallback = computed(() => {
  const btn = themeButtonColors()
  return { bg: btn.bg, border: btn.border, name: btn.text }
})

const groupEffective = computed(() => ({
  bg: activeGroup.value?.bgColor || groupFallback.value.bg,
  border: activeGroup.value?.borderColor || groupFallback.value.border,
  name: activeGroup.value?.nameColor || groupFallback.value.name,
}))

const groupHueDragging = ref(false)
const groupHueDragValue = ref(0)
const groupHue = computed(() =>
  groupHueDragging.value
    ? groupHueDragValue.value
    : leadHue([groupEffective.value.border, groupEffective.value.bg, groupEffective.value.name]),
)

let groupHueSnap = null
let groupHueSnapHue = 0

function beginGroupHue() {
  groupHueSnap = { ...groupEffective.value }
  groupHueSnapHue = leadHue([groupHueSnap.border, groupHueSnap.bg, groupHueSnap.name])
  groupHueDragValue.value = groupHueSnapHue
  groupHueDragging.value = true
}

function endGroupHue() {
  groupHueSnap = null
  groupHueDragging.value = false
}

function onGroupHue(next) {
  if (!groupHueSnap) beginGroupHue()
  groupHueDragging.value = true
  groupHueDragValue.value = next
  const shifted = shiftColorRecord(groupHueSnap, next - groupHueSnapHue)
  jsonStore.updateSeparator(appStore.contextMenu.targetName, {
    bgColor: shifted.bg,
    borderColor: shifted.border,
    nameColor: shifted.name,
  })
}

function onGroupBgColor(e) {
  const id = appStore.contextMenu.targetName
  jsonStore.updateSeparator(id, { bgColor: e.target.value })
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
  jsonStore.updateSeparator(id, { borderColor: undefined, nameColor: undefined, bgColor: undefined })
}

function addGroup() {
  const { type, targetIndex } = appStore.contextMenu
  const tab = appStore.currentTab
  appStore.closeContextMenu()
  resetPanels()
  if (type === 'board') {
    const seps = (jsonStore.configFile.separators ?? []).filter((s) => s.tab === tab)
    const sounds = jsonStore.configFile.files.filter((f) => f.tabs.includes(tab))
    let max = -1
    for (const s of seps) {
      if (Number.isFinite(s.position)) max = Math.max(max, s.position)
    }
    for (const f of sounds) {
      const order = tab === 'All' ? (f.index ?? 0) : (f.tabIndexes?.[tab] ?? 0)
      if (Number.isFinite(order)) max = Math.max(max, order)
    }
    jsonStore.addSeparator(tab, max + 1)
    return
  }
  const sound = jsonStore.configFile.files[targetIndex]
  if (!sound) return
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
    tabAlignOpen.value,
    groupAlignOpen.value,
    groupColorsOpen.value,
  ],
  ([visible]) => {
    if (visible) {
      volumeHistoryPushed = false
      measureMenu()
    }
  },
  { flush: 'post' },
)

watch(activeFlyout, (kind) => {
  if (kind) positionFlyout()
}, { flush: 'post' })

watch(
  () => [
    appStore.contextMenu.x,
    appStore.contextMenu.y,
    appStore.contextMenu.type,
    appStore.contextMenu.targetName,
    appStore.contextMenu.targetIndex,
  ],
  () => {
    if (!appStore.contextMenu.visible) return
    activeFlyout.value = null
    flyoutReady.value = false
    clearFlyoutClose()
  },
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
  clearFlyoutClose()
  activeFlyout.value = null
  flyoutReady.value = false
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

async function showInFolder() {
  const sound = currentSound()
  close()
  if (!sound?.path) return
  try {
    await revealItemInDir(sound.path)
  } catch {
    appStore.setErrorActive($t('contextMenu.showInFolderFailed'))
  }
}

async function copySoundPath() {
  const sound = currentSound()
  close()
  if (sound?.path) await copyText(sound.path)
}

async function replaceAudio() {
  const { targetIndex } = appStore.contextMenu
  const sound = currentSound()
  close()
  if (!sound || typeof targetIndex !== 'number' || targetIndex < 0) return
  try {
    const selected = await openDialog({
      multiple: false,
      title: $t('contextMenu.replaceAudio'),
      filters: [{ name: $t('common.audioFiles'), extensions: ['mp3', 'wav', 'ogg'] }],
    })
    const nextPath = pickOpenPath(selected)
    if (!nextPath || nextPath === sound.path) return
    if (sound.id) {
      window.dispatchEvent(new CustomEvent('sn:stop-sound-id', { detail: sound.id }))
    }
    jsonStore.replaceSoundPath(targetIndex, nextPath)
    const selectedPaths = appStore.selectedSoundPaths
    const i = selectedPaths.indexOf(sound.path)
    if (i !== -1) selectedPaths.splice(i, 1, nextPath)
  } catch {
    appStore.setErrorActive($t('contextMenu.replaceAudioFailed'))
  }
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
  onUnmounted(() => {
    window.removeEventListener('keydown', handler)
    clearFlyoutClose()
  })
})
</script>
