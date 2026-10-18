import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/plugin-sql', () => ({ default: class Database {} }))

import { dedupeSoundsByPath, emptyConfig, mergeSoundInto, type SoundFile } from '~/utils/db'

function sound(path: string, extra: Partial<SoundFile> = {}): SoundFile {
  return {
    name: path,
    id: '',
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

describe('dedupeSoundsByPath', () => {
  it('merges tabs and tags when two sounds share a path', () => {
    const config = emptyConfig()
    config.files = [
      sound('/board/a.wav', { id: 'aaaaaaaa', tabs: ['All', 'Lolli'], tagIds: ['t1'] }),
      sound('/board/a.wav', { id: 'bbbbbbbb', tabs: ['All', 'Memes'], tagIds: ['t2'], gifId: 'gif1' }),
      sound('/board/b.wav', { id: 'cccccccc' }),
    ]
    expect(dedupeSoundsByPath(config)).toBe(1)
    expect(config.files).toHaveLength(2)
    expect(config.files[0]?.id).toBe('aaaaaaaa')
    expect(config.files[0]?.tabs).toEqual(['All', 'Lolli', 'Memes'])
    expect(config.files[0]?.tagIds).toEqual(['t1', 't2'])
    expect(config.files[0]?.gifId).toBe('gif1')
    expect(config.files[1]?.path).toBe('/board/b.wav')
  })

  it('keeps paths that differ only by case', () => {
    const config = emptyConfig()
    config.files = [
      sound('/board/Foo.wav'),
      sound('/board/foo.wav'),
    ]
    expect(dedupeSoundsByPath(config)).toBe(0)
    expect(config.files).toHaveLength(2)
  })

  it('is a no-op when every path is unique', () => {
    const config = emptyConfig()
    config.files = [sound('/a.wav'), sound('/b.wav')]
    expect(dedupeSoundsByPath(config)).toBe(0)
    expect(config.files.map((f) => f.path)).toEqual(['/a.wav', '/b.wav'])
  })
})

describe('mergeSoundInto', () => {
  it('does not overwrite an existing gif', () => {
    const keep = sound('/a.wav', { gifId: 'keep' })
    mergeSoundInto(keep, sound('/a.wav', { gifId: 'extra' }))
    expect(keep.gifId).toBe('keep')
  })
})
