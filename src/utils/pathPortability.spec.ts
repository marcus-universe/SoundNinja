import { describe, expect, it } from 'vitest'
import {
  dirOf,
  joinOsPath,
  relinkCandidates,
  resolveMovedPaths,
  toRelativePath,
} from '~/utils/pathPortability'

describe('toRelativePath', () => {
  it('returns the path below the project folder with forward slashes', () => {
    expect(toRelativePath('C:\\Music\\Board\\Lolli\\a.wav', 'C:\\Music\\Board')).toBe('Lolli/a.wav')
    expect(toRelativePath('/home/deck/Board/Lolli/a.wav', '/home/deck/Board')).toBe('Lolli/a.wav')
  })

  it('ignores case on Windows paths only', () => {
    expect(toRelativePath('c:\\music\\BOARD\\a.wav', 'C:\\Music\\Board')).toBe('a.wav')
    expect(toRelativePath('/home/deck/BOARD/a.wav', '/home/deck/Board')).toBeNull()
  })

  it('is null for a sound outside the project folder', () => {
    expect(toRelativePath('D:\\Other\\a.wav', 'C:\\Music\\Board')).toBeNull()
    expect(toRelativePath('/home/deck/Boardx/a.wav', '/home/deck/Board')).toBeNull()
  })
})

describe('relinkCandidates', () => {
  it('tries the stored relative path first, then shorter tails of the old path', () => {
    expect(relinkCandidates('C:\\Music\\Board\\Lolli\\a.wav', 'Lolli/a.wav', '/home/deck/Board')).toEqual([
      '/home/deck/Board/Lolli/a.wav',
      '/home/deck/Board/Music/Board/Lolli/a.wav',
      '/home/deck/Board/Board/Lolli/a.wav',
      '/home/deck/Board/a.wav',
    ])
  })

  it('builds Windows candidates from a Linux path', () => {
    expect(relinkCandidates('/home/deck/Board/Lolli/a.wav', null, 'D:\\Sounds')).toContain('D:\\Sounds\\Lolli\\a.wav')
  })

  it('never escapes the project folder', () => {
    const out = relinkCandidates('/x/a.wav', '../../etc/passwd', '/home/deck/Board')
    expect(out.every((p) => p.startsWith('/home/deck/Board/'))).toBe(true)
    expect(out.some((p) => p.includes('..'))).toBe(false)
  })
})

describe('resolveMovedPaths', () => {
  const onDisk = new Set(['/home/deck/Board/Lolli/a.wav', '/home/deck/Board/kick.wav'])
  const exists = async (paths: string[]) => paths.map((p) => onDisk.has(p))

  it('finds sounds of a Windows project on Linux next to the project file', async () => {
    const map = await resolveMovedPaths(
      [
        { path: 'C:\\Users\\me\\Music\\Board\\Lolli\\a.wav', relPath: 'Lolli/a.wav' },
        { path: 'C:\\Users\\me\\Music\\Board\\kick.wav' },
        { path: 'C:\\Users\\me\\Music\\Board\\gone.wav' },
      ],
      '/home/deck/Board',
      [],
      exists,
    )
    expect(map.get('C:\\Users\\me\\Music\\Board\\Lolli\\a.wav')).toBe('/home/deck/Board/Lolli/a.wav')
    expect(map.get('C:\\Users\\me\\Music\\Board\\kick.wav')).toBe('/home/deck/Board/kick.wav')
    expect(map.has('C:\\Users\\me\\Music\\Board\\gone.wav')).toBe(false)
  })

  it('does not hand one file to two sounds or steal a path already in use', async () => {
    const two = await resolveMovedPaths(
      [{ path: 'C:\\a\\kick.wav' }, { path: 'D:\\b\\kick.wav' }],
      '/home/deck/Board',
      [],
      exists,
    )
    expect(two.size).toBe(1)
    const taken = await resolveMovedPaths(
      [{ path: 'C:\\a\\kick.wav' }],
      '/home/deck/Board',
      ['/home/deck/Board/kick.wav'],
      exists,
    )
    expect(taken.size).toBe(0)
  })
})

describe('path helpers', () => {
  it('joins with the separator of the base path', () => {
    expect(joinOsPath('C:\\Board', 'Lolli/a.wav')).toBe('C:\\Board\\Lolli\\a.wav')
    expect(joinOsPath('/home/deck/Board/', 'Lolli\\a.wav')).toBe('/home/deck/Board/Lolli/a.wav')
  })

  it('returns the folder of a file', () => {
    expect(dirOf('C:\\Board\\p.sninja')).toBe('C:\\Board')
    expect(dirOf('/home/deck/Board/p.sninja')).toBe('/home/deck/Board')
  })
})
