<template>
  <select
    v-if="!linux"
    :id="inputId"
    class="settings-select"
    :class="selectClass"
    :value="modelValue"
    :title="title"
    :aria-label="ariaLabel"
    :disabled="disabled"
    @change="onNative"
  >
    <template v-for="(row, index) in rows" :key="index">
      <optgroup v-if="row.kind === 'group'" :label="row.label">
        <option v-for="opt in row.options" :key="opt.value + opt.label" :value="opt.value">{{ opt.label }}</option>
      </optgroup>
      <option v-else :value="row.value">{{ row.label }}</option>
    </template>
  </select>

  <div
    v-else
    ref="rootEl"
    class="ui-select"
    :class="selectClass"
    @keydown="onKey"
  >
    <button
      :id="inputId"
      type="button"
      class="ui-select__trigger"
      :title="title"
      :aria-label="ariaLabel"
      :aria-expanded="open"
      aria-haspopup="listbox"
      :disabled="disabled"
      @click="toggle"
    >
      <span>{{ currentLabel }}</span>
      <span class="ui-select__arrow" aria-hidden="true">▾</span>
    </button>
    <div v-show="open" class="ui-select__list" role="listbox">
      <template v-for="(row, index) in rows" :key="index">
        <div v-if="row.kind === 'group'" class="ui-select__group">
          <div class="ui-select__sep">{{ row.label }}</div>
          <button
            v-for="opt in row.options"
            :key="opt.value + opt.label"
            type="button"
            class="ui-select__option"
            :class="{ active: opt.value === modelValue, 'is-key': opt.value === highlight }"
            role="option"
            :aria-selected="opt.value === modelValue"
            @click="pick(opt.value)"
          >{{ opt.label }}</button>
        </div>
        <button
          v-else
          type="button"
          class="ui-select__option"
          :class="{ active: row.value === modelValue, 'is-key': row.value === highlight }"
          role="option"
          :aria-selected="row.value === modelValue"
          @click="pick(row.value)"
        >{{ row.label }}</button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ensureOs, osName } from '~/utils/osName'

export type UISelectOption = {
  value: string
  label: string
  group?: string
}

const props = withDefaults(defineProps<{
  modelValue: string
  options: UISelectOption[]
  ariaLabel?: string
  title?: string
  inputId?: string
  selectClass?: string | string[]
  disabled?: boolean
}>(), {
  ariaLabel: '',
  title: '',
  inputId: undefined,
  selectClass: undefined,
  disabled: false,
})

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

ensureOs()

const linux = computed(() => osName.value === 'linux')

type OptionRow = { kind: 'option'; value: string; label: string }
type GroupRow = { kind: 'group'; label: string; options: { value: string; label: string }[] }

const rows = computed(() => {
  const out: Array<OptionRow | GroupRow> = []
  let group: GroupRow | null = null
  for (const opt of props.options) {
    if (!opt.group) {
      group = null
      out.push({ kind: 'option', value: opt.value, label: opt.label })
      continue
    }
    if (!group || group.label !== opt.group) {
      group = { kind: 'group', label: opt.group, options: [] }
      out.push(group)
    }
    group.options.push({ value: opt.value, label: opt.label })
  }
  return out
})

const currentLabel = computed(() => {
  const hit = props.options.find((opt) => opt.value === props.modelValue)
  return hit?.label || props.modelValue || ''
})

const open = ref(false)
const rootEl = ref<HTMLElement | null>(null)
const highlight = ref('')

function toggle() {
  open.value = !open.value
  if (open.value) highlight.value = props.modelValue
}

function pick(value: string) {
  open.value = false
  emit('update:modelValue', value)
}

function onNative(event: Event) {
  emit('update:modelValue', (event.target as HTMLSelectElement).value)
}

function onKey(event: KeyboardEvent) {
  if (!linux.value || props.disabled) return
  if (event.key === 'Escape') {
    open.value = false
    return
  }
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    if (!open.value) {
      toggle()
      return
    }
    const opts = props.options
    if (!opts.length) return
    const dir = event.key === 'ArrowDown' ? 1 : -1
    let index = opts.findIndex((opt) => opt.value === highlight.value)
    if (index < 0) index = 0
    else index = (index + dir + opts.length) % opts.length
    highlight.value = opts[index].value
    return
  }
  if (event.key === 'Enter' || event.key === ' ') {
    event.preventDefault()
    if (!open.value) {
      toggle()
      return
    }
    const opt = props.options.find((item) => item.value === highlight.value) || props.options[0]
    if (opt) pick(opt.value)
  }
}

function onDocPointerDown(event: PointerEvent) {
  const root = rootEl.value
  if (!root || !open.value) return
  if (event.target instanceof Node && !root.contains(event.target)) open.value = false
}

watch(open, (isOpen) => {
  if (isOpen) document.addEventListener('pointerdown', onDocPointerDown, true)
  else document.removeEventListener('pointerdown', onDocPointerDown, true)
})

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onDocPointerDown, true)
})
</script>
