<template>
    <div
        class="Soundbtn flex_c_v flex_wrap"
        :class="{ active: sound.active, loading, selected }"
        :style="btnStyle"
        :data-sound-index="sound.index"
        @click="$emit('play')"
        @contextmenu.prevent="(e) => $emit('contextmenu', e)"
    >
        <span class="sound-label">{{ sound.name }}</span>
        <span v-if="loading" class="sound-spinner" aria-hidden="true" />
    </div>
</template>

<script setup>
defineProps({
  sound: { type: Object, required: true },
  btnStyle: { type: Object, default: () => ({}) },
  loading: { type: Boolean, default: false },
  selected: { type: Boolean, default: false },
})
defineEmits(['play', 'contextmenu'])
</script>
