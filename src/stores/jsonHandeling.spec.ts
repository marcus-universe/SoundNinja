import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async () => false),
  convertFileSrc: (p: string) => p,
}))

vi.mock('@tauri-apps/api/app', () => ({ getVersion: vi.fn(async () => '9.9.9') }))

vi.mock('~/utils/db', async (importOriginal) => {
  const actual = await importOriginal<typeof import('~/utils/db')>()
  return {
    ...actual,
    ensureSchema: vi.fn(async () => {}),
    copyProject: vi.fn(async () => {}),
    forgetProject: vi.fn(),
    extractProjectMedia: vi.fn(async () => []),
    commitProject: vi.fn(async (_path: string, fn: (d: unknown) => Promise<unknown>) => ({
      value: await fn({}),
      report: null,
    })),
    withProjectDb: vi.fn(async (_path: string, fn: (d: unknown) => Promise<unknown>) => fn({})),
    saveConfig: vi.fn(async () => {}),
    saveMeta: vi.fn(async () => {}),
    saveHistory: vi.fn(async () => new Map()),
    saveSnapshot: vi.fn(async () => {}),
    loadLatestSnapshot: vi.fn(async () => null),
    gcOrphanGifs: vi.fn(async () => {}),
    loadProject: vi.fn(),
  }
})

import {
  commitProject, copyProject, emptyConfig, extractProjectMedia, loadLatestSnapshot, loadProject,
  saveConfig, saveHistory, saveSnapshot,
  type LoadedProject, type ProjectConfig, type SoundFile,
} from '~/utils/db'
import { useJsonHandelingStore } from '~/stores/jsonHandeling'

function sound(path: string, extra: Partial<SoundFile> = {}): SoundFile {
  return {
    name: path,
    id: 'aaaaaaaa',
    volume: 1,
    tabs: ['All'],
    active: false,
    index: 0,
    tabIndexes: {},
    tagIds: [],
    fileSize: 8,
    addedAt: 1,
    ...extra,
    path,
  }
}

/** A project as `loadProject` returns it: current format, already stamped. */
function loaded(config: ProjectConfig = emptyConfig(), extra: Partial<LoadedProject> = {}): LoadedProject {
  config.settings.perSoundVolumeV1 = true
  return {
    config,
    formatVersion: 2,
    marker: config.files.length,
    meta: { projectId: 'pid', embedAudio: false, embedGifs: true, saveOptionsSet: true },
    history: { undo: [], redo: [], sizes: new Map() },
    ...extra,
  }
}

function boot() {
  const store = useJsonHandelingStore()
  store.setCurrentProjectPath('C:/board/project.sninja')
  return store
}

async function flushPersist(store: { persistNow: () => Promise<unknown> }, ms = 200) {
  const spy = vi.spyOn(store, 'persistNow')
  await vi.advanceTimersByTimeAsync(ms)
  await Promise.all(spy.mock.results.map((r) => r.value))
  spy.mockRestore()
}

const allowEmpty = expect.objectContaining({ allowEmpty: true })

function reset() {
  setActivePinia(createPinia())
  vi.useFakeTimers()
  // A debounced save scheduled by the previous test must not fire in this one.
  vi.clearAllTimers()
  vi.clearAllMocks()
  vi.mocked(invoke).mockImplementation(async () => false)
  vi.mocked(loadProject).mockImplementation(async () => loaded())
  vi.mocked(loadLatestSnapshot).mockResolvedValue(null)
  vi.mocked(extractProjectMedia).mockResolvedValue([])
}

describe('jsonHandeling persistence', () => {
  beforeEach(reset)

  it('saves deleting the last sound as an intentional empty board', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav')]
    store.removeSound(0)
    await flushPersist(store)
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), allowEmpty)
  })

  it('saves a bulk delete of every sound as an intentional empty board', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav'), sound('/b.wav', { id: 'bbbbbbbb', index: 1 })]
    store.removeSoundsMany(['/a.wav', '/b.wav'])
    await flushPersist(store)
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), allowEmpty)
  })

  it('saves resetAll as an intentional empty board', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav')]
    store.resetAll()
    await flushPersist(store)
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), allowEmpty)
  })

  it('saves undo back to an empty board', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav', { name: 'A' })]
    store.undoStack.push(emptyConfig())
    store.undo()
    await flushPersist(store)
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), allowEmpty)
  })

  it('saves an imported empty project', async () => {
    const store = boot()
    await store.importConfig(emptyConfig(), 'C:/board/imported.sninja')
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), allowEmpty)
  })

  it('never passes allowEmpty for a board that still has sounds', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav'), sound('/b.wav', { id: 'bbbbbbbb', index: 1 })]
    store.removeSound(0)
    await flushPersist(store)
    const opts = vi.mocked(saveConfig).mock.calls[0]?.[2]
    expect(opts?.allowEmpty).toBeUndefined()
    expect(opts?.appVersion).toBe('9.9.9')
  })

  it('writes board, options and history in one commit and backs up once per session', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav', { name: 'A' })]
    store.renameSound(0, 'B')
    await flushPersist(store)
    store.renameSound(0, 'C')
    await flushPersist(store)
    const calls = vi.mocked(commitProject).mock.calls
    expect(calls).toHaveLength(2)
    expect(saveHistory).toHaveBeenCalledTimes(2)
    expect(calls[0]?.[2]).toMatchObject({ backup: true, embedAudio: false, embedGifs: true })
    expect(calls[1]?.[2]).toMatchObject({ backup: false })
  })

  it('keeps the project unsaved and reports the error when a background save fails', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav', { name: 'A' })]
    vi.mocked(commitProject).mockRejectedValueOnce(new Error('disk full'))
    const quiet = vi.spyOn(console, 'error').mockImplementation(() => {})
    store.renameSound(0, 'B')
    await vi.advanceTimersByTimeAsync(200)
    await vi.waitFor(() => expect(store.saveError).toBe('disk full'))
    quiet.mockRestore()
    expect(store.dirty).toBe(true)
    expect(store.saving).toBe(false)
    await store.persistNow()
    expect(store.dirty).toBe(false)
    expect(store.saveError).toBeNull()
  })
})

describe('jsonHandeling opening a project', () => {
  beforeEach(reset)

  it('does not restore or rewrite a finished empty project', async () => {
    const store = boot()
    vi.mocked(loadProject).mockResolvedValue(loaded(emptyConfig(), { marker: 0 }))
    await store.openProject('C:/board/empty.sninja')
    expect(store.configFile.files).toHaveLength(0)
    expect(invoke).not.toHaveBeenCalled()
    expect(commitProject).not.toHaveBeenCalled()
    expect(store.dirty).toBe(false)
  })

  it('checks the backup when an empty project has no soundCount marker', async () => {
    const store = boot()
    vi.mocked(loadProject).mockResolvedValue(loaded(emptyConfig(), { marker: null }))
    await store.openProject('C:/board/empty.sninja')
    expect(invoke).toHaveBeenCalledWith('path_exists_abs', { path: 'C:/board/empty.sninja.bak' })
  })

  it('recovers a lost board from the snapshot stored inside the file', async () => {
    const store = boot()
    const snap = emptyConfig()
    snap.files = [sound('/a.wav'), sound('/b.wav', { id: 'bbbbbbbb', index: 1 })]
    vi.mocked(loadProject).mockResolvedValue(loaded(emptyConfig(), { marker: 2 }))
    vi.mocked(loadLatestSnapshot).mockResolvedValue(snap)
    vi.mocked(invoke).mockImplementation(async (cmd) => (cmd === 'paths_exist_abs' ? [true, true] : false))
    await store.openProject('C:/board/lost.sninja')
    expect(store.configFile.files.map((f) => f.path)).toEqual(['/a.wav', '/b.wav'])
    expect(store.lastOpenReport.recovered).toBe('snapshot')
    // Written back, but without copying the damaged file over the backup.
    expect(vi.mocked(commitProject).mock.calls[0]?.[2]).toMatchObject({ backup: false })
  })

  it('falls back to the backup file when the project cannot be read at all', async () => {
    const store = boot()
    const good = emptyConfig()
    good.files = [sound('/a.wav')]
    vi.mocked(loadProject).mockImplementation(async (path) => {
      if (path === 'C:/board/broken.sninja.bak') return loaded(good)
      if (vi.mocked(copyProject).mock.calls.length) return loaded(JSON.parse(JSON.stringify(good)))
      throw new Error('file is not a database')
    })
    vi.mocked(invoke).mockImplementation(async (cmd) => (cmd === 'paths_exist_abs' ? [true] : true))
    const quiet = vi.spyOn(console, 'error').mockImplementation(() => {})
    await store.openProject('C:/board/broken.sninja')
    quiet.mockRestore()
    expect(copyProject).toHaveBeenCalledWith('C:/board/broken.sninja.bak', 'C:/board/broken.sninja')
    expect(invoke).toHaveBeenCalledWith('copy_file_to_abs', {
      src: 'C:/board/broken.sninja',
      dst: 'C:/board/broken.sninja.damaged',
    })
    expect(store.configFile.files).toHaveLength(1)
    expect(store.lastOpenReport.recovered).toBe('backup')
  })

  it('rethrows when the project is unreadable and there is no backup', async () => {
    const store = boot()
    vi.mocked(loadProject).mockRejectedValue(new Error('file is not a database'))
    await expect(store.openProject('C:/board/broken.sninja')).rejects.toThrow('file is not a database')
  })

  it('stamps a file from an older build and stores its original board as a snapshot', async () => {
    const store = boot()
    const old = emptyConfig()
    old.files = [sound('/a.wav')]
    vi.mocked(loadProject).mockResolvedValue(loaded(old, { formatVersion: null }))
    vi.mocked(invoke).mockImplementation(async (cmd) => (cmd === 'paths_exist_abs' ? [true] : false))
    await store.openProject('C:/board/old.sninja')
    expect(store.lastOpenReport.upgraded).toBe(true)
    expect(commitProject).toHaveBeenCalledTimes(1)
    expect(vi.mocked(commitProject).mock.calls[0]?.[2]).toMatchObject({ backup: true })
    expect(saveSnapshot).toHaveBeenCalledTimes(1)
    expect(store.dirty).toBe(false)
  })

  it('finds sounds next to the project file when their stored path is gone', async () => {
    const store = boot()
    const config = emptyConfig()
    config.files = [
      sound('C:\\Users\\me\\Music\\Board\\Lolli\\a.wav', { relPath: 'Lolli/a.wav' }),
      sound('C:\\Users\\me\\Music\\Board\\gone.wav', { id: 'bbbbbbbb', index: 1 }),
    ]
    const onDisk = new Set(['/home/deck/Board/Lolli/a.wav'])
    vi.mocked(loadProject).mockResolvedValue(loaded(config))
    vi.mocked(invoke).mockImplementation(async (cmd, args) => {
      if (cmd === 'paths_exist_abs') return (args as { paths: string[] }).paths.map((p) => onDisk.has(p))
      return false
    })
    await store.openProject('/home/deck/Board/board.sninja')
    expect(store.lastOpenReport.relinked).toBe(1)
    expect(store.configFile.files[0]?.path).toBe('/home/deck/Board/Lolli/a.wav')
    expect(store.configFile.files[1]?.path).toBe('C:\\Users\\me\\Music\\Board\\gone.wav')
    expect(commitProject).toHaveBeenCalled()
    await store.validateSoundPaths()
    expect(store.missingPaths).toEqual(['C:\\Users\\me\\Music\\Board\\gone.wav'])
  })

  it('unpacks audio stored in the project for sounds missing on this machine', async () => {
    const store = boot()
    const config = emptyConfig()
    config.files = [sound('C:\\far\\away\\kick.wav', { id: 'kick0001' })]
    vi.mocked(loadProject).mockResolvedValue(
      loaded(config, { meta: { projectId: 'pid', embedAudio: true, embedGifs: true, saveOptionsSet: true } }),
    )
    vi.mocked(invoke).mockImplementation(async (cmd) => (cmd === 'paths_exist_abs' ? [false] : false))
    vi.mocked(extractProjectMedia).mockResolvedValue([{ soundId: 'kick0001', path: '/data/project-media/pid/kick.wav' }])
    await store.openProject('/home/deck/Board/board.sninja')
    expect(extractProjectMedia).toHaveBeenCalledWith('/home/deck/Board/board.sninja', 'pid')
    expect(store.lastOpenReport.extracted).toBe(1)
    expect(store.configFile.files[0]?.path).toBe('/data/project-media/pid/kick.wav')
    expect(vi.mocked(commitProject).mock.calls.at(-1)?.[2]).toMatchObject({ embedAudio: true })
  })

  it('brings undo and redo back with the project', async () => {
    const store = boot()
    const now = emptyConfig()
    now.files = [sound('/a.wav', { name: 'Now' })]
    const earlier = emptyConfig()
    earlier.files = [sound('/a.wav', { name: 'Earlier' })]
    earlier._hid = 7
    const later = emptyConfig()
    later.files = [sound('/a.wav', { name: 'Later' })]
    later._hid = 9
    vi.mocked(loadProject).mockResolvedValue(
      loaded(now, { history: { undo: [earlier], redo: [later], sizes: new Map([[7, 10], [9, 10]]) } }),
    )
    vi.mocked(invoke).mockImplementation(async (cmd) => (cmd === 'paths_exist_abs' ? [true] : false))
    await store.openProject('C:/board/project.sninja')
    expect(store.canUndo).toBe(true)
    expect(store.canRedo).toBe(true)

    expect(store.redo()).toBe(true)
    expect(store.configFile.files[0]?.name).toBe('Later')
    expect(store.undo()).toBe(true)
    expect(store.configFile.files[0]?.name).toBe('Now')
    expect(store.undo()).toBe(true)
    expect(store.configFile.files[0]?.name).toBe('Earlier')
    // History ids continue after the stored ones and never reach the live board.
    expect(store.redoStack.every((s) => (s._hid ?? 0) > 9 || s._hid === 9)).toBe(true)
    expect(store.configFile._hid).toBeUndefined()
  })
})

describe('jsonHandeling autosave', () => {
  beforeEach(reset)

  it('with autosave off, edits stay unsaved until Save', async () => {
    const store = boot()
    store.setAutosave(false)
    store.configFile.files = [sound('/a.wav', { name: 'A' })]
    store.renameSound(0, 'B')
    await vi.advanceTimersByTimeAsync(60_000)
    expect(commitProject).not.toHaveBeenCalled()
    expect(store.dirty).toBe(true)
    await store.persistNow()
    expect(commitProject).toHaveBeenCalledTimes(1)
    expect(store.dirty).toBe(false)
  })

  it('with an interval, several edits share one save', async () => {
    const store = boot()
    store.setAutosave(true, 30)
    store.configFile.files = [sound('/a.wav', { name: 'A' })]
    store.renameSound(0, 'B')
    await vi.advanceTimersByTimeAsync(20_000)
    store.renameSound(0, 'C')
    expect(commitProject).not.toHaveBeenCalled()
    await flushPersist(store, 10_000)
    expect(commitProject).toHaveBeenCalledTimes(1)
    expect(store.dirty).toBe(false)
  })

  it('saves pending edits when autosave is switched back on', async () => {
    const store = boot()
    store.setAutosave(false)
    store.configFile.files = [sound('/a.wav', { name: 'A' })]
    store.renameSound(0, 'B')
    store.setAutosave(true)
    await flushPersist(store)
    expect(commitProject).toHaveBeenCalledTimes(1)
  })

  it('does not mark the project unsaved for playback state or cached metadata', async () => {
    const store = boot()
    store.setAutosave(false)
    store.configFile.files = [sound('/a.wav')]
    store.setActiveSound({ soundindex: 0, status: true })
    store.ReturnStatusAll()
    store.writeConfig({ silent: true })
    await vi.advanceTimersByTimeAsync(1000)
    expect(store.dirty).toBe(false)
    expect(commitProject).not.toHaveBeenCalled()
  })

  it('discard returns to the saved board and its saved history', async () => {
    const store = boot()
    store.setAutosave(false)
    store.configFile.files = [sound('/a.wav', { name: 'A' })]
    store.renameSound(0, 'B')
    await store.persistNow()
    expect(store.undoStack).toHaveLength(1)
    store.renameSound(0, 'C')
    store.renameSound(0, 'D')
    await store.discardChanges()
    expect(store.configFile.files[0]?.name).toBe('B')
    expect(store.undoStack).toHaveLength(1)
    expect(store.dirty).toBe(false)
    expect(store.undo()).toBe(true)
    expect(store.configFile.files[0]?.name).toBe('A')
  })

  it('save as copies the project, then writes the current board with the chosen options', async () => {
    const store = boot()
    store.setAutosave(false)
    store.configFile.files = [sound('/a.wav')]
    await store.saveAs('D:/share/board.sninja', { embedAudio: true, embedGifs: false })
    expect(copyProject).toHaveBeenCalledWith('C:/board/project.sninja', 'D:/share/board.sninja')
    expect(store.currentProjectPath).toBe('D:/share/board.sninja')
    expect(store.projectMeta).toMatchObject({ embedAudio: true, embedGifs: false, saveOptionsSet: true })
    const [path, , options] = vi.mocked(commitProject).mock.calls.at(-1)!
    expect(path).toBe('D:/share/board.sninja')
    expect(options).toMatchObject({ embedAudio: true, embedGifs: false })
  })
})

describe('jsonHandeling board edits', () => {
  beforeEach(reset)

  it('adds, renames, reorders, and removes tabs', () => {
    const store = boot()
    store.addTab('  ')
    store.addTab('Lolli')
    store.addTab('Lolli')
    store.addTab('Memes')
    expect(store.configFile.tabList.map((t) => t.name)).toEqual(['Lolli', 'Memes'])
    store.reorderTabs('Memes', 'Lolli')
    expect(store.configFile.tabList.map((t) => t.name)).toEqual(['Memes', 'Lolli'])
    store.renameTab('Memes', 'Jokes')
    store.removeTab('Jokes')
    expect(store.configFile.tabList.map((t) => t.name)).toEqual(['Lolli'])
  })

  it('removing a tab takes it off every sound, so it cannot come back on reopen', () => {
    const store = boot()
    store.addTab('Lolli')
    store.configFile.files = [sound('/a.wav', { tabs: ['All', 'Lolli'], tabIndexes: { Lolli: 0 } })]
    store.removeTab('Lolli')
    expect(store.configFile.files[0]?.tabs).toEqual(['All'])
    expect(store.configFile.files[0]?.tabIndexes).toEqual({})
  })

  it('clamps volume, ignores an unchanged value, and toggles loop', () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav', { volume: 0.5 })]
    store.setSoundVolume(0, 5)
    expect(store.configFile.files[0]?.volume).toBe(1)
    const undoLen = store.undoStack.length
    store.setSoundVolume(0, 1)
    expect(store.undoStack.length).toBe(undoLen)
    store.setSoundVolume(0, -1)
    expect(store.configFile.files[0]?.volume).toBe(0)
    store.setSoundLoop(0, true)
    expect(store.configFile.files[0]?.looping).toBe(true)
    const afterLoop = store.undoStack.length
    store.setSoundLoop(0, true)
    expect(store.undoStack.length).toBe(afterLoop)
  })

  it('adds, toggles, and removes a tag', () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav')]
    const tag = store.addTag('Hits', '#ffffff')
    store.toggleSoundTag('/a.wav', tag.id)
    expect(store.configFile.files[0]?.tagIds).toEqual([tag.id])
    store.removeTag(tag.id)
    expect(store.configFile.tags).toEqual([])
    expect(store.configFile.files[0]?.tagIds).toEqual([])
  })

  it('adds and removes a separator', () => {
    const store = boot()
    store.addSeparator('All', 1, 'Group')
    expect(store.configFile.separators).toHaveLength(1)
    expect(store.configFile.separators[0]?.name).toBe('Group')
    store.removeSeparator(store.configFile.separators[0]!.id)
    expect(store.configFile.separators).toEqual([])
  })

  it('gives a new group a whole-number slot between its neighbours', () => {
    const store = boot()
    store.configFile.files = [
      sound('/a.wav', { index: 0 }),
      sound('/b.wav', { id: 'bbbbbbbb', index: 1 }),
      sound('/c.wav', { id: 'cccccccc', index: 2 }),
    ]
    store.addSeparator('All', 0.5, 'Group')
    expect(store.captureBoardLayout('All')).toEqual({
      orphans: ['/a.wav'],
      groups: [{ id: store.configFile.separators[0]!.id, paths: ['/b.wav', '/c.wav'] }],
    })
    const slots = [...store.configFile.files.map((f) => f.index), store.configFile.separators[0]!.position]
    expect(slots.every(Number.isInteger)).toBe(true)
    expect(new Set(slots).size).toBe(4)
  })

  it('reorders sounds on All', () => {
    const store = boot()
    store.configFile.files = [
      sound('/a.wav', { index: 0 }),
      sound('/b.wav', { id: 'bbbbbbbb', index: 1 }),
    ]
    store.reorderSounds(0, 1, 'All')
    expect(store.configFile.files.find((f) => f.path === '/a.wav')?.index).toBe(1)
    expect(store.configFile.files.find((f) => f.path === '/b.wav')?.index).toBe(0)
  })

  it('keeps hidden sounds in their groups when a filtered view is rearranged', () => {
    const store = boot()
    store.configFile.files = [
      sound('/a.wav', { index: 0 }),
      sound('/b.wav', { id: 'bbbbbbbb', index: 2 }),
      sound('/c.wav', { id: 'cccccccc', index: 3 }),
      sound('/d.wav', { id: 'dddddddd', index: 5 }),
    ]
    store.configFile.separators = [
      { id: 'g1', tab: 'All', position: 1 },
      { id: 'g2', tab: 'All', position: 4 },
    ]
    // A search shows only a + d; the user drops "a" behind "d" in group g2.
    store.applyBoardLayout('All', {
      orphans: [],
      groups: [{ id: 'g1', paths: [] }, { id: 'g2', paths: ['/d.wav', '/a.wav'] }],
    })
    expect(store.captureBoardLayout('All')).toEqual({
      orphans: [],
      groups: [{ id: 'g1', paths: ['/b.wav', '/c.wav'] }, { id: 'g2', paths: ['/d.wav', '/a.wav'] }],
    })
  })

  it('undoes and redoes one edit, and caps the undo stack', () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav', { name: 'A' })]
    store.renameSound(0, 'Z')
    expect(store.configFile.files[0]?.name).toBe('Z')
    expect(store.undo()).toBe(true)
    expect(store.configFile.files[0]?.name).toBe('A')
    expect(store.redo()).toBe(true)
    expect(store.configFile.files[0]?.name).toBe('Z')

    for (let i = 0; i < 51; i++) store.renameSound(0, `n${i}`)
    expect(store.undoStack.length).toBe(50)
  })

  it('discards edits back to the opening snapshot without saving', async () => {
    const store = boot()
    const opened = emptyConfig()
    opened.files = [sound('/a.wav', { name: 'A' })]
    store.configFile = JSON.parse(JSON.stringify(opened))
    store.openingSnapshot = JSON.parse(JSON.stringify(opened))
    store.renameSound(0, 'B')
    await store.discardChanges()
    expect(store.configFile.files[0]?.name).toBe('A')
    expect(store.dirty).toBe(false)
    expect(saveConfig).not.toHaveBeenCalled()
  })
})
