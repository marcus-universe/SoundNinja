<template>
    <div class="BGBlur" :class="[customClass]">
        
    </div>
</template>
<script setup>
defineProps({
  customClass: {
    type: String,
    default: ''
  }
})
</script>
<style lang=""></style>