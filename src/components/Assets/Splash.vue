<template>
  <div class="splash" role="status" :aria-label="label">
    <img src="/designs/Logo_Animated.gif" alt="Sound Ninja" class="splash__logo" />
    <div class="splash__bar-wrap">
      <span class="splash__label">{{ label }}</span>
      <div class="splash__bar" role="progressbar" :aria-valuenow="percent" aria-valuemin="0" aria-valuemax="100">
        <div class="splash__fill" :style="{ width: `${percent}%` }" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  percent: number
  label: string
}>()
</script>
