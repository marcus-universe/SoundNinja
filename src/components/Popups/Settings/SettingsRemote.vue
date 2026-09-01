<template>
  <section>
    <h2 class="settings-content__title">{{ $t('settings.remote.title') }}</h2>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.remote.enable') }}</span>
        <span class="settings-hint">{{ $t('settings.remote.enableHint') }}</span>
      </div>
      <UICheckbox :modelValue="enabled" @update:modelValue="onEnable" />
    </div>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.remote.port') }}</label>
      <input
        type="number"
        class="settings-input"
        v-model.number="port"
        min="1"
        max="65535"
        @change="onPort"
      />
    </div>

    <div class="settings-group settings-group--stacked">
      <label class="settings-label">{{ $t('settings.remote.token') }}</label>
      <p class="settings-hint">{{ $t('settings.remote.tokenHint') }}</p>
      <div class="settings-row settings-row--nowrap" style="margin-top: 0.5rem; width: 100%">
        <input
          type="text"
          class="settings-input settings-input--wide"
          v-model="token"
          autocomplete="off"
          spellcheck="false"
          @change="onToken"
        />
        <button type="button" class="settings-btn" @click="generateToken">
          {{ $t('settings.remote.generate') }}
        </button>
      </div>
    </div>

    <div class="settings-group settings-group--stacked">
      <span class="settings-label">{{ $t('settings.remote.status') }}</span>
      <p class="settings-hint">
        {{ statusLine }}
      </p>
      <p v-if="errorText" class="settings-error" style="margin-top: 0.4rem">{{ errorText }}</p>
    </div>

    <div class="settings-group settings-group--stacked">
      <span class="settings-label">{{ $t('settings.remote.address') }}</span>
      <p class="settings-hint">{{ $t('settings.remote.firewallHint') }}</p>
      <ul v-if="ips.length" class="remote-addr-list">
        <li v-for="row in ips" :key="row.ip" class="remote-addr">
          <span class="remote-addr__text">
            <span class="remote-addr__url">{{ urlFor(row.ip) }}</span>
            <span v-if="row.primary" class="remote-addr__tag">{{ row.name }}</span>
            <span v-else class="remote-addr__name">{{ row.name }}</span>
          </span>
          <button
            type="button"
            class="settings-btn settings-btn--icon-only"
            :title="$t('settings.remote.copy')"
            @click="copyUrl(row.ip)"
          >
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="settings-btn-icon">
              <rect x="9" y="9" width="13" height="13" rx="2"/>
              <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/>
            </svg>
          </button>
        </li>
      </ul>
      <p v-if="copied" class="settings-success" style="margin-top: 0.4rem">{{ $t('settings.remote.copied') }}</p>
    </div>
  </section>
</template>

<script setup lang="ts">
import { copyText } from '~/utils/clipboard'
import { getLocalIps, remoteStatus, type LocalIp, type RemoteStatus } from '~/utils/remote'

const { t } = useI18n()
const appSettings = useAppSettingsStore()

const enabled = computed({
  get: () => appSettings.remoteEnabled,
  set: (v: boolean) => { onEnable(v) },
})
const port = ref(appSettings.remotePort)
const token = ref(appSettings.remoteToken)
const ips = ref<LocalIp[]>([])
const live = ref<RemoteStatus>({ running: false, port: appSettings.remotePort, clients: 0 })
const copied = ref(false)
let copiedTimer: ReturnType<typeof setTimeout> | null = null
let pollTimer: ReturnType<typeof setInterval> | null = null

const errorText = computed(() => {
  const raw = appSettings.remoteError || ''
  if (!raw) return ''
  const match = raw.match(/Port (\d+) is already in use/i)
  if (match) return t('settings.remote.portInUse', { port: match[1] })
  return raw
})

const statusLine = computed(() => {
  if (!appSettings.remoteEnabled) return t('settings.remote.stopped')
  if (live.value.running) {
    return `${t('settings.remote.running')} — ${t('settings.remote.clients', { count: live.value.clients })}`
  }
  if (appSettings.remoteError) return t('settings.remote.stopped')
  return t('settings.remote.running')
})

function urlFor(ip: string) {
  return `http://${ip}:${appSettings.remotePort}`
}

async function onEnable(v: boolean) {
  try {
    await appSettings.setRemoteEnabled(!!v)
    await refreshStatus()
  } catch { /* remoteError set in store */ }
}

async function onPort() {
  try {
    await appSettings.setRemotePort(port.value)
    await refreshStatus()
  } catch { /* remoteError set in store */ }
}

async function onToken() {
  try {
    await appSettings.setRemoteToken(token.value)
    await refreshStatus()
  } catch { /* remoteError set in store */ }
}

async function generateToken() {
  const buf = new Uint8Array(16)
  crypto.getRandomValues(buf)
  token.value = Array.from(buf, (b) => b.toString(16).padStart(2, '0')).join('')
  await onToken()
}

async function copyUrl(ip: string) {
  const ok = await copyText(urlFor(ip))
  if (!ok) return
  copied.value = true
  if (copiedTimer) clearTimeout(copiedTimer)
  copiedTimer = setTimeout(() => { copied.value = false }, 1600)
}

async function refreshIps() {
  try {
    ips.value = await getLocalIps()
  } catch {
    ips.value = []
  }
}

async function refreshStatus() {
  try {
    live.value = await remoteStatus()
  } catch {
    live.value = { running: false, port: appSettings.remotePort, clients: 0 }
  }
}

onMounted(async () => {
  port.value = appSettings.remotePort
  token.value = appSettings.remoteToken
  await refreshIps()
  await refreshStatus()
  pollTimer = setInterval(refreshStatus, 1000)
})

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer)
  if (copiedTimer) clearTimeout(copiedTimer)
})
</script>
