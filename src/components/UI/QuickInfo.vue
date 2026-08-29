<template>
  <span
    class="qtip"
    :class="{
      'qtip--below': below && !side,
      'qtip--left': side === 'left',
      'qtip--right': side === 'right',
    }"
  >
    <slot />
    <span class="qtip__tip">{{ text }}</span>
  </span>
</template>

<script setup lang="ts">
defineProps<{
  text: string
  /** Show tip below the trigger instead of above. */
  below?: boolean
  /** Show tip to the left or right (sidebar). Overrides above/below. */
  side?: 'left' | 'right'
}>()
</script>
