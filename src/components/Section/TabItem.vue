<template>
    <div
        class="tab grid_c"
        :class="{ active: isActive }"
        :style="tabStyle"
        :data-tab-name="tabName"
        @click="$emit('select')"
        @contextmenu.prevent="(e) => $emit('contextmenu', e)"
    >
        {{ tabName }}
    </div>
</template>

<script setup>
import { parseOverride } from '~/utils/colorOverride'
import { withAlpha } from '~/utils/themeTokens'

const props = defineProps({
  tabName: { type: String, required: true },
  isActive: { type: Boolean, default: false },
  tabColor: { type: String, default: '' },
})
defineEmits(['select', 'contextmenu'])

// Resolve per-tab override → --tab-* CSS vars (falls back to theme tokens).
const tabStyle = computed(() => {
  const o = parseOverride(props.tabColor)
  const style = {}
  // Legacy single-hex override: treat as accent → bg tint + border.
  if (o.border && !o.bg && !o.text) {
    style['--tab-bg'] = withAlpha(o.border, 0.2)
    style['--tab-bg-hover'] = withAlpha(o.border, 0.4)
    style['--tab-border'] = o.border
    style['--tab-border-hover'] = o.border
    style['--tab-text-active'] = '#eeeeee'
  } else {
    if (o.bg) style['--tab-bg'] = o.bg
    if (o.bgHover) style['--tab-bg-hover'] = o.bgHover
    if (o.text) {
      style['--tab-text'] = o.text
      style['--tab-text-active'] = o.text
    }
    if (o.textHover) style['--tab-text-hover'] = o.textHover
    if (o.border) style['--tab-border'] = o.border
    if (o.borderHover) style['--tab-border-hover'] = o.borderHover
  }
  return style
})
</script>
