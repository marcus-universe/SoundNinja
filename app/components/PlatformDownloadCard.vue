<script setup lang="ts">
export type AssetKind = 'exe' | 'msi' | 'dmg' | 'appimage' | 'deb' | 'other'
export type PlatformKey = 'windows' | 'mac' | 'linux'

export interface ClassifiedAsset {
  name: string
  url: string
  kind: AssetKind
  platform: string
}

const props = defineProps<{
  platform: PlatformKey
  icon: string
  assets: ClassifiedAsset[]
  highlighted?: boolean
}>()

const { t } = useI18n()

const open = ref(false)
const root = ref<HTMLElement | null>(null)

const preferredOrder: Record<PlatformKey, AssetKind[]> = {
  windows: ['exe', 'msi'],
  mac: ['dmg'],
  linux: ['appimage', 'deb'],
}

function pickDefault(assets: ClassifiedAsset[]) {
  for (const kind of preferredOrder[props.platform]) {
    const match = assets.find((a) => a.kind === kind)
    if (match) return match
  }
  return assets[0] ?? null
}

const selected = ref<ClassifiedAsset | null>(pickDefault(props.assets))

watch(
  () => props.assets,
  (assets) => {
    selected.value = pickDefault(assets)
  },
  { deep: true },
)

function shortLabel(asset: ClassifiedAsset) {
  if (asset.kind === 'other') {
    return t('downloadPage.assetsShort.other', { name: asset.name })
  }
  return t(`downloadPage.assetsShort.${asset.kind}`)
}

function toggle() {
  if (!props.assets.length) return
  open.value = !open.value
}

function choose(asset: ClassifiedAsset) {
  selected.value = asset
  open.value = false
  const a = document.createElement('a')
  a.href = asset.url
  a.target = '_blank'
  a.rel = 'noopener noreferrer'
  document.body.appendChild(a)
  a.click()
  a.remove()
}

function onDocClick(e: MouseEvent) {
  if (!root.value?.contains(e.target as Node)) open.value = false
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') open.value = false
}

onMounted(() => {
  document.addEventListener('click', onDocClick)
  document.addEventListener('keydown', onKey)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick)
  document.removeEventListener('keydown', onKey)
})
</script>

<template>
  <article
    ref="root"
    class="card flex flex-col items-center px-6 py-8 text-center transition-colors duration-200 sm:px-8 sm:py-10"
    :class="
      highlighted
        ? 'border-2 border-primary'
        : 'border-2 border-transparent'
    "
  >
    <div class="flex items-center justify-center py-2 sm:py-4">
      <Icon
        :name="icon"
        :size="112"
        class="text-ink"
        aria-hidden="true"
      />
    </div>

    <h3 class="mt-2 text-xl font-extrabold sm:text-2xl">
      {{ t(`downloadPage.platforms.${platform}`) }}
    </h3>
    <p class="mt-1 text-sm text-ink/55">
      {{ t(`downloadPage.platformHint.${platform}`) }}
    </p>

    <div v-if="assets.length" class="relative mt-8 w-full">
      <button
        type="button"
        class="flex w-full cursor-pointer items-center justify-between gap-3 rounded-xl bg-bg/80 px-4 py-3 text-left text-sm font-semibold text-ink transition-colors duration-200 hover:bg-bg"
        :aria-expanded="open"
        :aria-haspopup="true"
        :aria-label="t('downloadPage.selectFormat')"
        @click.stop="toggle"
      >
        <span class="truncate">
          {{ selected ? shortLabel(selected) : t('downloadPage.selectFormat') }}
        </span>
        <svg
          xmlns="http://www.w3.org/2000/svg"
          viewBox="0 0 24 24"
          fill="currentColor"
          class="size-5 shrink-0 text-ink/60 transition-transform duration-200"
          :class="{ 'rotate-180': open }"
          aria-hidden="true"
        >
          <path
            d="M12 15.5a1 1 0 0 1-.7-.3l-5-5a1 1 0 1 1 1.4-1.4L12 13.1l4.3-4.3a1 1 0 1 1 1.4 1.4l-5 5a1 1 0 0 1-.7.3Z"
          />
        </svg>
      </button>

      <ul
        v-if="open"
        class="absolute inset-x-0 top-full z-20 mt-2 overflow-hidden rounded-xl border border-white/10 bg-bg shadow-xl"
        role="listbox"
      >
        <li v-for="asset in assets" :key="asset.url" role="option">
          <button
            type="button"
            class="flex w-full cursor-pointer items-center px-4 py-3 text-left text-sm font-semibold text-ink transition-colors duration-200 hover:bg-surface-hover"
            :class="{ 'text-primary': selected?.url === asset.url }"
            @click.stop="choose(asset)"
          >
            {{ shortLabel(asset) }}
          </button>
        </li>
      </ul>
    </div>
    <p v-else class="mt-8 text-sm text-ink/50">
      {{ t('downloadPage.noAsset') }}
    </p>
  </article>
</template>
