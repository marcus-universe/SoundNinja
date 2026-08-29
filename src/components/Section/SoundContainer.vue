<template>
    <div
        class="SoundContainer"
        :class="{
          'SoundContainer--player-large': showPlayer && playerLarge,
          'SoundContainer--bulk': appStore.multiSelectActive,
        }"
    >
        <div class="SoundContainer__scroll">
        <Transition
            :name="tabTransitionName"
            :mode="tabTransitionMode"
            :css="tabTransition !== 'none'"
            :duration="tabTransitionDuration"
        >
        <div :key="'tab:' + currentTab" class="SoundTab-wrap">
        <div
            class="SoundTab flex_c_v flex_start button-gaps"
            ref="boardRef"
            :style="uniformHeight ? { '--btn-min-height': uniformHeight + 'px' } : {}"
        >
            <!-- Orphan strip: sounds before first group -->
            <div
                class="sound-orphans sound-section-body flex_c_h flex_start button-gaps flex_wrap"
                :class="orphanAlignClass"
                data-section-id="orphans"
                ref="orphansRef"
            >
                <SoundButton
                    v-for="sound in orphanSounds"
                    :key="'s:' + sound.path"
                    :sound="sound"
                    :btnStyle="getBtnStyle(sound)"
                    :loading="loadingPaths.has(sound.path)"
                    :selected="appStore.multiSelectActive && appStore.selectedSoundPaths.includes(sound.path)"
                    :gifSrc="gifSrcFor(sound)"
                    :gifPosX="sound.gifPosX ?? 50"
                    :gifPosY="sound.gifPosY ?? 50"
                    :hasGif="!!sound.gifId"
                    :progressPaused="!!playingSounds.get(sound.index)?.paused || !windowFocused"
                    :missing="!!jsonStore.missingPaths?.includes(sound.path)"
                    :data-sound-path="sound.path"
                    @play="onSoundClick(sound)"
                    @contextmenu="(e) => { e.stopPropagation(); openSoundMenu(e, sound) }"
                    @gifhover="(on) => onGifHover(sound, on)"
                />
            </div>

            <!-- Group cards -->
            <div class="sound-groups-outer flex_c_v flex_start button-gaps" ref="groupsOuterRef">
                <div
                    v-for="sec in groupSections"
                    :key="'g:' + sec.sep.id"
                    class="sound-group tab-separator"
                    :data-sep-id="sec.sep.id"
                    :style="groupCardStyle(sec.sep)"
                    @contextmenu.prevent="(e) => openSeparatorMenu(e, sec.sep)"
                >
                    <div class="sound-group__name" :style="groupNameStyle(sec.sep)">
                        {{ sec.sep.name?.trim() || $t('contextMenu.untitledGroup') }}
                    </div>
                    <div
                        class="sound-group__body sound-section-body flex_c_h flex_start button-gaps flex_wrap"
                        :class="alignClassFor(sec.sep)"
                        :data-section-id="sec.sep.id"
                    >
                        <SoundButton
                            v-for="sound in sec.sounds"
                            :key="'s:' + sound.path"
                            :sound="sound"
                            :btnStyle="getBtnStyle(sound)"
                            :loading="loadingPaths.has(sound.path)"
                            :selected="appStore.multiSelectActive && appStore.selectedSoundPaths.includes(sound.path)"
                            :gifSrc="gifSrcFor(sound)"
                            :gifPosX="sound.gifPosX ?? 50"
                            :gifPosY="sound.gifPosY ?? 50"
                            :hasGif="!!sound.gifId"
                            :progressPaused="!!playingSounds.get(sound.index)?.paused || !windowFocused"
                            :missing="!!jsonStore.missingPaths?.includes(sound.path)"
                            :data-sound-path="sound.path"
                            @play="onSoundClick(sound)"
                            @contextmenu="(e) => { e.stopPropagation(); openSoundMenu(e, sound) }"
                            @gifhover="(on) => onGifHover(sound, on)"
                        />
                    </div>
                </div>
            </div>
        </div>
        </div>
        </Transition>
        </div>

        <Transition name="fade">
            <div v-if="appStore.multiSelectActive" class="bulk-bar flex_c_h align_c">
                <span class="bulk-bar__count">{{ $t('bulk.selected', { count: appStore.selectedSoundPaths.length }) }}</span>
                <div class="bulk-bar__color">
                    <ColorGroupPicker
                        :model-value="bulkOverride"
                        :base-colors="bulkBaseColors"
                        :title="$t('bulk.color')"
                        placement="bottom-right"
                        @change="onBulkOverride"
                    />
                </div>
                <select class="bulk-bar__select" v-model="bulkTab" @change="applyBulkTab">
                    <option value="">{{ $t('bulk.moveToTab') }}</option>
                    <option v-for="t in tabOptions" :key="t" :value="t">{{ t }}</option>
                </select>
                <button class="bulk-bar__btn bulk-bar__btn--danger" :disabled="appStore.selectedSoundPaths.length === 0" @click="applyBulkDelete">
                    {{ $t('bulk.delete') }}
                </button>
                <button class="bulk-bar__btn bulk-bar__btn--primary" @click="appStore.setMultiSelectActive(false)">{{ $t('bulk.done') }}</button>
            </div>
        </Transition>

        <PlayerBar v-if="showPlayer" />
    </div>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import Sortable from 'sortablejs'
import { parseOverride, serializeOverride, resolveEffectiveColors } from '~/utils/colorOverride'
import { withProjectDb, loadGifBlobsByIds, normalizeTabTransition } from '~/utils/db'
import { cacheGifRow, peekGifUrls, revokeGifUrls } from '~/utils/gifCache'

const MAX_ANIM_GIFS = 8

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const boardRef = ref(null)
const orphansRef = ref(null)
const groupsOuterRef = ref(null)
let outerSortable = null
/** @type {import('sortablejs').default[]} */
const innerSortables = []

const showPlayer = computed(() => jsonStore.configFile?.settings?.showPlayer !== false)
const playerLarge = computed(() => jsonStore.configFile?.settings?.playerLarge === true)

const loadingPaths = reactive(new Set())
const uniformHeight = ref(0)
const bulkOverride = ref({})
const bulkTab = ref('')
const bulkBaseColors = computed(() => {
  void appStore.multiSelectActive
  const s = jsonStore.configFile?.settings
  void s?.theme
  void s?.btnBg
  void s?.btnBorder
  void s?.primaryColor
  return resolveEffectiveColors(bulkOverride.value || {}, 'button')
})

const currentTab = computed(() => appStore.currentTab)
const playingSounds = reactive(new Map())
const windowFocused = ref(true)

const JSONFile = computed(() => {
  const tab = currentTab.value
  const sortByIndex =
    tab === 'All'
      ? (a, b) => a.index - b.index
      : (a, b) => (a.tabIndexes?.[tab] ?? 0) - (b.tabIndexes?.[tab] ?? 0)
  const filesToFilter = appStore.Searchbar.SearchbarActive
    ? jsonStore.filteredFiles
    : jsonStore.configFile?.files

  return filesToFilter
    ?.filter((sound) => sound.tabs?.includes(tab))
    .sort(sortByIndex)
})

const Settings = computed(() => jsonStore.configFile?.settings)

const tabTransition = computed(() => normalizeTabTransition(Settings.value?.tabTransition))
const slideDir = ref('left')
const tabOrderNames = computed(() => [
  'All',
  ...(jsonStore.configFile?.tabList ?? []).map((t) => t.name),
])

watch(currentTab, (next, prev) => {
  const order = tabOrderNames.value
  const a = order.indexOf(prev)
  const b = order.indexOf(next)
  if (a < 0 || b < 0 || a === b) return
  slideDir.value = b > a ? 'left' : 'right'
})

const tabTransitionName = computed(() => {
  const t = tabTransition.value
  if (t === 'none') return ''
  if (t === 'slide') return slideDir.value === 'left' ? 'tab-slide-left' : 'tab-slide-right'
  if (t === 'fade') return 'tab-fade'
  return 'tab-stagger'
})

const tabTransitionMode = computed(() => {
  const t = tabTransition.value
  return t === 'fade' || t === 'stagger' ? 'out-in' : undefined
})

const tabTransitionDuration = computed(() => {
  if (tabTransition.value === 'stagger') return { enter: 380, leave: 380 }
  return undefined
})

const tabEntry = computed(() =>
  (jsonStore.configFile?.tabList ?? []).find((t) => t.name === currentTab.value),
)

const tabButtonAlign = computed(() => tabEntry.value?.buttonAlign ?? 'left')

function resolveAlign(sep) {
  return sep?.buttonAlign ?? tabButtonAlign.value
}

function alignClassFor(sep) {
  const a = resolveAlign(sep)
  return {
    'sound-align--left': a === 'left',
    'sound-align--center': a === 'center',
    'sound-align--right': a === 'right',
  }
}

const orphanAlignClass = computed(() => alignClassFor(null))

function groupCardStyle(sep) {
  const style = {}
  if (sep.borderColor) style['--group-border'] = sep.borderColor
  return style
}

function groupNameStyle(sep) {
  if (sep.nameColor) return { color: sep.nameColor }
  return {}
}

function tabOrder(sound) {
  return currentTab.value === 'All' ? sound.index : (sound.tabIndexes?.[currentTab.value] ?? 0)
}

/** Orphans + group sections for the current tab, positional membership. */
const displaySections = computed(() => {
  const sounds = JSONFile.value ?? []
  const seps = (jsonStore.separators ?? [])
    .filter((s) => s.tab === currentTab.value)
    .slice()
    .sort((a, b) => a.position - b.position)

  if (seps.length === 0) {
    return [{ kind: 'orphans', sounds }]
  }

  const orphans = []
  const groups = seps.map((sep) => ({ kind: 'group', sep, sounds: [] }))

  for (const sound of sounds) {
    const order = tabOrder(sound)
    let placed = false
    for (let i = 0; i < seps.length; i++) {
      const start = seps[i].position
      const end = i + 1 < seps.length ? seps[i + 1].position : Number.POSITIVE_INFINITY
      if (order >= start && order < end) {
        groups[i].sounds.push(sound)
        placed = true
        break
      }
    }
    if (!placed) orphans.push(sound)
  }

  return [{ kind: 'orphans', sounds: orphans }, ...groups]
})

const orphanSounds = computed(() => displaySections.value.find((s) => s.kind === 'orphans')?.sounds ?? [])
const groupSections = computed(() => displaySections.value.filter((s) => s.kind === 'group'))

const staggerIndexByKey = computed(() => {
  const map = {}
  let i = 0
  for (const s of orphanSounds.value) map[s.path] = i++
  for (const sec of groupSections.value) {
    for (const s of sec.sounds) map[s.path] = i++
  }
  return map
})

const allDisplaySounds = computed(() => {
  const out = []
  for (const sec of displaySections.value) {
    for (const s of sec.sounds) out.push(s)
  }
  return out
})

function reorderDisabled() {
  return jsonStore.configFile?.settings?.allowReorder === false
}

function destroySortables() {
  outerSortable?.destroy()
  outerSortable = null
  while (innerSortables.length) {
    innerSortables.pop()?.destroy()
  }
}

function revertDom(evt) {
  const { oldIndex, item, from } = evt
  if (oldIndex == null) return
  from.removeChild(item)
  from.insertBefore(item, from.children[oldIndex] ?? null)
}

function layoutFromDom() {
  const orphans = [...(orphansRef.value?.querySelectorAll('[data-sound-path]') ?? [])]
    .map((el) => el.getAttribute('data-sound-path'))
    .filter(Boolean)
  const groups = []
  for (const card of groupsOuterRef.value?.children ?? []) {
    const id = card.getAttribute('data-sep-id')
    if (!id) continue
    const body = card.querySelector('.sound-group__body')
    const paths = [...(body?.querySelectorAll('[data-sound-path]') ?? [])]
      .map((el) => el.getAttribute('data-sound-path'))
      .filter(Boolean)
    groups.push({ id, paths })
  }
  return { orphans, groups }
}

function onOuterEnd(evt) {
  const { oldIndex, newIndex } = evt
  if (oldIndex === newIndex || oldIndex == null || newIndex == null) return
  revertDom(evt)
  const ids = groupSections.value.map((s) => s.sep.id)
  const arr = ids.slice()
  const [moved] = arr.splice(oldIndex, 1)
  arr.splice(newIndex, 0, moved)
  jsonStore.moveGroupWithMembers(currentTab.value, arr)
}

function onInnerEnd(evt) {
  const { oldIndex, newIndex, from, to } = evt
  if (from === to && (oldIndex === newIndex || oldIndex == null || newIndex == null)) return
  // Read target layout from DOM *before* revert (Sortable already moved the node).
  const layout = layoutFromDom()
  revertDom(evt)
  jsonStore.applyBoardLayout(currentTab.value, layout)
}

function bindInnerSortable(el) {
  if (!el) return
  const s = Sortable.create(el, {
    group: { name: 'sounds', pull: true, put: true },
    animation: 180,
    disabled: reorderDisabled(),
    draggable: '.Soundbtn',
    ghostClass: 'drag-over',
    onEnd: onInnerEnd,
  })
  innerSortables.push(s)
}

function setupSortables() {
  destroySortables()
  if (!groupsOuterRef.value || !orphansRef.value) return

  outerSortable = Sortable.create(groupsOuterRef.value, {
    animation: 180,
    disabled: reorderDisabled(),
    draggable: '.sound-group',
    handle: '.sound-group__name',
    ghostClass: 'drag-over',
    onEnd: onOuterEnd,
  })

  bindInnerSortable(orphansRef.value)
  groupsOuterRef.value.querySelectorAll('.sound-group__body').forEach((el) => {
    bindInnerSortable(el)
  })
}

onMounted(() => {
  nextTick(() => setupSortables())
})

onUnmounted(() => {
  destroySortables()
  gifObserver?.disconnect()
  gifObserver = null
})

const gifPlayOnHover = computed(() => Settings.value?.gifPlayOnHover !== false)
const visibleByPath = reactive({})
const hoveredPath = ref(null)
const gifUrls = reactive({})
let gifObserver = null

function onGifHover(sound, on) {
  if (on) hoveredPath.value = sound.path
  else if (hoveredPath.value === sound.path) hoveredPath.value = null
}

function shouldAnimate(sound) {
  if (!sound.gifId) return false
  if (!windowFocused.value) return false
  if (!visibleByPath[sound.path]) return false
  if (gifPlayOnHover.value) return hoveredPath.value === sound.path
  return true
}

const animatingPaths = computed(() => {
  const paths = []
  for (const sound of allDisplaySounds.value) {
    if (!shouldAnimate(sound)) continue
    paths.push(sound.path)
    if (paths.length >= MAX_ANIM_GIFS) break
  }
  return new Set(paths)
})

function gifSrcFor(sound) {
  if (!sound.gifId) return ''
  const urls = gifUrls[sound.gifId] || peekGifUrls(sound.gifId)
  if (!urls) return ''
  if (animatingPaths.value.has(sound.path)) return urls.animUrl
  return urls.posterUrl
}

async function loadVisibleGifs() {
  const path = jsonStore.currentProjectPath
  if (!path) return
  const ids = []
  for (const sound of allDisplaySounds.value) {
    if (!sound.gifId) continue
    if (!visibleByPath[sound.path]) continue
    if (gifUrls[sound.gifId] || peekGifUrls(sound.gifId)) continue
    ids.push(sound.gifId)
  }
  if (!ids.length) return
  try {
    const rows = await withProjectDb(path, (d) => loadGifBlobsByIds(d, ids))
    for (const row of rows) {
      gifUrls[row.id] = cacheGifRow(row)
    }
  } catch (e) {
    console.error('Failed to load GIF blob', e)
  }
}

function releaseOffscreenGif(soundPath) {
  const file = jsonStore.configFile?.files?.find((f) => f.path === soundPath)
  const id = file?.gifId
  if (!id) return
  const stillVisible = jsonStore.configFile.files.some(
    (f) => f.gifId === id && f.path !== soundPath && visibleByPath[f.path],
  )
  if (stillVisible) return
  revokeGifUrls(id)
  delete gifUrls[id]
}

function setupGifObserver() {
  gifObserver?.disconnect()
  const root = boardRef.value?.closest('.SoundContainer__scroll') || null
  gifObserver = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        const path = e.target.getAttribute('data-sound-path')
        if (!path) continue
        if (e.isIntersecting) visibleByPath[path] = true
        else {
          delete visibleByPath[path]
          releaseOffscreenGif(path)
        }
      }
      loadVisibleGifs()
    },
    { root, rootMargin: '80px', threshold: 0.01 },
  )
  nextTick(() => {
    boardRef.value?.querySelectorAll('.Soundbtn').forEach((el) => gifObserver.observe(el))
  })
}

watch(
  () => Settings.value?.allowReorder,
  (v) => {
    const disabled = v === false
    outerSortable?.option('disabled', disabled)
    for (const s of innerSortables) s.option('disabled', disabled)
  },
)

function updateUniformHeight() {
  if (!Settings.value?.uniformButtonHeight) {
    uniformHeight.value = 0
    return
  }
  uniformHeight.value = 0
  nextTick(() => {
    const el = boardRef.value
    if (!el) return
    let max = 0
    el.querySelectorAll('.Soundbtn').forEach((b) => {
      max = Math.max(max, b.offsetHeight)
    })
    uniformHeight.value = max
  })
}

const tabOptions = computed(() => (jsonStore.configFile?.tabList ?? []).map((t) => t.name).filter(Boolean))

const boardSignature = computed(() => {
  const parts = []
  for (const sec of displaySections.value) {
    if (sec.kind === 'orphans') {
      parts.push('o:' + sec.sounds.map((s) => s.path).join(','))
    } else {
      parts.push('g:' + sec.sep.id + ':' + sec.sounds.map((s) => s.path).join(','))
    }
  }
  return parts.join('|')
})

watch(
  [
    boardSignature,
    () => jsonStore.configFile.files.map((f) => f.gifId || '').join('|'),
  ],
  () => {
    for (const f of jsonStore.configFile.files) {
      if (!f.gifId || gifUrls[f.gifId]) continue
      const urls = peekGifUrls(f.gifId)
      if (urls) gifUrls[f.gifId] = urls
    }
    nextTick(() => {
      setupSortables()
      setupGifObserver()
      loadVisibleGifs()
    })
  },
  { flush: 'post' },
)

watch(currentTab, () => {
  destroySortables()
  gifObserver?.disconnect()
  gifObserver = null
  for (const k of Object.keys(visibleByPath)) delete visibleByPath[k]
  nextTick(() => {
    setupSortables()
    setupGifObserver()
    loadVisibleGifs()
  })
})

watch(
  [() => Settings.value?.uniformButtonHeight, () => allDisplaySounds.value.length],
  () => updateUniformHeight(),
  { flush: 'post' },
)

function openSeparatorMenu(event, sep) {
  appStore.openContextMenu({
    x: event.clientX,
    y: event.clientY,
    type: 'separator',
    targetName: sep.id,
    targetIndex: -1,
  })
}

function getBtnStyle(sound) {
  const style = {}
  const si = staggerIndexByKey.value[sound.path]
  if (si != null) style['--stagger-i'] = si
  const info = playingSounds.get(sound.index)
  if (sound.active && info && info.duration > 0) {
    const elapsed = info.paused
      ? (info.elapsedMs || 0) / 1000
      : Math.min(info.duration, (Date.now() - info.startTime) / 1000)
    const from = Math.min(100, (elapsed / info.duration) * 100)
    style['--sound-progress-from'] = from + '%'
    if (info.paused || !windowFocused.value) {
      style['--sound-progress'] = from + '%'
      style['--sound-progress-dur'] = '0s'
    } else {
      style['--sound-progress'] = '100%'
      style['--sound-progress-dur'] = Math.max(0, info.duration - elapsed) + 's'
    }
  }
  const o = parseOverride(sound.color)
  if (o.bg) style['--color-btn'] = o.bg
  if (o.bgHover) style['--btn-bg-hover'] = o.bgHover
  if (o.text) style['--sound-text'] = o.text
  if (o.textHover) style['--btn-text-hover'] = o.textHover
  if (o.border) style['--btn-border'] = o.border
  if (o.borderHover) style['--btn-border-hover'] = o.borderHover
  return style
}

function openSoundMenu(event, sound) {
  const fileArrayIndex = jsonStore.configFile.files.indexOf(sound)
  appStore.openContextMenu({
    x: event.clientX,
    y: event.clientY,
    type: 'sound',
    targetName: sound.name,
    targetIndex: fileArrayIndex,
  })
}

function startProgress(soundFileIndex, duration) {
  playingSounds.set(soundFileIndex, {
    duration,
    startTime: Date.now(),
    paused: false,
    elapsedMs: 0,
  })
}

function stopProgress(soundFileIndex) {
  playingSounds.delete(soundFileIndex)
}

function stopAllProgress() {
  playingSounds.clear()
}

function syncProgressPauseState(list) {
  const files = jsonStore.configFile?.files || []
  const byPath = new Map((list || []).map((p) => [p.path, p]))

  for (const [fileIndex, info] of playingSounds) {
    const file = files.find((f) => f.index === fileIndex)
    if (!file || !byPath.has(file.path)) continue
    const snap = byPath.get(file.path)
    const shouldPause = !!snap.paused
    const posSec = Number(snap.positionSecs)
    if (Number.isFinite(posSec) && info.duration > 0) {
      info.elapsedMs = Math.max(0, posSec * 1000)
      info.startTime = Date.now() - info.elapsedMs
    }

    if (shouldPause && !info.paused) {
      if (!Number.isFinite(posSec)) {
        info.elapsedMs = Math.max(0, Date.now() - info.startTime)
      }
      info.paused = true
    } else if (!shouldPause && info.paused) {
      info.startTime = Date.now() - (info.elapsedMs || 0)
      info.paused = false
    } else if (!shouldPause) {
      info.paused = false
    }
  }
}

let unlistenFinished = null
let unlistenPlaying = null

function syncWindowFocus() {
  windowFocused.value = !document.hidden && document.hasFocus()
}

onMounted(async () => {
  setupGifObserver()
  document.addEventListener('visibilitychange', syncWindowFocus)
  window.addEventListener('focus', syncWindowFocus)
  window.addEventListener('blur', syncWindowFocus)
  unlistenFinished = await listen('sound_finished', (event) => {
    const path = event.payload
    const idx = jsonStore.configFile.files.findIndex((f) => f.path === path)
    if (idx !== -1) {
      const fileIndex = jsonStore.configFile.files[idx].index
      stopProgress(fileIndex)
      jsonStore.setActiveSound({ soundindex: idx, status: false })
    } else {
      stopAllProgress()
      jsonStore.ReturnStatusAll()
    }
  })
  unlistenPlaying = await listen('playing_changed', (event) => {
    syncProgressPauseState(event.payload ?? [])
  })
})

onUnmounted(() => {
  stopAllProgress()
  document.removeEventListener('visibilitychange', syncWindowFocus)
  window.removeEventListener('focus', syncWindowFocus)
  window.removeEventListener('blur', syncWindowFocus)
  if (unlistenFinished) unlistenFinished()
  if (unlistenPlaying) unlistenPlaying()
})

function onSoundClick(sound) {
  if (appStore.multiSelectActive) {
    appStore.toggleSoundSelection(sound.path)
    return
  }
  if (jsonStore.missingPaths.includes(sound.path)) {
    appStore.setRelinkActive(true)
    return
  }
  setActiveSound(sound)
}

function onBulkOverride(override) {
  bulkOverride.value = override
  const paths = appStore.selectedSoundPaths
  if (paths.length) jsonStore.setSoundColorMany(paths, serializeOverride(override))
}

function applyBulkTab() {
  const tab = bulkTab.value
  const paths = appStore.selectedSoundPaths
  if (tab && paths.length) jsonStore.setSoundTabsMany(paths, tab)
  bulkTab.value = ''
}

function applyBulkDelete() {
  const paths = appStore.selectedSoundPaths
  if (!paths.length) return
  jsonStore.removeSoundsMany(paths)
  appStore.clearSoundSelection()
}

async function setActiveSound(sound) {
  const fileArrayIndex = jsonStore.configFile.files.indexOf(sound)
  const overlapSounds = Settings.value.overlapSounds ?? false
  const stopOnRetrigger = Settings.value.stopOnRetrigger ?? true

  if (!sound.active) {
    if (!overlapSounds) {
      jsonStore.ReturnStatusAll()
      stopAllProgress()
    }
    jsonStore.setActiveSound({ soundindex: fileArrayIndex, status: true })

    loadingPaths.add(sound.path)
    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: appSettings.outputSource,
      hostName: appSettings.outputHost || null,
      active: false,
      overlap: overlapSounds,
    })
      .then((duration) => {
        if (!sound.active) return
        if (typeof duration === 'number' && duration > 0) {
          startProgress(sound.index, duration)
        }
      })
      .catch((e) => console.error('Sound playback error', e))
      .finally(() => loadingPaths.delete(sound.path))
  } else {
    if (!stopOnRetrigger) return
    stopProgress(sound.index)
    loadingPaths.delete(sound.path)
    jsonStore.setActiveSound({ soundindex: fileArrayIndex, status: false })
    invoke('play_sound', {
      soundPath: sound.path,
      deviceName: appSettings.outputSource,
      hostName: appSettings.outputHost || null,
      active: true,
      overlap: overlapSounds,
    }).catch((e) => console.error('Stop error', e))
  }
}
</script>
