<script setup lang="ts">
const route = useRoute()
</script>

<template>
  <NuxtLayout>
    <NuxtPage :page-key="route.path" />
  </NuxtLayout>
</template>
