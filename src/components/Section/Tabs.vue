<template>
    <div
        class="TabContainer flex_c_h flex_start"
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
            :key="tab.name"
            class="tab grid_c"
            ref="tab"
            :class="{ active: tab.name === appStore.currentTab }"
            :style="tab.color ? { '--tab-accent': tab.color } : {}"
            @click="CheckTabContent(tab.name)"
            @contextmenu.prevent="(e) => openTabMenu(e, tab.name)"
        >
            {{ tab.name }}
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

function openTabMenu(event, tabName) {
  appStore.openContextMenu({
    x: event.clientX,
    y: event.clientY,
    type: 'tab',
    targetName: tabName,
    targetIndex: -1,
  })
}
</script>

