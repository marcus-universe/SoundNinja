<template>
  <div class="settings-web">
    <h3 class="settings-content__title settings-web__title">{{ $t('settings.remote.web.title') }}</h3>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.remote.web.enable') }}</span>
        <span class="settings-hint">{{ $t('settings.remote.web.enableHint') }}</span>
      </div>
      <UICheckbox :modelValue="enabled" @update:modelValue="onEnable" />
    </div>

    <div class="settings-group">
      <label class="settings-label">{{ $t('settings.remote.web.port') }}</label>
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
      <label class="settings-label">{{ $t('settings.remote.web.token') }}</label>
      <p class="settings-hint">{{ $t('settings.remote.web.tokenHint') }}</p>
      <div class="settings-row settings-row--nowrap" style="margin-top: 0.5rem; width: 100%">
        <input
          type="text"
          class="settings-input settings-input--wide"
          v-model="token"
          autocomplete="off"
          spellcheck="false"
          @change="onToken"
        />
        <button type="button" class="settings-btn" @click="regenerate">
          {{ $t('settings.remote.web.generate') }}
        </button>
      </div>
    </div>

    <div class="settings-group settings-group--stacked">
      <span class="settings-label">{{ $t('settings.remote.web.status') }}</span>
      <p class="settings-hint">{{ statusLine }}</p>
      <p v-if="errorText" class="settings-error" style="margin-top: 0.4rem">{{ errorText }}</p>
    </div>

    <div class="settings-group settings-group--stacked">
      <span class="settings-label">{{ $t('settings.remote.web.address') }}</span>
      <p class="settings-hint">{{ $t('settings.remote.web.security') }}</p>
      <ul v-if="ips.length" class="remote-addr-list">
        <li v-for="row in ips" :key="row.ip" class="remote-addr">
          <button
            type="button"
            class="remote-addr__pick"
            :class="{ 'is-on': row.ip === qrIp }"
            @click="qrIp = row.ip"
          >
            <span class="remote-addr__url">{{ urlFor(row.ip) }}</span>
            <span v-if="row.primary" class="remote-addr__tag">{{ row.name }}</span>
            <span v-else class="remote-addr__name">{{ row.name }}</span>
          </button>
          <button type="button" class="settings-btn" @click="copyUrl(row.ip)">{{ $t('settings.remote.copy') }}</button>
          <button type="button" class="settings-btn" @click="openUrl(row.ip)">{{ $t('settings.remote.web.open') }}</button>
        </li>
      </ul>
      <p v-if="copied" class="settings-success" style="margin-top: 0.4rem">{{ $t('settings.remote.copied') }}</p>
      <div v-if="qrSvg" class="settings-web__qr">
        <p class="settings-label">{{ $t('settings.remote.web.qr') }}</p>
        <div class="settings-web__qr-pad" v-html="qrSvg" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { copyText } from '~/utils/clipboard'
import { openInSystemBrowser } from '~/utils/openExternal'
import { getLocalIps, type LocalIp } from '~/utils/remote'
import { makeWebToken, webuiQrSvg, webuiStatus, type WebuiStatus } from '~/utils/webui'

const { t } = useI18n()
const appSettings = useAppSettingsStore()

const enabled = computed({
  get: () => appSettings.webuiEnabled,
  set: (v: boolean) => { onEnable(v) },
})
const port = ref(appSettings.webuiPort)
const token = ref(appSettings.webuiToken)
const ips = ref<LocalIp[]>([])
const qrIp = ref('')
const qrSvg = ref('')
const live = ref<WebuiStatus>({ running: false, port: appSettings.webuiPort, clients: 0, token: '' })
const copied = ref(false)
let copiedTimer: ReturnType<typeof setTimeout> | null = null
let pollTimer: ReturnType<typeof setInterval> | null = null

const errorText = computed(() => {
  const raw = appSettings.webuiError || ''
  if (!raw) return ''
  const match = raw.match(/Port (\d+) is already in use/i)
  if (match) return t('settings.remote.portInUse', { port: match[1] })
  return raw
})

const statusLine = computed(() => {
  if (!appSettings.webuiEnabled) return t('settings.remote.stopped')
  if (live.value.running) {
    return `${t('settings.remote.running')} — ${t('settings.remote.clients', { count: live.value.clients })}`
  }
  if (appSettings.webuiError) return t('settings.remote.stopped')
  return t('settings.remote.running')
})

function urlFor(ip: string) {
  const code = encodeURIComponent(appSettings.webuiToken || token.value || '')
  const host = ip.includes(':') ? `[${ip}]` : ip
  return `http://${host}:${appSettings.webuiPort}/?t=${code}`
}

async function refreshQr() {
  const ip = qrIp.value || ips.value.find((row) => row.primary)?.ip || ips.value[0]?.ip || ''
  qrIp.value = ip
  if (!ip || !appSettings.webuiEnabled) {
    qrSvg.value = ''
    return
  }
  try { qrSvg.value = await webuiQrSvg(urlFor(ip)) } catch { qrSvg.value = '' }
}

async function onEnable(v: boolean) {
  try {
    if (v && !token.value.trim()) token.value = makeWebToken()
    await appSettings.setWebuiEnabled(!!v)
    if (appSettings.webuiToken) token.value = appSettings.webuiToken
    await refreshStatus()
    await refreshQr()
  } catch { /* webuiError set in store */ }
}

async function onPort() {
  try {
    await appSettings.setWebuiPort(port.value)
    await refreshStatus()
    await refreshQr()
  } catch { /* webuiError set in store */ }
}

async function onToken() {
  try {
    await appSettings.setWebuiToken(token.value)
    await refreshStatus()
    await refreshQr()
  } catch { /* webuiError set in store */ }
}

async function regenerate() {
  token.value = makeWebToken()
  await onToken()
}

async function copyUrl(ip: string) {
  const ok = await copyText(urlFor(ip))
  if (!ok) return
  copied.value = true
  if (copiedTimer) clearTimeout(copiedTimer)
  copiedTimer = setTimeout(() => { copied.value = false }, 1600)
}

function openUrl(ip: string) {
  openInSystemBrowser(urlFor(ip)).catch(() => {})
}

async function refreshIps() {
  try { ips.value = await getLocalIps() } catch { ips.value = [] }
  if (!qrIp.value) qrIp.value = ips.value.find((row) => row.primary)?.ip || ips.value[0]?.ip || ''
}

async function refreshStatus() {
  try { live.value = await webuiStatus() } catch {
    live.value = { running: false, port: appSettings.webuiPort, clients: 0, token: appSettings.webuiToken }
  }
}

watch(qrIp, () => { refreshQr() })

onMounted(async () => {
  port.value = appSettings.webuiPort
  token.value = appSettings.webuiToken
  await refreshIps()
  await refreshStatus()
  await refreshQr()
  pollTimer = setInterval(refreshStatus, 1000)
})

onUnmounted(() => {
  if (pollTimer) clearInterval(pollTimer)
  if (copiedTimer) clearTimeout(copiedTimer)
})
</script>

<style scoped lang="scss">
.settings-web {
  margin-top: 2.4rem;
  padding-top: 1.6rem;
  border-top: 0.1rem solid color-mix(in srgb, var(--color-text, #eee) 12%, transparent);
}

.settings-web__title {
  margin-bottom: 1.2rem;
}

.remote-addr__pick {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.2rem;
  padding: 0.4rem 0.6rem;
  border: 0.1rem solid transparent;
  border-radius: var(--radius-sm, 0.4rem);
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;

  &.is-on {
    border-color: var(--primary_color);
  }
}

.settings-web__qr {
  margin-top: 1.2rem;
}

.settings-web__qr-pad {
  margin-top: 0.6rem;
  width: 22rem;
  max-width: 100%;
  padding: 0.8rem;
  background: #fff;
  border-radius: var(--radius-md, 0.64rem);

  :deep(svg) {
    display: block;
    width: 100%;
    height: auto;
  }
}
</style>
