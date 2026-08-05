<template>
  <DialogField
    v-if="open"
    :title="dialogTitle"
    @close="close"
  >
    <div class="update-dialog flex_c_v align_c gap1">
      <template v-if="phase === 'checking'">
        <p class="dialog-text">{{ $t('updater.checking') }}</p>
      </template>

      <template v-else-if="phase === 'available'">
        <p class="dialog-text">
          {{ $t('updater.available', { current: currentVersion, latest: latestVersion }) }}
        </p>
        <p v-if="notes" class="update-dialog__notes dialog-text">{{ notes }}</p>
        <div class="flex_c_h gap1 dialog-actions">
          <UIButton @click="install">{{ $t('updater.updateNow') }}</UIButton>
          <UIButton @click="close">{{ $t('updater.later') }}</UIButton>
        </div>
      </template>

      <template v-else-if="phase === 'availableManual'">
        <p class="dialog-text">
          {{ $t('updater.available', { current: currentVersion, latest: latestVersion }) }}
        </p>
        <p class="dialog-text">{{ $t('updater.manualHint') }}</p>
        <p v-if="notes" class="update-dialog__notes dialog-text">{{ notes }}</p>
        <div class="flex_c_h gap1 dialog-actions">
          <UIButton @click="openManual">{{ $t('updater.openReleases') }}</UIButton>
          <UIButton @click="close">{{ $t('updater.later') }}</UIButton>
        </div>
      </template>

      <template v-else-if="phase === 'upToDate'">
        <p class="dialog-text">
          {{ $t('updater.upToDate', { version: currentVersion }) }}
        </p>
        <div class="flex_c_h gap1 dialog-actions">
          <UIButton @click="close">{{ $t('dialog.ok') }}</UIButton>
        </div>
      </template>

      <template v-else-if="phase === 'error'">
        <p class="dialog-text dialog-error">{{ errorMessage }}</p>
        <div class="flex_c_h gap1 dialog-actions">
          <UIButton @click="close">{{ $t('dialog.ok') }}</UIButton>
        </div>
      </template>

      <template v-else-if="phase === 'downloading'">
        <p class="dialog-text">{{ $t('updater.downloading') }}</p>
        <div class="update-dialog__bar" role="progressbar" :aria-valuenow="progressPct" aria-valuemin="0" aria-valuemax="100">
          <div class="update-dialog__bar-fill" :style="{ width: `${progressPct}%` }" />
        </div>
        <p class="dialog-text update-dialog__pct">{{ progressPct }}%</p>
      </template>
    </div>
  </DialogField>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Update } from '~/utils/updater'
import {
  checkForAppUpdate,
  getAppVersion,
  installAppUpdate,
  openReleasePage,
} from '~/utils/updater'

type Phase =
  | 'checking'
  | 'available'
  | 'availableManual'
  | 'upToDate'
  | 'error'
  | 'downloading'

const { t } = useI18n()

const open = ref(false)
const phase = ref<Phase>('checking')
const currentVersion = ref('')
const latestVersion = ref('')
const notes = ref('')
const errorMessage = ref('')
const progressPct = ref(0)
const pending = ref<Update | null>(null)
const manualUrl = ref('')
let busy = false

const dialogTitle = computed(() => {
  switch (phase.value) {
    case 'checking':
      return t('updater.titleChecking')
    case 'available':
    case 'availableManual':
      return t('updater.titleAvailable')
    case 'upToDate':
      return t('updater.titleUpToDate')
    case 'downloading':
      return t('updater.titleDownloading')
    default:
      return t('updater.titleError')
  }
})

function close() {
  if (phase.value === 'downloading') return
  open.value = false
  pending.value = null
  manualUrl.value = ''
  busy = false
}

async function runCheck(opts: { forceUi: boolean }) {
  if (busy) return
  busy = true
  currentVersion.value = await getAppVersion()
  latestVersion.value = ''
  notes.value = ''
  errorMessage.value = ''
  progressPct.value = 0
  pending.value = null
  manualUrl.value = ''

  if (opts.forceUi) {
    phase.value = 'checking'
    open.value = true
  }

  try {
    const result = await checkForAppUpdate()
    if (result.status === 'none') {
      if (opts.forceUi) {
        phase.value = 'upToDate'
        open.value = true
      }
      busy = false
      return
    }
    if (result.status === 'available') {
      pending.value = result.update
      latestVersion.value = result.update.version
      notes.value = (result.update.body || '').trim()
      phase.value = 'available'
      open.value = true
      return
    }
    // availableManual
    latestVersion.value = result.version
    notes.value = result.notes
    manualUrl.value = result.url
    phase.value = 'availableManual'
    open.value = true
  } catch (e) {
    if (opts.forceUi) {
      errorMessage.value = e instanceof Error ? e.message : String(e)
      phase.value = 'error'
      open.value = true
    }
  } finally {
    if (phase.value !== 'available' && phase.value !== 'downloading') {
      busy = false
    }
  }
}

async function install() {
  const update = pending.value
  if (!update || phase.value === 'downloading') return
  phase.value = 'downloading'
  progressPct.value = 0
  let downloaded = 0
  let contentLength: number | undefined
  try {
    await installAppUpdate(update, (event) => {
      if (event.event === 'Started') {
        contentLength = event.data.contentLength ?? undefined
        downloaded = 0
        progressPct.value = 0
      } else if (event.event === 'Progress') {
        downloaded += event.data.chunkLength
        if (contentLength && contentLength > 0) {
          progressPct.value = Math.min(99, Math.round((downloaded / contentLength) * 100))
        }
      } else if (event.event === 'Finished') {
        progressPct.value = 100
      }
    })
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : String(e)
    phase.value = 'error'
    busy = false
  }
}

async function openManual() {
  if (!manualUrl.value) return
  try {
    await openReleasePage(manualUrl.value)
  } catch (e) {
    errorMessage.value = e instanceof Error ? e.message : String(e)
    phase.value = 'error'
  }
}

/** Silent start check — popup only when an update exists. */
async function checkOnStart() {
  await runCheck({ forceUi: false })
}

/** Help menu — always show dialog. */
async function checkManual() {
  await runCheck({ forceUi: true })
}

defineExpose({ checkOnStart, checkManual })
</script>

<style scoped lang="scss">
.update-dialog {
  width: 100%;
  max-width: 28rem;
  align-items: center;
  text-align: center;
}

.update-dialog .dialog-actions {
  justify-content: center;
  width: 100%;
}

.update-dialog__notes {
  text-align: left;
  align-self: stretch;
  max-height: 12rem;
  overflow-y: auto;
  white-space: pre-wrap;
  opacity: 0.9;
}

.update-dialog__bar {
  width: 100%;
  height: 0.6rem;
  border-radius: 0.4rem;
  background: color-mix(in srgb, var(--primary_color) 20%, transparent);
  overflow: hidden;
}

.update-dialog__bar-fill {
  height: 100%;
  background: var(--primary_color);
  transition: width 0.15s ease;
}

.update-dialog__pct {
  font-size: var(--font-size-sm);
  opacity: 0.85;
}
</style>
