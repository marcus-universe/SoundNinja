<template>
    <div
        class="tab grid_c"
        :class="{ active: isActive }"
        :style="tabColor ? { '--tab-accent': tabColor } : {}"
        :data-tab-name="tabName"
        @click="$emit('select')"
        @contextmenu.prevent="(e) => $emit('contextmenu', e)"
    >
        {{ tabName }}
    </div>
</template>

<script setup>
defineProps({
  tabName: { type: String, required: true },
  isActive: { type: Boolean, default: false },
  tabColor: { type: String, default: '' },
})
defineEmits(['select', 'contextmenu'])
</script>
