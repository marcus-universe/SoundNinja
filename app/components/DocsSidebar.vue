<script setup lang="ts">
const props = defineProps<{
  pages: Array<{ path: string, title: string }>
  activePath: string
}>()

const { t } = useI18n()
const localePath = useLocalePath()

function docsTo(path: string) {
  const stem = path.replace(/^\//, '')
  return stem === 'overview' ? localePath('/docs') : localePath(`/docs/${stem}`)
}

function isActive(path: string) {
  return path === props.activePath
}
</script>

<template>
  <nav
    class="card p-4 lg:sticky lg:top-24"
    :aria-label="t('docsPage.onThisPage')"
  >
    <p class="mb-3 text-xs font-bold tracking-wide text-ink/50 uppercase">
      {{ t('docsPage.title') }}
    </p>
    <ul class="flex flex-wrap gap-2 lg:flex-col lg:gap-1">
      <li v-for="item in pages" :key="item.path">
        <NuxtLink
          :to="docsTo(item.path)"
          class="inline-flex rounded-full px-3 py-1.5 text-sm font-semibold transition-colors duration-200 lg:w-full lg:rounded-xl"
          :class="isActive(item.path)
            ? 'bg-primary/15 text-primary'
            : 'text-ink/80 hover:bg-surface-hover hover:text-ink'"
        >
          {{ item.title }}
        </NuxtLink>
      </li>
    </ul>
  </nav>
</template>
