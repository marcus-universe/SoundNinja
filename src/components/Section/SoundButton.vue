<template>
    <div
        class="Soundbtn flex_c_v flex_wrap"
        :class="{ active: sound.active }"
        :style="btnStyle"
        :data-sound-index="sound.index"
        @click="$emit('play')"
        @contextmenu.prevent="(e) => $emit('contextmenu', e)"
    >
        <span class="sound-label">{{ sound.name }}</span>
    </div>
</template>

<script setup>
defineProps({
  sound: { type: Object, required: true },
  btnStyle: { type: Object, default: () => ({}) },
})
defineEmits(['play', 'contextmenu'])
</script>
