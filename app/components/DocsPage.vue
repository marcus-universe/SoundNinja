<script setup lang="ts">
import type { Collections } from '@nuxt/content'

const { t, locale } = useI18n()
const route = useRoute()

const contentPath = computed(() => {
  const raw = route.params.slug
  if (!raw || raw === '') return '/overview'
  return `/${String(raw)}`
})

function collectionName(code: string) {
  return (`content_${code}`) as keyof Collections
}

const { data: page } = await useAsyncData(
  () => `docs-page-${locale.value}-${contentPath.value}`,
  async () => {
    const content = await queryCollection(collectionName(locale.value))
      .path(contentPath.value)
      .first()

    if (!content && locale.value !== 'en') {
      return await queryCollection('content_en').path(contentPath.value).first()
    }

    return content
  },
  { watch: [locale, contentPath] },
)

const { data: navPages } = await useAsyncData(
  () => `docs-nav-${locale.value}`,
  async () => {
    let items = await queryCollection(collectionName(locale.value)).all()
    if ((!items || items.length === 0) && locale.value !== 'en') {
      items = await queryCollection('content_en').all()
    }
    return [...(items ?? [])].sort((a, b) => (a.order ?? 0) - (b.order ?? 0))
  },
  { watch: [locale] },
)

useHead(() => ({
  title: page.value?.title
    ? `${page.value.title} — SoundNinja`
    : `${t('docsPage.title')} — SoundNinja`,
  meta: page.value?.description
    ? [{ name: 'description', content: page.value.description }]
    : [],
}))
</script>

<template>
  <div class="mx-auto max-w-6xl px-4 pt-24 pb-16 sm:px-6 sm:pt-28">
    <div class="lg:grid lg:grid-cols-[16rem_minmax(0,1fr)] lg:items-start lg:gap-8">
      <DocsSidebar :pages="navPages ?? []" :active-path="contentPath" />

      <article class="card docs-prose mt-6 p-6 sm:p-8 lg:mt-0">
        <ContentRenderer v-if="page" :value="page" />
        <p v-else class="text-ink/80">
          {{ t('docsPage.notFound') }}
        </p>
      </article>
    </div>
  </div>
</template>
