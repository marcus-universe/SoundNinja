<template>
  <div
    class="main flex_c_v flex_start"
    :class="{ 'main--bulk': appStore.multiSelectActive }"
  >
    <Tabs />
    <SoundContainer />
  </div>
</template>

<script setup>
const appStore = useAppStore()
</script>
