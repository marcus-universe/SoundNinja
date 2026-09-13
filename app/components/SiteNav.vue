<script setup lang="ts">
const { t } = useI18n()
const localePath = useLocalePath()
const config = useRuntimeConfig()
const base = computed(() => {
  const b = config.app.baseURL || '/'
  return b.endsWith('/') ? b : `${b}/`
})
</script>

<template>
  <header
    class="fixed top-0 right-0 left-0 z-50 border-b border-white/5 bg-bg/80 backdrop-blur-md"
  >
    <nav
      class="mx-auto flex h-14 max-w-6xl items-center justify-between px-4 sm:h-16 sm:px-6"
      aria-label="Primary"
    >
      <NuxtLink
        :to="localePath('/')"
        class="flex items-center gap-2 text-lg font-extrabold uppercase tracking-tight text-ink transition-colors hover:text-primary sm:text-xl"
      >
        <img
          :src="`${base}assets/Logo_Icon.svg`"
          :alt="t('hero.logoAlt')"
          width="40"
          height="32"
          class="h-8 w-auto sm:h-9"
        />
        Sound Ninja
      </NuxtLink>

      <div class="flex items-center gap-3 sm:gap-4">
        <NuxtLink
          :to="localePath('/docs')"
          class="text-sm font-semibold text-ink/80 transition-colors hover:text-primary"
        >
          {{ t('nav.docs') }}
        </NuxtLink>
        <NuxtLink
          :to="{ path: localePath('/'), hash: '#features' }"
          class="hidden text-sm font-semibold text-ink/80 transition-colors hover:text-primary sm:inline"
        >
          {{ t('nav.features') }}
        </NuxtLink>
        <NuxtLink
          :to="localePath('/download')"
          class="hidden text-sm font-semibold text-ink/80 transition-colors hover:text-primary sm:inline"
        >
          {{ t('nav.download') }}
        </NuxtLink>
        <a
          href="https://github.com/marcus-universe/SoundNinja"
          target="_blank"
          rel="noopener noreferrer"
          class="hidden text-sm font-semibold text-ink/80 transition-colors hover:text-primary sm:inline"
        >
          {{ t('nav.github') }}
        </a>

        <LanguageSwitcher />
      </div>
    </nav>
  </header>
</template>
