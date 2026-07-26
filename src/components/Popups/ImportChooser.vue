<template>
  <DialogField
    v-if="appStore.importChooserActive"
    :title="$t('importChooser.title')"
    @close="close"
  >
    <p class="dialog-text">{{ $t('importChooser.message') }}</p>
    <div class="flex_c_v gap1 dialog-actions">
      <UIButton :full-width="true" @click="choose('audio')">
        {{ $t('importChooser.audioFiles') }}
      </UIButton>
      <UIButton :full-width="true" @click="choose('folders')">
        {{ $t('importChooser.folders') }}
      </UIButton>
      <UIButton :full-width="true" @click="close">
        {{ $t('dialog.cancel') }}
      </UIButton>
    </div>
  </DialogField>
</template>

<script setup lang="ts">
const appStore = useAppStore()

const emit = defineEmits<{
  choose: [mode: 'audio' | 'folders']
}>()

function close() {
  appStore.setImportChooserActive(false)
}

function choose(mode: 'audio' | 'folders') {
  close()
  emit('choose', mode)
}
</script>
