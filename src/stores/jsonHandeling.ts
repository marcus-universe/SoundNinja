import { defineStore } from 'pinia'
import { writeTextFile, BaseDirectory } from '@tauri-apps/plugin-fs'

interface SoundFile {
  name: string
  path: string
  volume: number
  tabs: string[]
  active: boolean
  index: number
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
      this.filteredFiles = contents.files
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

    addFiles(files: SoundFile[]) {
      this.configFile.files = [...this.configFile.files, ...files]
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
      // re-index
      this.configFile.files.forEach((f, i) => { f.index = i })
      this.writeConfig()
    },

    setSoundColor(soundindex: number, color: string) {
      this.configFile.files[soundindex].color = color
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
