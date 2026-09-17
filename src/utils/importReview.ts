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
  return folder.rootAudio.length + folder.groups.reduce((n, g) => n + g.audio.length, 0)
}
