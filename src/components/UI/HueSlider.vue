<template>
  <input
    type="range"
    class="hue-slider"
    min="0"
    max="360"
    step="1"
    :value="modelValue"
    :title="hint"
    :aria-label="ariaLabel || hint || 'Hue'"
    @pointerdown="$emit('dragStart')"
    @input="onInput"
    @change="$emit('dragEnd')"
  />
</template>

<script setup lang="ts">
defineProps<{
  modelValue: number
  hint?: string
  ariaLabel?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [number]
  dragStart: []
  dragEnd: []
}>()

function onInput(e: Event) {
  emit('update:modelValue', Number((e.target as HTMLInputElement).value))
}
</script>
