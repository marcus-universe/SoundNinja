<template>
    <div
        class="Soundbtn flex_c_v flex_wrap"
        :class="{ active: sound.active, loading, selected, 'has-gif': hasGif, 'gif-title-chip': showGifTitleChip, missing }"
        :style="btnStyle"
        :data-sound-index="sound.index"
        :data-sound-path="sound.path"
        @click="onClick"
        @pointerdown="onPointerDown"
        @pointermove="lp.move"
        @pointerup="lp.end"
        @pointercancel="lp.end"
        @contextmenu.prevent="(e) => $emit('contextmenu', e)"
        @mouseenter="hovered = true"
        @mouseleave="hovered = false"
    >
        <img
            v-if="gifSrc"
            class="sound-gif"
            :src="gifSrc"
            alt=""
            decoding="async"
            draggable="false"
            :style="{ objectPosition: gifPosX + '% ' + gifPosY + '%' }"
        />
        <span v-if="progress" ref="progressEl" class="sound-progress" aria-hidden="true" />
        <span class="sound-label">{{ sound.name }}</span>
        <span v-if="showBadges && badgeTags.length" class="sound-tag-badges" aria-hidden="true">
          <span
            v-for="tag in badgeTags"
            :key="tag.id"
            class="tag-badge"
            :style="{ '--tag-color': tag.color }"
          >{{ tag.name }}</span>
        </span>
        <span
          v-if="multiSelect && sound.id"
          class="sound-id-chip"
          :title="sound.id"
          @contextmenu.prevent.stop="copyId"
        >{{ sound.id }}</span>
        <span v-if="loading" class="sound-spinner" aria-hidden="true" />
    </div>
</template>

<script setup>
import { computed, inject, onBeforeUnmount, ref, watch } from 'vue'
import { copyText } from '~/utils/clipboard'
import { BOARD_STATE_KEY } from '~/utils/boardState'
import { BOARD_GIF_KEY, peekGifUrls } from '~/utils/gifCache'
import { isRemote } from '~/utils/runtime'
import { useRemoteLongPress } from '~/utils/longPress'
import { progressPositionMs } from '~/utils/soundProgress'

const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const props = defineProps({
  sound: { type: Object, required: true },
  btnStyle: { type: Object, default: () => ({}) },
  multiSelect: { type: Boolean, default: false },
  /** Animate while on screen (the "play GIFs always" setting). */
  gifAnimate: { type: Boolean, default: false },
  /** Animate while the pointer is over this button and it is on screen. */
  gifAnimateOnHover: { type: Boolean, default: false },
  gifPosX: { type: Number, default: 50 },
  gifPosY: { type: Number, default: 50 },
  hasGif: { type: Boolean, default: false },
})
const emit = defineEmits(['play', 'contextmenu'])
const lp = useRemoteLongPress()

function onPointerDown(e) {
  lp.start(e, (x, y) => {
    emit('contextmenu', { preventDefault() {}, stopPropagation() {}, clientX: x, clientY: y })
  })
}

function onClick(e) {
  if (lp.eaten(e)) return
  if (isRemote) {
    try { navigator.vibrate?.(8) } catch { /* unsupported */ }
  }
  emit('play', e)
}

// Hover lives here on purpose. Held in the parent it invalidated a computed
// spanning every sound, so moving the mouse re-rendered the whole board.
const hovered = ref(false)

// Per-button reads of the board's maps: only this button re-renders when its
// own visibility, URL, loading, selection or playback state changes.
const boardGif = inject(BOARD_GIF_KEY, null)
const board = inject(BOARD_STATE_KEY, null)

const loading = computed(() => !!board?.loading.has(props.sound.path))
const selected = computed(() => !!board?.selected.value.has(props.sound.path))
const missing = computed(() => !!board?.missing.value.has(props.sound.path))
const progress = computed(() => board?.progress.get(props.sound.path) ?? null)

const gifUrls = computed(() => {
  const id = props.sound.gifId
  if (!id) return null
  return boardGif?.gifUrls[id] || peekGifUrls(id)
})

const onScreen = computed(() => !!boardGif?.visibleByPath[props.sound.path])

const gifSrc = computed(() => {
  const urls = gifUrls.value
  if (!urls?.animUrl) return ''
  const animating = onScreen.value
    && (props.gifAnimate || (props.gifAnimateOnHover && hovered.value))
  // Animated file only while it plays; the still frame otherwise so no decoder stays alive.
  return animating ? urls.animUrl : urls.posterUrl || urls.animUrl
})

// Progress sweep. The animation is created at the position the audio is at
// right now, so a remount (tab switch) or a re-render picks up mid-sweep
// instead of starting over, and the compositor runs it without layout.
const progressEl = ref(null)
let sweep = null

function syncSweep() {
  sweep?.cancel()
  sweep = null
  const el = progressEl.value
  const info = progress.value
  if (!el || !info || !(info.duration > 0)) return
  const durMs = info.duration * 1000
  const pos = progressPositionMs(info, Date.now())
  if (typeof el.animate !== 'function') {
    el.style.transform = `scaleX(${pos / durMs})`
    return
  }
  sweep = el.animate(
    [{ transform: 'scaleX(0)' }, { transform: 'scaleX(1)' }],
    { duration: durMs, iterations: info.looping ? Infinity : 1, fill: 'both', easing: 'linear' },
  )
  sweep.currentTime = pos
  if (info.paused) sweep.pause()
}

watch([progress, progressEl], syncSweep, { flush: 'post' })

onBeforeUnmount(() => {
  sweep?.cancel()
  sweep = null
})

function copyId() {
  copyText(props.sound.id)
}

const showBadges = computed(() => appSettings.showTagBadges !== false)
const showGifTitleChip = computed(() => appSettings.gifTitleChip !== false)

const badgeTags = computed(() => {
  const ids = props.sound.tagIds ?? []
  if (!ids.length) return []
  const byId = board?.tagsById.value
    ?? new Map((jsonStore.configFile.tags ?? []).map((t) => [t.id, t]))
  const cap = board?.badgeCap.value ?? 3
  const out = []
  for (const id of ids) {
    const tag = byId.get(id)
    if (tag) out.push(tag)
    if (out.length >= cap) break
  }
  return out
})
</script>
