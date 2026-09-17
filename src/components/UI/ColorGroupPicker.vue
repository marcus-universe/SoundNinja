<template>
  <div
    class="color-group-picker"
    :class="{
      'color-group-picker--inline': inline,
      'color-group-picker--row': layout === 'row',
    }"
    ref="rootEl"
  >
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
      <div class="color-group-picker__hue" :title="$t('contextMenu.colorHueHint')">
        <div class="color-group-picker__hue-head">
          <span>{{ $t('contextMenu.colorHue') }}</span>
          <span>{{ displayHue }}°</span>
        </div>
        <HueSlider
          :model-value="displayHue"
          :hint="$t('contextMenu.colorHueHint')"
          :aria-label="$t('contextMenu.colorHue')"
          @drag-start="beginHueDrag"
          @update:model-value="onHue"
          @drag-end="onHueDragEnd"
        />
      </div>
      <div v-if="appSettings.basicButtonColors" class="color-group-picker__scheme" role="group">
        <button
          type="button"
          class="color-group-picker__scheme-btn"
          :class="{ 'is-active': scheme === 'dark' }"
          @click="setScheme('dark')"
        >{{ $t('contextMenu.colorSchemeDark') }}</button>
        <button
          type="button"
          class="color-group-picker__scheme-btn"
          :class="{ 'is-active': scheme === 'light' }"
          @click="setScheme('light')"
        >{{ $t('contextMenu.colorSchemeLight') }}</button>
      </div>
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
          @change="flushEmit"
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

    <Teleport v-else-if="layout === 'row'" defer to="[data-bulk-palette]">
      <div
        class="color-group-picker__expand"
        :class="{ 'color-group-picker__expand--open': open }"
      >
        <div class="color-group-picker__expand-inner">
          <div class="color-group-picker__panel color-group-picker__panel--row" @click.stop>
            <div class="color-group-picker__hue" :title="$t('contextMenu.colorHueHint')">
              <div class="color-group-picker__hue-head">
                <span>{{ $t('contextMenu.colorHue') }}</span>
                <span>{{ displayHue }}°</span>
              </div>
              <HueSlider
                :model-value="displayHue"
                :hint="$t('contextMenu.colorHueHint')"
                :aria-label="$t('contextMenu.colorHue')"
                @drag-start="beginHueDrag"
                @update:model-value="onHue"
                @drag-end="onHueDragEnd"
              />
            </div>
            <div v-if="appSettings.basicButtonColors" class="color-group-picker__scheme" role="group">
              <button
                type="button"
                class="color-group-picker__scheme-btn"
                :class="{ 'is-active': scheme === 'dark' }"
                @click="setScheme('dark')"
              >{{ $t('contextMenu.colorSchemeDark') }}</button>
              <button
                type="button"
                class="color-group-picker__scheme-btn"
                :class="{ 'is-active': scheme === 'light' }"
                @click="setScheme('light')"
              >{{ $t('contextMenu.colorSchemeLight') }}</button>
            </div>
            <div
              v-for="row in rows"
              :key="row.key"
              class="color-group-picker__row"
            >
              <label class="color-group-picker__row-label">{{ $t(row.labelKey) }}</label>
              <div class="color-group-picker__row-tools">
                <input
                  type="color"
                  class="color-group-picker__wheel"
                  :value="wheelValue(row.key)"
                  @input="onWheel(row.key, $event)"
                  @change="flushEmit"
                />
                <button
                  type="button"
                  class="color-group-picker__row-reset"
                  :title="$t('contextMenu.resetColor')"
                  @click="clearKey(row.key)"
                >↺</button>
              </div>
            </div>
            <div class="color-group-picker__footer">
              <button type="button" class="color-group-picker__reset-all" @click="resetAll">
                {{ $t('contextMenu.resetAllColors') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>

    <Teleport v-else to="body">
      <div
        v-if="open"
        ref="panelEl"
        class="color-group-picker__panel color-group-picker__panel--floating"
        :style="panelStyle"
        @click.stop
      >
        <div class="color-group-picker__hue" :title="$t('contextMenu.colorHueHint')">
          <div class="color-group-picker__hue-head">
            <span>{{ $t('contextMenu.colorHue') }}</span>
            <span>{{ displayHue }}°</span>
          </div>
          <HueSlider
            :model-value="displayHue"
            :hint="$t('contextMenu.colorHueHint')"
            :aria-label="$t('contextMenu.colorHue')"
            @drag-start="beginHueDrag"
            @update:model-value="onHue"
            @drag-end="onHueDragEnd"
          />
        </div>
        <div v-if="appSettings.basicButtonColors" class="color-group-picker__scheme" role="group">
          <button
            type="button"
            class="color-group-picker__scheme-btn"
            :class="{ 'is-active': scheme === 'dark' }"
            @click="setScheme('dark')"
          >{{ $t('contextMenu.colorSchemeDark') }}</button>
          <button
            type="button"
            class="color-group-picker__scheme-btn"
            :class="{ 'is-active': scheme === 'light' }"
            @click="setScheme('light')"
          >{{ $t('contextMenu.colorSchemeLight') }}</button>
        </div>
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
            @change="flushEmit"
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
import { leadHue, shiftColorRecord } from '~/utils/hue'
import {
  applyBasicPick,
  applyScheme,
  inferScheme,
  type ColorScheme,
} from '~/utils/monoPalette'

const appSettings = useAppSettingsStore()

const props = withDefaults(defineProps<{
  modelValue?: ColorOverride
  title?: string
  showLabel?: boolean
  /** Render panel inline (no trigger / no floating teleport). */
  inline?: boolean
  /**
   * stack: vertical rows (context menu / floating).
   * row: in-flow horizontal fields (multi-select bulk bar).
   */
  layout?: 'stack' | 'row'
  /**
   * Floating panel anchor:
   * - auto: below trigger (context menu / default)
   * - bottom-right: SoundContainer corner, above player (legacy)
   */
  placement?: 'auto' | 'bottom-right'
  /** Theme/applied colors shown when a key has no override. */
  baseColors?: ColorOverride
}>(), {
  modelValue: () => ({}),
  showLabel: true,
  inline: false,
  layout: 'stack',
  placement: 'auto',
  baseColors: () => ({}),
})

const emit = defineEmits<{
  'update:modelValue': [ColorOverride]
  change: [ColorOverride]
  preview: [ColorOverride]
}>()

const open = defineModel<boolean>('open', { default: false })

const rootEl = ref<HTMLElement | null>(null)
const panelEl = ref<HTMLElement | null>(null)
const panelPos = ref({ top: 0, left: 0 })

const ALL_KEYS: (keyof ColorOverride)[] = ['bg', 'bgHover', 'text', 'textHover', 'border', 'borderHover']

const FULL_ROWS: { key: keyof ColorOverride; labelKey: string }[] = [
  { key: 'bg', labelKey: 'contextMenu.colorBg' },
  { key: 'bgHover', labelKey: 'contextMenu.colorBgHover' },
  { key: 'text', labelKey: 'contextMenu.colorText' },
  { key: 'textHover', labelKey: 'contextMenu.colorTextHover' },
  { key: 'border', labelKey: 'contextMenu.colorBorder' },
  { key: 'borderHover', labelKey: 'contextMenu.colorBorderHover' },
]

const BASIC_ROWS: { key: keyof ColorOverride; labelKey: string }[] = [
  { key: 'text', labelKey: 'contextMenu.colorBase' },
]

const rows = computed(() => (appSettings.basicButtonColors ? BASIC_ROWS : FULL_ROWS))

const draft = ref<ColorOverride | null>(null)
const local = computed(() => draft.value ?? (props.modelValue || {}))
const scheme = computed(() => inferScheme(local.value))
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

function effectiveHex(key: keyof ColorOverride): string {
  const v = local.value[key]
  if (v && /^#[0-9a-f]{3,8}$/i.test(v)) return v
  const base = props.baseColors?.[key]
  if (base && /^#[0-9a-f]{3,8}$/i.test(base)) return base
  return wheelValue(key)
}

function snapshotColors(): ColorOverride {
  const out: ColorOverride = {}
  for (const key of ALL_KEYS) out[key] = effectiveHex(key)
  return out
}

const HUE_LEAD: (keyof ColorOverride)[] = ['border', 'bg', 'bgHover', 'borderHover', 'textHover', 'text']

const hueDragging = ref(false)
const hueDragValue = ref(0)

const displayHue = computed(() =>
  hueDragging.value ? hueDragValue.value : leadHue(HUE_LEAD.map((k) => effectiveHex(k))),
)

let hueSnap: ColorOverride | null = null
let hueSnapHue = 0

function beginHueDrag() {
  hueSnap = snapshotColors()
  hueSnapHue = leadHue(HUE_LEAD.map((k) => hueSnap![k]))
  hueDragValue.value = hueSnapHue
  hueDragging.value = true
}

function endHueDrag() {
  hueSnap = null
  hueDragging.value = false
}

function onHue(next: number) {
  if (!hueSnap) beginHueDrag()
  hueDragging.value = true
  hueDragValue.value = next
  emitValue(shiftColorRecord(hueSnap!, next - hueSnapHue))
}

function onHueDragEnd() {
  endHueDrag()
  flushEmit()
}

let rafId = 0
let pendingOverride: ColorOverride | null = null
let lastLive: ColorOverride | null = null

function firePreview(next: ColorOverride) {
  draft.value = next
  emit('preview', next)
}

function fireCommit(next: ColorOverride) {
  draft.value = next
  lastLive = next
  emit('update:modelValue', next)
  emit('change', next)
}

function emitNow(next: ColorOverride) {
  pendingOverride = null
  lastLive = next
  if (rafId) {
    cancelAnimationFrame(rafId)
    rafId = 0
  }
  fireCommit(next)
}

function flushEmit() {
  if (rafId) {
    cancelAnimationFrame(rafId)
    rafId = 0
  }
  const v = pendingOverride || lastLive
  pendingOverride = null
  if (v) fireCommit(v)
}

function emitValue(next: ColorOverride) {
  pendingOverride = next
  lastLive = next
  draft.value = next
  if (rafId) return
  rafId = requestAnimationFrame(() => {
    rafId = 0
    const v = pendingOverride
    pendingOverride = null
    if (v) firePreview(v)
  })
}

function setScheme(next: ColorScheme) {
  if (!appSettings.basicButtonColors) return
  if (scheme.value === next) return
  emitNow(applyScheme(local.value, next, effectiveHex('text')))
}

function onWheel(key: keyof ColorOverride, e: Event) {
  const hex = (e.target as HTMLInputElement).value
  if (appSettings.basicButtonColors && key === 'text') {
    emitValue(applyBasicPick(local.value, hex, scheme.value))
    return
  }
  emitValue({ ...local.value, [key]: hex })
}

function clearKey(key: keyof ColorOverride) {
  const next = { ...local.value }
  if (appSettings.basicButtonColors && key === 'text') {
    delete next.text
    delete next.bg
    delete next.border
    delete next.textHover
    delete next.bgHover
    delete next.borderHover
  } else {
    delete next[key]
  }
  emitNow(next)
}

function resetAll() {
  emitNow({})
}

function toggle() {
  open.value = !open.value
  if (open.value && props.layout !== 'row') positionPanel()
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
  if (props.inline || props.layout === 'row' || !open.value) return
  const t = e.target
  if (!(t instanceof Node)) return
  if (rootEl.value?.contains(t)) return
  if (panelEl.value?.contains(t)) return
  open.value = false
}

function onViewportChange() {
  if (open.value && props.layout !== 'row') positionPanel()
}

onMounted(() => {
  document.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('resize', onViewportChange)
})
onBeforeUnmount(() => {
  flushEmit()
  document.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('resize', onViewportChange)
})

defineExpose({ open, isEmpty: () => isEmptyOverride(local.value) })
</script>
