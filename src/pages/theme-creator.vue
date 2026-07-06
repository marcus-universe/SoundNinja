<template>
  <div class="theme-creator-window">
    <SettingsThemeCreator />
  </div>
</template>

<script setup lang="ts">
// Standalone window that hosts the Theme Creator. The controls emit live
// `theme_preview` events consumed by the main window (the real preview surface).
</script>

<style lang="scss" scoped>
.theme-creator-window {
  width: 100%;
  height: 100vh;
  overflow-y: auto;
  padding: 1.25rem 1.25rem 3rem;
  background: var(--color-bg, #222831);
  color: #eee;
}
</style>
