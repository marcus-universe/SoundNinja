<template>
  <span class="qtip" :class="{ 'qtip--below': below }">
    <slot />
    <span class="qtip__tip">{{ text }}</span>
  </span>
</template>

<script setup lang="ts">
defineProps<{
  text: string
  /** Show tip below the trigger instead of above. */
  below?: boolean
}>()
</script>
