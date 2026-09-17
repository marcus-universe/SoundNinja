export const AUDIO_EXTENSIONS = ['mp3', 'wav', 'ogg'] as const

/** Folder-row dest: create a tab named after the folder. */
export const NEW_TAB_DEST = '__new__'

export type ScannedAudioFile = {
  path: string
  fileName: string
}

export type ScannedFolderGroup = {
  name: string
  path: string
  audio: ScannedAudioFile[]
}

export type ScannedFolder = {
  path: string
  name: string
  rootAudio: ScannedAudioFile[]
  groups: ScannedFolderGroup[]
  /** IPC may send snake_case if camelCase rename is skipped. */
  root_audio?: ScannedAudioFile[]
}

export type ImportDropScan = {
  files: ScannedAudioFile[]
  folders: ScannedFolder[]
  skipped: number
}

export type ReviewFileRow = ScannedAudioFile & {
  destTab: string
}

export type ReviewFolderRow = ScannedFolder & {
  destTab: string
}

export type ImportReviewState = {
  files: ReviewFileRow[]
  folders: ReviewFolderRow[]
  skipped: number
}

type AudioLike = ScannedAudioFile & { file_name?: string }

function normalizeAudio(file: AudioLike): ScannedAudioFile {
  return {
    path: file.path,
    fileName: file.fileName || file.file_name || '',
  }
}

export function folderRootAudio(folder: {
  rootAudio?: ScannedAudioFile[]
  root_audio?: ScannedAudioFile[]
}): ScannedAudioFile[] {
  return (folder.rootAudio ?? folder.root_audio ?? []).map(normalizeAudio)
}

export function folderGroups(folder: { groups?: ScannedFolderGroup[] }): ScannedFolderGroup[] {
  return (folder.groups ?? []).map((g) => ({
    ...g,
    audio: (g.audio ?? []).map(normalizeAudio),
  }))
}

/** Canonical camelCase arrays after inspect_import_drop IPC. */
export function normalizeImportScan(scan: ImportDropScan | null | undefined): ImportDropScan {
  const files = (scan?.files ?? []).map(normalizeAudio)
  const folders = (scan?.folders ?? []).map((folder) => ({
    ...folder,
    rootAudio: folderRootAudio(folder),
    groups: folderGroups(folder),
  }))
  return {
    files,
    folders,
    skipped: scan?.skipped ?? 0,
  }
}

export function audioDisplayName(fileName: string): string {
  return String(fileName || '')
    .replace(/\.(wav|mp3|ogg)$/i, '')
    .replaceAll('_', ' ')
    .replace(/([A-Z])/g, ' $1')
    .trim()
}

export function pathKey(p: string): string {
  return String(p || '').replace(/\\/g, '/').toLowerCase()
}

export function folderAudioCount(folder: ScannedFolder): number {
  return folderRootAudio(folder).length
    + folderGroups(folder).reduce((n, g) => n + g.audio.length, 0)
}
