import { invoke } from '@tauri-apps/api/core'
import { copyProject, inspectProject, PROJECT_FORMAT_VERSION, type ProjectInspect } from '~/utils/db'
import { dirOf, joinOsPath, pathSegments } from '~/utils/pathPortability'
import { projectNameFromDbPath } from '~/utils/projects'
import { tt } from '~/utils/tt'
import { useAppStore } from '~/stores/app'
import { useAppSettingsStore } from '~/stores/appSettings'
import { useJsonHandelingStore } from '~/stores/jsonHandeling'

function isFolderLayout(dbPath: string): boolean {
  return /^project\.(sninja|db)$/i.test(pathSegments(dbPath).pop() ?? '')
}

/**
 * Free path for a converted copy next to the original.
 * `<dir>/Board.sninja` → `<dir>/Board_<suffix>.sninja`
 * `<projects>/Board/project.sninja` → `<projects>/Board_<suffix>/project.sninja`
 */
export async function siblingCopyPath(dbPath: string, suffix: string): Promise<string> {
  const dir = dirOf(dbPath)
  const build = (n: number): string => {
    const tail = n > 1 ? `_${suffix}_${n}` : `_${suffix}`
    if (isFolderLayout(dbPath)) {
      const folder = pathSegments(dir).pop() ?? 'Project'
      return joinOsPath(dirOf(dir), `${folder}${tail}`, 'project.sninja')
    }
    const stem = (pathSegments(dbPath).pop() ?? 'project').replace(/\.(sninja|db)$/i, '')
    return joinOsPath(dir, `${stem}${tail}.sninja`)
  }
  for (let n = 1; n < 500; n++) {
    const candidate = build(n)
    if (!(await invoke<boolean>('path_exists_abs', { path: candidate }))) return candidate
  }
  return build(Date.now())
}

/**
 * If the board has unsaved changes, save them (autosave) or ask.
 * False means the user cancelled and the caller must not switch projects.
 */
export async function settleUnsavedChanges(): Promise<boolean> {
  const jsonStore = useJsonHandelingStore()
  const appStore = useAppStore()
  if (!jsonStore.dirty || !jsonStore.currentProjectPath) return true
  let choice = 'save'
  if (!jsonStore.autosave) {
    choice = await appStore.askChoice({
      title: tt('dialog.unsavedTitle'),
      message: tt('dialog.unsavedMessage'),
      cancelId: 'cancel',
      choices: [
        { id: 'save', label: tt('dialog.save') },
        { id: 'discard', label: tt('dialog.discard') },
        { id: 'cancel', label: tt('dialog.cancel') },
      ],
    })
  }
  if (choice === 'cancel') return false
  if (choice === 'discard') {
    await jsonStore.discardChanges()
    return true
  }
  try {
    await jsonStore.persistNow()
    return true
  } catch (e) {
    // Never trap the user behind a save that keeps failing: offer a way on.
    const next = await appStore.askChoice({
      title: tt('common.failedSaveProject'),
      message: e instanceof Error ? e.message : String(e),
      cancelId: 'cancel',
      choices: [
        { id: 'discard', label: tt('dialog.discard') },
        { id: 'cancel', label: tt('dialog.cancel') },
      ],
    })
    return next === 'discard'
  }
}

/**
 * Decide which file to open for `dbPath`.
 *
 * A project written by an older build is converted — but only after asking,
 * and by default on a copy, so the original stays usable with the old
 * version. A project from a NEWER build is only ever opened as a copy: this
 * build would drop what it does not understand when saving.
 *
 * Returns the path to open, or null when the user cancelled.
 */
async function resolveOpenTarget(dbPath: string, info: ProjectInspect): Promise<string | null> {
  const appStore = useAppStore()
  // New file, or one that cannot be read: nothing to convert. An unreadable
  // file goes straight to the store, which recovers it from its backup.
  if (!info.ok || !info.exists || info.isNew) return dbPath

  const version = info.formatVersion
  if (version != null && version > PROJECT_FORMAT_VERSION) {
    const choice = await appStore.askChoice({
      title: tt('project.newerTitle'),
      message: tt('project.newerMessage', { version: info.appVersion || '?' }),
      cancelId: 'cancel',
      choices: [
        { id: 'copy', label: tt('project.newerCopy') },
        { id: 'cancel', label: tt('dialog.cancel') },
      ],
    })
    if (choice !== 'copy') return null
    const target = await siblingCopyPath(dbPath, 'copy')
    await copyProject(dbPath, target)
    return target
  }

  if (version == null || version < PROJECT_FORMAT_VERSION) {
    // Nothing to lose in an empty project: convert it without a question.
    if (info.soundCount === 0) return dbPath
    const choice = await appStore.askChoice({
      title: tt('project.upgradeTitle'),
      message: tt('project.upgradeMessage'),
      cancelId: 'cancel',
      choices: [
        { id: 'copy', label: tt('project.upgradeCopy'), hint: tt('project.upgradeCopyHint') },
        { id: 'inplace', label: tt('project.upgradeInPlace'), hint: tt('project.upgradeInPlaceHint') },
        { id: 'cancel', label: tt('dialog.cancel') },
      ],
    })
    if (choice === 'copy') {
      const target = await siblingCopyPath(dbPath, 'upgraded')
      await copyProject(dbPath, target)
      return target
    }
    if (choice === 'inplace') {
      // Even "this file" keeps a way back.
      await copyProject(dbPath, `${dbPath}.pre-upgrade.bak`)
      return dbPath
    }
    return null
  }
  return dbPath
}

export interface OpenFlowOptions {
  /** Save / ask about unsaved changes of the current project first. Default true. */
  settleUnsaved?: boolean
}

/**
 * The one way a project gets opened from the UI: settle unsaved changes,
 * convert old formats on request, load, remember it as recent, and point out
 * anything that needs attention (recovered board, missing sound files).
 *
 * Returns false when the user cancelled. Throws when the project cannot be
 * opened at all.
 */
export async function openProjectFlow(dbPath: string, opts: OpenFlowOptions = {}): Promise<boolean> {
  const jsonStore = useJsonHandelingStore()
  const appStore = useAppStore()
  const appSettings = useAppSettingsStore()

  if (opts.settleUnsaved !== false && !(await settleUnsavedChanges())) return false

  const info = await inspectProject(dbPath)
  const target = await resolveOpenTarget(dbPath, info)
  if (!target) return false

  await jsonStore.openProject(target)
  try {
    await appSettings.touchRecent(target, projectNameFromDbPath(target))
  } catch (e) {
    // Opening is the primary action; a recents failure must not abort it.
    console.error('Failed to update recent projects', e)
  }

  const report = jsonStore.lastOpenReport
  if (report.recovered) {
    await appStore.askChoice({
      title: tt('project.recoveredTitle'),
      message: tt(
        report.recovered === 'snapshot' ? 'project.recoveredSnapshot' : 'project.recoveredBackup',
        { count: jsonStore.configFile.files.length },
      ),
      cancelId: 'ok',
      choices: [{ id: 'ok', label: tt('dialog.ok') }],
    })
  }

  await jsonStore.validateSoundPaths()
  if (jsonStore.missingPaths.length) appStore.setRelinkActive(true)
  return true
}
