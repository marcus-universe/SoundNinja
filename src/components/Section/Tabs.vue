<template>
    <div
        class="TabContainer flex_c_h flex_start gap1"
        :class="{ searchMove: appStore.Searchbar.SearchbarActive }"
    >
        <div
            class="tab grid_c"
            ref="tab"
            :class="{ active: 'All' === appStore.currentTab }"
            @click="CheckTabContent('All')"
        >
            All
        </div>

        <div
            v-for="tab in jsonStore.configFile.tabList"
            :key="tab"
            class="tab grid_c"
            ref="tab"
            :class="{ active: tab === appStore.currentTab }"
            @click="CheckTabContent(tab)"
        >
            {{ tab }}
        </div>

        <Icons :customClass="'addTab'" :icon="'plus'" @triggered="AddTab" />

        <Transition name="fade">
            <RenameField v-if="appStore.PopupActive.active" />
        </Transition>
    </div>
</template>

<script setup>
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

function AddTab() {
  appStore.setPopupActive({ active: true, type: 'addTab' })
}

function CheckTabContent(tab) {
  appStore.setCurrentTab(tab)
}
</script>
