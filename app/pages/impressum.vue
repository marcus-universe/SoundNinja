<script setup lang="ts">
const { t } = useI18n()
const localePath = useLocalePath()

const LICENSE_URL =
  'https://github.com/marcus-universe/SoundNinja/blob/main/LICENSE'
const REPO_URL = 'https://github.com/marcus-universe/SoundNinja'

const contactEmail = computed(() => t('impressum.contactEmail'))

useHead(() => ({
  title: `${t('impressum.title')} — SoundNinja`,
}))
</script>

<template>
  <div class="mx-auto max-w-3xl px-4 pt-24 pb-16 sm:px-6 sm:pt-28">
    <NuxtLink
      :to="localePath('/')"
      class="mb-8 inline-flex items-center gap-1 text-sm font-semibold text-primary hover:underline"
    >
      ← {{ t('impressum.back') }}
    </NuxtLink>

    <h1 class="text-3xl font-extrabold sm:text-4xl">
      {{ t('impressum.title') }}
    </h1>

    <div class="card mt-8 space-y-6 p-6 sm:p-8">
      <section>
        <h2 class="text-lg font-bold text-primary">
          {{ t('impressum.nameHeading') }}
        </h2>
        <p class="mt-2 text-ink/80">{{ t('impressum.placeholder') }}</p>
      </section>

      <section>
        <h2 class="text-lg font-bold text-primary">
          {{ t('impressum.contactHeading') }}
        </h2>
        <p class="mt-2 text-ink/80">
          {{ t('impressum.contactLabel') }}:
          <a
            :href="`mailto:${contactEmail}`"
            class="font-semibold text-primary hover:underline"
          >
            {{ contactEmail }}
          </a>
        </p>
      </section>

      <section>
        <h2 class="text-lg font-bold text-primary">
          {{ t('impressum.repoHeading') }}
        </h2>
        <p class="mt-2 text-ink/80">{{ t('impressum.repoText') }}</p>
        <a
          :href="REPO_URL"
          target="_blank"
          rel="noopener noreferrer"
          class="mt-2 inline-block font-semibold text-primary hover:underline"
        >
          github.com/marcus-universe/SoundNinja
        </a>
      </section>

      <section>
        <h2 class="text-lg font-bold text-primary">
          {{ t('impressum.licenseHeading') }}
        </h2>
        <p class="mt-2 text-ink/80">{{ t('impressum.licenseText') }}</p>
        <a
          :href="LICENSE_URL"
          target="_blank"
          rel="noopener noreferrer"
          class="mt-2 inline-block font-semibold text-primary hover:underline"
        >
          {{ t('impressum.licenseLink') }}
        </a>
      </section>
    </div>
  </div>
</template>
