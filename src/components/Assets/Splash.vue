<template>
  <div class="splash" role="status" :aria-label="label">
    <!-- Animated WebP at display resolution: a third of the GIF's bytes and a
         quarter of its per-frame decode work, right when the app is booting. -->
    <img
      src="/designs/Logo_Animated.webp"
      width="700"
      height="250"
      alt="Sound Ninja"
      class="splash__logo"
      decoding="async"
    />
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
