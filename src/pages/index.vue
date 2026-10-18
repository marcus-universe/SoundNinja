<template>
  <div
    class="main flex_c_v flex_start"
    :class="{ 'main--bulk': appStore.multiSelectActive, 'main--locked': boardLocked }"
    :inert="boardLocked ? '' : undefined"
  >
    <Tabs />
    <SoundContainer />
  </div>
</template>

<script setup>
const appStore = useAppStore()
const boardLocked = computed(() =>
  !!appStore.activeOverlay || !!appStore.importReview || appStore.gifPickerIndex != null,
)
</script>
