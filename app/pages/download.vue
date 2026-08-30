<script setup lang="ts">
type AssetKind = 'exe' | 'msi' | 'dmg' | 'appimage' | 'deb' | 'other'
type PlatformKey = 'windows' | 'mac' | 'linux'

interface ClassifiedAsset {
  name: string
  url: string
  kind: AssetKind
  platform: string
}

const { t, locale } = useI18n()
const localePath = useLocalePath()
const { latest, older, loading, error, releasesPageUrl } = useReleases()

useHead(() => ({
  title: `${t('downloadPage.title')} — SoundNinja`,
}))

const platforms = ['windows', 'mac', 'linux'] as const

const platformIcons: Record<PlatformKey, string> = {
  windows: 'ri:windows-fill',
  mac: 'ic:baseline-apple',
  linux: 'fa7-brands:linux',
}

const detectedPlatform = ref<PlatformKey | 'unknown'>('unknown')

onMounted(() => {
  const ua = navigator.userAgent.toLowerCase()
  if (ua.includes('win')) detectedPlatform.value = 'windows'
  else if (ua.includes('mac')) detectedPlatform.value = 'mac'
  else if (ua.includes('linux') || ua.includes('x11'))
    detectedPlatform.value = 'linux'
})

function assetLabel(asset: ClassifiedAsset) {
  if (asset.kind === 'other') {
    return t('downloadPage.assets.other', { name: asset.name })
  }
  return t(`downloadPage.assets.${asset.kind}`)
}

function assetPlatformIcon(asset: ClassifiedAsset): string | null {
  if (
    asset.platform === 'windows' ||
    asset.platform === 'mac' ||
    asset.platform === 'linux'
  ) {
    return platformIcons[asset.platform]
  }
  return null
}

function formatDate(iso: string | null) {
  if (!iso) return null
  try {
    return new Intl.DateTimeFormat(locale.value, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    }).format(new Date(iso))
  } catch {
    return iso.slice(0, 10)
  }
}
</script>

<template>
  <div class="mx-auto max-w-5xl px-4 pt-24 pb-16 sm:px-6 sm:pt-28">
    <NuxtLink
      :to="localePath('/')"
      class="mb-8 inline-flex items-center gap-1 text-sm font-semibold text-primary hover:underline"
    >
      ← {{ t('downloadPage.back') }}
    </NuxtLink>

    <h1 class="text-3xl font-extrabold sm:text-4xl">
      {{ t('downloadPage.title') }}
    </h1>
    <p class="mt-3 max-w-xl text-ink/75">
      {{ t('downloadPage.subtitle') }}
    </p>

    <div
      v-if="loading"
      class="card mt-10 p-8 text-center text-ink/70"
      role="status"
    >
      {{ t('downloadPage.loading') }}
    </div>

    <div
      v-else-if="error || !latest"
      class="card mt-10 space-y-4 p-8 text-center"
    >
      <p class="text-ink/80">{{ t('downloadPage.error') }}</p>
      <a
        :href="releasesPageUrl"
        target="_blank"
        rel="noopener noreferrer"
        class="btn-primary"
      >
        {{ t('downloadPage.viewOnGithub') }}
      </a>
    </div>

    <template v-else>
      <section class="mt-10" aria-labelledby="latest-heading">
        <div class="flex flex-wrap items-baseline justify-between gap-2">
          <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
            <h2 id="latest-heading" class="text-xl font-bold text-primary">
              {{ t('downloadPage.latest') }}
            </h2>
            <a
              :href="latest.htmlUrl"
              target="_blank"
              rel="noopener noreferrer"
              class="text-sm font-semibold text-primary hover:underline"
            >
              {{ t('downloadPage.releaseNotes') }}
            </a>
          </div>
          <p class="text-sm font-semibold text-ink/60">
            {{ t('downloadPage.version', { version: latest.tag }) }}
            <span v-if="formatDate(latest.publishedAt)">
              · {{ t('downloadPage.published', { date: formatDate(latest.publishedAt) }) }}
            </span>
          </p>
        </div>

        <div class="mt-6 grid gap-4 sm:grid-cols-3 sm:gap-5">
          <PlatformDownloadCard
            v-for="key in platforms"
            :key="key"
            :platform="key"
            :icon="platformIcons[key]"
            :assets="latest.byPlatform[key]"
            :highlighted="detectedPlatform === key"
          />
        </div>
      </section>

      <section class="mt-14" aria-labelledby="older-heading">
        <h2 id="older-heading" class="text-xl font-bold text-primary">
          {{ t('downloadPage.older') }}
        </h2>

        <p v-if="!older.length" class="mt-4 text-ink/60">
          {{ t('downloadPage.olderEmpty') }}
        </p>

        <ul v-else class="mt-6 space-y-4">
          <li
            v-for="release in older"
            :key="release.tag"
            class="card p-5 sm:p-6"
          >
            <div class="flex flex-wrap items-baseline justify-between gap-2">
              <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                <h3 class="text-lg font-extrabold">
                  {{ release.tag }}
                  <span
                    v-if="release.prerelease"
                    class="ml-2 text-xs font-semibold text-ink/50"
                  >
                    {{ t('downloadPage.prerelease') }}
                  </span>
                </h3>
                <a
                  :href="release.htmlUrl"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-sm font-semibold text-primary hover:underline"
                >
                  {{ t('downloadPage.releaseNotes') }}
                </a>
              </div>
              <span
                v-if="formatDate(release.publishedAt)"
                class="text-sm text-ink/50"
              >
                {{ t('downloadPage.published', { date: formatDate(release.publishedAt) }) }}
              </span>
            </div>

            <ul
              v-if="release.assets.length"
              class="mt-4 flex flex-wrap gap-2"
            >
              <li v-for="asset in release.assets" :key="asset.url">
                <a
                  :href="asset.url"
                  class="btn-download px-4 py-2 text-sm"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  <Icon
                    v-if="assetPlatformIcon(asset)"
                    :name="assetPlatformIcon(asset)!"
                    class="size-5 shrink-0"
                    aria-hidden="true"
                  />
                  {{ assetLabel(asset) }}
                </a>
              </li>
            </ul>
            <a
              v-else
              :href="release.htmlUrl"
              target="_blank"
              rel="noopener noreferrer"
              class="mt-3 inline-block text-sm font-semibold text-primary hover:underline"
            >
              {{ t('downloadPage.viewOnGithub') }}
            </a>
          </li>
        </ul>
      </section>
    </template>
  </div>
</template>
