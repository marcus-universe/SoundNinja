<template>
    <div
        class="Soundbtn flex_c_v flex_wrap"
        :class="{ active: sound.active, loading, selected, 'has-gif': hasGif, 'progress-paused': progressPaused, missing }"
        :style="btnStyle"
        :data-sound-index="sound.index"
        @click="$emit('play')"
        @contextmenu.prevent="(e) => $emit('contextmenu', e)"
        @mouseenter="hovered = true"
        @mouseleave="hovered = false"
    >
        <img
            v-if="gifSrc"
            class="sound-gif"
            :src="gifSrc"
            alt=""
            draggable="false"
            :style="{ objectPosition: gifPosX + '% ' + gifPosY + '%' }"
        />
        <span class="sound-label">{{ sound.name }}</span>
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
import { computed, ref } from 'vue'
import { copyText } from '~/utils/clipboard'

const props = defineProps({
  sound: { type: Object, required: true },
  btnStyle: { type: Object, default: () => ({}) },
  loading: { type: Boolean, default: false },
  selected: { type: Boolean, default: false },
  multiSelect: { type: Boolean, default: false },
  /** Animated file URL, shown while this button plays its GIF. */
  gifAnimSrc: { type: String, default: '' },
  /** Still first frame, shown the rest of the time so no decoder stays alive. */
  gifPosterSrc: { type: String, default: '' },
  /** Animate unconditionally (the "play GIFs always" setting). */
  gifAnimate: { type: Boolean, default: false },
  /** Animate while the pointer is over this button. */
  gifAnimateOnHover: { type: Boolean, default: false },
  gifPosX: { type: Number, default: 50 },
  gifPosY: { type: Number, default: 50 },
  hasGif: { type: Boolean, default: false },
  progressPaused: { type: Boolean, default: false },
  missing: { type: Boolean, default: false },
})
defineEmits(['play', 'contextmenu'])

// Hover lives here on purpose. Held in the parent it invalidated a computed
// spanning every sound, so moving the mouse re-rendered the whole board.
const hovered = ref(false)

const gifSrc = computed(() => {
  if (!props.gifAnimSrc) return ''
  const animating = props.gifAnimate || (props.gifAnimateOnHover && hovered.value)
  return animating ? props.gifAnimSrc : props.gifPosterSrc || props.gifAnimSrc
})

function copyId() {
  copyText(props.sound.id)
}
</script>
