import type { ProjectConfig, SoundFile } from '~/utils/db'

/**
 * Board layout of one tab: loose sounds first, then each group with its
 * members. Sounds are referenced by path, groups by separator id.
 *
 * In memory a tab keeps one shared number line: `sound.index` /
 * `sound.tabIndexes[tab]` and `separator.position`. A sound belongs to the
 * group whose marker precedes it. That is compact but fragile — two entries
 * with the same number are enough to move a sound into another group. So the
 * project file stores membership explicitly (see `sound_tabs.group_id`) and
 * the number line is rebuilt from it on load.
 */
export interface BoardLayout {
  orphans: string[]
  groups: { id: string; paths: string[] }[]
}

function finite(n: unknown): number {
  const v = Number(n)
  return Number.isFinite(v) ? v : Number.MAX_SAFE_INTEGER
}

/** Tab order key for a sound (global index on "All"). */
export function soundOrderOnTab(sound: SoundFile, tab: string): number {
  if (tab === 'All') return sound.index ?? 0
  return sound.tabIndexes?.[tab] ?? 0
}

export function setSoundOrderOnTab(sound: SoundFile, tab: string, order: number): void {
  if (tab === 'All') {
    sound.index = order
    return
  }
  if (!sound.tabIndexes) sound.tabIndexes = {}
  sound.tabIndexes[tab] = order
}

/** `All` plus every tab the project lists. */
export function boardTabNames(config: ProjectConfig): string[] {
  const out = ['All']
  for (const t of config.tabList ?? []) {
    if (t?.name && !out.includes(t.name)) out.push(t.name)
  }
  return out
}

function soundsOnTab(config: ProjectConfig, tab: string): SoundFile[] {
  // Array.prototype.sort is stable: equal keys keep file order.
  return (config.files ?? [])
    .filter((f) => f.tabs?.includes(tab))
    .sort((a, b) => finite(soundOrderOnTab(a, tab)) - finite(soundOrderOnTab(b, tab)))
}

function separatorsOnTab(config: ProjectConfig, tab: string) {
  return (config.separators ?? [])
    .filter((s) => s.tab === tab)
    .slice()
    .sort((a, b) => finite(a.position) - finite(b.position))
}

/** Snapshot orphans + groups with member paths from current positions. */
export function captureBoardLayout(config: ProjectConfig, tab: string): BoardLayout {
  const sounds = soundsOnTab(config, tab)
  const seps = separatorsOnTab(config, tab)
  if (seps.length === 0) {
    return { orphans: sounds.map((s) => s.path), groups: [] }
  }

  const orphans: string[] = []
  const groups = seps.map((s) => ({ id: s.id, paths: [] as string[] }))
  for (const sound of sounds) {
    const order = finite(soundOrderOnTab(sound, tab))
    let owner = -1
    for (let i = 0; i < seps.length; i++) {
      if (order >= finite(seps[i]!.position)) owner = i
      else break
    }
    if (owner < 0) orphans.push(sound.path)
    else groups[owner]!.paths.push(sound.path)
  }
  return { orphans, groups }
}

/**
 * Build a layout from explicit membership (`groupOf(path)` → separator id).
 * A sound naming a group that no longer exists on this tab becomes an orphan.
 */
export function layoutFromMembership(
  config: ProjectConfig,
  tab: string,
  groupOf: (path: string) => string | null | undefined,
): BoardLayout {
  const seps = separatorsOnTab(config, tab)
  const groups = seps.map((s) => ({ id: s.id, paths: [] as string[] }))
  const byId = new Map(groups.map((g) => [g.id, g]))
  const orphans: string[] = []
  for (const sound of soundsOnTab(config, tab)) {
    const group = byId.get(groupOf(sound.path) ?? '')
    if (group) group.paths.push(sound.path)
    else orphans.push(sound.path)
  }
  return { orphans, groups }
}

/**
 * Keep sounds the incoming layout does not mention.
 *
 * A layout read from the screen only lists the buttons that are visible —
 * with a search or tag filter active that is a subset of the tab. Writing it
 * back as-is left the hidden sounds on their old numbers, which then collided
 * with the new ones and reshuffled groups. Hidden sounds stay in their
 * section, right behind the visible neighbour they followed before.
 */
export function mergeHiddenIntoLayout(current: BoardLayout, incoming: BoardLayout): BoardLayout {
  const listed = new Set<string>([...incoming.orphans, ...incoming.groups.flatMap((g) => g.paths)])
  const seen = new Set<string>()
  const unique = (paths: string[]) => paths.filter((p) => {
    if (!p || seen.has(p)) return false
    seen.add(p)
    return true
  })

  const merge = (before: string[], next: string[]): string[] => {
    const out = unique(next)
    let anchor = -1
    for (const path of before) {
      if (listed.has(path)) {
        const at = out.indexOf(path)
        if (at >= 0) anchor = at
        continue
      }
      if (seen.has(path)) continue
      seen.add(path)
      out.splice(anchor + 1, 0, path)
      anchor += 1
    }
    return out
  }

  const currentById = new Map(current.groups.map((g) => [g.id, g.paths]))
  const orphans = merge(current.orphans, incoming.orphans)
  const groups = incoming.groups.map((g) => ({
    id: g.id,
    paths: merge(currentById.get(g.id) ?? [], g.paths),
  }))
  // Members of a group the incoming layout dropped fall back to the orphans.
  const incomingIds = new Set(incoming.groups.map((g) => g.id))
  for (const g of current.groups) {
    if (incomingIds.has(g.id)) continue
    for (const path of g.paths) {
      if (listed.has(path) || seen.has(path)) continue
      seen.add(path)
      orphans.push(path)
    }
  }
  return { orphans, groups }
}

/**
 * Rewrite dense interleaved order for a tab: orphan sounds, then each group
 * marker + its members. Sounds of the tab that `layout` does not list are
 * appended to the orphans / kept via `mergeHiddenIntoLayout` by the caller;
 * anything still unlisted lands after the last entry so numbers never collide.
 */
export function applyBoardLayout(config: ProjectConfig, tab: string, layout: BoardLayout): void {
  const byPath = new Map((config.files ?? []).map((f) => [f.path, f]))
  const sepById = new Map((config.separators ?? []).map((s) => [s.id, s]))
  const placed = new Set<string>()
  let seq = 0
  const place = (path: string) => {
    const f = byPath.get(path)
    if (!f || placed.has(path) || !f.tabs?.includes(tab)) return
    placed.add(path)
    setSoundOrderOnTab(f, tab, seq++)
  }
  for (const path of layout.orphans) place(path)
  for (const g of layout.groups) {
    const sep = sepById.get(g.id)
    if (sep && sep.tab === tab) sep.position = seq++
    for (const path of g.paths) place(path)
  }
  for (const f of config.files ?? []) {
    if (f.tabs?.includes(tab) && !placed.has(f.path)) setSoundOrderOnTab(f, tab, seq++)
  }
  // Group markers of this tab the layout skipped still need a unique slot.
  const listedSeps = new Set(layout.groups.map((g) => g.id))
  for (const s of config.separators ?? []) {
    if (s.tab === tab && !listedSeps.has(s.id)) s.position = seq++
  }
}

/** Path → separator id for every grouped sound of a tab. */
export function groupMembership(config: ProjectConfig, tab: string): Map<string, string> {
  const out = new Map<string, string>()
  for (const g of captureBoardLayout(config, tab).groups) {
    for (const path of g.paths) out.set(path, g.id)
  }
  return out
}
