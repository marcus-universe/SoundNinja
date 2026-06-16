<template>
  <DialogField :title="dialogTitle" :error-message="appStore.ErrorMessage" @close="Exit">
    <UIInput v-model="typedName" placeholder="Type here a name" />
    <UIButton :full-width="true" @click="setName">Confirm</UIButton>
  </DialogField>
</template>

<script setup>
const appStore = useAppStore()
const typedName = ref('')

const dialogTitle = computed(() => {
  switch (appStore.PopupActive.type) {
    case 'renameTab': return 'Rename Tab'
    case 'renameSound': return 'Rename Button'
    default: return 'Add Tab'
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

