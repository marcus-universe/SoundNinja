import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { tt } from '~/utils/tt'
import { pickOpenPaths } from '~/utils/projects'
import { useAppStore } from '~/stores/app'
import { useJsonHandelingStore } from '~/stores/jsonHandeling'
import {
  AUDIO_EXTENSIONS,
  NEW_TAB_DEST,
  audioDisplayName,
  pathKey,
  type ImportDropScan,
  type ImportReviewState,
  type ScannedAudioFile,
} from '~/utils/importReview'

async function restoreFocus() {
  try {
    await getCurrentWindow().setFocus()
  } catch {
    /* non-tauri */
  }
}

export async function inspectImportDrop(paths: string[]): Promise<ImportDropScan> {
  return await invoke<ImportDropScan>('inspect_import_drop', {
    paths,
    exts: [...AUDIO_EXTENSIONS],
  })
}

function dedupeByPath<T extends { path: string }>(items: T[]): T[] {
  const seen = new Set<string>()
  const out: T[] = []
  for (const item of items) {
    const key = pathKey(item.path)
    if (seen.has(key)) continue
    seen.add(key)
    out.push(item)
  }
  return out
}

function boardPathSet(): Set<string> {
  const jsonStore = useJsonHandelingStore()
  return new Set((jsonStore.configFile.files ?? []).map((f) => pathKey(f.path)))
}

function filterScan(scan: ImportDropScan): ImportDropScan {
  const onBoard = boardPathSet()
  const keep = (f: ScannedAudioFile) => !onBoard.has(pathKey(f.path))
  const files = dedupeByPath(scan.files.filter(keep))
  const folders = dedupeByPath(scan.folders)
    .map((folder) => {
      const groups = folder.groups
        .map((g) => ({ ...g, audio: dedupeByPath(g.audio.filter(keep)) }))
        .filter((g) => g.audio.length > 0)
      return {
        ...folder,
        rootAudio: dedupeByPath(folder.rootAudio.filter(keep)),
        groups,
      }
    })
    .filter((f) => f.rootAudio.length > 0 || f.groups.length > 0)
  return { files, folders, skipped: scan.skipped }
}

function scanHasItems(scan: ImportDropScan): boolean {
  return scan.files.length > 0 || scan.folders.length > 0
}

function toReviewState(scan: ImportDropScan, defaultFileTab: string): ImportReviewState {
  return {
    files: scan.files.map((f) => ({ ...f, destTab: defaultFileTab })),
    folders: scan.folders.map((f) => ({ ...f, destTab: NEW_TAB_DEST })),
    skipped: scan.skipped,
  }
}

function mergeReviewState(current: ImportReviewState, incoming: ImportReviewState): ImportReviewState {
  const fileKeys = new Set(current.files.map((f) => pathKey(f.path)))
  const folderKeys = new Set(current.folders.map((f) => pathKey(f.path)))
  return {
    files: [...current.files, ...incoming.files.filter((f) => !fileKeys.has(pathKey(f.path)))],
    folders: [...current.folders, ...incoming.folders.filter((f) => !folderKeys.has(pathKey(f.path)))],
    skipped: current.skipped + incoming.skipped,
  }
}

function addLooseFilesToAll(files: ScannedAudioFile[]) {
  const jsonStore = useJsonHandelingStore()
  const indexLength = jsonStore.configFile.files.length
  jsonStore.addFiles(
    files.map((file, index) => ({
      name: audioDisplayName(file.fileName),
      path: file.path,
      id: '',
      volume: 1,
      tabs: ['All'],
      active: false,
      index: index + indexLength,
      tabIndexes: {},
    })),
  )
}

export async function presentImportPaths(paths: string[]) {
  const appStore = useAppStore()
  const jsonStore = useJsonHandelingStore()
  if (!paths.length) return

  let scan: ImportDropScan
  try {
    scan = await inspectImportDrop(paths)
  } catch (e) {
    appStore.setErrorActive(String(e))
    return
  }

  scan = filterScan(scan)
  if (!scanHasItems(scan)) {
    appStore.setErrorActive(tt('importReview.nothingToImport'))
    return
  }

  const namedTabs = jsonStore.configFile.tabList ?? []
  const reviewOpen = !!appStore.importReview
  if (!scan.folders.length && namedTabs.length === 0 && !reviewOpen) {
    addLooseFilesToAll(scan.files)
    return
  }

  const defaultFileTab = appStore.currentTab || 'All'
  const next = toReviewState(scan, defaultFileTab)
  if (reviewOpen && appStore.importReview) {
    appStore.setImportReview(mergeReviewState(appStore.importReview, next))
  } else {
    appStore.setImportReview(next)
  }
}

export async function pickAudioFilesAndPresent() {
  const selected = await open({
    multiple: true,
    title: tt('common.importAudioFiles'),
    filters: [{ name: tt('common.audioFiles'), extensions: [...AUDIO_EXTENSIONS] }],
  })
  await restoreFocus()
  await presentImportPaths(pickOpenPaths(selected))
}

export async function pickFoldersAndPresent() {
  const selected = await open({
    directory: true,
    multiple: true,
    title: tt('importReview.selectFolders'),
  })
  await restoreFocus()
  await presentImportPaths(pickOpenPaths(selected))
}
