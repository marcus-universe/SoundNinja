import { defineStore } from 'pinia'
import {
  openDb, withProjectDb, reopenDb, loadConfig, saveConfig, emptyConfig, gcOrphanGifs,
  healFolderTabMembership, mergeTabsFromUsage,
  persistSoundIds,
  type ProjectConfig, type SoundFile, type TabEntry, type Separator, type Settings,
  type ButtonAlign,
} from '~/utils/db'
import { ensureSoundIds, newSoundId } from '~/utils/soundId'
import { revokeAllGifUrls } from '~/utils/gifCache'

/** Deep clone helper. Config is pure JSON data, so a JSON round-trip both
 *  deep-clones and strips Vue reactive Proxies (which structuredClone rejects). */
function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v))
}

/** Cap undo/redo stacks so memory stays bounded for large projects. */
const MAX_HISTORY = 50

/** Lowercase + strip spaces/punctuation so "alf" matches "A L F". */
function compactSearch(s: string): string {
  return s.toLowerCase().replace(/[^a-z0-9]+/gi, '')
}

export const useJsonHandelingStore = defineStore('JsonHandeling', {
  state: () => ({
    currentProjectPath: null as string | null,
    configFile: emptyConfig() as ProjectConfig,
    filteredFiles: [] as SoundFile[],
    /** Snapshot taken when a project is opened — used to Discard changes. */
    openingSnapshot: null as ProjectConfig | null,
    dirty: false,
    saving: false,
    missingPaths: [] as string[],
    _persistTimer: null as ReturnType<typeof setTimeout> | null,
    /** Stepwise undo history for soundboard mutations (sounds/tabs/separators). */
    undoStack: [] as ProjectConfig[],
    redoStack: [] as ProjectConfig[],
    /** When true, mutations skip pushBeforeChange (undo/redo restore path). */
    _historySuspended: false,
  }),

  getters: {
    getConfig: (state) => state.configFile,
    separators: (state) => state.configFile.separators ?? [],
    canUndo: (state) => state.undoStack.length > 0,
    canRedo: (state) => state.redoStack.length > 0,
  },

  actions: {
    // ── Undo / redo ───────────────────────────────────────────────────────────
    /** Snapshot current config before a tracked mutation. Clears redo. */
    pushBeforeChange() {
      if (this._historySuspended) return
      this.undoStack.push(clone(this.configFile))
      if (this.undoStack.length > MAX_HISTORY) this.undoStack.shift()
      this.redoStack = []
    },

    clearHistory() {
      this.undoStack = []
      this.redoStack = []
    },

    applySnapshot(snap: ProjectConfig) {
      this._historySuspended = true
      try {
        this.configFile = clone(snap)
        this.normalizeIndexes()
        this.filteredFiles = this.configFile.files
        this.writeConfig()
      } finally {
        this._historySuspended = false
      }
    },

    undo() {
      if (!this.undoStack.length) return false
      this.redoStack.push(clone(this.configFile))
      this.applySnapshot(this.undoStack.pop()!)
      return true
    },

    redo() {
      if (!this.redoStack.length) return false
      this.undoStack.push(clone(this.configFile))
      this.applySnapshot(this.redoStack.pop()!)
      return true
    },

    // ── Project lifecycle ─────────────────────────────────────────────────────
    /** Opens a project DB, loads it into state, and snapshots for Discard. */
    async openProject(dbAbsPath: string) {
      revokeAllGifUrls()
      let config = await withProjectDb(dbAbsPath, (d) => loadConfig(d))
      if (!config.files.length) {
        config = await this.tryRestoreFromBak(dbAbsPath, config)
      }
      this.configFile = config
      this.normalizeIndexes()
      this.filteredFiles = this.configFile.files
      this.currentProjectPath = dbAbsPath
      this.openingSnapshot = clone(this.configFile)
      this.dirty = false
      this.missingPaths = []
      this.clearHistory()
      if (ensureSoundIds(this.configFile.files)) {
        try {
          await withProjectDb(dbAbsPath, (d) => persistSoundIds(d, this.configFile.files))
        } catch (e) {
          console.error('Failed to persist sound ids', e)
        }
      }
    },

    /** If a failed save emptied `sounds`, recover files from project.sninja.bak. */
    async tryRestoreFromBak(dbAbsPath: string, current: ProjectConfig): Promise<ProjectConfig> {
      const bak = dbAbsPath + '.bak'
      const restored = dbAbsPath.replace(/\.sninja$/i, '.restored.sninja')
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const candidates: string[] = []
        if (await invoke<boolean>('path_exists_abs', { path: bak })) candidates.push(bak)
        if (await invoke<boolean>('path_exists_abs', { path: restored })) candidates.push(restored)
        if (!candidates.length) return current
        let bakConfig: ProjectConfig | null = null
        for (const source of candidates) {
          const loaded = await withProjectDb(source, (d) => loadConfig(d))
          if (loaded.files.length) {
            bakConfig = loaded
            break
          }
        }
        await reopenDb(dbAbsPath)
        if (!bakConfig?.files.length) return current
        this.configFile = bakConfig
        this.currentProjectPath = dbAbsPath
        await this.persistNow()
        return this.configFile
      } catch (e) {
        console.error('Failed to restore project from backup', e)
        try { await reopenDb(dbAbsPath) } catch { /* ignore */ }
        return current
      }
    },

    async validateSoundPaths() {
      const paths = this.configFile.files.map((f) => f.path).filter(Boolean)
      this.missingPaths = []
      if (!paths.length) return
      const { invoke } = await import('@tauri-apps/api/core')
      const exists = await invoke<boolean[]>('paths_exist_abs', { paths })
      this.missingPaths = paths.filter((_, i) => !exists[i])
    },

    /** Loads config into an already-open project DB (used by JSON import/migration). */
    async importConfig(config: ProjectConfig, dbAbsPath?: string) {
      if (dbAbsPath) {
        await openDb(dbAbsPath)
        this.currentProjectPath = dbAbsPath
      }
      this.configFile = {
        settings: config.settings,
        tabList: config.tabList ?? [],
        files: config.files ?? [],
        separators: config.separators ?? [],
      }
      mergeTabsFromUsage(this.configFile)
      healFolderTabMembership(this.configFile)
      ensureSoundIds(this.configFile.files)
      this.normalizeIndexes()
      this.filteredFiles = this.configFile.files
      this.openingSnapshot = clone(this.configFile)
      this.dirty = false
      this.clearHistory()
      await this.persistNow()
    },

    /** Reverts in-memory state to the last opened snapshot. No disk write is
     *  needed because the snapshot always mirrors what is already on disk, so
     *  we simply clear the dirty flag (avoids a costly, freeze-inducing save). */
    async discardChanges() {
      if (!this.openingSnapshot) return
      this.configFile = clone(this.openingSnapshot)
      this.normalizeIndexes()
      this.filteredFiles = this.configFile.files
      this.dirty = false
      this.clearHistory()
    },

    setCurrentProjectPath(p: string | null) {
      this.currentProjectPath = p
    },

    /** @deprecated legacy no-op kept for the old settings page. */
    setHue(_val: number) {
      // hue was replaced by the theme system; intentionally does nothing.
    },

    // ── Persistence ───────────────────────────────────────────────────────────
    /** Public compat alias — schedules a debounced save to the project DB. */
    writeConfig() {
      this.dirty = true
      if (this._persistTimer) clearTimeout(this._persistTimer)
      this._persistTimer = setTimeout(() => {
        this.persistNow().catch((e) => console.error('Failed to persist project', e))
      }, 200)
    },

    /** Flushes any pending changes to the project DB immediately. */
    async persistNow() {
      if (this._persistTimer) {
        clearTimeout(this._persistTimer)
        this._persistTimer = null
      }
      if (!this.currentProjectPath) {
        throw new Error('No project database is open.')
      }
      const path = this.currentProjectPath
      const bak = path + '.bak'
      const { invoke } = await import('@tauri-apps/api/core')
      try {
        await invoke('copy_file_to_abs', { src: path, dst: bak })
      } catch { /* first save may have no file yet */ }
      this.saving = true
      try {
        await withProjectDb(path, async (d) => {
          await saveConfig(d, this.configFile)
          const keep = this.configFile.files.map((f) => f.gifId).filter((id): id is string => !!id)
          await gcOrphanGifs(d, keep)
        })
        this.openingSnapshot = clone(this.configFile)
        this.dirty = false
      } catch (e) {
        console.error('Failed to persist project', e)
        try {
          await invoke('copy_file_to_abs', { src: bak, dst: path })
          await reopenDb(path)
        } catch { /* restore best-effort */ }
        throw e
      } finally {
        this.saving = false
      }
    },

    /**
     * Save As: flush live DB, then copy the SQLite file (gif_blobs included).
     * Avoids pulling every GIF across IPC and rewriting the whole DB.
     */
    async saveAs(dbAbsPath: string) {
      const { ensureProjectParentDir, removeInvalidProjectPlaceholder } = await import('~/utils/projects')
      const { invoke } = await import('@tauri-apps/api/core')
      await ensureProjectParentDir(dbAbsPath)
      await removeInvalidProjectPlaceholder(dbAbsPath)

      const src = this.currentProjectPath
      if (!src) {
        await this.importConfig(clone(this.configFile), dbAbsPath)
        return
      }

      const same = src.replace(/\\/g, '/').toLowerCase() === dbAbsPath.replace(/\\/g, '/').toLowerCase()
      if (same) {
        await this.persistNow()
        return
      }

      await this.persistNow()
      await withProjectDb(src, async (d) => {
        try { await d.execute('PRAGMA wal_checkpoint(TRUNCATE)') } catch { /* copy main file anyway */ }
      })
      await invoke('copy_file_to_abs', { src, dst: dbAbsPath })
      await reopenDb(dbAbsPath)
      this.currentProjectPath = dbAbsPath
      this.openingSnapshot = clone(this.configFile)
      this.dirty = false
      this.clearHistory()
    },

    updateConfigFile(contents: ProjectConfig) {
      this.pushBeforeChange()
      this.configFile = {
        settings: contents.settings,
        tabList: contents.tabList ?? [],
        files: contents.files ?? [],
        separators: contents.separators ?? [],
      }
      this.normalizeIndexes()
      this.filteredFiles = this.configFile.files
      this.writeConfig()
    },

    /**
     * Fill missing tab/order keys, then densify each tab so group markers
     * keep slots on the same number line as sounds. Sound-only compacting
     * (0..n) steals those slots and shuffles group membership on load.
     */
    normalizeIndexes() {
      const files = this.configFile.files
      if (!files) return

      for (const f of files) {
        if (!Array.isArray(f.tabs)) f.tabs = ['All']
        if (!f.tabs.includes('All')) f.tabs.unshift('All')
        if (!f.tabIndexes) f.tabIndexes = {}
        for (const key of Object.keys(f.tabIndexes)) {
          if (!f.tabs.includes(key)) delete f.tabIndexes[key]
        }
      }

      const tabs = ['All', ...(this.configFile.tabList ?? []).map((t) => t.name)]
      for (const tab of tabs) {
        const inTab = files.filter((f) => f.tabs?.includes(tab))
        const seps = (this.configFile.separators ?? []).filter((s) => s.tab === tab)
        let max = -1
        for (const f of inTab) {
          const o = this.soundOrderOnTab(f, tab)
          if (Number.isFinite(o)) max = Math.max(max, o)
        }
        for (const s of seps) {
          if (Number.isFinite(s.position)) max = Math.max(max, s.position)
        }
        for (const f of inTab) {
          const raw = tab === 'All' ? f.index : f.tabIndexes[tab]
          if (raw == null || !Number.isFinite(Number(raw))) {
            this.setSoundOrderOnTab(f, tab, ++max)
          }
        }
        this.applyBoardLayoutSilent(tab, this.captureBoardLayout(tab))
      }
    },

    // ── Settings ──────────────────────────────────────────────────────────────
    setTheme(val: string) {
      this.configFile.settings.theme = val
      this.writeConfig()
    },
    setCustomCss(val: string) {
      this.configFile.settings.customCss = val
      this.writeConfig()
    },
    setOutSource(val: string) {
      this.configFile.settings.outputSource = val
      this.writeConfig()
    },
    setStopOnRetrigger(val: boolean) {
      this.configFile.settings.stopOnRetrigger = val
      this.writeConfig()
    },
    setOverlapSounds(val: boolean) {
      this.configFile.settings.overlapSounds = val
      this.writeConfig()
    },
    setCacheConfig(maxSizeMib: number, maxEntryMib: number) {
      this.configFile.settings.cacheMaxSizeMib = maxSizeMib
      this.configFile.settings.cacheMaxEntryMib = maxEntryMib
      this.writeConfig()
    },
    setOutputVolume(val: number) {
      this.configFile.settings.outputVolume = val
      this.writeConfig()
    },

    setOutputHost(val: string) {
      this.configFile.settings.outputHost = val
      this.writeConfig()
    },
    setAsioChannels(left: number | null, right: number | null) {
      this.configFile.settings.asioLeftChannel = left ?? undefined
      this.configFile.settings.asioRightChannel = right ?? undefined
      this.writeConfig()
    },

    setUniformButtonHeight(val: boolean) {
      this.configFile.settings.uniformButtonHeight = val
      this.writeConfig()
    },
    setAllowReorder(val: boolean) {
      this.configFile.settings.allowReorder = val
      this.writeConfig()
    },
    setGifPlayOnHover(val: boolean) {
      this.configFile.settings.gifPlayOnHover = val
      this.writeConfig()
    },
    setPreloadGifs(val: boolean) {
      this.configFile.settings.preloadGifs = val
      this.writeConfig()
    },
    // Generic single-setting update — avoids a dedicated action per field.
    setSetting(key: keyof Settings, val: unknown) {
      (this.configFile.settings as Record<string, unknown>)[key] = val
      this.writeConfig()
    },
    // Bulk-apply theme color-model fields (used by builtin presets + reset).
    setThemeColors(colors: Partial<Settings>) {
      Object.assign(this.configFile.settings, colors)
      this.writeConfig()
    },

    // ── Sounds ────────────────────────────────────────────────────────────────
    addFiles(files: SoundFile[]) {
      this.pushBeforeChange()
      const used = new Set(this.configFile.files.map((f) => f.id).filter(Boolean))
      const withIds = files.map((f) => {
        if (f.id) {
          used.add(f.id)
          return f
        }
        const id = newSoundId(used)
        used.add(id)
        return { ...f, id }
      })
      this.configFile.files = [...this.configFile.files, ...withIds]
      this.normalizeIndexes()
      this.writeConfig()
    },

    setActiveSound({ soundindex, status }: { soundindex: number; status: boolean }) {
      this.configFile.files[soundindex].active = status
      this.writeConfig()
    },

    renameSound(soundindex: number, newName: string) {
      this.pushBeforeChange()
      this.configFile.files[soundindex].name = newName
      this.writeConfig()
    },

    removeSound(soundindex: number) {
      this.pushBeforeChange()
      this.configFile.files.splice(soundindex, 1)
      this.normalizeIndexes()
      this.writeConfig()
    },

    setSoundColor(soundindex: number, color: string) {
      this.pushBeforeChange()
      this.configFile.files[soundindex].color = color
      this.writeConfig()
    },

    setSoundGif(soundindex: number, gifId: string | null, gifPosX = 50, gifPosY = 50) {
      const file = this.configFile.files[soundindex]
      if (!file) return
      this.pushBeforeChange()
      if (gifId) {
        file.gifId = gifId
        file.gifPosX = gifPosX
        file.gifPosY = gifPosY
      } else {
        delete file.gifId
        delete file.gifPosX
        delete file.gifPosY
      }
      this.writeConfig()
    },

    setSoundTabs(soundFileIndex: number, tabs: string[]) {
      this.pushBeforeChange()
      this.configFile.files[soundFileIndex].tabs = tabs
      this.normalizeIndexes()
      this.writeConfig()
    },

    // ── Bulk (multi-select) actions, keyed by sound path ──────────────────────
    setSoundColorMany(paths: string[], color: string) {
      this.pushBeforeChange()
      const set = new Set(paths)
      for (const f of this.configFile.files) {
        if (set.has(f.path)) f.color = color
      }
      this.writeConfig()
    },

    setSoundTabsMany(paths: string[], tab: string) {
      this.pushBeforeChange()
      const set = new Set(paths)
      for (const f of this.configFile.files) {
        if (set.has(f.path) && !f.tabs.includes(tab)) f.tabs = [...f.tabs, tab]
      }
      this.normalizeIndexes()
      this.writeConfig()
    },

    relinkSounds(pairs: { from: string; to: string }[]) {
      if (!pairs.length) return
      this.pushBeforeChange()
      const map = new Map(pairs.map((p) => [p.from, p.to]))
      for (const f of this.configFile.files) {
        const next = map.get(f.path)
        if (next) f.path = next
      }
      this.missingPaths = this.missingPaths.filter((p) => !map.has(p))
      this.writeConfig()
    },

    removeSoundsMany(paths: string[]) {
      this.pushBeforeChange()
      const set = new Set(paths)
      this.configFile.files = this.configFile.files.filter((f) => !set.has(f.path))
      this.normalizeIndexes()
      this.writeConfig()
    },

    reorderSounds(draggedIdx: number, targetIdx: number, tab: string) {
      const files = this.configFile.files
      if (tab === 'All') {
        const sorted = [...files].sort((a, b) => a.index - b.index)
        const from = sorted.findIndex((f) => f.index === draggedIdx)
        const to = sorted.findIndex((f) => f.index === targetIdx)
        if (from === -1 || to === -1 || from === to) return
        this.pushBeforeChange()
        const [item] = sorted.splice(from, 1)
        sorted.splice(to, 0, item)
        sorted.forEach((f, i) => { f.index = i })
      } else {
        const inTab = files
          .filter((f) => f.tabs.includes(tab))
          .sort((a, b) => (a.tabIndexes?.[tab] ?? 0) - (b.tabIndexes?.[tab] ?? 0))
        const from = inTab.findIndex((f) => f.index === draggedIdx)
        const to = inTab.findIndex((f) => f.index === targetIdx)
        if (from === -1 || to === -1 || from === to) return
        this.pushBeforeChange()
        const [item] = inTab.splice(from, 1)
        inTab.splice(to, 0, item)
        inTab.forEach((f, i) => {
          if (!f.tabIndexes) f.tabIndexes = {}
          f.tabIndexes[tab] = i
        })
      }
      this.writeConfig()
    },

    // ── Tabs ──────────────────────────────────────────────────────────────────
    addTab(name: string) {
      const trimmed = String(name ?? '').trim()
      if (!trimmed) return
      if (this.configFile.tabList.some((t) => t.name === trimmed)) return
      this.pushBeforeChange()
      this.configFile.tabList.push({ name: trimmed })
      this.writeConfig()
    },

    removeTab(name: string) {
      this.pushBeforeChange()
      this.configFile.tabList = this.configFile.tabList.filter((t) => t.name !== name)
      this.configFile.separators = (this.configFile.separators ?? []).filter((s) => s.tab !== name)
      this.writeConfig()
    },

    renameTab(oldName: string, newName: string) {
      const tab = this.configFile.tabList.find((t) => t.name === oldName)
      if (!tab) return
      const trimmed = String(newName ?? '').trim()
      if (!trimmed || trimmed === oldName) return
      if (this.configFile.tabList.some((t) => t.name === trimmed)) return
      this.pushBeforeChange()
      tab.name = trimmed
      this.configFile.files.forEach((f) => {
        const idx = f.tabs.indexOf(oldName)
        if (idx !== -1) f.tabs[idx] = trimmed
        if (f.tabIndexes && oldName in f.tabIndexes) {
          f.tabIndexes[trimmed] = f.tabIndexes[oldName]
          delete f.tabIndexes[oldName]
        }
      })
      ;(this.configFile.separators ?? []).forEach((s) => {
        if (s.tab === oldName) s.tab = trimmed
      })
      this.writeConfig()
    },

    setTabColor(name: string, color: string) {
      const tab = this.configFile.tabList.find((t) => t.name === name)
      if (tab) {
        this.pushBeforeChange()
        tab.color = color
        this.writeConfig()
      }
    },

    reorderTabs(draggedName: string, targetName: string) {
      const list = this.configFile.tabList
      const from = list.findIndex((t) => t.name === draggedName)
      const to = list.findIndex((t) => t.name === targetName)
      if (from === -1 || to === -1 || from === to) return
      this.pushBeforeChange()
      const [item] = list.splice(from, 1)
      list.splice(to, 0, item)
      this.writeConfig()
    },

    // ── Separators / Groups ───────────────────────────────────────────────────
    addSeparator(tab: string, position: number, name?: string) {
      this.pushBeforeChange()
      if (!this.configFile.separators) this.configFile.separators = []
      const id = `sep_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
      this.configFile.separators.push({
        id,
        tab,
        position,
        ...(name ? { name } : {}),
      })
      this.writeConfig()
    },

    removeSeparator(id: string) {
      this.pushBeforeChange()
      this.configFile.separators = (this.configFile.separators ?? []).filter((s) => s.id !== id)
      this.writeConfig()
    },

    setSeparatorPosition(id: string, position: number) {
      const sep = (this.configFile.separators ?? []).find((s) => s.id === id)
      if (sep) {
        this.pushBeforeChange()
        sep.position = position
        this.writeConfig()
      }
    },

    updateSeparator(id: string, patch: Partial<Omit<Separator, 'id'>>) {
      const sep = (this.configFile.separators ?? []).find((s) => s.id === id)
      if (!sep) return
      this.pushBeforeChange()
      if ('name' in patch) {
        const n = patch.name?.trim()
        if (n) sep.name = n
        else delete sep.name
      }
      if ('borderColor' in patch) {
        if (patch.borderColor) sep.borderColor = patch.borderColor
        else delete sep.borderColor
      }
      if ('nameColor' in patch) {
        if (patch.nameColor) sep.nameColor = patch.nameColor
        else delete sep.nameColor
      }
      if ('bgColor' in patch) {
        if (patch.bgColor) sep.bgColor = patch.bgColor
        else delete sep.bgColor
      }
      if ('buttonAlign' in patch) {
        if (patch.buttonAlign) sep.buttonAlign = patch.buttonAlign
        else delete sep.buttonAlign
      }
      if ('position' in patch && typeof patch.position === 'number') {
        sep.position = patch.position
      }
      if ('tab' in patch && patch.tab) sep.tab = patch.tab
      this.writeConfig()
    },

    setTabButtonAlign(tabName: string, align: ButtonAlign | undefined) {
      const tab = this.configFile.tabList.find((t) => t.name === tabName)
      if (!tab) return
      this.pushBeforeChange()
      if (align) tab.buttonAlign = align
      else delete tab.buttonAlign
      this.writeConfig()
    },

    /** Tab order key for a sound (global index on "All"). */
    soundOrderOnTab(sound: SoundFile, tab: string): number {
      if (tab === 'All') return sound.index ?? 0
      return sound.tabIndexes?.[tab] ?? 0
    },

    setSoundOrderOnTab(sound: SoundFile, tab: string, order: number) {
      if (tab === 'All') {
        sound.index = order
        return
      }
      if (!sound.tabIndexes) sound.tabIndexes = {}
      sound.tabIndexes[tab] = order
    },

    /**
     * Rewrite dense interleaved order for a tab: orphan sounds, then each
     * group marker + its members. Keeps relative membership from `layout`.
     */
    applyBoardLayoutSilent(
      tab: string,
      layout: { orphans: string[]; groups: { id: string; paths: string[] }[] },
    ) {
      const byPath = new Map(this.configFile.files.map((f) => [f.path, f]))
      let seq = 0
      for (const path of layout.orphans) {
        const f = byPath.get(path)
        if (f) this.setSoundOrderOnTab(f, tab, seq++)
      }
      for (const g of layout.groups) {
        const sep = (this.configFile.separators ?? []).find((s) => s.id === g.id)
        if (sep) sep.position = seq++
        for (const path of g.paths) {
          const f = byPath.get(path)
          if (f) this.setSoundOrderOnTab(f, tab, seq++)
        }
      }
    },

    applyBoardLayout(
      tab: string,
      layout: { orphans: string[]; groups: { id: string; paths: string[] }[] },
    ) {
      this.pushBeforeChange()
      this.applyBoardLayoutSilent(tab, layout)
      this.writeConfig()
    },

    /**
     * Rebuild board layout from current positional membership, then densify.
     * Call after cross-group drops so float anchors never drift.
     */
    normalizeBoardOrder(tab: string) {
      const layout = this.captureBoardLayout(tab)
      this.applyBoardLayout(tab, layout)
    },

    /** Snapshot orphans + groups with member paths from current positions. */
    captureBoardLayout(tab: string): { orphans: string[]; groups: { id: string; paths: string[] }[] } {
      const sounds = this.configFile.files
        .filter((f) => f.tabs.includes(tab))
        .sort((a, b) => this.soundOrderOnTab(a, tab) - this.soundOrderOnTab(b, tab))
      const seps = (this.configFile.separators ?? [])
        .filter((s) => s.tab === tab)
        .slice()
        .sort((a, b) => a.position - b.position)

      if (seps.length === 0) {
        return { orphans: sounds.map((s) => s.path), groups: [] }
      }

      const orphans: string[] = []
      const groups: { id: string; paths: string[] }[] = seps.map((s) => ({ id: s.id, paths: [] }))

      for (const sound of sounds) {
        const order = this.soundOrderOnTab(sound, tab)
        let placed = false
        for (let i = 0; i < seps.length; i++) {
          const start = seps[i].position
          const end = i + 1 < seps.length ? seps[i + 1].position : Number.POSITIVE_INFINITY
          if (order >= start && order < end) {
            groups[i].paths.push(sound.path)
            placed = true
            break
          }
        }
        if (!placed && order < seps[0].position) orphans.push(sound.path)
        else if (!placed) orphans.push(sound.path)
      }
      return { orphans, groups }
    },

    /**
     * Reorder groups on a tab (orphans stay first). Member sounds move with
     * their group. `orderedSepIds` is the new group order.
     */
    moveGroupWithMembers(tab: string, orderedSepIds: string[]) {
      const layout = this.captureBoardLayout(tab)
      const byId = new Map(layout.groups.map((g) => [g.id, g]))
      const groups = orderedSepIds
        .map((id) => byId.get(id))
        .filter((g): g is { id: string; paths: string[] } => !!g)
      // Keep any groups missing from orderedSepIds at the end (safety).
      for (const g of layout.groups) {
        if (!orderedSepIds.includes(g.id)) groups.push(g)
      }
      this.applyBoardLayout(tab, { orphans: layout.orphans, groups })
    },

    // ── Bulk / misc ───────────────────────────────────────────────────────────
    resetAll() {
      this.pushBeforeChange()
      this.configFile = emptyConfig()
      this.normalizeIndexes()
      this.filteredFiles = []
      this.writeConfig()
    },

    ReturnStatusAll() {
      this.configFile.files.forEach((file) => { file.active = false })
      this.writeConfig()
    },

    filterSounds(searchTerm: string) {
      const q = (searchTerm ?? '').trim()
      if (!q) {
        this.filteredFiles = this.configFile.files
        return
      }
      const qLower = q.toLowerCase()
      const qCompact = compactSearch(q)
      // Only use tokens with 2+ chars for AND matching. Single letters like
      // "A L F" would otherwise match any name containing a, l, and f.
      const tokens = qLower.split(/\s+/).filter((t) => t.length >= 2)

      this.filteredFiles = this.configFile.files.filter((file) => {
        const nameLower = (file.name ?? '').toLowerCase()
        const nameCompact = compactSearch(file.name ?? '')

        // 1) Exact-ish substring (case-insensitive)
        if (nameLower.includes(qLower)) return true
        // 2) Compact: strip spaces/punct so "A L F" ↔ "alf" both ways
        if (qCompact.length > 0 && nameCompact.includes(qCompact)) return true
        // 3) Multi-word queries ("hypnose cat"): every real token must appear
        if (tokens.length > 1) {
          return tokens.every((t) => {
            const tCompact = compactSearch(t)
            return nameLower.includes(t)
              || (tCompact.length > 0 && nameCompact.includes(tCompact))
          })
        }
        return false
      })
    },
  },
})
