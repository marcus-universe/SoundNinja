<template>
  <li class="ui-listitem flex_c flex_space_between gap1">
    <span class="ui-listitem-label">{{ label }}</span>
    <Icons
      :icon="'exit'"
      :customClass="'ui-listitem-remove icon'"
      @triggered="$emit('remove')"
    />
  </li>
</template>

<script setup>
defineProps({ label: { type: String, required: true } })
defineEmits(['remove'])
</script>
