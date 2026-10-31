import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/plugin-sql', () => ({ default: class Database {} }))

import { emptyConfig, type SoundFile } from '~/utils/db'
import { buildPortableExport, resolveImportedPaths } from '~/utils/soundboardZip'

function sound(path: string, extra: Partial<SoundFile> = {}): SoundFile {
  return {
    name: 'hit',
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

describe('soundboard zip paths', () => {
  it('exports an empty board with no copy entries', () => {
    const { portable, entries } = buildPortableExport(emptyConfig())
    expect(portable.files).toEqual([])
    expect(entries).toEqual([])
  })

  it('rewrites one sound to a relative path and back under the project dir', () => {
    const config = emptyConfig()
    config.files = [sound('C:/board/hits/kick.wav', { name: 'Kick', tabs: ['All', 'Hits'] })]
    const { portable, entries } = buildPortableExport(config)
    expect(entries).toEqual([{ src: 'C:/board/hits/kick.wav', destRel: 'sounds/Hits/kick.wav' }])
    expect(portable.files[0]?.path).toBe('sounds/Hits/kick.wav')
    expect(config.files[0]?.path).toBe('C:/board/hits/kick.wav')

    resolveImportedPaths(portable, 'D:/projects/imported')
    expect(portable.files[0]?.path).toBe('D:/projects/imported/sounds/Hits/kick.wav')
  })

  it('joins imported relative paths with the OS separator of the project dir', () => {
    const config = emptyConfig()
    config.files = [sound('sounds/All/kick.wav')]
    resolveImportedPaths(config, 'D:\\projects\\imported')
    expect(config.files[0]?.path).toBe('D:\\projects\\imported\\sounds\\All\\kick.wav')
  })
})
