/**
 * Save → load against a real SQLite engine (node:sqlite), through the same
 * `project_query` / `project_commit` contract the Rust side implements.
 * Skipped on Node builds that ship without node:sqlite.
 */
import { beforeEach, describe, expect, it, vi } from 'vitest'

const backend = vi.hoisted(() => {
  type Sqlite = {
    exec(sql: string): void
    prepare(sql: string): { all(...p: unknown[]): unknown[]; run(...p: unknown[]): unknown }
  }
  type NodeProcess = {
    getBuiltinModule?: (id: string) => { DatabaseSync: new (path: string) => Sqlite } | undefined
  }
  const mod = (globalThis as unknown as { process?: NodeProcess }).process?.getBuiltinModule?.('node:sqlite')
  const files = new Map<string, Sqlite>()

  const open = (path: string): Sqlite => {
    let db = files.get(path)
    if (!db) files.set(path, (db = new mod!.DatabaseSync(':memory:')))
    return db
  }

  /** `$N` placeholders → anonymous `?` in order of appearance. */
  const expand = (sql: string, params: unknown[] = []) => {
    const args: unknown[] = []
    const text = sql.replace(/\$(\d+)/g, (_m, n: string) => {
      const v = params[Number(n) - 1]
      args.push(typeof v === 'boolean' ? Number(v) : v ?? null)
      return '?'
    })
    return { text, args }
  }

  type Stmt = { sql: string; params?: unknown[]; tolerant?: boolean }

  const invoke = async (cmd: string, a: Record<string, unknown>): Promise<unknown> => {
    if (cmd === 'project_query') {
      const { text, args } = expand(a.sql as string, a.params as unknown[])
      return open(a.path as string).prepare(text).all(...args).map((row) => ({ ...(row as object) }))
    }
    if (cmd === 'project_commit') {
      const db = open(a.path as string)
      db.exec('BEGIN')
      try {
        for (const stmt of a.statements as Stmt[]) {
          const { text, args } = expand(stmt.sql, stmt.params)
          try {
            db.prepare(text).run(...args)
          } catch (e) {
            if (!stmt.tolerant) throw e
          }
        }
        db.exec('COMMIT')
      } catch (e) {
        db.exec('ROLLBACK')
        throw e
      }
      return { embeddedAudio: 0, embeddedBytes: 0, missingAudio: [], backupWritten: false }
    }
    throw new Error(`unexpected command ${cmd}`)
  }

  return { available: !!mod, files, open, invoke }
})

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args: Record<string, unknown>) => backend.invoke(cmd, args),
  convertFileSrc: (p: string) => p,
}))

import {
  commitProject,
  emptyConfig,
  forgetProject,
  loadHistory,
  loadLatestSnapshot,
  loadMeta,
  loadProject,
  PROJECT_FORMAT_VERSION,
  saveConfig,
  saveHistory,
  saveMeta,
  saveSnapshot,
  withProjectDb,
  type ProjectConfig,
  type SoundFile,
} from '~/utils/db'
import { applyBoardLayout, captureBoardLayout } from '~/utils/boardLayout'

const PATH = '/board/project.sninja'

function sound(path: string, extra: Partial<SoundFile> = {}): SoundFile {
  return {
    name: path.replace(/^.*\//, '').replace(/\.wav$/, ''),
    id: '',
    volume: 1,
    tabs: ['All'],
    active: false,
    index: 0,
    tabIndexes: {},
    tagIds: [],
    looping: false,
    ...extra,
    path,
  }
}

/**
 * All:    kick | [Drums] snare hat | [Voices] hello
 * Lolli:  [Pink] hello snare
 */
function board(): ProjectConfig {
  const config = emptyConfig()
  config.settings.theme = 'soundninja'
  config.settings.perSoundVolumeV1 = true
  config.settings.soundHotkeys = [{ id: 'h1', soundId: 'snare001', combo: 'Ctrl+1' }]
  config.tabList = [{ name: 'Lolli', color: '#ff00aa', buttonAlign: 'center' }, { name: 'Empty' }]
  config.tags = [{ id: 'tagdrums', name: 'Drums', color: '#112233' }]
  config.files = [
    sound('/board/kick.wav', { id: 'kick0001', color: '{"bg":"#ff0000"}', volume: 0.5, addedAt: 11, fileSize: 22, durationSecs: 1.5 }),
    sound('/board/snare.wav', { id: 'snare001', tabs: ['All', 'Lolli'], tagIds: ['tagdrums'], looping: true }),
    sound('/board/hat.wav', { id: 'hat00001', gifId: 'abc123', gifPosX: 10, gifPosY: 90 }),
    sound('/board/hello.wav', { id: 'hello001', tabs: ['All', 'Lolli'], name: 'Hello there' }),
  ]
  config.separators = [
    { id: 'gDrums', tab: 'All', position: 0, name: 'Drums', borderColor: '#aa0000', nameColor: '#00aa00', bgColor: '#0000aa', buttonAlign: 'right' },
    { id: 'gVoices', tab: 'All', position: 0, name: 'Voices' },
    { id: 'gPink', tab: 'Lolli', position: 0, name: 'Pink' },
  ]
  applyBoardLayout(config, 'All', {
    orphans: ['/board/kick.wav'],
    groups: [
      { id: 'gDrums', paths: ['/board/snare.wav', '/board/hat.wav'] },
      { id: 'gVoices', paths: ['/board/hello.wav'] },
    ],
  })
  applyBoardLayout(config, 'Lolli', {
    orphans: [],
    groups: [{ id: 'gPink', paths: ['/board/hello.wav', '/board/snare.wav'] }],
  })
  return config
}

async function save(config: ProjectConfig) {
  await commitProject(PATH, (d) => saveConfig(d, config, { appVersion: '9.9.9' }))
}

function exec(sql: string) {
  backend.open(PATH).exec(sql)
}

describe.skipIf(!backend.available)('project file round trip', () => {
  beforeEach(() => {
    backend.files.clear()
    forgetProject(PATH)
  })

  it('keeps tabs, groups, order, colors, names, tags and per-sound settings', async () => {
    const before = board()
    await save(before)
    const { config, formatVersion, marker } = await loadProject(PATH)

    expect(formatVersion).toBe(PROJECT_FORMAT_VERSION)
    expect(marker).toBe(4)
    expect(config.tabList).toEqual(before.tabList)
    expect(config.tags).toEqual(before.tags)
    expect(config.settings.soundHotkeys).toEqual(before.settings.soundHotkeys)
    for (const tab of ['All', 'Lolli', 'Empty']) {
      expect(captureBoardLayout(config, tab)).toEqual(captureBoardLayout(before, tab))
    }
    const sep = config.separators.find((s) => s.id === 'gDrums')
    expect(sep).toMatchObject({ name: 'Drums', borderColor: '#aa0000', nameColor: '#00aa00', bgColor: '#0000aa', buttonAlign: 'right' })

    const byPath = new Map(config.files.map((f) => [f.path, f]))
    expect(byPath.get('/board/kick.wav')).toMatchObject({
      id: 'kick0001', color: '{"bg":"#ff0000"}', volume: 0.5, addedAt: 11, fileSize: 22, durationSecs: 1.5,
    })
    expect(byPath.get('/board/snare.wav')).toMatchObject({ tabs: ['All', 'Lolli'], tagIds: ['tagdrums'], looping: true })
    expect(byPath.get('/board/hat.wav')).toMatchObject({ gifId: 'abc123', gifPosX: 10, gifPosY: 90 })
    expect(byPath.get('/board/hello.wav')?.name).toBe('Hello there')
  })

  it('is stable: saving what was loaded changes nothing', async () => {
    await save(board())
    const first = (await loadProject(PATH)).config
    await save(first)
    const second = (await loadProject(PATH)).config
    expect(second).toEqual(first)
  })

  it('keeps every sound in its group when all stored positions collide', async () => {
    const before = board()
    await save(before)
    exec('UPDATE sounds SET global_index = 0')
    exec('UPDATE sound_tabs SET tab_index = 0')
    exec('UPDATE separators SET position = 0')
    const { config } = await loadProject(PATH)
    const groups = (tab: string, c: ProjectConfig) =>
      captureBoardLayout(c, tab).groups.map((g) => [g.id, [...g.paths].sort()])
    expect(captureBoardLayout(config, 'All').orphans).toEqual(['/board/kick.wav'])
    expect(groups('All', config)).toEqual(groups('All', before))
    expect(groups('Lolli', config)).toEqual(groups('Lolli', before))
  })

  it('leaves the file untouched when any statement of a save fails', async () => {
    await save(board())
    await expect(
      commitProject(PATH, async (d) => {
        await saveConfig(d, emptyConfig(), { allowEmpty: true })
        await d.execute('INSERT INTO no_such_table (x) VALUES (1)')
      }),
    ).rejects.toThrow()
    const { config } = await loadProject(PATH)
    expect(config.files).toHaveLength(4)
    expect(config.separators).toHaveLength(3)
    expect(config.tabList.map((t) => t.name)).toEqual(['Lolli', 'Empty'])
  })

  it('writes nothing when building the save throws', async () => {
    await save(board())
    await expect(
      commitProject(PATH, async (d) => {
        await d.execute('DELETE FROM sounds')
        throw new Error('boom')
      }),
    ).rejects.toThrow('boom')
    expect((await loadProject(PATH)).config.files).toHaveLength(4)
  })

  it('reads a file written by an older build by position and reports no format version', async () => {
    await withProjectDb(PATH, async () => {}) // creates the schema
    exec("INSERT INTO settings (key, value) VALUES ('theme', 'soundninja'), ('soundCount', '3')")
    exec("INSERT INTO tabs (name, color, position) VALUES ('Lolli', NULL, 0)")
    exec(`INSERT INTO sounds (path, name, volume, global_index, active, sound_id) VALUES
      ('C:\\board\\a.wav', 'A', 1, 0, 1, 'aaaaaaaa'),
      ('C:\\board\\b.wav', 'B', 1, 2, 0, 'bbbbbbbb'),
      ('C:\\board\\c.wav', 'C', 1, 3, 0, 'cccccccc')`)
    exec(`INSERT INTO sound_tabs (sound_path, tab, tab_index) VALUES
      ('C:\\board\\a.wav', 'All', 0), ('C:\\board\\b.wav', 'All', 2), ('C:\\board\\c.wav', 'All', 3),
      ('C:\\board\\b.wav', 'Lolli', 0), ('C:\\board\\b.wav', 'Lolli', 0)`)
    exec("INSERT INTO separators (id, tab, position, name) VALUES ('g1', 'All', 1, 'Old group')")

    const { config, formatVersion } = await loadProject(PATH)
    expect(formatVersion).toBeNull()
    expect(captureBoardLayout(config, 'All')).toEqual({
      orphans: ['C:\\board\\a.wav'],
      groups: [{ id: 'g1', paths: ['C:\\board\\b.wav', 'C:\\board\\c.wav'] }],
    })
    // A duplicated sound_tabs row must not list the tab twice.
    expect(config.files.find((f) => f.name === 'B')?.tabs).toEqual(['All', 'Lolli'])
    // "Playing" is never restored from disk.
    expect(config.files.every((f) => f.active === false)).toBe(true)
  })

  it('trusts positions again after an older build rewrote a new-format file', async () => {
    await save(board())
    // An older build rewrites `settings` (dropping formatVersion) and
    // `sound_tabs` (without group_id), and moves "kick" into the Drums group.
    exec("DELETE FROM settings WHERE key = 'formatVersion'")
    exec('UPDATE sound_tabs SET group_id = NULL')
    exec("UPDATE separators SET position = 0 WHERE id = 'gDrums'")
    exec("UPDATE sounds SET global_index = 1 WHERE path = '/board/kick.wav'")
    exec("UPDATE sounds SET global_index = 2 WHERE path = '/board/snare.wav'")
    exec("UPDATE sounds SET global_index = 3 WHERE path = '/board/hat.wav'")
    const { config, formatVersion } = await loadProject(PATH)
    expect(formatVersion).toBeNull()
    const layout = captureBoardLayout(config, 'All')
    expect(layout.orphans).toEqual([])
    expect(layout.groups[0]).toEqual({ id: 'gDrums', paths: ['/board/kick.wav', '/board/snare.wav', '/board/hat.wav'] })
  })

  it('carries settings of a newer build through load and save', async () => {
    await save(board())
    exec("INSERT INTO settings (key, value) VALUES ('futureFeature', 'on')")
    const { config } = await loadProject(PATH)
    expect(config.extraSettings).toEqual({ futureFeature: 'on' })
    await save(config)
    const rows = await withProjectDb(PATH, (d) =>
      d.select<{ value: string }[]>("SELECT value FROM settings WHERE key = 'futureFeature'"))
    expect(rows).toEqual([{ value: 'on' }])
  })

  it('drops a tag filter that names tags which no longer exist', async () => {
    const config = board()
    config.settings.selectedTagIds = ['tagdrums', 'ghost']
    await save(config)
    exec("DELETE FROM tags WHERE id = 'tagdrums'")
    const loaded = (await loadProject(PATH)).config
    // Otherwise every tab would render empty.
    expect(loaded.settings.selectedTagIds).toEqual([])
    expect(loaded.files.every((f) => (f.tagIds ?? []).length === 0)).toBe(true)
  })

  it('stores project options and keeps them when an older build rewrites the board', async () => {
    await commitProject(PATH, async (d) => {
      await saveConfig(d, board())
      await saveMeta(d, { projectId: 'pid-1', embedAudio: true, embedGifs: false, saveOptionsSet: true })
    })
    exec('DELETE FROM settings')
    const meta = await withProjectDb(PATH, (d) => loadMeta(d))
    expect(meta).toEqual({ projectId: 'pid-1', embedAudio: true, embedGifs: false, saveOptionsSet: true })
  })

  it('persists undo and redo stacks and only sends new entries again', async () => {
    const a = { ...board(), _hid: 1 }
    const b = { ...board(), _hid: 2 }
    b.files[0]!.name = 'Renamed'
    const c = { ...emptyConfig(), _hid: 3 }

    let stored = new Map<number, number>()
    await commitProject(PATH, async (d) => {
      stored = await saveHistory(d, [a, b], [c], stored)
    })
    expect([...stored.keys()].sort()).toEqual([1, 2, 3])

    const loaded = await withProjectDb(PATH, (d) => loadHistory(d))
    expect(loaded.undo.map((s) => s._hid)).toEqual([1, 2])
    expect(loaded.redo.map((s) => s._hid)).toEqual([3])
    expect(loaded.undo[1]?.files[0]?.name).toBe('Renamed')

    // Undo once: entry 2 leaves the undo stack, a new entry 4 lands on redo.
    const d4 = { ...board(), _hid: 4 }
    const sent: string[] = []
    await commitProject(PATH, async (d) => {
      const spy = { select: d.select.bind(d), execute: async (sql: string, p?: unknown[]) => { sent.push(sql); return d.execute(sql, p) } }
      stored = await saveHistory(spy, [a], [c, d4], stored)
    })
    expect(sent.filter((s) => s.startsWith('INSERT'))).toHaveLength(1)
    const after = await withProjectDb(PATH, (d) => loadHistory(d))
    expect(after.undo.map((s) => s._hid)).toEqual([1])
    expect(after.redo.map((s) => s._hid)).toEqual([3, 4])
  })

  it('keeps recovery snapshots and returns the newest one that has sounds', async () => {
    await commitProject(PATH, async (d) => {
      await saveSnapshot(d, board(), 'session', '1.0.0')
      await saveSnapshot(d, emptyConfig(), 'session', '1.0.0')
    })
    const snap = await withProjectDb(PATH, (d) => loadLatestSnapshot(d))
    expect(snap?.files).toHaveLength(4)
    expect(captureBoardLayout(snap!, 'All').groups.map((g) => g.id)).toEqual(['gDrums', 'gVoices'])

    for (let i = 0; i < 8; i++) {
      await commitProject(PATH, (d) => saveSnapshot(d, board(), 'session'))
    }
    const rows = await withProjectDb(PATH, (d) => d.select<{ c: number }[]>('SELECT COUNT(*) AS c FROM snapshots'))
    expect(rows[0]?.c).toBe(5)
  })
})
