import { describe, expect, it, vi } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(), convertFileSrc: (p: string) => p }))

import { emptyConfig, type ProjectConfig, type SoundFile } from '~/utils/db'
import {
  applyBoardLayout,
  captureBoardLayout,
  groupMembership,
  layoutFromMembership,
  mergeHiddenIntoLayout,
} from '~/utils/boardLayout'

function sound(path: string, index: number, extra: Partial<SoundFile> = {}): SoundFile {
  return {
    name: path,
    id: '',
    volume: 1,
    tabs: ['All'],
    active: false,
    index,
    tabIndexes: {},
    tagIds: [],
    ...extra,
    path,
  }
}

/** All: a | [g1] b c | [g2] d */
function board(): ProjectConfig {
  const config = emptyConfig()
  config.files = [sound('a', 0), sound('b', 2), sound('c', 3), sound('d', 5)]
  config.separators = [
    { id: 'g1', tab: 'All', position: 1, name: 'One' },
    { id: 'g2', tab: 'All', position: 4, name: 'Two' },
  ]
  return config
}

describe('captureBoardLayout', () => {
  it('splits a tab into orphans and groups by position', () => {
    expect(captureBoardLayout(board(), 'All')).toEqual({
      orphans: ['a'],
      groups: [{ id: 'g1', paths: ['b', 'c'] }, { id: 'g2', paths: ['d'] }],
    })
  })

  it('returns every sound as an orphan on a tab without groups', () => {
    const config = board()
    config.separators = []
    expect(captureBoardLayout(config, 'All')).toEqual({ orphans: ['a', 'b', 'c', 'd'], groups: [] })
  })

  it('ignores groups and sounds of other tabs', () => {
    const config = board()
    config.tabList = [{ name: 'Lolli' }]
    config.files.push(sound('x', 9, { tabs: ['All', 'Lolli'], tabIndexes: { Lolli: 0 } }))
    expect(captureBoardLayout(config, 'Lolli')).toEqual({ orphans: ['x'], groups: [] })
  })
})

describe('applyBoardLayout', () => {
  it('writes one dense number line shared by sounds and group markers', () => {
    const config = board()
    applyBoardLayout(config, 'All', {
      orphans: [],
      groups: [{ id: 'g2', paths: ['d', 'a'] }, { id: 'g1', paths: ['c', 'b'] }],
    })
    const order = Object.fromEntries(config.files.map((f) => [f.path, f.index]))
    const seps = Object.fromEntries(config.separators.map((s) => [s.id, s.position]))
    expect(seps).toEqual({ g2: 0, g1: 3 })
    expect(order).toEqual({ d: 1, a: 2, c: 4, b: 5 })
  })

  it('gives a sound the layout forgot a slot of its own', () => {
    const config = board()
    applyBoardLayout(config, 'All', { orphans: ['a'], groups: [{ id: 'g1', paths: ['b'] }, { id: 'g2', paths: ['d'] }] })
    const used = [...config.files.map((f) => f.index), ...config.separators.map((s) => s.position)]
    expect(new Set(used).size).toBe(used.length)
  })

  it('round-trips through capture', () => {
    const config = board()
    const before = captureBoardLayout(config, 'All')
    applyBoardLayout(config, 'All', before)
    expect(captureBoardLayout(config, 'All')).toEqual(before)
  })
})

describe('layoutFromMembership', () => {
  it('rebuilds groups from explicit membership even when every position collides', () => {
    const config = board()
    const membership = groupMembership(config, 'All')
    // What a damaged file looks like: all sounds and markers on slot 0.
    for (const f of config.files) f.index = 0
    for (const s of config.separators) s.position = 0
    const layout = layoutFromMembership(config, 'All', (p) => membership.get(p))
    expect(layout.orphans).toEqual(['a'])
    expect(layout.groups.map((g) => [g.id, g.paths])).toEqual([['g1', ['b', 'c']], ['g2', ['d']]])
  })

  it('turns a sound of a deleted group into an orphan', () => {
    const config = board()
    const layout = layoutFromMembership(config, 'All', (p) => (p === 'a' ? 'gone' : undefined))
    expect(layout.orphans).toEqual(['a', 'b', 'c', 'd'])
  })
})

describe('mergeHiddenIntoLayout', () => {
  it('keeps hidden sounds in their group when a filtered view is dropped back', () => {
    const current = captureBoardLayout(board(), 'All')
    // Search shows only "a" and "d"; the user drags "a" into group g2.
    const fromScreen = { orphans: [], groups: [{ id: 'g1', paths: [] }, { id: 'g2', paths: ['d', 'a'] }] }
    expect(mergeHiddenIntoLayout(current, fromScreen)).toEqual({
      orphans: [],
      groups: [{ id: 'g1', paths: ['b', 'c'] }, { id: 'g2', paths: ['d', 'a'] }],
    })
  })

  it('keeps a hidden sound behind the visible neighbour it followed', () => {
    const current = { orphans: ['a', 'h', 'b', 'c'], groups: [] }
    const fromScreen = { orphans: ['c', 'a', 'b'], groups: [] }
    expect(mergeHiddenIntoLayout(current, fromScreen).orphans).toEqual(['c', 'a', 'h', 'b'])
  })

  it('changes nothing when the screen showed the whole tab', () => {
    const current = captureBoardLayout(board(), 'All')
    const moved = { orphans: ['d', 'a'], groups: [{ id: 'g1', paths: ['c', 'b'] }, { id: 'g2', paths: [] }] }
    expect(mergeHiddenIntoLayout(current, moved)).toEqual(moved)
  })

  it('never lists a sound twice', () => {
    const current = { orphans: ['a', 'b'], groups: [] }
    const merged = mergeHiddenIntoLayout(current, { orphans: ['a', 'a', 'b'], groups: [] })
    expect(merged.orphans).toEqual(['a', 'b'])
  })
})
