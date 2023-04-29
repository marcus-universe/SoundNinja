<template>
    <div
        class="TabContainer flex_c_h flex_start gap1"
        :class="{ searchMove: store.Searchbar.SearchbarActive }"
    >
        <div
            class="tab grid_c"
            ref="tab"
            :class="{ active: 'All' === store.currentTab }"
            @click="CheckTabContent('All')"
        >
            All
        </div>

        <div
            v-for="tab in store.JsonHandeling.configFile.tabList"
            :key="tab"
            class="tab grid_c"
            ref="tab"
            :class="{ active: tab === store.currentTab }"
            @click="CheckTabContent(tab)"
        >
            {{ tab }}
        </div>

        <Icons :customClass="'addTab'" :icon="'plus'" @triggered="AddTab" />

        <Transition name="fade">
            <RenameField v-if="store.PopupActive.active" />
        </Transition>
    </div>
</template>

<script>
import Icons from "@/components/Assets/Icons.vue";
import RenameField from "@/components/Popups/RenameField.vue";
export default {
    components: {
        Icons,
        RenameField,
    },
    computed: {
        store() {
            return this.$store.state;
        },
    },
    methods: {
        AddTab() {
            this.$store.dispatch("setPopupActive", { active: true, type: "addTab" });
        },
        CheckTabContent(tab) {
            this.$store.dispatch("setCurrentTab", tab);
        },
    },
};
</script>
