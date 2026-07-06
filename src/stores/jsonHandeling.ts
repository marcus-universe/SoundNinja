import { defineStore } from 'pinia'
import {
  openDb, getDb, loadConfig, saveConfig, emptyConfig,
  type ProjectConfig, type SoundFile, type TabEntry, type Separator,
} from '~/utils/db'

/** Deep clone helper. Config is pure JSON data, so a JSON round-trip both
 *  deep-clones and strips Vue reactive Proxies (which structuredClone rejects). */
function clone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v))
}

export const useJsonHandelingStore = defineStore('JsonHandeling', {
  state: () => ({
    currentProjectPath: null as string | null,
    configFile: emptyConfig() as ProjectConfig,
    filteredFiles: [] as SoundFile[],
    /** Snapshot taken when a project is opened — used to Discard changes. */
    openingSnapshot: null as ProjectConfig | null,
    dirty: false,
    _persistTimer: null as ReturnType<typeof setTimeout> | null,
  }),

  getters: {
    getConfig: (state) => state.configFile,
    separators: (state) => state.configFile.separators ?? [],
  },

  actions: {
    // ── Project lifecycle ─────────────────────────────────────────────────────
    /** Opens a project DB, loads it into state, and snapshots for Discard. */
    async openProject(dbAbsPath: string) {
      const d = await openDb(dbAbsPath)
      const config = await loadConfig(d)
      this.configFile = config
      this.normalizeIndexes()
      this.filteredFiles = this.configFile.files
      this.currentProjectPath = dbAbsPath
      this.openingSnapshot = clone(this.configFile)
      this.dirty = false
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
      this.normalizeIndexes()
      this.filteredFiles = this.configFile.files
      this.openingSnapshot = clone(this.configFile)
      this.dirty = false
      await this.persistNow()
    },

    /** Reverts in-memory state to the last opened snapshot and re-saves. */
    async discardChanges() {
      if (!this.openingSnapshot) return
      this.configFile = clone(this.openingSnapshot)
      this.normalizeIndexes()
      this.filteredFiles = this.configFile.files
      this.dirty = false
      await this.persistNow()
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
      this._persistTimer = setTimeout(() => { this.persistNow() }, 200)
    },

    /** Flushes any pending changes to the project DB immediately. */
    async persistNow() {
      if (this._persistTimer) {
        clearTimeout(this._persistTimer)
        this._persistTimer = null
      }
      const d = getDb()
      if (!d) return
      try {
        await saveConfig(d, this.configFile)
        this.openingSnapshot = clone(this.configFile)
        this.dirty = false
      } catch (e) {
        console.error('Failed to persist project', e)
      }
    },

    updateConfigFile(contents: ProjectConfig) {
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
     * Ensure every sound has a compact global `index` and a `tabIndexes` entry
     * for each tab it belongs to. Prunes stale tab entries and appends new ones.
     */
    normalizeIndexes() {
      const files = this.configFile.files
      if (!files) return

      const byGlobal = [...files].sort((a, b) => (a.index ?? 0) - (b.index ?? 0))
      byGlobal.forEach((f, i) => { f.index = i })

      for (const f of files) {
        if (!f.tabIndexes) f.tabIndexes = {}
        for (const key of Object.keys(f.tabIndexes)) {
          if (!f.tabs.includes(key)) delete f.tabIndexes[key]
        }
      }
      for (const tab of (this.configFile.tabList ?? []).map((t) => t.name)) {
        const inTab = files.filter((f) => f.tabs.includes(tab))
        inTab.sort((a, b) => {
          const av = a.tabIndexes[tab] ?? Number.MAX_SAFE_INTEGER
          const bv = b.tabIndexes[tab] ?? Number.MAX_SAFE_INTEGER
          return av !== bv ? av - bv : a.index - b.index
        })
        inTab.forEach((f, i) => { f.tabIndexes[tab] = i })
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

    // ── Sounds ────────────────────────────────────────────────────────────────
    addFiles(files: SoundFile[]) {
      this.configFile.files = [...this.configFile.files, ...files]
      this.normalizeIndexes()
      this.writeConfig()
    },

    setActiveSound({ soundindex, status }: { soundindex: number; status: boolean }) {
      this.configFile.files[soundindex].active = status
      this.writeConfig()
    },

    renameSound(soundindex: number, newName: string) {
      this.configFile.files[soundindex].name = newName
      this.writeConfig()
    },

    removeSound(soundindex: number) {
      this.configFile.files.splice(soundindex, 1)
      this.normalizeIndexes()
      this.writeConfig()
    },

    setSoundColor(soundindex: number, color: string) {
      this.configFile.files[soundindex].color = color
      this.writeConfig()
    },

    setSoundTabs(soundFileIndex: number, tabs: string[]) {
      this.configFile.files[soundFileIndex].tabs = tabs
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
      this.configFile.tabList.push({ name })
      this.writeConfig()
    },

    removeTab(name: string) {
      this.configFile.tabList = this.configFile.tabList.filter((t) => t.name !== name)
      this.configFile.separators = (this.configFile.separators ?? []).filter((s) => s.tab !== name)
      this.writeConfig()
    },

    renameTab(oldName: string, newName: string) {
      const tab = this.configFile.tabList.find((t) => t.name === oldName)
      if (!tab) return
      tab.name = newName
      this.configFile.files.forEach((f) => {
        const idx = f.tabs.indexOf(oldName)
        if (idx !== -1) f.tabs[idx] = newName
        if (f.tabIndexes && oldName in f.tabIndexes) {
          f.tabIndexes[newName] = f.tabIndexes[oldName]
          delete f.tabIndexes[oldName]
        }
      })
      ;(this.configFile.separators ?? []).forEach((s) => {
        if (s.tab === oldName) s.tab = newName
      })
      this.writeConfig()
    },

    setTabColor(name: string, color: string) {
      const tab = this.configFile.tabList.find((t) => t.name === name)
      if (tab) {
        tab.color = color
        this.writeConfig()
      }
    },

    reorderTabs(draggedName: string, targetName: string) {
      const list = this.configFile.tabList
      const from = list.findIndex((t) => t.name === draggedName)
      const to = list.findIndex((t) => t.name === targetName)
      if (from === -1 || to === -1 || from === to) return
      const [item] = list.splice(from, 1)
      list.splice(to, 0, item)
      this.writeConfig()
    },

    // ── Separators ────────────────────────────────────────────────────────────
    addSeparator(tab: string, position: number) {
      if (!this.configFile.separators) this.configFile.separators = []
      const id = `sep_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`
      this.configFile.separators.push({ id, tab, position })
      this.writeConfig()
    },

    removeSeparator(id: string) {
      this.configFile.separators = (this.configFile.separators ?? []).filter((s) => s.id !== id)
      this.writeConfig()
    },

    setSeparatorPosition(id: string, position: number) {
      const sep = (this.configFile.separators ?? []).find((s) => s.id === id)
      if (sep) {
        sep.position = position
        this.writeConfig()
      }
    },

    // ── Bulk / misc ───────────────────────────────────────────────────────────
    resetAll() {
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
      if (!searchTerm) {
        this.filteredFiles = this.configFile.files
      } else {
        this.filteredFiles = this.configFile.files.filter((file) =>
          file.name.toLowerCase().includes(searchTerm.toLowerCase())
        )
      }
    },
  },
})
