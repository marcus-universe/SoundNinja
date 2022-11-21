<template>
<div class="TabContainer flex_c_h flex_start gap1" :class="{'searchMove': store.Searchbar.SearchbarActive}">
    <div class="tab grid_c" ref="tab" :class="{'active': CheckTabContent() === store.currentTab}">All</div>

    <Icons
        :customClass="'addTab'"
        :icon="'plus'"
        @triggered="AddTab" />

        <Transition name="fade">
            <RenameField v-if="store.PopupActive" />
        </Transition>
    
</div>
</template>

<script>
import Icons from '@/components/Assets/Icons.vue'
import RenameField from '@/components/Popups/RenameField.vue'
export default {
    components: {
        Icons,
        RenameField
    },
    computed: {
        store() {
            return this.$store.state
        }
    },
    methods: {
        AddTab() {
            this.$store.dispatch('setPopupActive', true)
        },
        CheckTabContent() {
            console.log()
        }
    },

}
</script>
