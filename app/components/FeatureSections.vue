<script setup lang="ts">
const { t } = useI18n()
const config = useRuntimeConfig()
const base = computed(() => {
  const b = config.app.baseURL || '/'
  return b.endsWith('/') ? b : `${b}/`
})

const FEATURE_KEYS = [
  'themes',
  'tabs',
  'recording',
  'performance',
  'opensource',
] as const

const LICENSE_URL =
  'https://github.com/marcus-universe/SoundNinja/blob/main/LICENSE'

const features = computed(() =>
  FEATURE_KEYS.map((key, index) => ({
    key,
    title: t(`features.items.${key}.title`),
    description: t(`features.items.${key}.description`),
    image: `${base.value}screenshots/${key}.svg`,
    imageRight: index % 2 === 0,
    licenseLink:
      key === 'opensource' ? t('features.items.opensource.licenseLink') : null,
  })),
)
</script>

<template>
  <section
    id="features"
    class="relative pt-8 pb-16 sm:pt-10 sm:pb-24"
    :aria-label="t('nav.features')"
  >
    <article
      v-for="feature in features"
      :key="feature.key"
      class="px-4 py-16 first:pt-8 sm:px-6 sm:py-20 sm:first:pt-10"
    >
      <div
        class="mx-auto grid max-w-6xl items-center gap-8 md:grid-cols-2 md:gap-12 lg:gap-16"
      >
        <div
          class="order-1"
          :class="feature.imageRight ? 'md:order-1' : 'md:order-2'"
        >
          <h3 class="text-2xl font-extrabold text-primary sm:text-3xl">
            {{ feature.title }}
          </h3>
          <p class="mt-3 text-base text-ink/80 sm:text-lg">
            {{ feature.description }}
          </p>
          <a
            v-if="feature.licenseLink"
            :href="LICENSE_URL"
            target="_blank"
            rel="noopener noreferrer"
            class="mt-4 inline-flex items-center gap-1 font-semibold text-primary underline-offset-4 hover:underline"
          >
            {{ feature.licenseLink }}
            <span aria-hidden="true">→</span>
          </a>
        </div>

        <div
          class="order-2 overflow-hidden rounded-3xl bg-surface shadow-lg"
          :class="feature.imageRight ? 'md:order-2' : 'md:order-1'"
        >
          <img
            :src="feature.image"
            :alt="feature.title"
            class="aspect-video w-full object-contain"
            width="1280"
            height="720"
            loading="lazy"
          />
        </div>
      </div>
    </article>
  </section>
</template>
