<template>
  <section>
    <h2 class="settings-content__title">{{ $t('settings.tabs.hotkeys') }}</h2>

    <div class="settings-section-divider">{{ $t('settings.hotkeys.actions') }}</div>
    <p class="settings-hint settings-hotkeys__hint">{{ $t('settings.hotkeys.actionsHint') }}</p>

    <div
      v-for="action in actions"
      :key="action"
      class="settings-group settings-hotkeys__row"
    >
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t(`settings.hotkeys.action.${action}`) }}</span>
      </div>
      <button
        type="button"
        class="settings-hotkeys__capture"
        :class="{ 'is-capturing': capturing === action }"
        @click="startCapture(action)"
      >
        {{ captureLabel(action) }}
      </button>
    </div>

    <div class="settings-section-divider">{{ $t('settings.hotkeys.triggers') }}</div>
    <p class="settings-hint settings-hotkeys__hint">{{ $t('settings.hotkeys.triggersHint') }}</p>

    <div class="settings-group settings-group--toggle">
      <div class="settings-toggle-text">
        <span class="settings-label">{{ $t('settings.hotkeys.globalTriggers') }}</span>
        <span class="settings-hint">{{ $t('settings.hotkeys.globalTriggersHint') }}</span>
      </div>
      <UICheckbox :modelValue="soundTriggersGlobal" @update:modelValue="onGlobalToggle" />
    </div>

    <div
      v-for="row in soundHotkeys"
      :key="row.id"
      class="settings-hotkeys__trigger"
    >
      <input
        class="settings-input settings-hotkeys__id"
        :value="row.soundId"
        :placeholder="$t('settings.hotkeys.soundIdPlaceholder')"
        maxlength="8"
        spellcheck="false"
        @change="onSoundIdInput(row, $event)"
      />
      <span class="settings-hotkeys__name" :class="{ 'is-missing': !soundName(row.soundId) }">
        {{ soundName(row.soundId) || $t('settings.hotkeys.unknownId') }}
      </span>
      <button
        type="button"
        class="settings-hotkeys__capture"
        :class="{ 'is-capturing': capturing === row.id }"
        @click="startCapture(row.id)"
      >
        {{ triggerCaptureLabel(row) }}
      </button>
      <button type="button" class="settings-hotkeys__remove" @click="removeTrigger(row.id)">
        {{ $t('settings.hotkeys.remove') }}
      </button>
    </div>

    <div class="settings-group">
      <UIButton @click="addTrigger">{{ $t('settings.hotkeys.addTrigger') }}</UIButton>
    </div>

    <p v-if="conflictHint" class="settings-hint settings-hotkeys__conflict">{{ conflictHint }}</p>
  </section>
</template>

<script setup lang="ts">
import {
  APP_HOTKEY_ACTIONS,
  eventToCombo,
  newHotkeyRowId,
  type AppHotkeyAction,
  type SoundHotkey,
} from '~/utils/hotkeys'

const { t } = useI18n()
const appStore = useAppStore()
const appSettings = useAppSettingsStore()
const jsonStore = useJsonHandelingStore()

const actions = APP_HOTKEY_ACTIONS
const capturing = ref<string | null>(null)
const conflictHint = ref('')

const soundTriggersGlobal = computed(() => appSettings.soundTriggersGlobal)
const soundHotkeys = computed<SoundHotkey[]>(() => jsonStore.configFile.settings.soundHotkeys ?? [])

function soundName(soundId: string): string {
  if (!soundId) return ''
  return jsonStore.configFile.files.find((f) => f.id === soundId)?.name || ''
}

function captureLabel(action: AppHotkeyAction): string {
  if (capturing.value === action) return t('settings.hotkeys.pressKey')
  return appSettings.hotkeys[action] || t('settings.hotkeys.unset')
}

function triggerCaptureLabel(row: SoundHotkey): string {
  if (capturing.value === row.id) return t('settings.hotkeys.pressKey')
  return row.combo || t('settings.hotkeys.unset')
}

function startCapture(target: string) {
  capturing.value = target
  conflictHint.value = ''
}

function clearCombo(target: string) {
  if (APP_HOTKEY_ACTIONS.includes(target as AppHotkeyAction)) {
    appSettings.setAppHotkey(target as AppHotkeyAction, '')
  } else {
    writeTriggers(soundHotkeys.value.map((r) => r.id === target ? { ...r, combo: '' } : r))
  }
}

function comboOwner(combo: string, except: string): string | null {
  if (!combo) return null
  for (const action of APP_HOTKEY_ACTIONS) {
    if (action !== except && appSettings.hotkeys[action] === combo) return action
  }
  for (const row of soundHotkeys.value) {
    if (row.id !== except && row.combo === combo) return row.soundId || row.id
  }
  return null
}

function applyCombo(target: string, combo: string) {
  const owner = comboOwner(combo, target)
  if (owner) {
    for (const action of APP_HOTKEY_ACTIONS) {
      if (appSettings.hotkeys[action] === combo) appSettings.setAppHotkey(action, '')
    }
    writeTriggers(soundHotkeys.value.map((r) => r.combo === combo && r.id !== target ? { ...r, combo: '' } : r))
    conflictHint.value = t('settings.hotkeys.conflict', { combo })
  }
  if (APP_HOTKEY_ACTIONS.includes(target as AppHotkeyAction)) {
    appSettings.setAppHotkey(target as AppHotkeyAction, combo)
  } else {
    writeTriggers(soundHotkeys.value.map((r) => r.id === target ? { ...r, combo } : r))
  }
}

function onCaptureKey(e: KeyboardEvent) {
  if (!capturing.value) return
  if (e.key === 'Escape') {
    e.preventDefault()
    capturing.value = null
    return
  }
  if (e.key === 'Backspace' || e.key === 'Delete') {
    e.preventDefault()
    clearCombo(capturing.value)
    capturing.value = null
    return
  }
  const combo = eventToCombo(e)
  if (!combo) return
  e.preventDefault()
  e.stopPropagation()
  applyCombo(capturing.value, combo)
  capturing.value = null
}

function writeTriggers(next: SoundHotkey[]) {
  jsonStore.setSetting('soundHotkeys', next)
}

function addTrigger() {
  const pending = appStore.consumePendingHotkeySoundId()
  writeTriggers([
    ...soundHotkeys.value,
    { id: newHotkeyRowId(), soundId: pending || '', combo: '' },
  ])
}

function removeTrigger(id: string) {
  writeTriggers(soundHotkeys.value.filter((r) => r.id !== id))
}

function onSoundIdInput(row: SoundHotkey, e: Event) {
  const val = (e.target as HTMLInputElement).value.toLowerCase().replace(/[^a-z0-9]/g, '').slice(0, 8)
  writeTriggers(soundHotkeys.value.map((r) => r.id === row.id ? { ...r, soundId: val } : r))
}

function onGlobalToggle(val: boolean) {
  appSettings.setSoundTriggersGlobal(val)
}

function consumePending() {
  const id = appStore.pendingHotkeySoundId
  if (!id) return
  const existing = soundHotkeys.value.find((r) => r.soundId === id)
  if (existing) {
    appStore.consumePendingHotkeySoundId()
    capturing.value = existing.id
    return
  }
  addTrigger()
}

watch(() => appStore.activeOverlay, (val) => {
  if (val === 'settings') consumePending()
})
watch(() => appStore.pendingHotkeySoundId, (id) => {
  if (id && appStore.activeOverlay === 'settings') consumePending()
})

onMounted(() => {
  window.addEventListener('keydown', onCaptureKey, true)
  consumePending()
})

onUnmounted(() => {
  window.removeEventListener('keydown', onCaptureKey, true)
})
</script>
