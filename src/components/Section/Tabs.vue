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

        <div class="tab-list" ref="tabListRef">
            <TabItem
                v-for="tab in jsonStore.configFile.tabList"
                :key="tab.name"
                :tabName="tab.name"
                :isActive="tab.name === appStore.currentTab"
                :tabColor="tab.color || ''"
                @select="CheckTabContent(tab.name)"
                @contextmenu="(e) => openTabMenu(e, tab.name)"
            />
        </div>

        <Icons :customClass="'addTab'" :icon="'plus'" @triggered="AddTab" />

        <Transition name="fade">
            <RenameField v-if="appStore.PopupActive.active" />
        </Transition>
    </div>
</template>

<script setup>
import Sortable from 'sortablejs'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const tabListRef = ref(null)
let sortable = null

onMounted(() => {
  sortable = Sortable.create(tabListRef.value, {
    animation: 180,
    draggable: '.tab',
    ghostClass: 'drag-over',
    onEnd(evt) {
      const { oldIndex, newIndex, item, from } = evt
      if (oldIndex === newIndex || oldIndex == null || newIndex == null) return
      // Undo SortableJS's DOM mutation so Vue owns the final order.
      from.removeChild(item)
      from.insertBefore(item, from.children[oldIndex] ?? null)
      const list = jsonStore.configFile.tabList
      const fromTab = list[oldIndex]
      const toTab = list[newIndex]
      if (fromTab && toTab) {
        jsonStore.reorderTabs(fromTab.name, toTab.name)
      }
    },
  })
})

onUnmounted(() => {
  sortable?.destroy()
  sortable = null
})

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

