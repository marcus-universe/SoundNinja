import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/plugin-sql', () => ({ default: class Database {} }))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(async () => false),
  convertFileSrc: (p: string) => p,
}))

vi.mock('~/utils/db', async (importOriginal) => {
  const actual = await importOriginal<typeof import('~/utils/db')>()
  return {
    ...actual,
    openDb: vi.fn(async () => {}),
    reopenDb: vi.fn(async () => {}),
    withProjectDb: vi.fn(async (_path: string, fn: (d: unknown) => Promise<unknown>) => fn({})),
    saveConfig: vi.fn(async () => {}),
    gcOrphanGifs: vi.fn(async () => {}),
    persistSoundIds: vi.fn(async () => {}),
    loadConfig: vi.fn(async () => actual.emptyConfig()),
    readSoundCountMarker: vi.fn(async () => 0),
  }
})

import { emptyConfig, loadConfig, readSoundCountMarker, saveConfig, type SoundFile } from '~/utils/db'
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

function boot() {
  const store = useJsonHandelingStore()
  store.setCurrentProjectPath('C:/board/project.sninja')
  return store
}

async function flushPersist(store: { persistNow: () => Promise<unknown> }) {
  const spy = vi.spyOn(store, 'persistNow')
  await vi.advanceTimersByTimeAsync(200)
  await Promise.all(spy.mock.results.map((r) => r.value))
  spy.mockRestore()
}

describe('jsonHandeling persistence', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
    vi.clearAllMocks()
  })

  it('saves deleting the last sound as an intentional empty board', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav')]
    store.removeSound(0)
    await flushPersist(store)
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), { allowEmpty: true })
  })

  it('saves a bulk delete of every sound as an intentional empty board', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav'), sound('/b.wav', { id: 'bbbbbbbb', index: 1 })]
    store.removeSoundsMany(['/a.wav', '/b.wav'])
    await flushPersist(store)
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), { allowEmpty: true })
  })

  it('saves resetAll as an intentional empty board', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav')]
    store.resetAll()
    await flushPersist(store)
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), { allowEmpty: true })
  })

  it('saves undo back to an empty board', async () => {
    const store = boot()
    store.configFile.files = [sound('/a.wav', { name: 'A' })]
    store.undoStack.push(emptyConfig())
    store.undo()
    await flushPersist(store)
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), { allowEmpty: true })
  })

  it('saves an imported empty project', async () => {
    const store = boot()
    await store.importConfig(emptyConfig(), 'C:/board/imported.sninja')
    expect(store.configFile.files).toHaveLength(0)
    expect(saveConfig).toHaveBeenCalledWith(expect.anything(), expect.anything(), { allowEmpty: true })
  })

  it('does not restore a finished empty project', async () => {
    const store = boot()
    const config = emptyConfig()
    config.settings.perSoundVolumeV1 = true
    vi.mocked(loadConfig).mockResolvedValue(config)
    vi.mocked(readSoundCountMarker).mockResolvedValue(0)
    await store.openProject('C:/board/empty.sninja')
    expect(store.configFile.files).toHaveLength(0)
    expect(invoke).not.toHaveBeenCalled()
  })

  it('checks the backup when an empty project has no soundCount marker', async () => {
    const store = boot()
    const config = emptyConfig()
    config.settings.perSoundVolumeV1 = true
    vi.mocked(loadConfig).mockResolvedValue(config)
    vi.mocked(readSoundCountMarker).mockResolvedValue(null)
    await store.openProject('C:/board/empty.sninja')
    expect(invoke).toHaveBeenCalledWith('path_exists_abs', { path: 'C:/board/empty.sninja.bak' })
  })
})

describe('jsonHandeling board edits', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
    vi.clearAllMocks()
  })

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
