<template>
  <div
    class="nslider"
    role="slider"
    tabindex="0"
    :aria-label="label"
    :aria-valuemin="min"
    :aria-valuemax="max"
    :aria-valuenow="modelValue"
    :aria-valuetext="shown"
    @pointerdown="onPointerDown"
    @dblclick.prevent="reset"
    @contextmenu.prevent="reset"
    @keydown="onKey"
  >
    <div class="nslider__fill" :style="{ width: fill }" />
    <span class="nslider__label">{{ label }}</span>
    <input
      v-if="editing"
      ref="inputEl"
      class="nslider__input"
      type="text"
      inputmode="decimal"
      :value="shown"
      @click.stop
      @pointerdown.stop
      @blur="commit"
      @keydown.enter.prevent="commit"
      @keydown.esc.prevent="cancel"
    />
    <span v-else class="nslider__value">{{ shown }}</span>
  </div>
</template>

<script setup lang="ts">
import { clampUiScale } from '~/utils/runtime'

const props = withDefaults(defineProps<{
  label: string
  modelValue: number
  min?: number
  max?: number
  step?: number
  resetValue?: number
}>(), {
  min: 0.5,
  max: 2,
  step: 0.01,
  resetValue: 1,
})

const emit = defineEmits<{ 'update:modelValue': [number] }>()

const editing = ref(false)
const inputEl = ref<HTMLInputElement | null>(null)
const shown = computed(() => Number(props.modelValue).toFixed(2))
const fill = computed(() => {
  const span = props.max - props.min
  if (span <= 0) return '0%'
  const pct = ((props.modelValue - props.min) / span) * 100
  return `${Math.min(100, Math.max(0, pct))}%`
})

function emitValue(n: number) {
  const clamped = Math.min(props.max, Math.max(props.min, n))
  const snapped = Math.round(clamped / props.step) * props.step
  emit('update:modelValue', clampUiScale(snapped))
}

function reset() {
  editing.value = false
  emitValue(props.resetValue)
}

let dragging = false
let moved = false
let startX = 0
let startVal = 0
let dragWidth = 240

function onPointerDown(e: PointerEvent) {
  if (editing.value || e.button !== 0) return
  dragging = true
  moved = false
  startX = e.clientX
  startVal = props.modelValue
  dragWidth = Math.max(1, (e.currentTarget as HTMLElement).clientWidth || 240)
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
  window.addEventListener('pointermove', onPointerMove)
  window.addEventListener('pointerup', onPointerUp)
}

function onPointerMove(e: PointerEvent) {
  if (!dragging) return
  const dx = e.clientX - startX
  if (Math.abs(dx) > 3) moved = true
  const fine = e.shiftKey ? 0.1 : 1
  const span = props.max - props.min
  emitValue(startVal + (dx / dragWidth) * span * fine)
}

function onPointerUp() {
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
  if (!dragging) return
  dragging = false
  if (!moved) beginEdit()
}

async function beginEdit() {
  editing.value = true
  await nextTick()
  inputEl.value?.focus()
  inputEl.value?.select()
}

function commit() {
  const raw = Number(String(inputEl.value?.value || '').replace(',', '.'))
  editing.value = false
  if (Number.isFinite(raw)) emitValue(raw)
}

function cancel() {
  editing.value = false
}

function onKey(e: KeyboardEvent) {
  if (editing.value) return
  const fine = e.shiftKey ? props.step : Math.max(props.step, 0.05)
  if (e.key === 'ArrowRight' || e.key === 'ArrowUp') {
    e.preventDefault()
    emitValue(props.modelValue + fine)
  } else if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') {
    e.preventDefault()
    emitValue(props.modelValue - fine)
  } else if (e.key === 'Home') {
    e.preventDefault()
    emitValue(props.min)
  } else if (e.key === 'End') {
    e.preventDefault()
    emitValue(props.max)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    beginEdit()
  }
}

onUnmounted(() => {
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
})
</script>

<style scoped lang="scss">
.nslider {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  min-height: 2.8rem;
  padding: 0 0.8rem;
  border-radius: var(--radius-sm, 0.4rem);
  background: var(--color-bg-2, #1a1e25);
  border: 0.1rem solid color-mix(in srgb, var(--color-text, #eee) 16%, transparent);
  cursor: ew-resize;
  user-select: none;
  touch-action: none;
  overflow: hidden;

  &:focus-visible {
    outline: 0.2rem solid var(--primary_color);
    outline-offset: 0.2rem;
  }
}

.nslider__fill {
  position: absolute;
  inset: 0 auto 0 0;
  background: color-mix(in srgb, var(--primary_color) 42%, transparent);
  pointer-events: none;
}

.nslider__label,
.nslider__value {
  position: relative;
  z-index: 1;
  font-size: var(--font-size-sm, 1.6rem);
  color: var(--color-text, #eee);
}

.nslider__value {
  font-variant-numeric: tabular-nums;
}

.nslider__input {
  position: relative;
  z-index: 1;
  width: 6rem;
  border: none;
  background: transparent;
  color: var(--color-text, #eee);
  font: inherit;
  font-variant-numeric: tabular-nums;
  text-align: right;
  outline: none;
}
</style>
