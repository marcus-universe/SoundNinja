import { defineStore } from 'pinia'
import {
  ensureSchema, withProjectDb, commitProject, copyProject, forgetProject, loadProject,
  extractProjectMedia, saveConfig, saveMeta, saveHistory, saveSnapshot, loadLatestSnapshot,
  emptyConfig, defaultMeta, gcOrphanGifs, sanitizeConfig,
  healFolderTabMembership, mergeTabsFromUsage, mergeSoundInto, dedupeSoundsByPath,
  ensurePerSoundVolume, shouldRestoreClearedBoard, PROJECT_FORMAT_VERSION,
  type ProjectConfig, type ProjectMeta, type SoundFile, type TabEntry, type Separator, type Settings,
  type ButtonAlign, type SoundTag, type SortMode, type CommitReport, type LoadedProject, normalizeSortMode,
} from '~/utils/db'
import {
  applyBoardLayout, boardTabNames, captureBoardLayout, mergeHiddenIntoLayout,
  setSoundOrderOnTab, soundOrderOnTab, type BoardLayout,
} from '~/utils/boardLayout'
import { dirOf, resolveMovedPaths, toRelativePath } from '~/utils/pathPortability'
import { ensureSoundIds, newSoundId } from '~/utils/soundId'
import { isRemote } from '~/utils/runtime'
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

/** Debounce for "save after every change". */
const AUTOSAVE_DEBOUNCE_MS = 200

/** What happened while a project was opened — shown to the user when notable. */
export interface OpenReport {
  /** The board came from a recovery copy instead of the live tables. */
  recovered: 'snapshot' | 'backup' | null
  /** Sounds found again relative to the project file (moved folder / other OS). */
  relinked: number
  /** Sounds unpacked from audio stored inside the project. */
  extracted: number
  /** File was written by an older format and has been brought up to date. */
  upgraded: boolean
}

function emptyOpenReport(): OpenReport {
  return { recovered: null, relinked: 0, extracted: 0, upgraded: false }
}

/** History entries already in the project file: id → JSON size. */
let storedHistory = new Map<number, number>()

let appVersionCache: string | null = null
async function appVersion(): Promise<string> {
  if (appVersionCache != null) return appVersionCache
  try {
    const { getVersion } = await import('@tauri-apps/api/app')
    appVersionCache = await getVersion()
  } catch {
    appVersionCache = ''
  }
  return appVersionCache
}

function newProjectId(): string {
  try {
    return crypto.randomUUID()
  } catch {
    return `p${Date.now().toString(36)}${Math.random().toString(36).slice(2, 10)}`
  }
}

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
    /** Next persist may write zero sounds. Set only by an intentional clear. */
    _allowEmptyPersist: false,
    /** Embed options + stable id of the open project. */
    projectMeta: defaultMeta() as ProjectMeta,
    /** Save automatically after changes (app setting). Off = manual Save only. */
    autosave: true,
    /** 0 = right after every change, otherwise at most every N seconds. */
    autosaveIntervalSec: 0,
    /** Message of the last failed background save. Cleared by the next good one. */
    saveError: null as string | null,
    /** Result of the last save (embedded audio counts, files that were missing). */
    lastSaveReport: null as CommitReport | null,
    /** What the last `openProject` had to do (recover, relink, unpack). */
    lastOpenReport: emptyOpenReport() as OpenReport,
    /** Undo/redo stacks as of the last save — what Discard goes back to. */
    _savedUndo: [] as ProjectConfig[],
    _savedRedo: [] as ProjectConfig[],
    /** Source of undo/redo entry ids. */
    _hidSeq: 0,
    /** Bumped by every change; tells a finishing save whether it is already stale. */
    _changeSeq: 0,
    /** First save of a session writes a `.bak` and an in-file snapshot. */
    _sessionSaved: false,
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
    /** Copy of the live board, tagged with a fresh history id. */
    takeSnapshot(): ProjectConfig {
      const snap = clone(this.configFile)
      snap._hid = ++this._hidSeq
      return snap
    },

    /** Snapshot current config before a tracked mutation. Clears redo. */
    pushBeforeChange() {
      if (this._historySuspended) return
      this.undoStack.push(this.takeSnapshot())
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
        const next = clone(snap)
        delete next._hid
        // History may come from disk or an older build: never trust its shape.
        sanitizeConfig(next)
        this.configFile = next
        this.normalizeIndexes()
        this.applyBoardFilter()
        this.noteIntentionalEmpty()
        this.writeConfig()
      } finally {
        this._historySuspended = false
      }
    },

    undo() {
      if (!this.undoStack.length) return false
      this.redoStack.push(this.takeSnapshot())
      this.applySnapshot(this.undoStack.pop()!)
      return true
    },

    redo() {
      if (!this.redoStack.length) return false
      this.undoStack.push(this.takeSnapshot())
      this.applySnapshot(this.redoStack.pop()!)
      return true
    },

    // ── Project lifecycle ─────────────────────────────────────────────────────
    /**
     * Opens a project file and loads it into state.
     *
     * Order matters: read (recovering from a backup if the file is unreadable
     * or its board was lost), normalise, then make the project work on THIS
     * machine — find moved sounds next to the project file and unpack embedded
     * audio — and finally write back whatever had to be repaired.
     */
    async openProject(dbAbsPath: string) {
      revokeAllGifUrls()
      this._allowEmptyPersist = false
      if (this._persistTimer) {
        clearTimeout(this._persistTimer)
        this._persistTimer = null
      }
      const report = emptyOpenReport()

      let loaded: LoadedProject
      try {
        loaded = await loadProject(dbAbsPath)
      } catch (e) {
        loaded = await this.recoverUnreadable(dbAbsPath, e)
        report.recovered = 'backup'
      }

      let config = loaded.config
      if (shouldRestoreClearedBoard(config.files.length, loaded.marker)) {
        const restored = await this.recoverEmptyBoard(dbAbsPath)
        if (restored) {
          config = restored.config
          report.recovered = restored.source
        }
      }

      this.configFile = config
      if (!this.configFile.tags) this.configFile.tags = []
      // The board as found on disk. The first save of the session stores it
      // as a recovery snapshot (and, for an old file, as its pre-upgrade state).
      this.openingSnapshot = clone(config)
      this.projectMeta = { ...defaultMeta(), ...loaded.meta }
      this.currentProjectPath = dbAbsPath
      this.missingPaths = []
      this.saveError = null
      this.lastSaveReport = null
      // After a recovery the file on disk is the damaged one. Copying it over
      // `.bak` would destroy the very backup the board was just read from.
      this._sessionSaved = !!report.recovered

      // Undo/redo come back with the project. After a recovery the stored
      // history describes a different board, so it is dropped.
      const history = report.recovered ? { undo: [], redo: [], sizes: new Map<number, number>() } : loaded.history
      this.undoStack = history.undo.slice(-MAX_HISTORY)
      this.redoStack = history.redo.slice(-MAX_HISTORY)
      storedHistory = new Map(history.sizes)
      this._hidSeq = Math.max(0, ...history.sizes.keys())

      this.normalizeIndexes()
      const idsChanged = ensureSoundIds(this.configFile.files)
      const volumeMigrated = ensurePerSoundVolume(this.configFile)
      const needsStamp = loaded.formatVersion !== PROJECT_FORMAT_VERSION || !this.projectMeta.projectId
      report.upgraded = loaded.formatVersion !== PROJECT_FORMAT_VERSION && config.files.length > 0

      this.syncFilterSessionFromSettings()
      this.applyBoardFilter()

      try {
        report.relinked = await this.relinkMovedSounds()
        if (this.projectMeta.embedAudio) {
          // Unpacking reads paths from the file, so relinked ones go in first.
          if (report.relinked) await this.persistNow()
          report.extracted = await this.extractEmbeddedSounds()
        }
      } catch (e) {
        console.error('Failed to resolve sound locations', e)
      }

      const repaired = !!report.recovered || report.relinked > 0 || report.extracted > 0
      if (repaired || needsStamp || idsChanged || volumeMigrated) {
        try {
          await this.persistNow()
        } catch (e) {
          console.error('Failed to write back the opened project', e)
        }
      }
      this.openingSnapshot = clone(this.configFile)
      this._savedUndo = [...this.undoStack]
      this._savedRedo = [...this.redoStack]
      this.dirty = false
      this.lastOpenReport = report

      void this.warmSoundMeta()
      if (normalizeSortMode(this.configFile.settings.sortMode) === 'duration') {
        void this.warmDurations()
      }
    },

    /**
     * The project file itself cannot be read (truncated copy, disk error).
     * Put the damaged file aside and continue from `<file>.bak`.
     */
    async recoverUnreadable(dbAbsPath: string, cause: unknown): Promise<LoadedProject> {
      const bak = dbAbsPath + '.bak'
      const { invoke } = await import('@tauri-apps/api/core')
      const hasBak = await invoke<boolean>('path_exists_abs', { path: bak }).catch(() => false)
      if (!hasBak) throw cause
      let fromBak: LoadedProject
      try {
        fromBak = await loadProject(bak)
      } catch {
        throw cause
      }
      console.error('Project unreadable, restoring from backup', cause)
      try {
        await invoke('copy_file_to_abs', { src: dbAbsPath, dst: `${dbAbsPath}.damaged` })
      } catch { /* keeping the damaged copy is best effort */ }
      for (const suffix of ['', '-wal', '-shm', '-journal']) {
        try { await invoke('delete_file_abs', { path: dbAbsPath + suffix }) } catch { /* ignore */ }
      }
      forgetProject(dbAbsPath)
      await copyProject(bak, dbAbsPath)
      try {
        return await loadProject(dbAbsPath)
      } catch {
        return fromBak
      }
    },

    /**
     * The file opens but holds no sounds although the last save expected some
     * (an interrupted save by a build without atomic saves). Look for the board
     * in the in-file snapshots, then in the backup files next to the project.
     */
    async recoverEmptyBoard(
      dbAbsPath: string,
    ): Promise<{ config: ProjectConfig; source: 'snapshot' | 'backup' } | null> {
      try {
        const snap = await withProjectDb(dbAbsPath, (d) => loadLatestSnapshot(d))
        if (snap?.files.length) return { config: snap, source: 'snapshot' }
      } catch (e) {
        console.error('Failed to read project snapshots', e)
      }
      const candidates = [dbAbsPath + '.bak', dbAbsPath.replace(/\.sninja$/i, '.restored.sninja')]
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        for (const source of candidates) {
          if (source === dbAbsPath) continue
          if (!(await invoke<boolean>('path_exists_abs', { path: source }))) continue
          try {
            const loaded = await loadProject(source)
            if (loaded.config.files.length) return { config: loaded.config, source: 'backup' }
          } catch (e) {
            console.error('Backup unreadable', source, e)
          }
        }
      } catch (e) {
        console.error('Failed to restore project from backup', e)
      }
      return null
    },

    /** Rewrite sound paths in the live board and in every history entry. */
    remapSoundPaths(map: Map<string, string>) {
      if (!map.size) return
      const apply = (config: ProjectConfig): boolean => {
        let changed = false
        for (const f of config.files ?? []) {
          const to = map.get(f.path)
          if (to) { f.path = to; changed = true }
        }
        return changed
      }
      apply(this.configFile)
      for (const snap of [...this.undoStack, ...this.redoStack]) {
        // A rewritten entry differs from its stored copy: send it again.
        if (apply(snap) && snap._hid != null) storedHistory.delete(snap._hid)
      }
      dedupeSoundsByPath(this.configFile)
    },

    /**
     * Sounds whose absolute path does not exist here (project moved, other
     * computer, other OS) are looked up relative to the project file.
     * Returns how many were found.
     */
    async relinkMovedSounds(): Promise<number> {
      const path = this.currentProjectPath
      const files = this.configFile.files
      if (isRemote || !path || !files.length) return 0
      const { invoke } = await import('@tauri-apps/api/core')
      const exists = (paths: string[]) => invoke<boolean[]>('paths_exist_abs', { paths })
      const flags = await exists(files.map((f) => f.path))
      const missing = files.filter((_, i) => !flags[i])
      if (!missing.length) return 0
      const present = files.filter((_, i) => flags[i]).map((f) => f.path)
      const map = await resolveMovedPaths(
        missing.map((f) => ({ path: f.path, relPath: f.relPath })),
        dirOf(path),
        present,
        exists,
      )
      this.remapSoundPaths(map)
      return map.size
    },

    /** Unpack audio stored in the project for sounds still missing on disk. */
    async extractEmbeddedSounds(): Promise<number> {
      const path = this.currentProjectPath
      if (isRemote || !path) return 0
      const extracted = await extractProjectMedia(path, this.projectMeta.projectId || 'project')
      if (!extracted.length) return 0
      const byId = new Map(extracted.map((e) => [e.soundId, e.path]))
      const taken = new Set(this.configFile.files.map((f) => f.path))
      const map = new Map<string, string>()
      for (const f of this.configFile.files) {
        const to = f.id ? byId.get(f.id) : undefined
        if (!to || to === f.path || taken.has(to)) continue
        taken.add(to)
        map.set(f.path, to)
      }
      this.remapSoundPaths(map)
      return map.size
    },

    async validateSoundPaths() {
      const paths = this.configFile.files.map((f) => f.path).filter(Boolean)
      this.missingPaths = []
      if (!paths.length) return
      const { invoke } = await import('@tauri-apps/api/core')
      const exists = await invoke<boolean[]>('paths_exist_abs', { paths })
      this.missingPaths = paths.filter((_, i) => !exists[i])
    },

    /** Loads config into a project file (used by JSON import/migration). */
    async importConfig(config: ProjectConfig, dbAbsPath?: string) {
      if (dbAbsPath) {
        await ensureSchema(dbAbsPath)
        this.currentProjectPath = dbAbsPath
        this.projectMeta = defaultMeta()
        this._sessionSaved = false
      }
      this.configFile = {
        settings: config.settings,
        tabList: config.tabList ?? [],
        files: config.files ?? [],
        separators: config.separators ?? [],
        tags: config.tags ?? [],
        ...(config.extraSettings ? { extraSettings: config.extraSettings } : {}),
      }
      sanitizeConfig(this.configFile, { resetActive: true })
      mergeTabsFromUsage(this.configFile)
      healFolderTabMembership(this.configFile)
      ensureSoundIds(this.configFile.files)
      ensurePerSoundVolume(this.configFile)
      this.normalizeIndexes()
      this.syncFilterSessionFromSettings()
      this.applyBoardFilter()
      this.clearHistory()
      storedHistory = new Map()
      this.noteIntentionalEmpty()
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
      // Back to the history that belongs to the saved board, so undo/redo
      // keep matching what is on disk.
      this.undoStack = [...this._savedUndo]
      this.redoStack = [...this._savedRedo]
      if (this._persistTimer) {
        clearTimeout(this._persistTimer)
        this._persistTimer = null
      }
    },

    setCurrentProjectPath(p: string | null) {
      this.currentProjectPath = p
    },

    /** Autosave preference from the app settings. */
    setAutosave(enabled: boolean, intervalSec = 0) {
      this.autosave = !!enabled
      this.autosaveIntervalSec = Number.isFinite(intervalSec) && intervalSec > 0 ? Math.round(intervalSec) : 0
      if (this._persistTimer) {
        clearTimeout(this._persistTimer)
        this._persistTimer = null
      }
      if (this.autosave && this.dirty) this.schedulePersist()
    },

    /** Whether sound files / GIFs are stored inside the project file. */
    setSaveOptions(opts: { embedAudio: boolean; embedGifs: boolean }) {
      this.projectMeta = {
        ...this.projectMeta,
        embedAudio: !!opts.embedAudio,
        embedGifs: !!opts.embedGifs,
        saveOptionsSet: true,
      }
    },

    /** @deprecated legacy no-op kept for the old settings page. */
    setHue(_val: number) {
      // hue was replaced by the theme system; intentionally does nothing.
    },

    // ── Persistence ───────────────────────────────────────────────────────────
    /** Remember a user clear so the next save may persist zero sounds. */
    noteIntentionalEmpty() {
      if ((this.configFile.files?.length ?? 0) === 0) this._allowEmptyPersist = true
    },

    /**
     * Note a change. With autosave on this schedules a save: right away
     * (debounced) or, with an interval, at most once per interval. With
     * autosave off the project just becomes "unsaved".
     *
     * `silent` is for housekeeping the user did not do (cached durations, file
     * sizes): it is written with the next save but never asks to be saved.
     */
    schedulePersist(opts?: { silent?: boolean }) {
      this._changeSeq++
      if (!opts?.silent) this.dirty = true
      if (!this.autosave) return
      const interval = this.autosaveIntervalSec > 0
      // Interval mode: the first change starts the clock, later ones ride along.
      if (interval && this._persistTimer) return
      if (this._persistTimer) clearTimeout(this._persistTimer)
      this._persistTimer = setTimeout(() => {
        this._persistTimer = null
        this.persistNow().catch((e) => {
          console.error('Failed to persist project', e)
          this.saveError = e instanceof Error ? e.message : String(e)
        })
      }, interval ? this.autosaveIntervalSec * 1000 : AUTOSAVE_DEBOUNCE_MS)
    },

    /** Public compat alias — rebuilds the filtered board, then schedules a save. */
    writeConfig(opts?: { silent?: boolean }) {
      this.applyBoardFilter()
      this.schedulePersist(opts)
    },

    /**
     * Writes the project to disk now, as one atomic transaction: board,
     * project options, undo/redo history and (first save of a session) a
     * recovery snapshot. On failure the file is left exactly as it was.
     */
    async persistNow() {
      if (isRemote) {
        this.dirty = false
        return
      }
      if (this._persistTimer) {
        clearTimeout(this._persistTimer)
        this._persistTimer = null
      }
      if (!this.currentProjectPath) {
        throw new Error('No project database is open.')
      }
      const path = this.currentProjectPath
      const seq = this._changeSeq
      const version = await appVersion()
      const allowEmpty = this._allowEmptyPersist && (this.configFile.files?.length ?? 0) === 0

      // Remember where each sound sits relative to the project file, so the
      // project still finds it after moving to another machine.
      const dir = dirOf(path)
      for (const f of this.configFile.files) {
        const rel = toRelativePath(f.path, dir)
        if (rel) {
          if (f.relPath !== rel) f.relPath = rel
        } else if (f.relPath != null) {
          delete f.relPath
        }
      }
      if (!this.projectMeta.projectId) {
        this.projectMeta = { ...this.projectMeta, projectId: newProjectId() }
      }
      for (const snap of [...this.undoStack, ...this.redoStack]) {
        if (snap._hid == null) snap._hid = ++this._hidSeq
      }

      const firstOfSession = !this._sessionSaved
      const before = firstOfSession && this.openingSnapshot?.files?.length ? this.openingSnapshot : null
      const meta = this.projectMeta
      this.saving = true
      try {
        let sizes = storedHistory
        const { report } = await commitProject(path, async (d) => {
          await saveConfig(d, this.configFile, { ...(allowEmpty ? { allowEmpty: true } : {}), appVersion: version })
          // History entries may bring a button background back on undo: keep those images.
          const keep = new Set<string>()
          for (const config of [this.configFile, ...this.undoStack, ...this.redoStack]) {
            for (const f of config.files ?? []) if (f.gifId) keep.add(f.gifId)
          }
          await gcOrphanGifs(d, [...keep])
          await saveMeta(d, meta)
          sizes = await saveHistory(d, this.undoStack, this.redoStack, storedHistory)
          if (before) await saveSnapshot(d, before, 'session', version)
        }, {
          embedAudio: meta.embedAudio,
          embedGifs: meta.embedGifs,
          backup: firstOfSession,
        })
        storedHistory = sizes
        this._sessionSaved = true
        this.lastSaveReport = report
        this.openingSnapshot = clone(this.configFile)
        this._savedUndo = [...this.undoStack]
        this._savedRedo = [...this.redoStack]
        // A change that arrived while the save ran is still unsaved.
        this.dirty = this._changeSeq !== seq
        this._allowEmptyPersist = false
        this.saveError = null
      } catch (e) {
        console.error('Failed to persist project', e)
        throw e
      } finally {
        this.saving = false
      }
    },

    /**
     * Save As: copy the project file (images, stored audio and history come
     * along), switch to the copy, then write the current board into it. The
     * original keeps the state of its last save.
     */
    async saveAs(dbAbsPath: string, options?: { embedAudio: boolean; embedGifs: boolean }) {
      const { ensureProjectParentDir, removeInvalidProjectPlaceholder } = await import('~/utils/projects')
      await ensureProjectParentDir(dbAbsPath)
      await removeInvalidProjectPlaceholder(dbAbsPath)

      const src = this.currentProjectPath
      if (!src) {
        await this.importConfig(clone(this.configFile), dbAbsPath)
        if (options) {
          this.setSaveOptions(options)
          await this.persistNow()
        }
        return
      }

      const same = src.replace(/\\/g, '/').toLowerCase() === dbAbsPath.replace(/\\/g, '/').toLowerCase()
      if (!same) {
        await copyProject(src, dbAbsPath)
        this.currentProjectPath = dbAbsPath
        // The copy is new: no backup of "its previous state" to take.
        this._sessionSaved = true
      }
      if (options) this.setSaveOptions(options)
      await this.persistNow()
    },

    updateConfigFile(contents: ProjectConfig) {
      this.pushBeforeChange()
      this.configFile = {
        settings: contents.settings,
        tabList: contents.tabList ?? [],
        files: contents.files ?? [],
        separators: contents.separators ?? [],
        tags: contents.tags ?? [],
        ...(this.configFile.extraSettings ? { extraSettings: this.configFile.extraSettings } : {}),
      }
      sanitizeConfig(this.configFile)
      this.normalizeIndexes()
      this.applyBoardFilter()
      this.noteIntentionalEmpty()
      this.writeConfig()
    },

    /**
     * Fill missing tab/order keys, then densify each tab so group markers
     * keep slots on the same number line as sounds. Sound-only compacting
     * (0..n) steals those slots and shuffles group membership on load.
     */
    normalizeIndexes() {
      const config = this.configFile
      const files = config.files
      if (!files) return

      for (const f of files) {
        if (!Array.isArray(f.tabs)) f.tabs = ['All']
        if (!f.tabs.includes('All')) f.tabs.unshift('All')
        if (!f.tabIndexes) f.tabIndexes = {}
        for (const key of Object.keys(f.tabIndexes)) {
          if (!f.tabs.includes(key)) delete f.tabIndexes[key]
        }
      }

      for (const tab of boardTabNames(config)) {
        const inTab = files.filter((f) => f.tabs?.includes(tab))
        const seps = (config.separators ?? []).filter((s) => s.tab === tab)
        let max = -1
        const isSet = (v: unknown): v is number =>
          typeof v === 'number' && Number.isFinite(v) && v < Number.MAX_SAFE_INTEGER
        for (const f of inTab) {
          const raw = tab === 'All' ? f.index : f.tabIndexes[tab]
          if (isSet(raw)) max = Math.max(max, raw)
        }
        for (const s of seps) {
          if (isSet(s.position)) max = Math.max(max, s.position)
        }
        // A sound new to this tab has no slot yet: it goes to the end.
        for (const f of inTab) {
          const raw = tab === 'All' ? f.index : f.tabIndexes[tab]
          if (!isSet(raw)) setSoundOrderOnTab(f, tab, ++max)
        }
        applyBoardLayout(config, tab, captureBoardLayout(config, tab))
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
      const indexByPath = new Map<string, number>()
      this.configFile.files.forEach((f, i) => {
        if (f.path && !indexByPath.has(f.path)) indexByPath.set(f.path, i)
      })
      const used = new Set(this.configFile.files.map((f) => f.id).filter(Boolean))
      const now = Date.now()
      const fresh: SoundFile[] = []
      for (const f of files) {
        const path = f.path ?? ''
        const existingIdx = path ? indexByPath.get(path) : undefined
        if (existingIdx != null) {
          const existing = this.configFile.files[existingIdx]
          if (existing) mergeSoundInto(existing, f)
          continue
        }
        const next: SoundFile = {
          ...f,
          tagIds: f.tagIds ?? [],
          addedAt: f.addedAt ?? now,
          id: f.id || '',
        }
        if (!next.id) next.id = newSoundId(used)
        used.add(next.id)
        if (path) indexByPath.set(path, this.configFile.files.length + fresh.length)
        fresh.push(next)
      }
      if (fresh.length) this.configFile.files = [...this.configFile.files, ...fresh]
      dedupeSoundsByPath(this.configFile)
      this.normalizeIndexes()
      this.applyBoardFilter()
      this.writeConfig()
      void this.warmDurations()
    },

    commitImportReview(review: ImportReviewState) {
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
      // Playback state only. Saving on every play/stop cost a full project
      // write per click and could leave a button stored as "playing".
      file.active = status
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
      this.noteIntentionalEmpty()
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

    /**
     * Swap the audio file on a button. Name, id, color, gif, tabs, volume stay.
     * Returns false when `nextPath` is already another button — `sounds.path`
     * is the primary key, so two buttons cannot share one file path.
     */
    replaceSoundPath(soundindex: number, nextPath: string): boolean {
      const file = this.configFile.files[soundindex]
      if (!file || !nextPath || file.path === nextPath) return false
      if (this.configFile.files.some((f, i) => i !== soundindex && f.path === nextPath)) return false
      this.pushBeforeChange()
      const old = file.path
      file.path = nextPath
      this.missingPaths = this.missingPaths.filter((p) => p !== old)
      this.writeConfig()
      return true
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

    /**
     * Point missing buttons at found files.
     * A destination already used by another button is skipped (returns that
     * count). Two missing files with the same name must not collapse onto
     * one path — that is SQLITE 1555 on the next save.
     */
    relinkSounds(pairs: { from: string; to: string }[]): number {
      if (!pairs.length) return 0
      this.pushBeforeChange()
      const reserved = new Set(this.configFile.files.map((f) => f.path))
      const moved = new Set<string>()
      let skipped = 0
      for (const { from, to } of pairs) {
        if (!from || !to || from === to) continue
        const file = this.configFile.files.find((f) => f.path === from)
        if (!file || moved.has(from)) {
          skipped++
          continue
        }
        if (reserved.has(to)) {
          skipped++
          continue
        }
        reserved.delete(from)
        reserved.add(to)
        file.path = to
        moved.add(from)
      }
      dedupeSoundsByPath(this.configFile)
      this.missingPaths = this.missingPaths.filter((p) => {
        if (!moved.has(p)) return true
        return this.configFile.files.some((f) => f.path === p)
      })
      this.writeConfig()
      return skipped
    },

    removeSoundsMany(paths: string[]) {
      this.pushBeforeChange()
      const set = new Set(paths)
      this.configFile.files = this.configFile.files.filter((f) => !set.has(f.path))
      this.normalizeIndexes()
      this.noteIntentionalEmpty()
      this.writeConfig()
    },

    reorderSounds(draggedIdx: number, targetIdx: number, tab: string) {
      const files = this.configFile.files
      const dragged = files.find((f) => f.tabs.includes(tab) && f.index === draggedIdx)
      const target = files.find((f) => f.tabs.includes(tab) && f.index === targetIdx)
      if (!dragged || !target || dragged === target) return
      // Move within the tab's layout so group markers keep their own slots —
      // renumbering sounds alone lets them collide with the markers.
      const layout = captureBoardLayout(this.configFile, tab)
      const sections = [layout.orphans, ...layout.groups.map((g) => g.paths)]
      const to = sections.find((sec) => sec.includes(target.path))
      const from = sections.find((sec) => sec.includes(dragged.path))
      if (!to || !from) return
      const at = to.indexOf(target.path)
      from.splice(from.indexOf(dragged.path), 1)
      to.splice(at, 0, dragged.path)
      this.pushBeforeChange()
      applyBoardLayout(this.configFile, tab, layout)
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
      // Sounds must drop the tab too. A leftover reference made the deleted
      // tab reappear on the next open (tabs are rebuilt from what sounds use).
      for (const f of this.configFile.files) {
        if (f.tabs?.includes(name)) f.tabs = f.tabs.filter((t) => t !== name)
        if (f.tabIndexes && name in f.tabIndexes) delete f.tabIndexes[name]
      }
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
      // Callers pass a slot between two sounds (e.g. 2.5). Renumber the tab so
      // the marker owns a whole number like everything else.
      applyBoardLayout(this.configFile, tab, captureBoardLayout(this.configFile, tab))
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
      return soundOrderOnTab(sound, tab)
    },

    setSoundOrderOnTab(sound: SoundFile, tab: string, order: number) {
      setSoundOrderOnTab(sound, tab, order)
    },

    /**
     * Rewrite dense interleaved order for a tab: orphan sounds, then each
     * group marker + its members. Keeps relative membership from `layout`.
     */
    applyBoardLayoutSilent(tab: string, layout: BoardLayout) {
      applyBoardLayout(this.configFile, tab, layout)
    },

    /**
     * Apply a layout read from the screen. With a search or tag filter active
     * the screen shows only part of the tab; the hidden sounds keep their
     * place instead of colliding with the renumbered visible ones.
     */
    applyBoardLayout(tab: string, layout: BoardLayout) {
      const merged = mergeHiddenIntoLayout(captureBoardLayout(this.configFile, tab), layout)
      this.pushBeforeChange()
      applyBoardLayout(this.configFile, tab, merged)
      this.writeConfig()
    },

    /**
     * Rebuild board layout from current positional membership, then densify.
     * Call after cross-group drops so float anchors never drift.
     */
    normalizeBoardOrder(tab: string) {
      const layout = captureBoardLayout(this.configFile, tab)
      this.applyBoardLayout(tab, layout)
    },

    /** Snapshot orphans + groups with member paths from current positions. */
    captureBoardLayout(tab: string): BoardLayout {
      return captureBoardLayout(this.configFile, tab)
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
      const extra = this.configFile.extraSettings
      this.configFile = emptyConfig()
      if (extra) this.configFile.extraSettings = extra
      this.normalizeIndexes()
      this.filteredFiles = []
      this.noteIntentionalEmpty()
      this.writeConfig()
    },

    ReturnStatusAll() {
      this.configFile.files.forEach((file) => { file.active = false })
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
        if (changed) this.writeConfig({ silent: true })
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
      if (changed) this.writeConfig({ silent: true })
    },
  },
})
