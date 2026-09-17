<template>
  <div
    v-if="appStore.dropActive"
    class="drop-overlay"
    role="status"
    aria-live="polite"
  >
    <div class="drop-overlay__frame">
      <Icons icon="upload" custom-class="drop-overlay__icon" />
      <p class="drop-overlay__title">{{ $t('dropZone.title') }}</p>
      <p class="drop-overlay__hint">{{ $t('dropZone.hint') }}</p>
    </div>
  </div>
</template>

<script setup>
const appStore = useAppStore()
</script>
