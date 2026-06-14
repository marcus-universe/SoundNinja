import { defineStore } from 'pinia'
import { writeTextFile, BaseDirectory } from '@tauri-apps/api/fs'

interface SoundFile {
  name: string
  path: string
  volume: number
  tabs: string[]
  active: boolean
  index: number
}

interface Config {
  tabList: string[]
  settings: {
    hue: number
    outputSource: string
  }
  files: SoundFile[]
}

export const useJsonHandelingStore = defineStore('JsonHandeling', {
  state: () => ({
    NewJsonData: {} as Record<string, unknown>,
    FileStruct: [] as unknown[],
    TabList: [] as string[],
    Settings: {
      hue: 0,
    },
    JSONFile: null as unknown,
    path: null as string | null,
    currentProjectPath: null as string | null,
    configFile: {
      tabList: [],
      settings: {
        hue: 0,
        outputSource: 'default',
      },
      files: [],
    } as Config,
  }),

  getters: {
    getJsonFile: (state) => state.JSONFile,
    getConfig: (state) => state.configFile,
  },

  actions: {
    updateConfigFile(contents: Config) {
      this.configFile = contents
    },

    writeConfig() {
      writeTextFile(
        { path: 'config.json', contents: JSON.stringify(this.configFile, null, 2) },
        { dir: BaseDirectory.App }
      )
    },

    saveToPath(filePath: string): Promise<void> {
      return writeTextFile(filePath, JSON.stringify(this.configFile, null, 2))
    },

    setCurrentProjectPath(p: string | null) {
      this.currentProjectPath = p
    },

    setHue(val: number) {
      this.configFile.settings.hue = val
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

    resetAll() {
      this.configFile = {
        tabList: [],
        settings: { hue: 189, outputSource: 'default' },
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
  },
})
