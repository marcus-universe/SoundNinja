<template>
  <DialogField
    v-if="appStore.ErrorActive"
    :title="$t('dialog.errorTitle')"
    @close="Exit"
  >
    <p class="dialog-text">{{ appStore.ErrorMessage }}</p>
    <div class="flex_c_h gap1 dialog-actions">
      <UIButton @click="Exit">{{ $t('dialog.ok') }}</UIButton>
    </div>
  </DialogField>
</template>

<script setup>
const { t: $t } = useI18n()
const appStore = useAppStore()

function Exit() {
  appStore.setErrorActive(false)
}
</script>
