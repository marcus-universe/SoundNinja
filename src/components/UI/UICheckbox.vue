<template>
  <label class="ui-checkbox">
    <input
      type="checkbox"
      class="ui-checkbox__input"
      :checked="modelValue"
      @change="$emit('update:modelValue', $event.target.checked)"
    />
    <span class="ui-checkbox__box" aria-hidden="true">
      <svg class="ui-checkbox__check" viewBox="0 0 12 10" fill="none" xmlns="http://www.w3.org/2000/svg">
        <polyline points="1.5,5 4.5,8.5 10.5,1.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </span>
    <slot />
  </label>
</template>

<script setup>
defineProps({ modelValue: { type: Boolean, default: false } })
defineEmits(['update:modelValue'])
</script>
