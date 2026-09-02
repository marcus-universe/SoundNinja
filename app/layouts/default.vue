<script setup lang="ts">
const { t, locale } = useI18n()

useHead(() => ({
  htmlAttrs: { lang: locale.value },
  title: t('meta.title'),
  meta: [{ name: 'description', content: t('meta.description') }],
}))
</script>

<template>
  <div class="relative flex min-h-screen flex-col">
    <SiteBackdrop />
    <SiteNav />
    <main class="relative z-10 flex-1 overflow-x-hidden">
      <slot />
    </main>
    <SiteFooter class="relative z-10" />
  </div>
</template>
