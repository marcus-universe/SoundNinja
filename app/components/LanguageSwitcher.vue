<script setup lang="ts">
const { t, locale } = useI18n()
const switchLocalePath = useSwitchLocalePath()

const locales = [
  { code: 'en', label: 'EN' },
  { code: 'de', label: 'DE' },
  { code: 'fr', label: 'FR' },
  { code: 'es', label: 'ES' },
  { code: 'ja', label: 'JA' },
  { code: 'zh', label: 'ZH' },
] as const

async function onChange(event: Event) {
  const code = (event.target as HTMLSelectElement).value
  await navigateTo(switchLocalePath(code))
}
</script>

<template>
  <label class="relative inline-flex items-center">
    <span class="sr-only">{{ t('nav.language') }}</span>
    <select
      :value="locale"
      class="cursor-pointer appearance-none rounded-full bg-surface py-1 pr-7 pl-3 text-xs font-bold text-ink transition-colors hover:bg-surface-hover sm:text-sm"
      :aria-label="t('nav.language')"
      @change="onChange"
    >
      <option
        v-for="loc in locales"
        :key="loc.code"
        :value="loc.code"
      >
        {{ loc.label }}
      </option>
    </select>
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="currentColor"
      class="pointer-events-none absolute right-2 size-3.5 text-ink/60"
      aria-hidden="true"
    >
      <path
        d="M12 15.5a1 1 0 0 1-.7-.3l-5-5a1 1 0 1 1 1.4-1.4L12 13.1l4.3-4.3a1 1 0 1 1 1.4 1.4l-5 5a1 1 0 0 1-.7.3Z"
      />
    </svg>
  </label>
</template>
