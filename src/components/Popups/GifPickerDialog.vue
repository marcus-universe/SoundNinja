<template>
  <DialogField :title="step === 'position' ? $t('gifPicker.positionTitle') : $t('gifPicker.title')" @close="close">
    <p v-if="error" class="dialog-error">{{ error }}</p>

    <div v-if="step === 'browse'" class="gif-picker">
      <nav class="gif-picker__rail" :aria-label="$t('gifPicker.title')">
        <button
          type="button"
          :class="['gif-picker__rail-item', { active: browseTab === 'local' }]"
          @click="browseTab = 'local'"
        >
          <span class="gif-picker__rail-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"/>
            </svg>
          </span>
          <span class="gif-picker__rail-label">{{ $t('gifPicker.tabLocal') }}</span>
        </button>
        <button
          type="button"
          :class="['gif-picker__rail-item', { active: browseTab === 'klipy' }]"
          @click="browseTab = 'klipy'"
        >
          <span class="gif-picker__rail-icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2"/>
              <circle cx="8.5" cy="8.5" r="1.5"/>
              <path d="M21 15l-5-5L5 21"/>
            </svg>
          </span>
          <span class="gif-picker__rail-label">{{ $t('gifPicker.tabKlipy') }}</span>
        </button>
      </nav>

      <div class="gif-picker__pane">
        <template v-if="browseTab === 'local'">
          <div class="gif-picker__toolbar">
            <input
              class="ui-input gif-picker__search"
              type="search"
              :placeholder="$t('gifPicker.localSearchPlaceholder')"
              v-model="localQuery"
            />
            <UIButton @click="addFolder">{{ $t('gifPicker.addFolder') }}</UIButton>
            <UIButton @click="pickLocal">{{ $t('gifPicker.localFile') }}</UIButton>
          </div>
          <div v-if="folderChips.length" class="gif-picker__chips">
            <span
              v-for="folder in folderChips"
              :key="folder"
              class="gif-picker__chip"
              :title="folder"
            >
              <span class="gif-picker__chip-name">{{ folderName(folder) }}</span>
              <button
                type="button"
                class="gif-picker__chip-remove"
                :aria-label="$t('gifPicker.removeFolder')"
                @click="removeFolder(folder)"
              >
                ×
              </button>
            </span>
          </div>
          <p v-if="localTruncated" class="gif-picker__hint">{{ $t('gifPicker.localTruncated', { count: LOCAL_CAP }) }}</p>
          <div v-if="filteredLocalItems.length" ref="localGridRef" class="gif-picker__grid">
            <div
              v-for="item in filteredLocalItems"
              :key="item.path"
              class="gif-picker__cell gif-picker__cell--local"
              :data-path="item.path"
            >
              <button
                type="button"
                class="gif-picker__cell-pick"
                :title="item.name"
                @click="selectLocal(item.path)"
              >
                <img
                  v-if="localPreviewUrls[item.path]"
                  :src="localPreviewUrls[item.path]"
                  :alt="item.name"
                  draggable="false"
                />
                <span v-else class="gif-picker__cell-ph" />
              </button>
              <button
                type="button"
                class="gif-picker__cell-remove"
                :aria-label="$t('gifPicker.removeFromLibrary')"
                @click="removeLocalItem(item)"
              >
                ×
              </button>
            </div>
          </div>
          <p v-else class="gif-picker__hint">
            {{ localQuery.trim() ? $t('gifPicker.empty') : $t('gifPicker.localEmpty') }}
          </p>
        </template>

        <template v-else>
          <div class="gif-picker__info">
            <p class="gif-picker__hint">{{ hasKey ? $t('gifPicker.info') : $t('gifPicker.noKey') }}</p>
            <div class="gif-picker__info-actions">
              <UIButton @click="openPartner">{{ $t('settings.main.klipyDocs') }}</UIButton>
              <UIButton @click="openKlipySettings">{{ $t('gifPicker.openKlipySettings') }}</UIButton>
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
          <div v-if="items.length" ref="gridRef" class="gif-picker__grid" @scroll.passive="onGridScroll">
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
      </div>
    </div>

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

const LOCAL_CAP = 400
const PREVIEW_LRU = 36

const { t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const step = ref('browse')
const browseTab = ref('local')
const query = ref('')
const items = ref([])
const loading = ref(false)
const loadingMore = ref(false)
const searched = ref(false)
const page = ref(1)
const hasMore = ref(false)
const gridRef = ref(null)
const sentinelRef = ref(null)
const localGridRef = ref(null)
const error = ref('')
let searchGen = 0
let io = null
let localIo = null
let klipyVisited = false
const posX = ref(50)
const posY = ref(50)
const previewUrl = ref('')
const pendingMime = ref('image/gif')
/** Keep bytes off Vue reactivity — Uint8Array + Proxy is a bad mix. */
let pendingBytes = null
let drag = null

const localQuery = ref('')
const localItems = ref([])
const localTruncated = ref(false)
const localPreviewUrls = reactive({})
const previewLoading = new Set()
const previewOrder = []
const reduceMotion = ref(false)
let motionMq = null

const hasKey = computed(() => !!appSettings.klipyApiKey?.trim())
const canLoadMore = computed(() => hasMore.value)
const targetIndex = computed(() => appStore.gifPickerIndex)
const existingGif = computed(() => {
  const i = targetIndex.value
  if (i == null || i < 0) return false
  return !!jsonStore.configFile.files[i]?.gifId
})
const folderChips = computed(() => appSettings.gifLocalFolders)
const filteredLocalItems = computed(() => {
  const q = localQuery.value.trim().toLowerCase()
  if (!q) return localItems.value
  return localItems.value.filter((item) => item.name.toLowerCase().includes(q))
})

function normPath(p) {
  return String(p || '').replace(/\\/g, '/').replace(/\/+$/, '').toLowerCase()
}

function basename(p) {
  const parts = String(p || '').replace(/\\/g, '/').split('/').filter(Boolean)
  return parts[parts.length - 1] || p
}

function folderName(folder) {
  return basename(folder)
}

function pathUnderFolder(file, folder) {
  return normPath(file).startsWith(normPath(folder) + '/')
}

function close() {
  revokePreview()
  revokeAllLocalPreviews()
  appStore.closeGifPicker()
}

function revokePreview() {
  if (previewUrl.value) {
    URL.revokeObjectURL(previewUrl.value)
    previewUrl.value = ''
  }
}

function revokeLocalPreview(path) {
  const u = localPreviewUrls[path]
  if (u) {
    URL.revokeObjectURL(u)
    delete localPreviewUrls[path]
  }
  const i = previewOrder.indexOf(path)
  if (i >= 0) previewOrder.splice(i, 1)
}

function revokeAllLocalPreviews() {
  for (const p of Object.keys(localPreviewUrls)) {
    URL.revokeObjectURL(localPreviewUrls[p])
    delete localPreviewUrls[p]
  }
  previewOrder.length = 0
}

async function openKlipy() {
  await openInSystemBrowser(KLIPY_HOME_URL)
}

async function openPartner() {
  await openInSystemBrowser(KLIPY_PARTNER_URL)
}

function openKlipySettings() {
  close()
  appStore.openSettingsTab('behavior', 'klipyApi')
}

function applyPage(result, append) {
  const next = result.items || []
  if (!append) {
    items.value = next
  } else {
    const seen = new Set(items.value.map((g) => g.id))
    for (const g of next) {
      if (!seen.has(g.id)) {
        seen.add(g.id)
        items.value.push(g)
      }
    }
  }
  page.value = result.page || page.value
  hasMore.value = !!result.hasNext && next.length > 0
}

async function loadTrending() {
  if (!hasKey.value) return
  const gen = ++searchGen
  loading.value = true
  error.value = ''
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
  if (gen === searchGen) await maybeRefill()
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
  if (gen === searchGen) await maybeRefill()
}

function isNearGridBottom() {
  const el = gridRef.value
  if (!el) return false
  return el.scrollTop + el.clientHeight >= el.scrollHeight - 120
}

function onGridScroll() {
  if (isNearGridBottom()) loadMore()
}

async function maybeRefill() {
  await nextTick()
  if (canLoadMore.value && isNearGridBottom()) loadMore()
}

async function loadMore() {
  if (!canLoadMore.value || loading.value || loadingMore.value || !hasKey.value) return
  const q = query.value.trim()
  const gen = searchGen
  const nextPage = page.value + 1
  loadingMore.value = true
  try {
    const result = q
      ? await searchKlipy(appSettings.klipyApiKey, q, nextPage)
      : await trendingKlipy(appSettings.klipyApiKey, nextPage)
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
  if (gen === searchGen) await maybeRefill()
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

function bindLocalPreviews() {
  if (typeof IntersectionObserver === 'undefined') return
  localIo?.disconnect()
  localIo = null
  const root = localGridRef.value
  if (!root) return
  localIo = new IntersectionObserver((entries) => {
    for (const e of entries) {
      if (!e.isIntersecting) continue
      const path = e.target.getAttribute('data-path')
      if (path) loadLocalPreview(path)
    }
  }, { root, rootMargin: '80px', threshold: 0 })
  for (const el of root.querySelectorAll('[data-path]')) localIo.observe(el)
}

async function loadLocalPreview(path) {
  if (localPreviewUrls[path]) {
    const i = previewOrder.indexOf(path)
    if (i >= 0) previewOrder.splice(i, 1)
    previewOrder.push(path)
    return
  }
  if (previewLoading.has(path)) return
  previewLoading.add(path)
  try {
    const b64 = await invoke('read_file_base64_abs', { path })
    const bytes = b64ToBytes(b64)
    if (bytes.length > MAX_GIF_BYTES) return
    const mime = detectImageMime(bytes)
    if (!mime || !isLocalImageMime(mime)) return
    let blobBytes = bytes
    let blobMime = mime
    if (reduceMotion.value && isAnimatedImageMime(mime)) {
      try {
        blobBytes = await extractPosterPng(bytes, mime)
        blobMime = 'image/png'
      } catch { /* keep animated */ }
    }
    while (previewOrder.length >= PREVIEW_LRU) {
      const old = previewOrder.shift()
      if (old && old !== path) revokeLocalPreview(old)
    }
    if (localPreviewUrls[path]) URL.revokeObjectURL(localPreviewUrls[path])
    localPreviewUrls[path] = URL.createObjectURL(new Blob([blobBytes], { type: blobMime }))
    previewOrder.push(path)
  } catch {
    /* skip unreadable files */
  } finally {
    previewLoading.delete(path)
  }
}

async function refreshLocalLibrary() {
  if (!appSettings.loaded) await appSettings.load()
  const hidden = new Set(appSettings.gifLocalHidden.map(normPath))
  const seen = new Set()
  const next = []
  let truncated = false

  for (const file of appSettings.gifLocalFiles) {
    const key = normPath(file)
    if (!key || seen.has(key) || hidden.has(key)) continue
    seen.add(key)
    next.push({ path: file, name: basename(file), extra: true })
    if (next.length >= LOCAL_CAP) {
      truncated = true
      break
    }
  }

  if (!truncated) {
    for (const folder of appSettings.gifLocalFolders) {
      let files = []
      try {
        files = await invoke('list_image_files_abs', { dir: folder })
      } catch {
        continue
      }
      for (const file of files) {
        const key = normPath(file)
        if (!key || seen.has(key) || hidden.has(key)) continue
        seen.add(key)
        next.push({ path: file, name: basename(file), extra: false })
        if (next.length >= LOCAL_CAP) {
          truncated = true
          break
        }
      }
      if (truncated) break
    }
  }

  localItems.value = next
  localTruncated.value = truncated
}

async function addFolder() {
  error.value = ''
  try {
    const dir = await open({
      directory: true,
      multiple: false,
      title: t('gifPicker.addFolder'),
    })
    if (!dir) return
    const path = Array.isArray(dir) ? dir[0] : dir
    if (!path) return
    const key = normPath(path)
    if (appSettings.gifLocalFolders.some((f) => normPath(f) === key)) {
      await refreshLocalLibrary()
      return
    }
    await appSettings.setGifLocalLibrary({
      folders: [...appSettings.gifLocalFolders, path],
    })
    await refreshLocalLibrary()
  } catch (e) {
    error.value = String(e)
  }
}

async function removeFolder(folder) {
  const folders = appSettings.gifLocalFolders.filter((f) => normPath(f) !== normPath(folder))
  const hidden = appSettings.gifLocalHidden.filter((h) => !pathUnderFolder(h, folder))
  await appSettings.setGifLocalLibrary({ folders, hidden })
  for (const item of localItems.value) {
    if (pathUnderFolder(item.path, folder)) revokeLocalPreview(item.path)
  }
  await refreshLocalLibrary()
}

async function removeLocalItem(item) {
  if (item.extra) {
    const files = appSettings.gifLocalFiles.filter((f) => normPath(f) !== normPath(item.path))
    await appSettings.setGifLocalLibrary({ files })
  } else {
    await appSettings.setGifLocalLibrary({
      hidden: [...appSettings.gifLocalHidden, item.path],
    })
  }
  revokeLocalPreview(item.path)
  await refreshLocalLibrary()
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

async function readLocalImage(filePath) {
  const b64 = await invoke('read_file_base64_abs', { path: filePath })
  const bytes = b64ToBytes(b64)
  if (bytes.length > MAX_GIF_BYTES) {
    error.value = t('gifPicker.tooLarge')
    return null
  }
  const mime = detectImageMime(bytes)
  if (!mime || !isLocalImageMime(mime)) {
    error.value = t('gifPicker.badType')
    return null
  }
  return { bytes, mime }
}

async function rememberLocalFile(filePath) {
  const key = normPath(filePath)
  const hidden = appSettings.gifLocalHidden.filter((h) => normPath(h) !== key)
  const wasHidden = hidden.length !== appSettings.gifLocalHidden.length
  const inFolder = appSettings.gifLocalFolders.some((f) => pathUnderFolder(filePath, f))
  const alreadyExtra = appSettings.gifLocalFiles.some((f) => normPath(f) === key)
  if (!wasHidden && (alreadyExtra || inFolder)) return
  const files = alreadyExtra || inFolder
    ? appSettings.gifLocalFiles
    : [...appSettings.gifLocalFiles, filePath]
  await appSettings.setGifLocalLibrary({ files, hidden })
  await refreshLocalLibrary()
}

async function selectLocal(filePath) {
  error.value = ''
  try {
    const img = await readLocalImage(filePath)
    if (!img) return
    beginPosition(img.bytes, img.mime)
  } catch (e) {
    error.value = String(e)
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
    const img = await readLocalImage(filePath)
    if (!img) return
    await rememberLocalFile(filePath)
    beginPosition(img.bytes, img.mime)
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

function onMotionChange(e) {
  reduceMotion.value = !!e.matches
  revokeAllLocalPreviews()
  nextTick(bindLocalPreviews)
}

watch(browseTab, (tab) => {
  if (tab !== 'klipy' || klipyVisited) return
  klipyVisited = true
  loadTrending()
})

watch([sentinelRef, gridRef, canLoadMore, () => items.value.length], () => {
  nextTick(bindSentinel)
})

watch([localGridRef, () => filteredLocalItems.value.length, localQuery], () => {
  nextTick(bindLocalPreviews)
})

onMounted(async () => {
  const i = targetIndex.value
  const file = i != null ? jsonStore.configFile.files[i] : null
  if (file?.gifId) {
    posX.value = file.gifPosX ?? 50
    posY.value = file.gifPosY ?? 50
  }
  if (typeof window !== 'undefined' && window.matchMedia) {
    motionMq = window.matchMedia('(prefers-reduced-motion: reduce)')
    reduceMotion.value = motionMq.matches
    if (motionMq.addEventListener) motionMq.addEventListener('change', onMotionChange)
    else motionMq.addListener?.(onMotionChange)
  }
  await refreshLocalLibrary()
})

onUnmounted(() => {
  io?.disconnect()
  io = null
  localIo?.disconnect()
  localIo = null
  if (motionMq) {
    if (motionMq.removeEventListener) motionMq.removeEventListener('change', onMotionChange)
    else motionMq.removeListener?.(onMotionChange)
  }
  revokePreview()
  revokeAllLocalPreviews()
  pendingBytes = null
})
</script>
