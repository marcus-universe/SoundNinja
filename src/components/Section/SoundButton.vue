<template>
    <div
        class="Soundbtn flex_c_v flex_wrap"
        :class="{ active: sound.active, loading, selected, 'has-gif': hasGif, 'gif-title-chip': showGifTitleChip, 'progress-paused': progressPaused, missing }"
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
import { computed, inject, ref } from 'vue'
import { copyText } from '~/utils/clipboard'
import { BOARD_GIF_KEY, peekGifUrls } from '~/utils/gifCache'
import { isRemote } from '~/utils/runtime'
import { useRemoteLongPress } from '~/utils/longPress'

const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const props = defineProps({
  sound: { type: Object, required: true },
  btnStyle: { type: Object, default: () => ({}) },
  loading: { type: Boolean, default: false },
  selected: { type: Boolean, default: false },
  multiSelect: { type: Boolean, default: false },
  /** Animate while on screen (the "play GIFs always" setting). */
  gifAnimate: { type: Boolean, default: false },
  /** Animate while the pointer is over this button and it is on screen. */
  gifAnimateOnHover: { type: Boolean, default: false },
  gifPosX: { type: Number, default: 50 },
  gifPosY: { type: Number, default: 50 },
  hasGif: { type: Boolean, default: false },
  progressPaused: { type: Boolean, default: false },
  missing: { type: Boolean, default: false },
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
// own visibility or URL changes.
const boardGif = inject(BOARD_GIF_KEY, null)

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

function copyId() {
  copyText(props.sound.id)
}

const showBadges = computed(() => appSettings.showTagBadges !== false)
const showGifTitleChip = computed(() => appSettings.gifTitleChip !== false)

const badgeCap = computed(() => {
  if (typeof document === 'undefined') return 3
  const raw = getComputedStyle(document.documentElement).getPropertyValue('--btn_width')
  const rem = parseFloat(raw) || 17.6
  if (rem < 8) return 1
  if (rem < 12) return 2
  return 3
})

const badgeTags = computed(() => {
  const ids = props.sound.tagIds ?? []
  if (!ids.length) return []
  const byId = new Map((jsonStore.configFile.tags ?? []).map((t) => [t.id, t]))
  const out = []
  for (const id of ids) {
    const tag = byId.get(id)
    if (tag) out.push(tag)
    if (out.length >= badgeCap.value) break
  }
  return out
})
</script>
