import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface AppSettings {
  projectsPath: string
  themesPath: string
  navbarSide: 'left' | 'right'
  lastProjectPath: string | null
  locale: string | null
}

export const useAppSettingsStore = defineStore('appSettings', {
  state: () => ({
    projectsPath: '',
    themesPath: '',
    navbarSide: 'left' as 'left' | 'right',
    lastProjectPath: null as string | null,
    locale: null as string | null,
    loaded: false,
  }),

  getters: {
    fontsPath: (state) => joinPath(state.themesPath, 'fonts'),
  },

  actions: {
    async load() {
      const s = await invoke<AppSettings>('get_app_settings')
      this.projectsPath = s.projectsPath
      this.themesPath = s.themesPath
      this.navbarSide = s.navbarSide === 'right' ? 'right' : 'left'
      this.lastProjectPath = s.lastProjectPath ?? null
      this.locale = s.locale ?? null
      this.loaded = true
      this.applyNavbarSide()
      return s
    },

    async save() {
      const payload: AppSettings = {
        projectsPath: this.projectsPath,
        themesPath: this.themesPath,
        navbarSide: this.navbarSide,
        lastProjectPath: this.lastProjectPath,
        locale: this.locale,
      }
      await invoke('set_app_settings', { settings: payload })
    },

    async setNavbarSide(side: 'left' | 'right') {
      this.navbarSide = side
      this.applyNavbarSide()
      await this.save()
    },

    applyNavbarSide() {
      if (typeof document === 'undefined') return
      const root = document.documentElement
      root.classList.toggle('navbar-right', this.navbarSide === 'right')
      root.classList.toggle('navbar-left', this.navbarSide !== 'right')
      root.style.setProperty('--navbar-side', this.navbarSide)
    },

    async setLastProject(path: string | null) {
      this.lastProjectPath = path
      await this.save()
    },

    /** Relocates projects/themes folder. mode: 'copy' | 'blank'. */
    async relocate(kind: 'projects' | 'themes', target: string, mode: 'copy' | 'blank') {
      const newPath = await invoke<string>('relocate_data', { kind, target, mode })
      if (kind === 'projects') this.projectsPath = newPath
      else this.themesPath = newPath
      await this.load()
      return newPath
    },
  },
})

function joinPath(base: string, child: string): string {
  if (!base) return child
  const sep = base.includes('\\') ? '\\' : '/'
  return base.replace(/[\\/]+$/, '') + sep + child
}
