<script setup lang="ts">
const { t } = useI18n()
const localePath = useLocalePath()
const year = new Date().getFullYear()

const LICENSE_URL =
  'https://github.com/marcus-universe/SoundNinja/blob/main/LICENSE'
const GITHUB_URL = 'https://github.com/marcus-universe/SoundNinja'
</script>

<template>
  <footer class="mt-auto border-t border-white/5 bg-surface/40">
    <div
      class="mx-auto flex max-w-6xl flex-col gap-6 px-4 py-10 sm:flex-row sm:items-start sm:justify-between sm:px-6"
    >
      <div class="max-w-sm">
        <p class="text-lg font-extrabold text-ink">SoundNinja</p>
        <p class="mt-1 text-sm text-ink/70">{{ t('footer.tagline') }}</p>
        <p class="mt-3 text-xs text-ink/50">
          {{ t('footer.copyright', { year }) }}
        </p>
      </div>

      <nav
        class="flex flex-wrap gap-x-6 gap-y-2 text-sm font-semibold"
        aria-label="Footer"
      >
        <NuxtLink
          :to="localePath('/docs')"
          class="text-ink/80 transition-colors hover:text-primary"
        >
          {{ t('nav.docs') }}
        </NuxtLink>
        <NuxtLink
          :to="localePath('/download')"
          class="text-ink/80 transition-colors hover:text-primary"
        >
          {{ t('nav.download') }}
        </NuxtLink>
        <a
          :href="LICENSE_URL"
          target="_blank"
          rel="noopener noreferrer"
          class="text-ink/80 transition-colors hover:text-primary"
        >
          {{ t('footer.license') }}
        </a>
        <a
          :href="GITHUB_URL"
          target="_blank"
          rel="noopener noreferrer"
          class="text-ink/80 transition-colors hover:text-primary"
        >
          {{ t('footer.github') }}
        </a>
        <NuxtLink
          :to="localePath('/impressum')"
          class="text-ink/80 transition-colors hover:text-primary"
        >
          {{ t('footer.impressum') }}
        </NuxtLink>
      </nav>
    </div>
  </footer>
</template>
