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
  normalizeImportScan,
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
  const keepNew = (f: ScannedAudioFile) => !onBoard.has(pathKey(f.path))
  // Loose files: skip paths already on the board. Folder rows stay even when
  // those files already exist (same folder name as a tab) — commit adds tab
  // membership instead of treating the folder as empty.
  const files = dedupeByPath((scan.files ?? []).filter(keepNew))
  const folders = dedupeByPath(scan.folders ?? [])
    .map((folder) => {
      const groups = (folder.groups ?? [])
        .map((g) => ({ ...g, audio: dedupeByPath(g.audio ?? []) }))
        .filter((g) => g.audio.length > 0)
      return {
        ...folder,
        rootAudio: dedupeByPath(folder.rootAudio ?? []),
        groups,
      }
    })
    .filter((f) => f.rootAudio.length > 0 || f.groups.length > 0)
  return { files, folders, skipped: scan.skipped }
}

function scanHasItems(scan: ImportDropScan): boolean {
  return scan.files.length > 0 || scan.folders.length > 0
}

function matchingTabName(folderName: string, tabs: { name: string }[]): string | null {
  const n = String(folderName || '').trim().toLowerCase()
  if (!n) return null
  return tabs.find((t) => t.name.toLowerCase() === n)?.name ?? null
}

function toReviewState(
  scan: ImportDropScan,
  defaultFileTab: string,
  namedTabs: { name: string }[],
): ImportReviewState {
  return {
    files: scan.files.map((f) => ({ ...f, destTab: defaultFileTab })),
    folders: scan.folders.map((f) => ({
      ...f,
      destTab: matchingTabName(f.name, namedTabs) || NEW_TAB_DEST,
    })),
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
    scan = normalizeImportScan(await inspectImportDrop(paths))
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
  const next = toReviewState(scan, defaultFileTab, namedTabs)
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
    multiple: false,
    title: tt('importReview.selectFolders'),
  })
  await restoreFocus()
  await presentImportPaths(pickOpenPaths(selected))
}
