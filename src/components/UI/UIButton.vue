<template>
  <button
    class="ui-btn"
    :class="{ 'ui-btn--full': fullWidth }"
    @click="$emit('click')"
  >
    <slot />
  </button>
</template>

<script setup>
defineProps({
  fullWidth: { type: Boolean, default: false },
})
defineEmits(['click'])
</script>
