<script setup lang="ts">
const { t } = useI18n()
const config = useRuntimeConfig()
const base = computed(() => {
  const b = config.app.baseURL || '/'
  return b.endsWith('/') ? b : `${b}/`
})
</script>

<template>
  <section
    class="relative flex min-h-[85vh] flex-col items-center justify-center px-4 pt-20 pb-16 text-center sm:px-6 sm:pt-24"
  >
    <img
      :src="`${base}assets/Logo_Icon.svg`"
      :alt="t('hero.logoAlt')"
      width="182"
      height="143"
      class="relative z-10 mb-6 h-20 w-auto sm:mb-8 sm:h-24 md:h-28"
    />

    <h1
      class="relative z-10 text-5xl font-black tracking-tight text-ink sm:text-6xl md:text-7xl"
    >
      {{ t('hero.title') }}
    </h1>

    <p
      class="relative z-10 mt-4 max-w-xl text-lg font-semibold text-ink/75 sm:text-xl md:text-2xl"
    >
      {{ t('hero.subtitle') }}
    </p>

    <div class="relative z-10 mt-10 flex flex-col items-center gap-3 sm:flex-row sm:gap-4">
      <DownloadButton />
      <a
        href="https://github.com/marcus-universe/SoundNinja"
        target="_blank"
        rel="noopener noreferrer"
        class="btn-secondary"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="currentColor"
          class="h-5 w-5"
          aria-hidden="true"
        >
          <path
            fill-rule="evenodd"
            d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0 1 12 6.844a9.59 9.59 0 0 1 2.504.337c1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.02 10.02 0 0 0 22 12.017C22 6.484 17.522 2 12 2Z"
            clip-rule="evenodd"
          />
        </svg>
        {{ t('hero.contribute') }}
      </a>
    </div>
  </section>
</template>
