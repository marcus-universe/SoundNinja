<template>
  <DialogField :title="step === 'position' ? $t('gifPicker.positionTitle') : $t('gifPicker.title')" @close="close">
    <p v-if="error" class="dialog-error">{{ error }}</p>

    <template v-if="step === 'browse'">
      <div class="gif-picker__info">
        <p class="gif-picker__hint">{{ $t('gifPicker.info') }}</p>
        <div class="gif-picker__info-actions">
          <UIButton @click="openPartner">{{ $t('gifPicker.getKey') }}</UIButton>
          <UIButton @click="pickLocal">{{ $t('gifPicker.localFile') }}</UIButton>
        </div>
      </div>
      <div class="gif-picker__toolbar">
        <input
          class="ui-input gif-picker__search"
          type="search"
          :placeholder="$t('gifPicker.searchPlaceholder')"
          :disabled="!hasKey || loading"
          v-model="query"
          @keydown.enter.prevent="runSearch"
        />
        <UIButton :disabled="!hasKey || loading" @click="runSearch">{{ $t('navbar.search') }}</UIButton>
      </div>
      <div v-if="items.length" ref="gridRef" class="gif-picker__grid">
        <button
          v-for="g in items"
          :key="g.id"
          type="button"
          class="gif-picker__cell"
          :title="g.title"
          @click="selectRemote(g)"
        >
          <img :src="g.previewUrl || g.thumbUrl" :alt="g.title" loading="lazy" draggable="false" />
        </button>
        <div
          v-if="canLoadMore"
          ref="sentinelRef"
          class="gif-picker__sentinel"
        >{{ loadingMore ? $t('gifPicker.loadingMore') : '' }}</div>
      </div>
      <p v-else-if="!loading && searched" class="gif-picker__hint">{{ $t('gifPicker.empty') }}</p>
      <p v-if="loading && !items.length" class="gif-picker__hint">…</p>
      <button type="button" class="gif-picker__attr" @click="openKlipy">{{ $t('gifPicker.poweredBy') }}</button>
    </template>

    <template v-else>
      <p class="gif-picker__hint">{{ $t('gifPicker.positionHint') }}</p>
      <div
        class="gif-picker__preview"
        @pointerdown.prevent="onDragStart"
        @pointermove="onDragMove"
        @pointerup="onDragEnd"
        @pointerleave="onDragEnd"
      >
        <img
          v-if="previewUrl"
          class="gif-picker__preview-img"
          :src="previewUrl"
          :style="{ objectPosition: posX + '% ' + posY + '%' }"
          draggable="false"
        />
      </div>
      <div class="flex_c_h gap1 dialog-actions">
        <UIButton :disabled="loading" @click="applyGif">{{ $t('gifPicker.apply') }}</UIButton>
        <UIButton v-if="existingGif" @click="removeGif">{{ $t('gifPicker.remove') }}</UIButton>
        <UIButton @click="backToBrowse">{{ $t('gifPicker.cancel') }}</UIButton>
      </div>
    </template>
  </DialogField>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { searchKlipy, trendingKlipy } from '~/utils/klipy'
import {
  MAX_GIF_BYTES,
  upsertGifBlob,
  withProjectDb,
} from '~/utils/db'
import {
  bytesToB64,
  b64ToBytes,
  cacheGifRow,
  detectImageMime,
  extractPosterPng,
  isAnimatedImageMime,
  isLocalImageMime,
  sha256Hex,
} from '~/utils/gifCache'
import { KLIPY_HOME_URL, KLIPY_PARTNER_URL, openInSystemBrowser } from '~/utils/openExternal'

const { t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const step = ref('browse')
const query = ref('')
const items = ref([])
const loading = ref(false)
const loadingMore = ref(false)
const searched = ref(false)
const page = ref(1)
const hasMore = ref(false)
const searchActive = ref(false)
const gridRef = ref(null)
const sentinelRef = ref(null)
const error = ref('')
let searchGen = 0
let io = null
const posX = ref(50)
const posY = ref(50)
const previewUrl = ref('')
const pendingMime = ref('image/gif')
/** Keep bytes off Vue reactivity — Uint8Array + Proxy is a bad mix. */
let pendingBytes = null
let drag = null

const hasKey = computed(() => !!appSettings.klipyApiKey?.trim())
const canLoadMore = computed(() => searchActive.value && hasMore.value)
const targetIndex = computed(() => appStore.gifPickerIndex)
const existingGif = computed(() => {
  const i = targetIndex.value
  if (i == null || i < 0) return false
  return !!jsonStore.configFile.files[i]?.gifId
})

function close() {
  revokePreview()
  appStore.closeGifPicker()
}

function revokePreview() {
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value)
    previewUrl.value = ''
  }
}

async function openKlipy() {
  await openInSystemBrowser(KLIPY_HOME_URL)
}

async function openPartner() {
  await openInSystemBrowser(KLIPY_PARTNER_URL)
}

function applyPage(result, append) {
  const next = result.items || []
  let added = next.length
  if (!append) {
    items.value = next
  } else {
    const seen = new Set(items.value.map((g) => g.id))
    added = 0
    for (const g of next) {
      if (!seen.has(g.id)) {
        seen.add(g.id)
        items.value.push(g)
        added++
      }
    }
  }
  page.value = result.page || page.value
  hasMore.value = !!result.hasNext && next.length > 0 && added > 0
}

async function loadTrending() {
  if (!hasKey.value) return
  const gen = ++searchGen
  loading.value = true
  error.value = ''
  searchActive.value = false
  hasMore.value = false
  page.value = 1
  try {
    const result = await trendingKlipy(appSettings.klipyApiKey, 1)
    if (gen !== searchGen) return
    applyPage(result, false)
    searched.value = true
  } catch (e) {
    if (gen !== searchGen) return
    error.value = String(e)
    items.value = []
  } finally {
    if (gen === searchGen) loading.value = false
  }
}

async function runSearch() {
  if (!hasKey.value) return
  const q = query.value.trim()
  if (!q) {
    await loadTrending()
    return
  }
  const gen = ++searchGen
  loading.value = true
  loadingMore.value = false
  error.value = ''
  searchActive.value = true
  page.value = 1
  try {
    const result = await searchKlipy(appSettings.klipyApiKey, q, 1)
    if (gen !== searchGen) return
    applyPage(result, false)
    searched.value = true
  } catch (e) {
    if (gen !== searchGen) return
    error.value = String(e)
    items.value = []
    hasMore.value = false
  } finally {
    if (gen === searchGen) loading.value = false
  }
}

async function loadMore() {
  if (!canLoadMore.value || loading.value || loadingMore.value || !hasKey.value) return
  const q = query.value.trim()
  if (!q) return
  const gen = searchGen
  const nextPage = page.value + 1
  loadingMore.value = true
  try {
    const result = await searchKlipy(appSettings.klipyApiKey, q, nextPage)
    if (gen !== searchGen) return
    applyPage(result, true)
    page.value = nextPage
  } catch (e) {
    if (gen !== searchGen) return
    hasMore.value = false
    error.value = String(e)
  } finally {
    if (gen === searchGen) loadingMore.value = false
  }
}

function bindSentinel() {
  if (typeof IntersectionObserver === 'undefined') return
  io?.disconnect()
  io = null
  const root = gridRef.value
  const target = sentinelRef.value
  if (!root || !target || !canLoadMore.value) return
  io = new IntersectionObserver((entries) => {
    if (entries.some((e) => e.isIntersecting)) loadMore()
  }, { root, rootMargin: '120px', threshold: 0 })
  io.observe(target)
}

function isSizeLimitError(err) {
  const msg = String(err ?? '')
  return msg.includes('exceeds') && msg.includes('byte limit')
}

async function downloadFirstFit(urls) {
  const list = (urls || []).filter(Boolean)
  let lastErr = null
  let hitLimit = false
  for (const url of list) {
    try {
      const b64 = await invoke('download_url_bytes', { url })
      const bytes = b64ToBytes(b64)
      if (bytes.length > MAX_GIF_BYTES) {
        hitLimit = true
        continue
      }
      return bytes
    } catch (e) {
      lastErr = e
      if (isSizeLimitError(e)) {
        hitLimit = true
        continue
      }
      throw e
    }
  }
  if (hitLimit) {
    const err = new Error('TOO_LARGE')
    err.code = 'TOO_LARGE'
    throw err
  }
  throw lastErr || new Error('download failed')
}

async function selectRemote(g) {
  error.value = ''
  loading.value = true
  try {
    const urls = (g.downloadUrls && g.downloadUrls.length) ? g.downloadUrls : [g.gifUrl]
    const bytes = await downloadFirstFit(urls)
    const mime = detectImageMime(bytes) || 'image/gif'
    beginPosition(bytes, mime)
  } catch (e) {
    if (e?.code === 'TOO_LARGE' || e?.message === 'TOO_LARGE') {
      error.value = t('gifPicker.tooLarge')
    } else {
      error.value = t('gifPicker.downloadFailed') + ' ' + String(e)
    }
  } finally {
    loading.value = false
  }
}

async function pickLocal() {
  error.value = ''
  try {
    const filePath = await open({
      multiple: false,
      title: t('gifPicker.localFile'),
      filters: [{ name: 'Images / GIF', extensions: ['gif', 'webp', 'png', 'jpg', 'jpeg'] }],
    })
    if (!filePath) return
    const b64 = await invoke('read_file_base64_abs', { path: filePath })
    const bytes = b64ToBytes(b64)
    if (bytes.length > MAX_GIF_BYTES) {
      error.value = t('gifPicker.tooLarge')
      return
    }
    const mime = detectImageMime(bytes)
    if (!mime || !isLocalImageMime(mime)) {
      error.value = t('gifPicker.badType')
      return
    }
    beginPosition(bytes, mime)
  } catch (e) {
    error.value = String(e)
  }
}

function beginPosition(bytes, mime) {
  revokePreview()
    pendingBytes = bytes
    pendingMime.value = mime
  previewUrl.value = URL.createObjectURL(new Blob([bytes], { type: mime }))
  const i = targetIndex.value
  const file = i != null ? jsonStore.configFile.files[i] : null
  posX.value = file?.gifPosX ?? 50
  posY.value = file?.gifPosY ?? 50
  step.value = 'position'
}

function onDragStart(e) {
  drag = { x: e.clientX, y: e.clientY, px: posX.value, py: posY.value, el: e.currentTarget }
  e.currentTarget.setPointerCapture?.(e.pointerId)
}

function onDragMove(e) {
  if (!drag) return
  const rect = drag.el.getBoundingClientRect()
  const dx = ((e.clientX - drag.x) / Math.max(1, rect.width)) * 100
  const dy = ((e.clientY - drag.y) / Math.max(1, rect.height)) * 100
  posX.value = Math.max(0, Math.min(100, drag.px - dx))
  posY.value = Math.max(0, Math.min(100, drag.py - dy))
}

function onDragEnd() {
  drag = null
}

function backToBrowse() {
  revokePreview()
  pendingBytes = null
  step.value = 'browse'
}

async function applyGif() {
  const i = targetIndex.value
  const bytes = pendingBytes
  const path = jsonStore.currentProjectPath
  if (i == null || i < 0 || !bytes || !path) return
  error.value = ''
  loading.value = true
  try {
    const mime = pendingMime.value
    const id = await sha256Hex(bytes)
    let posterB64 = null
    if (isAnimatedImageMime(mime)) {
      try {
        const poster = await extractPosterPng(bytes, mime)
        posterB64 = bytesToB64(poster)
      } catch { /* poster optional; hover-off will reuse the GIF */ }
    }
    const row = {
      id,
      mime,
      data: bytesToB64(bytes),
      poster: posterB64,
      byteLen: bytes.length,
    }
    await withProjectDb(path, (d) => upsertGifBlob(d, row))
    await cacheGifRow(row)
    jsonStore.setSoundGif(i, id, posX.value, posY.value)
    close()
  } catch (e) {
    error.value = t('gifPicker.saveFailed') + ' ' + String(e)
  } finally {
    loading.value = false
  }
}

function removeGif() {
  const i = targetIndex.value
  if (i == null || i < 0) return
  jsonStore.setSoundGif(i, null)
  close()
}

watch([sentinelRef, gridRef, canLoadMore, () => items.value.length], () => {
  nextTick(bindSentinel)
})

onMounted(() => {
  const i = targetIndex.value
  const file = i != null ? jsonStore.configFile.files[i] : null
  if (file?.gifId) {
    posX.value = file.gifPosX ?? 50
    posY.value = file.gifPosY ?? 50
  }
  loadTrending()
})

onUnmounted(() => {
  io?.disconnect()
  io = null
  revokePreview()
  pendingBytes = null
})
</script>
