import { defineStore } from 'pinia'
import {
  openDb, withProjectDb, reopenDb, loadConfig, saveConfig, emptyConfig, gcOrphanGifs,
  healFolderTabMembership, mergeTabsFromUsage,
  persistSoundIds, ensurePerSoundVolume,
  type ProjectConfig, type SoundFile, type TabEntry, type Separator, type Settings,
  type ButtonAlign, type SoundTag, type SortMode, normalizeSortMode,
} from '~/utils/db'
import { ensureSoundIds, newSoundId } from '~/utils/soundId'
import { revokeAllGifUrls } from '~/utils/gifCache'
import {
  NEW_TAB_DEST,
  audioDisplayName,
  folderGroups,
  folderRootAudio,
  pathKey,
  type ImportReviewState,
} from '~/utils/importReview'

/** Deep clone helper. Config is pure JSON data, so a JSON round-trip both
 *  deep-clones and strips Vue reactive Proxies (which structuredClone rejects). */
function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v))
}

/** Cap undo/redo stacks so memory stays bounded for large projects. */
const MAX_HISTORY = 50
/** Color-wheel ticks within this window share one undo snapshot. */
const COLOR_BURST_MS = 400

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
    _searchTerm: '',
    /** Stepwise undo history for soundboard mutations (sounds/tabs/separators). */
    undoStack: [] as ProjectConfig[],
    redoStack: [] as ProjectConfig[],
    /** When true, mutations skip pushBeforeChange (undo/redo restore path). */
    _historySuspended: false,
    /** Last color mutation timestamp for burst-undo coalescing. */
    _colorBurstAt: 0,
  }),

  getters: {
    getConfig: (state) => state.configFile,
    separators: (state) => state.configFile.separators ?? [],
    tags: (state) => state.configFile.tags ?? [],
    sortMode: (state) => normalizeSortMode(state.configFile.settings?.sortMode),
    visibleFiles: (state) => state.filteredFiles,
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

    /** One undo step per color-wheel drag. Ticks closer than COLOR_BURST_MS skip. */
    pushColorHistory() {
      const now = Date.now()
      if (now - this._colorBurstAt < COLOR_BURST_MS) {
        this._colorBurstAt = now
        return
      }
      this._colorBurstAt = now
      this.pushBeforeChange()
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
        this.applyBoardFilter()
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
      if (!this.configFile.tags) this.configFile.tags = []
      this.normalizeIndexes()
      this.currentProjectPath = dbAbsPath
      this.openingSnapshot = clone(this.configFile)
      this.dirty = false
      this.missingPaths = []
      this.clearHistory()
      this.syncFilterSessionFromSettings()
      this.applyBoardFilter()
      const idsChanged = ensureSoundIds(this.configFile.files)
      const volumeMigrated = ensurePerSoundVolume(this.configFile)
      if (volumeMigrated) {
        try {
          await this.persistNow()
        } catch (e) {
          console.error('Failed to persist per-sound volume migration', e)
        }
      } else if (idsChanged) {
        try {
          await withProjectDb(dbAbsPath, (d) => persistSoundIds(d, this.configFile.files))
        } catch (e) {
          console.error('Failed to persist sound ids', e)
        }
      }
      void this.warmSoundMeta()
      if (normalizeSortMode(this.configFile.settings.sortMode) === 'duration') {
        void this.warmDurations()
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
        tags: config.tags ?? [],
      }
      mergeTabsFromUsage(this.configFile)
      healFolderTabMembership(this.configFile)
      ensureSoundIds(this.configFile.files)
      ensurePerSoundVolume(this.configFile)
      this.normalizeIndexes()
      this.syncFilterSessionFromSettings()
      this.applyBoardFilter()
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
      this.syncFilterSessionFromSettings()
      this.applyBoardFilter()
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
    /** Debounced save. Does not rebuild the board file list. */
    schedulePersist() {
      this.dirty = true
      if (this._persistTimer) clearTimeout(this._persistTimer)
      this._persistTimer = setTimeout(() => {
        this.persistNow().catch((e) => console.error('Failed to persist project', e))
      }, 200)
    },

    /** Public compat alias — rebuilds the filtered board, then schedules a save. */
    writeConfig() {
      this.applyBoardFilter()
      this.schedulePersist()
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
        tags: contents.tags ?? [],
      }
      this.normalizeIndexes()
      this.applyBoardFilter()
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
      const now = Date.now()
      const withIds = files.map((f) => {
        const next = {
          ...f,
          tagIds: f.tagIds ?? [],
          addedAt: f.addedAt ?? now,
        }
        if (next.id) {
          used.add(next.id)
          return next
        }
        const id = newSoundId(used)
        used.add(id)
        return { ...next, id }
      })
      this.configFile.files = [...this.configFile.files, ...withIds]
      this.normalizeIndexes()
      this.applyBoardFilter()
      this.writeConfig()
      void this.warmDurations()
    },

    commitImportReview(review: ImportReviewState) {
      type BoardLayout = { orphans: string[]; groups: { id: string; paths: string[] }[] }
      const usedPaths = new Set(this.configFile.files.map((f) => pathKey(f.path)))
      const takePath = (path: string) => {
        const key = pathKey(path)
        if (usedPaths.has(key)) return false
        usedPaths.add(key)
        return true
      }

      const makeSound = (fileName: string, path: string, destTab: string): SoundFile => {
        const tabs = destTab && destTab !== 'All' ? ['All', destTab] : ['All']
        return {
          name: audioDisplayName(fileName),
          path,
          id: '',
          volume: 1,
          tabs,
          active: false,
          index: 0,
          tabIndexes: {},
        }
      }

      const newFiles: SoundFile[] = []
      const newSeps: Separator[] = []
      const pendingTabs = new Set<string>()
      let sepSeq = 0
      const newSepId = () => `sep_${Date.now()}_${sepSeq++}_${Math.random().toString(36).slice(2, 8)}`

      const layouts = new Map<string, BoardLayout>()
      const layoutFor = (tab: string): BoardLayout => {
        let layout = layouts.get(tab)
        if (!layout) {
          layout = this.captureBoardLayout(tab)
          layouts.set(tab, layout)
        }
        return layout
      }
      const appendToEnd = (tab: string, paths: string[]) => {
        if (!paths.length) return
        const layout = layoutFor(tab)
        const lastGroup = layout.groups[layout.groups.length - 1]
        if (lastGroup) lastGroup.paths.push(...paths)
        else layout.orphans.push(...paths)
      }
      const tabExists = (name: string) =>
        name === 'All'
        || this.configFile.tabList.some((t) => t.name === name)
        || pendingTabs.has(name)

      const existingTabIgnoreCase = (name: string) => {
        const n = String(name || '').trim()
        if (!n) return null
        if (n.toLowerCase() === 'all') return 'All'
        return this.configFile.tabList.find((t) => t.name.toLowerCase() === n.toLowerCase())?.name ?? null
      }

      const folderDest = (folder: ImportReviewState['folders'][number]) => {
        const raw = folder.destTab === NEW_TAB_DEST
          ? (folder.name || '').trim() || 'All'
          : folder.destTab || 'All'
        return existingTabIgnoreCase(raw) || raw
      }

      const fileOnBoard = (path: string) =>
        this.configFile.files.find((f) => pathKey(f.path) === pathKey(path))

      const existingTabAdds: { path: string; destTab: string }[] = []
      const adoptExisting = (path: string, destTab: string, bucket: string[]) => {
        const existing = fileOnBoard(path)
        if (!existing || destTab === 'All') return
        if (existing.tabs.includes(destTab)) return
        existingTabAdds.push({ path, destTab })
        bucket.push(path)
      }

      for (const folder of review.folders) {
        const destTab = folderDest(folder)
        if (destTab !== 'All' && !tabExists(destTab)) pendingTabs.add(destTab)
      }

      for (const file of review.files) {
        if (!takePath(file.path)) continue
        const dest = tabExists(file.destTab) ? file.destTab : 'All'
        const sound = makeSound(file.fileName, file.path, dest)
        newFiles.push(sound)
        appendToEnd(dest, [sound.path])
        if (dest !== 'All') appendToEnd('All', [sound.path])
      }

      for (const folder of review.folders) {
        const destTab = folderDest(folder)
        const destLayout = layoutFor(destTab)
        const rootPaths: string[] = []
        const allPaths: string[] = []

        for (const audio of folderRootAudio(folder)) {
          if (!takePath(audio.path)) {
            adoptExisting(audio.path, destTab, rootPaths)
            continue
          }
          const sound = makeSound(audio.fileName, audio.path, destTab)
          newFiles.push(sound)
          rootPaths.push(sound.path)
          allPaths.push(sound.path)
        }

        if (rootPaths.length) {
          if (destLayout.groups.length > 0) {
            const id = newSepId()
            newSeps.push({ id, tab: destTab, position: 0, name: folder.name })
            destLayout.groups.push({ id, paths: rootPaths })
          } else {
            destLayout.orphans.push(...rootPaths)
          }
        }

        for (const group of folderGroups(folder)) {
          const paths: string[] = []
          for (const audio of group.audio) {
            if (!takePath(audio.path)) {
              adoptExisting(audio.path, destTab, paths)
              continue
            }
            const sound = makeSound(audio.fileName, audio.path, destTab)
            newFiles.push(sound)
            paths.push(sound.path)
            allPaths.push(sound.path)
          }
          if (!paths.length) continue
          const id = newSepId()
          newSeps.push({ id, tab: destTab, position: 0, name: group.name })
          destLayout.groups.push({ id, paths })
        }

        if (destTab !== 'All') appendToEnd('All', allPaths)
      }

      if (!newFiles.length && !newSeps.length && pendingTabs.size === 0 && !existingTabAdds.length) return

      this.pushBeforeChange()
      for (const { path, destTab } of existingTabAdds) {
        const file = fileOnBoard(path)
        if (!file || destTab === 'All') continue
        if (!file.tabs.includes(destTab)) file.tabs = [...file.tabs, destTab]
      }
      for (const name of pendingTabs) {
        if (!this.configFile.tabList.some((t) => t.name === name)) {
          this.configFile.tabList.push({ name })
        }
      }

      const usedIds = new Set(this.configFile.files.map((f) => f.id).filter(Boolean))
      const now = Date.now()
      const withIds = newFiles.map((f) => {
        const next = { ...f, tagIds: f.tagIds ?? [], addedAt: f.addedAt ?? now }
        if (next.id) {
          usedIds.add(next.id)
          return next
        }
        const id = newSoundId(usedIds)
        usedIds.add(id)
        return { ...next, id }
      })
      this.configFile.files = [...this.configFile.files, ...withIds]
      if (!this.configFile.separators) this.configFile.separators = []
      this.configFile.separators = [...this.configFile.separators, ...newSeps]
      for (const [tab, layout] of layouts) {
        this.applyBoardLayoutSilent(tab, layout)
      }
      this.normalizeIndexes()
      this.applyBoardFilter()
      this.writeConfig()
      void this.warmDurations()
    },

    setActiveSound({ soundindex, status }: { soundindex: number; status: boolean }) {
      const file = this.configFile.files[soundindex]
      if (!file) return
      file.active = status
      this.writeConfig()
    },

    renameSound(soundindex: number, newName: string) {
      const file = this.configFile.files[soundindex]
      if (!file) return
      this.pushBeforeChange()
      file.name = newName
      this.writeConfig()
    },

    removeSound(soundindex: number) {
      this.pushBeforeChange()
      this.configFile.files.splice(soundindex, 1)
      this.normalizeIndexes()
      this.writeConfig()
    },

    setSoundVolume(soundindex: number, volume: number, opts?: { history?: boolean }) {
      const file = this.configFile.files[soundindex]
      if (!file) return
      const next = Math.min(1, Math.max(0, volume))
      if (file.volume === next) return
      if (opts?.history !== false) this.pushBeforeChange()
      file.volume = next
      this.writeConfig()
    },

    setSoundLoop(soundindex: number, looping: boolean, opts?: { history?: boolean }) {
      const file = this.configFile.files[soundindex]
      if (!file) return
      const next = !!looping
      if (!!file.looping === next) return
      if (opts?.history !== false) this.pushBeforeChange()
      file.looping = next
      this.writeConfig()
    },

    setSoundLoopByPath(path: string, looping: boolean, opts?: { history?: boolean }) {
      const idx = this.configFile.files.findIndex((f) => f.path === path)
      if (idx < 0) return
      this.setSoundLoop(idx, looping, opts)
    },

    setSoundColor(soundindex: number, color: string) {
      const file = this.configFile.files[soundindex]
      if (!file) return
      this.pushColorHistory()
      file.color = color
      this.schedulePersist()
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
      const file = this.configFile.files[soundFileIndex]
      if (!file) return
      this.pushBeforeChange()
      file.tabs = tabs
      this.normalizeIndexes()
      this.writeConfig()
    },

    /** Swap the audio file on a button. Name, id, color, gif, tabs, volume stay. */
    replaceSoundPath(soundindex: number, nextPath: string) {
      const file = this.configFile.files[soundindex]
      if (!file || !nextPath || file.path === nextPath) return
      this.pushBeforeChange()
      const old = file.path
      file.path = nextPath
      this.missingPaths = this.missingPaths.filter((p) => p !== old)
      this.writeConfig()
    },

    // ── Bulk (multi-select) actions, keyed by sound path ──────────────────────
    setSoundColorMany(paths: string[], color: string) {
      this.pushColorHistory()
      const set = new Set(paths)
      for (const f of this.configFile.files) {
        if (set.has(f.path)) f.color = color
      }
      this.schedulePersist()
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
        if (!item) return
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
        if (!item) return
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
          f.tabIndexes[trimmed] = f.tabIndexes[oldName] ?? 0
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
      if (!tab) return
      this.pushColorHistory()
      tab.color = color
      this.schedulePersist()
    },

    reorderTabs(draggedName: string, targetName: string) {
      const list = this.configFile.tabList
      const from = list.findIndex((t) => t.name === draggedName)
      const to = list.findIndex((t) => t.name === targetName)
      if (from === -1 || to === -1 || from === to) return
      this.pushBeforeChange()
      const [item] = list.splice(from, 1)
      if (!item) return
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
          const sep = seps[i]
          if (!sep) continue
          const start = sep.position
          const nextSep = seps[i + 1]
          const end = nextSep ? nextSep.position : Number.POSITIVE_INFINITY
          if (order >= start && order < end) {
            groups[i]?.paths.push(sound.path)
            placed = true
            break
          }
        }
        const firstSep = seps[0]
        if (!placed && firstSep && order < firstSep.position) orphans.push(sound.path)
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

    // ── Tags / filter / sort ─────────────────────────────────────────────────
    syncFilterSessionFromSettings() {
      const ids = this.configFile.settings?.selectedTagIds
      this.configFile.settings.selectedTagIds = Array.isArray(ids) ? ids.filter(Boolean) : []
      void import('./app').then(({ useAppStore }) => {
        useAppStore().hydrateSelectedTagIds(this.configFile.settings.selectedTagIds ?? [])
      })
    },

    persistSelectedTagIds(ids: string[]) {
      this.configFile.settings.selectedTagIds = [...ids]
      this.applyBoardFilter()
      this.writeConfig()
    },

    setSortMode(mode: SortMode) {
      const next = normalizeSortMode(mode)
      if (normalizeSortMode(this.configFile.settings.sortMode) === next) return
      this.configFile.settings.sortMode = next
      this.writeConfig()
      if (next === 'duration') void this.warmDurations()
    },

    addTag(name: string, color: string): SoundTag {
      this.pushBeforeChange()
      if (!this.configFile.tags) this.configFile.tags = []
      const used = new Set(this.configFile.tags.map((t) => t.id))
      const tag: SoundTag = {
        id: newSoundId(used),
        name: name.trim() || 'Tag',
        color: color || '#00d4ff',
      }
      this.configFile.tags.push(tag)
      this.writeConfig()
      return tag
    },

    updateTag(id: string, patch: Partial<Pick<SoundTag, 'name' | 'color'>>) {
      const tag = (this.configFile.tags ?? []).find((t) => t.id === id)
      if (!tag) return
      this.pushBeforeChange()
      if (patch.name != null) tag.name = patch.name
      if (patch.color != null) tag.color = patch.color
      this.writeConfig()
    },

    removeTag(id: string) {
      this.pushBeforeChange()
      this.configFile.tags = (this.configFile.tags ?? []).filter((t) => t.id !== id)
      for (const f of this.configFile.files) {
        if (!f.tagIds?.length) continue
        f.tagIds = f.tagIds.filter((tid) => tid !== id)
      }
      const selected = this.configFile.settings.selectedTagIds ?? []
      if (selected.includes(id)) {
        this.configFile.settings.selectedTagIds = selected.filter((tid) => tid !== id)
      }
      this.applyBoardFilter()
      this.writeConfig()
    },

    setSoundTags(path: string, tagIds: string[]) {
      const file = this.configFile.files.find((f) => f.path === path)
      if (!file) return
      this.pushBeforeChange()
      file.tagIds = [...tagIds]
      this.applyBoardFilter()
      this.writeConfig()
    },

    toggleSoundTag(path: string, tagId: string) {
      const file = this.configFile.files.find((f) => f.path === path)
      if (!file) return
      this.pushBeforeChange()
      const cur = file.tagIds ?? []
      file.tagIds = cur.includes(tagId) ? cur.filter((id) => id !== tagId) : [...cur, tagId]
      this.applyBoardFilter()
      this.writeConfig()
    },

    /** Name search (existing rules) AND OR-match on selected tags. */
    applyBoardFilter(searchTerm?: string) {
      if (searchTerm != null) this._searchTerm = searchTerm
      const q = (this._searchTerm ?? '').trim()
      const tagIds = this.configFile.settings?.selectedTagIds ?? []
      let files = this.configFile.files ?? []
      if (tagIds.length) {
        const want = new Set(tagIds)
        files = files.filter((file) => (file.tagIds ?? []).some((id) => want.has(id)))
      }
      if (!q) {
        this.filteredFiles = files
        return
      }
      const qLower = q.toLowerCase()
      const qCompact = compactSearch(q)
      const tokens = qLower.split(/\s+/).filter((t) => t.length >= 2)
      this.filteredFiles = files.filter((file) => {
        const nameLower = (file.name ?? '').toLowerCase()
        const nameCompact = compactSearch(file.name ?? '')
        if (nameLower.includes(qLower)) return true
        if (qCompact.length > 0 && nameCompact.includes(qCompact)) return true
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

    filterSounds(searchTerm: string) {
      this.applyBoardFilter(searchTerm)
    },

    async warmSoundMeta() {
      const files = this.configFile.files
      const need = files.filter((f) => f.path && (f.fileSize == null || f.addedAt == null))
      if (!need.length) return
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const metas = await invoke<{ path: string; size: number; mtime: number }[]>(
          'get_sound_file_meta',
          { paths: need.map((f) => f.path) },
        )
        const byPath = new Map(metas.map((m) => [m.path, m]))
        const fallback = Date.now()
        let changed = false
        for (const f of files) {
          const m = byPath.get(f.path)
          if (f.fileSize == null) {
            f.fileSize = m?.size ?? 0
            changed = true
          }
          if (f.addedAt == null) {
            f.addedAt = m?.mtime && m.mtime > 0 ? Number(m.mtime) : fallback
            changed = true
          }
        }
        if (changed) this.writeConfig()
      } catch (e) {
        console.error('Failed to warm sound file meta', e)
      }
    },

    async warmDurations() {
      const files = this.configFile.files.filter((f) => f.path && (f.durationSecs == null || f.durationSecs <= 0))
      if (!files.length) return
      const { invoke } = await import('@tauri-apps/api/core')
      let changed = false
      for (const f of files) {
        try {
          const dur = await invoke<number>('get_sound_duration', { soundPath: f.path })
          if (typeof dur === 'number' && dur > 0) {
            f.durationSecs = dur
            changed = true
          }
        } catch { /* missing / undecodable */ }
      }
      if (changed) this.writeConfig()
    },
  },
})
