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
import { readableTextColor } from '~/utils/contrast'

const props = defineProps({
  tabName: { type: String, required: true },
  isActive: { type: Boolean, default: false },
  tabColor: { type: String, default: '' },
})
defineEmits(['select', 'contextmenu'])

const jsonStore = useJsonHandelingStore()

// Resolve the tab's accent + a readable text color for its solid (active) state.
const tabStyle = computed(() => {
  let accent = props.tabColor
  if (!accent && typeof document !== 'undefined') {
    accent = getComputedStyle(document.documentElement).getPropertyValue('--primary_color').trim()
  }
  accent = accent || '#00d4ff'
  const s = jsonStore.configFile?.settings
  const tl = s?.textLight || '#eeeeee'
  const td = s?.textDark || '#222831'
  return {
    '--tab-accent': accent,
    '--tab-text-active': readableTextColor(accent, tl, td),
  }
})
</script>
