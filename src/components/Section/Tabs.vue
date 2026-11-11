<template>
    <div
        class="TabContainer flex_c_h flex_start"
        :class="{ searchMove: appStore.Searchbar.SearchbarActive }"
        :style="activeTabUnderline ? { '--tab-list-underline': activeTabUnderline } : {}"
        ref="tabContainerRef"
        @wheel.prevent="onTabWheel"
    >
        <div
            class="tab grid_c"
            ref="tab"
            :class="{ active: 'All' === appStore.currentTab }"
            @click="CheckTabContent('All')"
        >
            {{ $t('tabs.all') }}
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

        <QuickInfo class="addTab-tip" :text="$t('tabs.addTab')" below fixed>
          <Icons :customClass="'addTab icon'" :icon="'add'" @triggered="AddTab" />
        </QuickInfo>

        <Transition name="fade">
            <RenameField v-if="appStore.PopupActive.active" />
        </Transition>
    </div>
</template>

<script setup>
import Sortable from 'sortablejs'
import { parseOverride } from '~/utils/colorOverride'
import { isRemote } from '~/utils/runtime'
import { DRAG_TOLERANCE_PX, createPressTracker } from '~/utils/dragClick'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const tabListRef = ref(null)
const tabContainerRef = ref(null)
let sortable = null
/** Follows each press on the tab strip so a drag that went nowhere can count as a click. */
const press = createPressTracker()
let detachPress = null

const allowReorder = computed(() => jsonStore.configFile?.settings?.allowReorder !== false)

const activeTabUnderline = computed(() => {
  const name = appStore.currentTab
  if (!name || name === 'All') return ''
  const tab = (jsonStore.configFile?.tabList ?? []).find((t) => t.name === name)
  const o = parseOverride(tab?.color)
  return o.border || o.bg || ''
})

function onTabWheel(e) {
  if (tabListRef.value) {
    tabListRef.value.scrollBy({ left: e.deltaY + e.deltaX, behavior: 'smooth' })
  }
}

onMounted(() => {
  if (tabListRef.value) detachPress = press.attach(tabListRef.value)
  sortable = Sortable.create(tabListRef.value, {
    animation: 180,
    forceFallback: true,
    fallbackOnBody: true,
    // Dead zone before a press becomes a drag: without it any pointer
    // movement, even one pixel from an unsteady controller, started one.
    fallbackTolerance: DRAG_TOLERANCE_PX,
    ...(isRemote ? { delay: 120, delayOnTouchOnly: true } : {}),
    disabled: !allowReorder.value,
    draggable: '.tab',
    ghostClass: 'drag-over',
    onEnd(evt) {
      const { oldIndex, newIndex, item, from } = evt
      if (oldIndex === newIndex || oldIndex == null || newIndex == null) {
        // The tab did not move and the pointer barely did: a click that
        // SortableJS took for a drag and whose click event it swallows.
        const name = item?.getAttribute('data-tab-name')
        if (name && press.wasClick()) CheckTabContent(name)
        return
      }
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

  watch(
    [() => jsonStore.configFile?.settings?.allowReorder],
    () => {
      sortable?.option('disabled', jsonStore.configFile?.settings?.allowReorder === false)
    },
    { immediate: true }
  )
})

onUnmounted(() => {
  detachPress?.()
  detachPress = null
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

