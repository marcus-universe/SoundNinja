<template>
    <div
        class="Soundbtn flex_c_v flex_wrap"
        :class="{ active: sound.active, loading, selected, 'has-gif': hasGif, 'progress-paused': progressPaused, missing }"
        :style="btnStyle"
        :data-sound-index="sound.index"
        @click="$emit('play')"
        @contextmenu.prevent="(e) => $emit('contextmenu', e)"
        @mouseenter="$emit('gifhover', true)"
        @mouseleave="$emit('gifhover', false)"
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
import { copyText } from '~/utils/clipboard'

const props = defineProps({
  sound: { type: Object, required: true },
  btnStyle: { type: Object, default: () => ({}) },
  loading: { type: Boolean, default: false },
  selected: { type: Boolean, default: false },
  multiSelect: { type: Boolean, default: false },
  gifSrc: { type: String, default: '' },
  gifPosX: { type: Number, default: 50 },
  gifPosY: { type: Number, default: 50 },
  hasGif: { type: Boolean, default: false },
  progressPaused: { type: Boolean, default: false },
  missing: { type: Boolean, default: false },
})
defineEmits(['play', 'contextmenu', 'gifhover'])

function copyId() {
  copyText(props.sound.id)
}
</script>
