<template>
    <div class="renameBox">
        <div class="renameContainer flex_c_h flex_c gap1">
            <div class="ErrorMsg">{{ appStore.ErrorMessage }}</div>
            <Icons :icon="'exit'" :customClass="'exit'" @triggered="Exit" />
            <input v-model="typedName" type="text" placeholder="Type here a name" />
            <Icons :icon="'check'" :customClass="'icon'" @triggered="setName" />
        </div>
        <BlurBG />
    </div>
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
