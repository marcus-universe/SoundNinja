<template>
<transition name="fade">
    <div v-if="ErrorActive" class="ErrorAlert">
        <div class="ErrorMessageBox grid_c NormalMessage">
            <Icons :icon="'exit'" :customClass="'exit'" @triggered="Exit" />
            {{ErrorMessage}}
        </div>
        <BlurBG />
    </div>
</transition>
    
</template>
<script>
import BlurBG from '@/components/Assets/BlurBG.vue'
import Icons from '@/components/Assets/Icons.vue'
export default {
    components: {
        BlurBG,
        Icons
    },
    computed: {
        ErrorMessage() {
            return this.$store.state.ErrorMessage
        },
        ErrorActive() {
            return this.$store.state.ErrorActive
        }
    },
    methods: {
        Exit() {
            this.$store.dispatch('setErrorActive', false)
        }
    }
    
}
</script>
<style lang="">
    
</style>