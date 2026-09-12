<template>
  <Transition name="filter-blade">
    <aside
      v-if="appStore.filterPanelOpen"
      class="filter-panel"
      role="dialog"
      :aria-label="$t('filter.title')"
      @pointerdown.stop
    >
      <div class="filter-panel__section">
        <h2 class="filter-panel__title">{{ $t('filter.tags') }}</h2>
        <p v-if="!tags.length" class="filter-panel__empty">{{ $t('filter.noTags') }}</p>
        <ul v-else class="filter-panel__tags">
          <li v-for="tag in tags" :key="tag.id" class="filter-panel__tag">
            <button
              type="button"
              class="filter-panel__tag-btn"
              :class="{ 'is-on': selected.has(tag.id) }"
              @click="appStore.toggleSelectedTag(tag.id)"
            >
              <span class="filter-panel__swatch" :style="{ background: tag.color }" />
              <span v-if="editingId !== tag.id" class="filter-panel__tag-name">{{ tag.name }}</span>
              <input
                v-else
                :ref="(el) => setEditRef(el, tag.id)"
                v-model="editName"
                class="filter-panel__tag-input"
                :aria-label="$t('filter.tagName')"
                @keydown.enter.prevent="commitEdit(tag)"
                @keydown.esc.prevent="editingId = ''"
                @blur="commitEdit(tag)"
                @click.stop
              />
            </button>
            <input
              type="color"
              class="filter-panel__color"
              :value="tag.color"
              :aria-label="$t('filter.tagColor')"
              @input="onColor(tag.id, $event)"
            />
            <button
              type="button"
              class="filter-panel__icon-btn"
              :title="$t('contextMenu.rename')"
              @click.stop="startEdit(tag)"
            >
              <Icons icon="rename" custom-class="filter-panel__icon" />
            </button>
            <button
              type="button"
              class="filter-panel__icon-btn"
              :title="$t('filter.deleteTag')"
              @click.stop="jsonStore.removeTag(tag.id)"
            >
              <Icons icon="delete" custom-class="filter-panel__icon" />
            </button>
          </li>
        </ul>
        <form class="filter-panel__add" @submit.prevent="addTag">
          <input
            v-model="newName"
            class="filter-panel__add-input"
            type="text"
            :placeholder="$t('filter.tagName')"
            maxlength="32"
          />
          <input
            v-model="newColor"
            class="filter-panel__color"
            type="color"
            :aria-label="$t('filter.tagColor')"
          />
          <button type="submit" class="filter-panel__add-btn">{{ $t('filter.addTag') }}</button>
        </form>
      </div>

      <div class="filter-panel__section">
        <h2 class="filter-panel__title">{{ $t('filter.sort') }}</h2>
        <div class="filter-panel__sort" role="radiogroup" :aria-label="$t('filter.sort')">
          <label v-for="opt in sortOpts" :key="opt.id" class="filter-panel__sort-row">
            <input
              type="radio"
              name="board-sort"
              :value="opt.id"
              :checked="sortMode === opt.id"
              @change="jsonStore.setSortMode(opt.id)"
            />
            <span>{{ $t(opt.key) }}</span>
          </label>
        </div>
      </div>
    </aside>
  </Transition>
</template>

<script setup>
import { normalizeSortMode } from '~/utils/db'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const TAG_COLORS = ['#00d4ff', '#f86d6d', '#ffd166', '#06d6a0', '#c77dff', '#ffa94d']

const newName = ref('')
const newColor = ref(TAG_COLORS[0])
const editingId = ref('')
const editName = ref('')
const editRefs = new Map()

const tags = computed(() => jsonStore.configFile.tags ?? [])
const selected = computed(() => new Set(appStore.selectedTagIds))
const sortMode = computed(() => normalizeSortMode(jsonStore.configFile?.settings?.sortMode))

const sortOpts = [
  { id: 'user', key: 'filter.sortUser' },
  { id: 'name', key: 'filter.sortName' },
  { id: 'added', key: 'filter.sortAdded' },
  { id: 'duration', key: 'filter.sortDuration' },
  { id: 'size', key: 'filter.sortSize' },
]

function setEditRef(el, id) {
  if (el) editRefs.set(id, el)
  else editRefs.delete(id)
}

function startEdit(tag) {
  editingId.value = tag.id
  editName.value = tag.name
  nextTick(() => editRefs.get(tag.id)?.focus?.())
}

function commitEdit(tag) {
  if (editingId.value !== tag.id) return
  const name = editName.value.trim()
  editingId.value = ''
  if (name && name !== tag.name) jsonStore.updateTag(tag.id, { name })
}

function onColor(id, e) {
  const color = e.target?.value
  if (color) jsonStore.updateTag(id, { color })
}

function addTag() {
  const name = newName.value.trim()
  if (!name) return
  jsonStore.addTag(name, newColor.value)
  newName.value = ''
  const i = TAG_COLORS.indexOf(newColor.value)
  newColor.value = TAG_COLORS[(i + 1) % TAG_COLORS.length]
}

function onDocPointerDown(e) {
  const el = e.target
  if (!(el instanceof Element)) {
    appStore.setFilterPanelOpen(false)
    return
  }
  if (el.closest('.filter-panel') || el.closest('.filter-toggle')) return
  appStore.setFilterPanelOpen(false)
}

watch(
  () => appStore.filterPanelOpen,
  (open) => {
    document.removeEventListener('pointerdown', onDocPointerDown, true)
    if (!open) return
    nextTick(() => {
      document.addEventListener('pointerdown', onDocPointerDown, true)
    })
  },
)

onUnmounted(() => {
  document.removeEventListener('pointerdown', onDocPointerDown, true)
})
</script>
