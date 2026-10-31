import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/plugin-sql', () => ({ default: class Database {} }))

import {
  emptyConfig,
  readSoundCountMarker,
  saveConfig,
  shouldRestoreClearedBoard,
  type SoundFile,
} from '~/utils/db'

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
    ...extra,
    path,
  }
}

type Call = { sql: string; params?: unknown[] }

function fakeDb(soundCount: number) {
  const calls: Call[] = []
  let sounds = soundCount
  return {
    calls,
    async select<T>(sql: string): Promise<T> {
      calls.push({ sql })
      if (sql.includes('COUNT(*)')) return [{ c: sounds }] as T
      return [] as T
    },
    async execute(sql: string, params?: unknown[]) {
      calls.push({ sql, params })
      if (sql === 'DELETE FROM sounds') sounds = 0
    },
  }
}

function markerCall(calls: Call[]) {
  return calls.find((c) => c.sql.includes("'soundCount'"))
}

describe('shouldRestoreClearedBoard', () => {
  it('keeps a finished empty board', () => {
    expect(shouldRestoreClearedBoard(0, 0)).toBe(false)
  })

  it('restores when the marker never landed', () => {
    expect(shouldRestoreClearedBoard(0, null)).toBe(true)
  })

  it('restores when the marker still expects sounds', () => {
    expect(shouldRestoreClearedBoard(0, 4)).toBe(true)
  })

  it('leaves a non-empty board alone', () => {
    expect(shouldRestoreClearedBoard(2, null)).toBe(false)
    expect(shouldRestoreClearedBoard(2, 0)).toBe(false)
  })
})

describe('readSoundCountMarker', () => {
  it('reads a numeric marker', async () => {
    const d = {
      select: vi.fn(async () => [{ value: '0' }]),
      execute: vi.fn(),
    }
    expect(await readSoundCountMarker(d as never)).toBe(0)
  })

  it('returns null when the row is missing or the read fails', async () => {
    const empty = { select: vi.fn(async () => []), execute: vi.fn() }
    expect(await readSoundCountMarker(empty as never)).toBeNull()
    const broken = { select: vi.fn(async () => { throw new Error('no table') }), execute: vi.fn() }
    expect(await readSoundCountMarker(broken as never)).toBeNull()
  })
})

describe('saveConfig empty board', () => {
  it('refuses an unmarked empty list over existing sounds', async () => {
    const db = fakeDb(3)
    const config = emptyConfig()
    await expect(saveConfig(db as never, config)).rejects.toThrow(/Refusing to persist 0 sounds/)
    expect(db.calls.some((c) => c.sql.startsWith('DELETE'))).toBe(false)
  })

  it('persists an intentional empty list and writes soundCount 0 last', async () => {
    const db = fakeDb(3)
    const config = emptyConfig()
    await saveConfig(db as never, config, { allowEmpty: true })
    const marker = markerCall(db.calls)
    const deleteAt = db.calls.findIndex((c) => c.sql === 'DELETE FROM sounds')
    const markerAt = db.calls.findIndex((c) => c.sql.includes("'soundCount'"))
    const settingsBatch = db.calls.find(
      (c) => c.sql.startsWith('INSERT INTO settings') && !c.sql.includes('soundCount'),
    )
    expect(marker?.params).toEqual(['0'])
    expect(markerAt).toBeGreaterThan(deleteAt)
    expect(settingsBatch?.params ?? []).not.toContain('soundCount')
    expect(db.calls.some((c) => c.sql.includes('INSERT OR REPLACE INTO sounds'))).toBe(false)
  })

  it('writes soundCount equal to the sound rows after those rows', async () => {
    const db = fakeDb(0)
    const config = emptyConfig()
    config.files = [sound('/a.wav'), sound('/b.wav', { id: 'bbbbbbbb' })]
    await saveConfig(db as never, config)
    const markerAt = db.calls.findIndex((c) => c.sql.includes("'soundCount'"))
    const soundsAt = db.calls.findIndex((c) => c.sql.includes('INSERT OR REPLACE INTO sounds'))
    expect(markerCall(db.calls)?.params).toEqual(['2'])
    expect(markerAt).toBeGreaterThan(soundsAt)
  })
})
