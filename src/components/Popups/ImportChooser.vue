<template>
  <DialogField
    v-if="appStore.importChooserActive"
    :title="$t('importChooser.title')"
    @close="close"
  >
    <p class="dialog-text">{{ $t('importChooser.message') }}</p>
    <div class="flex_c_h gap1 dialog-actions import-chooser-actions">
      <button type="button" class="import-chooser-tile" @click="choose('folders')">
        <Icons icon="folders" custom-class="import-chooser-tile__icon" />
        <span class="import-chooser-tile__label">{{ $t('importChooser.folders') }}</span>
      </button>
      <button type="button" class="import-chooser-tile" @click="choose('audio')">
        <Icons icon="audio-file" custom-class="import-chooser-tile__icon" />
        <span class="import-chooser-tile__label">{{ $t('importChooser.audioFiles') }}</span>
      </button>
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
