import { defineCollection, defineContentConfig, z } from '@nuxt/content'

const docsSchema = z.object({
  title: z.string(),
  description: z.string(),
  order: z.number(),
})

const locales = ['en', 'de', 'fr', 'es', 'ja', 'zh'] as const

function localeCollection(code: (typeof locales)[number]) {
  return defineCollection({
    type: 'page',
    source: {
      include: `${code}/**`,
      prefix: '',
    },
    schema: docsSchema,
  })
}

export default defineContentConfig({
  collections: {
    content_en: localeCollection('en'),
    content_de: localeCollection('de'),
    content_fr: localeCollection('fr'),
    content_es: localeCollection('es'),
    content_ja: localeCollection('ja'),
    content_zh: localeCollection('zh'),
  },
})
