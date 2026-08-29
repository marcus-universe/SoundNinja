<template>
  <div class="settings-label-info" :class="{ 'settings-label-info--fluid': fluid }">
    <span class="settings-label"><slot /></span>
    <span class="settings-info-icon">
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
      <span class="settings-info-icon__tip">{{ tip }}</span>
    </span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  tip: string
  /** Drop fixed label width (stacked Theme Creator rows). */
  fluid?: boolean
}>()
</script>
