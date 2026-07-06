<template>
  <div class="dialog-box" @click.self="$emit('close')">
    <div class="dialog-panel flex_c_v gap1">
      <div class="dialog-header flex_c_h flex_space_between w100">
        <h2 class="dialog-title">{{ title }}</h2>
        <Icons :icon="'exit'" :customClass="'exit overlay-close'" @triggered="$emit('close')" />
      </div>
      <p v-if="errorMessage" class="dialog-error">{{ errorMessage }}</p>
      <slot />
    </div>
  </div>
</template>

<script setup>
defineProps({
  title: { type: String, required: true },
  errorMessage: { type: String, default: '' },
})
defineEmits(['close'])
</script>
