<template>
  <DialogField :title="$t('relink.title')" @close="close">
    <p class="dialog-text">{{ $t('relink.message', { count: missing.length }) }}</p>
    <ul class="relink-list">
      <li v-for="row in rows" :key="row.from" class="relink-list__item">
        <span class="relink-list__name">{{ row.name }}</span>
        <UISelect
          v-if="row.candidates.length"
          v-model="row.chosen"
          select-class="settings-input"
          :options="relinkOptions(row)"
        />
        <span v-else class="relink-list__none">{{ $t('relink.noMatch') }}</span>
      </li>
    </ul>
    <p v-if="scanError" class="dialog-error">{{ scanError }}</p>
    <div class="flex_c_h gap1 dialog-actions">
      <UIButton @click="pickFolder">{{ $t('relink.pickFolder') }}</UIButton>
      <UIButton :disabled="!canApply" @click="apply">{{ $t('relink.apply') }}</UIButton>
      <UIButton @click="close">{{ $t('dialog.cancel') }}</UIButton>
    </div>
  </DialogField>
</template>

<script setup>
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

const jsonStore = useJsonHandelingStore()
const appStore = useAppStore()
const { t } = useI18n()

const missing = computed(() => jsonStore.missingPaths ?? [])
const rows = ref([])
const scanError = ref('')

function relinkOptions(row) {
  return [
    { value: '', label: t('relink.skip') },
    ...row.candidates.map((path) => ({ value: path, label: path })),
  ]
}

function basename(p) {
  return String(p).replace(/^.*[\\/]/, '')
}

function rebuildRows(matchMap = new Map()) {
  const taken = new Set((jsonStore.configFile.files ?? []).map((f) => f.path).filter(Boolean))
  for (const path of missing.value) taken.delete(path)
  rows.value = missing.value.map((from) => {
    const name = basename(from)
    const candidates = matchMap.get(name.toLowerCase()) || matchMap.get(name) || []
    const chosen = candidates.find((c) => c && !taken.has(c)) || ''
    if (chosen) taken.add(chosen)
    return { from, name, candidates, chosen }
  })
}

watch(missing, () => rebuildRows(), { immediate: true })

const canApply = computed(() => rows.value.some((r) => r.chosen))

function close() {
  appStore.setRelinkActive(false)
}

async function pickFolder() {
  scanError.value = ''
  const dir = await open({ directory: true, multiple: false, title: t('native.findMissingSounds') })
  if (!dir || Array.isArray(dir)) return
  const names = [...new Set(missing.value.map(basename))]
  try {
    const hits = await invoke('find_files_by_names', {
      roots: [dir],
      names,
      maxDepth: 8,
    })
    const map = new Map()
    for (const h of hits || []) {
      map.set(h.name, h.paths)
      map.set(String(h.name).toLowerCase(), h.paths)
    }
    rebuildRows(map)
  } catch (e) {
    scanError.value = String(e)
  }
}

function apply() {
  scanError.value = ''
  const pairs = rows.value
    .filter((r) => r.chosen)
    .map((r) => ({ from: r.from, to: r.chosen }))
  const skipped = jsonStore.relinkSounds(pairs)
  if (skipped > 0) scanError.value = t('relink.duplicatePath')
  if (!jsonStore.missingPaths.length) close()
  else rebuildRows()
}
</script>
