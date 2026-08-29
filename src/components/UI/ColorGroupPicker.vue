<template>
  <div class="color-group-picker" :class="{ 'color-group-picker--inline': inline }" ref="rootEl">
    <button
      v-if="!inline"
      type="button"
      class="color-group-picker__trigger"
      :title="title || $t('contextMenu.colors')"
      @click.stop="toggle"
    >
      <span class="color-group-picker__swatch" :style="{ background: swatch }" />
      <span v-if="showLabel" class="color-group-picker__label">{{ title || $t('contextMenu.colors') }}</span>
      <span class="color-group-picker__chevron">{{ open ? '▲' : '▼' }}</span>
    </button>

    <div
      v-if="inline"
      class="color-group-picker__panel"
      @click.stop
    >
      <div
        v-for="row in rows"
        :key="row.key"
        class="color-group-picker__row"
      >
        <label class="color-group-picker__row-label">{{ $t(row.labelKey) }}</label>
        <input
          type="color"
          class="color-group-picker__wheel"
          :value="wheelValue(row.key)"
          @input="onWheel(row.key, $event)"
        />
        <button
          type="button"
          class="color-group-picker__row-reset"
          :title="$t('contextMenu.resetColor')"
          @click="clearKey(row.key)"
        >↺</button>
      </div>
      <div class="color-group-picker__footer">
        <button type="button" class="color-group-picker__reset-all" @click="resetAll">
          {{ $t('contextMenu.resetAllColors') }}
        </button>
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="!inline && open"
        ref="panelEl"
        class="color-group-picker__panel color-group-picker__panel--floating"
        :style="panelStyle"
        @click.stop
      >
        <div
          v-for="row in rows"
          :key="row.key"
          class="color-group-picker__row"
        >
          <label class="color-group-picker__row-label">{{ $t(row.labelKey) }}</label>
          <input
            type="color"
            class="color-group-picker__wheel"
            :value="wheelValue(row.key)"
            @input="onWheel(row.key, $event)"
          />
          <button
            type="button"
            class="color-group-picker__row-reset"
            :title="$t('contextMenu.resetColor')"
            @click="clearKey(row.key)"
          >↺</button>
        </div>
        <div class="color-group-picker__footer">
          <button type="button" class="color-group-picker__reset-all" @click="resetAll">
            {{ $t('contextMenu.resetAllColors') }}
          </button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import {
  type ColorOverride,
  isEmptyOverride,
  overrideSwatch,
} from '~/utils/colorOverride'

const props = withDefaults(defineProps<{
  modelValue?: ColorOverride
  title?: string
  showLabel?: boolean
  /** Render panel inline (no trigger / no floating teleport). */
  inline?: boolean
  /**
   * Floating panel anchor:
   * - auto: below trigger (context menu / default)
   * - bottom-right: SoundContainer corner, above player (multi-select bar)
   */
  placement?: 'auto' | 'bottom-right'
  /** Theme/applied colors shown when a key has no override. */
  baseColors?: ColorOverride
}>(), {
  modelValue: () => ({}),
  showLabel: true,
  inline: false,
  placement: 'auto',
  baseColors: () => ({}),
})

const emit = defineEmits<{
  'update:modelValue': [ColorOverride]
  change: [ColorOverride]
}>()

const open = ref(false)
const rootEl = ref<HTMLElement | null>(null)
const panelEl = ref<HTMLElement | null>(null)
const panelPos = ref({ top: 0, left: 0 })

const rows: { key: keyof ColorOverride; labelKey: string }[] = [
  { key: 'bg', labelKey: 'contextMenu.colorBg' },
  { key: 'bgHover', labelKey: 'contextMenu.colorBgHover' },
  { key: 'text', labelKey: 'contextMenu.colorText' },
  { key: 'textHover', labelKey: 'contextMenu.colorTextHover' },
  { key: 'border', labelKey: 'contextMenu.colorBorder' },
  { key: 'borderHover', labelKey: 'contextMenu.colorBorderHover' },
]

const local = computed(() => props.modelValue || {})
const swatch = computed(() =>
  overrideSwatch(local.value, props.baseColors?.border || props.baseColors?.bg || '#00d4ff'),
)

const panelStyle = computed(() => ({
  position: 'fixed' as const,
  top: `${panelPos.value.top}px`,
  left: `${panelPos.value.left}px`,
  zIndex: 1100,
}))

function wheelValue(key: keyof ColorOverride): string {
  const v = local.value[key]
  if (v && /^#[0-9a-f]{6,8}$/i.test(v)) return v.slice(0, 7)
  const base = props.baseColors?.[key]
  if (base && /^#[0-9a-f]{6,8}$/i.test(base)) return base.slice(0, 7)
  return '#00d4ff'
}

function emitValue(next: ColorOverride) {
  emit('update:modelValue', next)
  emit('change', next)
}

function onWheel(key: keyof ColorOverride, e: Event) {
  const hex = (e.target as HTMLInputElement).value
  emitValue({ ...local.value, [key]: hex })
}

function clearKey(key: keyof ColorOverride) {
  const next = { ...local.value }
  delete next[key]
  emitValue(next)
}

function resetAll() {
  emitValue({})
}

function toggle() {
  open.value = !open.value
  if (open.value) positionPanel()
}

async function positionPanel() {
  await nextTick()
  const el = rootEl.value
  if (!el || typeof window === 'undefined') return

  const panel = panelEl.value
  const panelW = panel?.offsetWidth || 220
  const panelH = panel?.offsetHeight || 280
  const pad = 10

  if (props.placement === 'bottom-right') {
    const host = el.closest('.SoundContainer') as HTMLElement | null
    const rect = (host ?? el).getBoundingClientRect()
    const playerClear = host?.classList.contains('SoundContainer--player-large') ? 92 : 68
    let left = rect.right - panelW - pad
    let top = rect.bottom - panelH - playerClear
    left = Math.min(Math.max(pad, left), window.innerWidth - panelW - pad)
    top = Math.min(Math.max(pad, top), window.innerHeight - panelH - pad)
    panelPos.value = { top, left }
    return
  }

  const rect = el.getBoundingClientRect()
  let left = rect.left
  let top = rect.bottom + 4
  left = Math.min(left, window.innerWidth - panelW - pad)
  top = Math.min(top, window.innerHeight - panelH - pad)
  panelPos.value = { top: Math.max(pad, top), left: Math.max(pad, left) }
}

function onPointerDown(e: PointerEvent) {
  if (props.inline || !open.value) return
  const t = e.target
  if (!(t instanceof Node)) return
  if (rootEl.value?.contains(t)) return
  if (panelEl.value?.contains(t)) return
  open.value = false
}

function onViewportChange() {
  if (open.value) positionPanel()
}

onMounted(() => {
  document.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('resize', onViewportChange)
})
onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('resize', onViewportChange)
})

defineExpose({ open, isEmpty: () => isEmptyOverride(local.value) })
</script>
