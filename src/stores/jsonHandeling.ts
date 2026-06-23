import { defineStore } from 'pinia'
import { writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs'

interface SoundFile {
  name: string
  path: string
  volume: number
  tabs: string[]
  active: boolean
  /** Global position used by the "All" tab */
  index: number
  /** Per-tab position, keyed by tab name (excludes "All") */
  tabIndexes: Record<string, number>
  color?: string
}

interface TabEntry {
  name: string
  color?: string
}

interface Config {
  settings: {
    theme: string
    customCss: string
    outputSource: string
    stopOnRetrigger: boolean
    overlapSounds: boolean
    cacheMaxSizeMib?: number
    cacheMaxEntryMib?: number
  }
  tabList: TabEntry[]
  files: SoundFile[]
}

export const useJsonHandelingStore = defineStore('JsonHandeling', {
  state: () => ({
    NewJsonData: {} as Record<string, unknown>,
    FileStruct: [] as unknown[],
    TabList: [] as TabEntry[],
    Settings: {
      theme: 'dark-cyan',
    },
    JSONFile: null as unknown,
    path: null as string | null,
    currentProjectPath: null as string | null,
    configFile: {
      settings: {
        theme: 'dark-cyan',
        customCss: '',
        outputSource: 'default',
        stopOnRetrigger: true,
        overlapSounds: false,
      },
      tabList: [],
      files: [],
    } as Config,
    filteredFiles: [] as SoundFile[],
  }),

  getters: {
    getJsonFile: (state) => state.JSONFile,
    getConfig: (state) => state.configFile,
  },

  actions: {
    updateConfigFile(contents: Config) {
      this.configFile = contents
      this.normalizeIndexes()
      this.filteredFiles = contents.files
    },

    /**
     * Ensure every sound has a compact global `index` and a `tabIndexes` entry
     * for each tab it belongs to. Prunes stale tab entries and appends new ones.
     * Migrates old configs that lack `tabIndexes`.
     */
    normalizeIndexes() {
      const files = this.configFile.files
      if (!files) return

      // Global index (drives the "All" tab): compact, preserving current order.
      const byGlobal = [...files].sort((a, b) => (a.index ?? 0) - (b.index ?? 0))
      byGlobal.forEach((f, i) => { f.index = i })

      // Per-tab indexes for real tabs only.
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

    writeConfig() {
      writeTextFile(
        'config.json',
        JSON.stringify(this.configFile, null, 2),
        { baseDir: BaseDirectory.AppData }
      )
    },

    saveToPath(filePath: string): Promise<void> {
      return writeTextFile(filePath, JSON.stringify(this.configFile, null, 2))
    },

    setCurrentProjectPath(p: string | null) {
      this.currentProjectPath = p
    },

    /** @deprecated use setTheme instead */
    setHue(val: number) {
      // legacy shim – ignored, kept so old call sites don't crash
    },

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

    addFiles(files: SoundFile[]) {
      this.configFile.files = [...this.configFile.files, ...files]
      this.normalizeIndexes()
      this.writeConfig()
    },

    setActiveSound({ soundindex, status }: { soundindex: number; status: boolean }) {
      this.configFile.files[soundindex].active = status
      this.writeConfig()
    },

    // ---- Tab actions ----
    addTab(name: string) {
      this.configFile.tabList.push({ name })
      this.writeConfig()
    },

    removeTab(name: string) {
      this.configFile.tabList = this.configFile.tabList.filter((t) => t.name !== name)
      this.writeConfig()
    },

    renameTab(oldName: string, newName: string) {
      const tab = this.configFile.tabList.find((t) => t.name === oldName)
      if (tab) {
        tab.name = newName
        // update all sounds that reference this tab
        this.configFile.files.forEach((f) => {
          const idx = f.tabs.indexOf(oldName)
          if (idx !== -1) f.tabs[idx] = newName
          if (f.tabIndexes && oldName in f.tabIndexes) {
            f.tabIndexes[newName] = f.tabIndexes[oldName]
            delete f.tabIndexes[oldName]
          }
        })
        this.writeConfig()
      }
    },

    setTabColor(name: string, color: string) {
      const tab = this.configFile.tabList.find((t) => t.name === name)
      if (tab) {
        tab.color = color
        this.writeConfig()
      }
    },

    // ---- Sound actions ----
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

    /**
     * Reorder sounds within a tab. `draggedIdx`/`targetIdx` are global `index`
     * values (stable keys). For the "All" tab the global index is updated;
     * for any other tab the per-tab `tabIndexes[tab]` is updated.
     */
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

    reorderTabs(draggedName: string, targetName: string) {
      const list = this.configFile.tabList
      const from = list.findIndex((t) => t.name === draggedName)
      const to = list.findIndex((t) => t.name === targetName)
      if (from === -1 || to === -1 || from === to) return
      const [item] = list.splice(from, 1)
      list.splice(to, 0, item)
      this.writeConfig()
    },

    resetAll() {
      this.configFile = {
        settings: { theme: 'dark-cyan', customCss: '', outputSource: 'default' },
        tabList: [],
        files: [],
      }
      this.writeConfig()
    },

    ReturnStatusAll() {
      this.configFile.files.forEach((file) => {
        file.active = false
      })
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
