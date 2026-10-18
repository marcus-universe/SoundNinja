<template>
  <DialogField
    v-if="review"
    :title="$t('importReview.title')"
    :error-message="skippedMessage"
    @close="close"
  >
    <ul class="ui-list import-review-list">
      <li
        v-for="folder in review.folders"
        :key="'folder:' + folder.path"
        class="ui-listitem import-review-row flex_c flex_space_between gap1"
      >
        <Icons icon="folders" custom-class="import-review-row__icon" />
        <div class="import-review-row__meta">
          <span class="ui-listitem-label">{{ folder.name }}</span>
          <span class="import-review-row__hint">{{ folderHint(folder) }}</span>
        </div>
        <UISelect
          v-model="folder.destTab"
          select-class="import-review-select"
          :options="folderDestOptions(folder)"
        />
      </li>
      <li
        v-for="file in review.files"
        :key="'file:' + file.path"
        class="ui-listitem import-review-row flex_c flex_space_between gap1"
      >
        <Icons icon="audio-file" custom-class="import-review-row__icon" />
        <span class="ui-listitem-label">{{ audioDisplayName(file.fileName) }}</span>
        <UISelect
          v-if="showFileSelects"
          v-model="file.destTab"
          select-class="import-review-select"
          :options="fileTabOptions"
        />
      </li>
    </ul>

    <div v-if="showApplyAll" class="import-review-apply">
      <label class="import-review-apply__label" for="import-review-apply-all">
        {{ $t('importReview.assignAllFiles') }}
      </label>
      <UISelect
        input-id="import-review-apply-all"
        :model-value="applyAllTab"
        select-class="import-review-select"
        :aria-label="$t('importReview.assignAllFiles')"
        :options="fileTabOptions"
        @update:model-value="pickApplyAll"
      />
    </div>

    <div class="flex_c_h gap1 dialog-actions import-review-actions">
      <UIButton @click="addFolder">{{ $t('importReview.addFolder') }}</UIButton>
      <UIButton @click="close">{{ $t('dialog.cancel') }}</UIButton>
      <UIButton @click="commit">{{ $t('importReview.import') }}</UIButton>
    </div>
  </DialogField>
</template>

<script setup>
import { NEW_TAB_DEST, audioDisplayName, folderAudioCount, folderGroups } from '~/utils/importReview'
import { pickFoldersAndPresent } from '~/utils/importDrop'

const { t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const review = computed(() => appStore.importReview)
const namedTabs = computed(() => jsonStore.configFile.tabList ?? [])
const showFileSelects = computed(() => namedTabs.value.length > 0)
const showApplyAll = computed(() =>
  showFileSelects.value && (review.value?.files.length ?? 0) > 0,
)

const applyAllTab = ref(appStore.currentTab || 'All')

const skippedMessage = computed(() => {
  const n = review.value?.skipped ?? 0
  return n > 0 ? t('importReview.skipped', { count: n }) : ''
})

function folderHint(folder) {
  const files = folderAudioCount(folder)
  const groups = folderGroups(folder).length
  const parts = []
  if (files) parts.push(t('importReview.fileCount', { count: files }))
  if (groups) parts.push(t('importReview.groupCount', { count: groups }))
  return parts.join(', ')
}

function folderDestOptions(folder) {
  return [
    { value: NEW_TAB_DEST, label: t('importReview.newTab', { name: folder.name }) },
    { value: 'All', label: t('tabs.all') },
    ...namedTabs.value.map((tab) => ({ value: tab.name, label: tab.name })),
  ]
}

const fileTabOptions = computed(() => [
  { value: 'All', label: t('tabs.all') },
  ...namedTabs.value.map((tab) => ({ value: tab.name, label: tab.name })),
])

function pickApplyAll(value) {
  applyAllTab.value = value
  assignAllFiles()
}

function assignAllFiles() {
  if (!review.value) return
  for (const file of review.value.files) {
    file.destTab = applyAllTab.value
  }
}

function addFolder() {
  void pickFoldersAndPresent()
}

function close() {
  appStore.clearImportReview()
}

function commit() {
  if (!review.value) return
  jsonStore.commitImportReview(review.value)
  close()
}
</script>
