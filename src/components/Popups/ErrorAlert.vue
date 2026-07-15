<template>
<transition name="fade">
    <div v-if="appStore.ErrorActive" class="ErrorAlert">
        <div class="ErrorMessageBox grid_c NormalMessage">
            <Icons :icon="'exit'" :customClass="'exit'" @triggered="Exit" />
            {{appStore.ErrorMessage}}
        </div>
        <BlurBG />
    </div>
</transition>
    
</template>
<script setup>
const appStore = useAppStore()

function Exit() {
  appStore.setErrorActive(false)
}
</script>
