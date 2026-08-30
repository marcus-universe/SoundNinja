<script setup lang="ts">
const props = defineProps<{ href?: string }>()

const href = computed(() => props.href ?? '')

const isInternal = computed(() => {
  return href.value.startsWith('/') && !href.value.startsWith('//')
})

const isHttp = computed(() => {
  return href.value.startsWith('http://') || href.value.startsWith('https://')
})
</script>

<template>
  <NuxtLinkLocale v-if="isInternal" :to="href">
    <slot />
  </NuxtLinkLocale>
  <a
    v-else-if="isHttp"
    :href="href"
    target="_blank"
    rel="noopener noreferrer"
  >
    <slot />
  </a>
  <a v-else :href="href">
    <slot />
  </a>
</template>
