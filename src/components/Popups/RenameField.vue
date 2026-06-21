<template>
  <DialogField :title="dialogTitle" :error-message="appStore.ErrorMessage" @close="Exit">
    <UIInput v-model="typedName" :placeholder="$t('rename.placeholder')" />
    <UIButton :full-width="true" @click="setName">{{ $t('rename.confirm') }}</UIButton>
  </DialogField>
</template>

<script setup>
const { t } = useI18n()
const appStore = useAppStore()
const typedName = ref('')

const dialogTitle = computed(() => {
  switch (appStore.PopupActive.type) {
    case 'renameTab': return t('rename.renameTab')
    case 'renameSound': return t('rename.renameSound')
    default: return t('rename.addTab')
  }
})

// Pre-fill with current name when renaming
watch(() => appStore.PopupActive, ({ active, type }) => {
  if (active && (type === 'renameTab' || type === 'renameSound')) {
    typedName.value = appStore.contextMenu.targetName
  } else {
    typedName.value = ''
  }
}, { immediate: true })

function Exit() {
  appStore.setPopupActive({ active: false, type: 'addTab' })
}

function setName() {
  appStore.setRenameContent({ name: typedName.value })
  if (!appStore.ErrorMessage) {
    appStore.setPopupActive({ active: false, type: 'addTab' })
  }
}

onMounted(() => {
  onKeyStroke('Enter', () => setName())
  onKeyStroke('Escape', () => Exit())
})
</script>
<style lang=""></style>

