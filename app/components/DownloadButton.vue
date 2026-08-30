<script setup lang="ts">
const { t } = useI18n()
const localePath = useLocalePath()
const { version, loading } = useReleases()

const label = computed(() =>
  version.value
    ? t('hero.downloadVersion', { version: version.value })
    : t('hero.download'),
)
</script>

<template>
  <NuxtLink
    :to="localePath('/download')"
    class="btn-primary min-w-[10rem]"
    :aria-busy="loading"
  >
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="currentColor"
      class="h-5 w-5"
      aria-hidden="true"
    >
      <path
        d="M12 3a1 1 0 0 1 1 1v9.586l2.293-2.293a1 1 0 1 1 1.414 1.414l-4 4a1 1 0 0 1-1.414 0l-4-4a1 1 0 1 1 1.414-1.414L11 13.586V4a1 1 0 0 1 1-1ZM5 18a1 1 0 1 0 0 2h14a1 1 0 1 0 0-2H5Z"
      />
    </svg>
    {{ label }}
  </NuxtLink>
</template>
