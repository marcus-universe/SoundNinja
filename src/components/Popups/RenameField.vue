<template>
  <DialogField title="Add Tab" :error-message="appStore.ErrorMessage" @close="Exit">
    <UIInput v-model="typedName" placeholder="Type here a name" />
    <UIButton :full-width="true" @click="setName">Confirm</UIButton>
  </DialogField>
</template>

<script setup>
const appStore = useAppStore()
const typedName = ref('')

function Exit() {
  appStore.setPopupActive({ active: false, type: 'addTab' })
}

function setName() {
  if (appStore.PopupActive.type === 'addTab') {
    appStore.setPopupActive({ active: false, type: 'addTab' })
  }
  appStore.setRenameContent({ name: typedName.value })
}

onMounted(() => {
  onKeyStroke('Enter', () => setName())
  onKeyStroke('Escape', () => Exit())
})
</script>
<style lang=""></style>
