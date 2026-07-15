<template>
  <button
    type="button"
    class="dialog-close-btn"
    :title="title"
    aria-label="Close"
    @click="$emit('close')"
  >
    <Icons icon="exit" custom-class="dialog-close" />
  </button>
</template>

<script setup lang="ts">
defineProps({
  title: { type: String, default: '' },
})

defineEmits(['close'])
</script>
