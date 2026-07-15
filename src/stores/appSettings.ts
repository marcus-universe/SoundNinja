import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { readTextFile, rename, exists, BaseDirectory } from '@tauri-apps/plugin-fs'
import {
  getDefaultPaths,
  openAppConfigDb,
  loadSettings as loadDbSettings,
  saveSetting,
  addRecent,
  listRecent,
  removeRecent,
  trimRecent,
  type RecentProject,
} from '~/utils/appConfig'
import type Database from '@tauri-apps/plugin-sql'

const DEFAULT_RECENT_LIMIT = 30

export const useAppSettingsStore = defineStore('appSettings', {
  state: () => ({
    projectsPath: '',
    themesPath: '',
    appConfigDbPath: '',
    navbarSide: 'left' as 'left' | 'right',
    lastProjectPath: null as string | null,
    locale: null as string | null,
    recentProjects: [] as RecentProject[],
    recentLimit: DEFAULT_RECENT_LIMIT,
    loaded: false,
  }),

  getters: {
    fontsPath: (state) => joinPath(state.themesPath, 'fonts'),
  },

  actions: {
    async _db(): Promise<Database> {
      return openAppConfigDb(this.appConfigDbPath)
    },

    async load() {
      const defaults = await getDefaultPaths()
      this.appConfigDbPath = defaults.appConfigDbPath
      const d = await openAppConfigDb(this.appConfigDbPath)

      let s = await loadDbSettings(d)
      // One-time migration from the legacy app-settings.json.
      if (!s.migrated) {
        await this.migrateFromJson(d, defaults.projectsPath)
        s = await loadDbSettings(d)
      }

      this.projectsPath = s.projectsPath || defaults.projectsPath
      this.themesPath = s.themesPath || defaults.themesPath
      this.navbarSide = s.navbarSide === 'right' ? 'right' : 'left'
      this.lastProjectPath = s.lastProjectPath || null
      this.locale = s.locale || null
      this.recentLimit = s.recentLimit ? Number(s.recentLimit) || DEFAULT_RECENT_LIMIT : DEFAULT_RECENT_LIMIT

      await this.refreshRecents()
      this.loaded = true
      this.applyNavbarSide()
      return s
    },

    /** Seeds app-config.db from the legacy app-settings.json (if present) and
     *  scans the old projects folder for the recent-projects list. */
    async migrateFromJson(d: Database, defaultProjectsPath: string) {
      let projectsPath = defaultProjectsPath
      try {
        if (await exists('app-settings.json', { baseDir: BaseDirectory.AppData })) {
          const txt = await readTextFile('app-settings.json', { baseDir: BaseDirectory.AppData })
          const old = JSON.parse(txt)
          if (old.projectsPath) { await saveSetting(d, 'projectsPath', old.projectsPath); projectsPath = old.projectsPath }
          if (old.themesPath) await saveSetting(d, 'themesPath', old.themesPath)
          if (old.navbarSide) await saveSetting(d, 'navbarSide', old.navbarSide)
          if (old.lastProjectPath) await saveSetting(d, 'lastProjectPath', old.lastProjectPath)
          if (old.locale) await saveSetting(d, 'locale', old.locale)
          try {
            await rename('app-settings.json', 'app-settings.migrated.json', {
              oldPathBaseDir: BaseDirectory.AppData,
              newPathBaseDir: BaseDirectory.AppData,
            })
          } catch { /* non-critical */ }
        }
      } catch { /* no legacy settings */ }

      // Seed recents from any projects found in the old folder-based layout.
      try {
        const projs = await listProjects(projectsPath)
        for (const p of projs) await addRecent(d, p.dbPath, p.name)
      } catch { /* ignore */ }

      await saveSetting(d, 'migrated', '1')
    },

    async refreshRecents() {
      const d = await this._db()
      this.recentProjects = await listRecent(d, this.recentLimit)
      await this.pushRecentsToMenu()
    },

    async pushRecentsToMenu() {
      try {
        await invoke('set_recent_projects', {
          recents: this.recentProjects.map((r) => ({ path: r.dbPath, name: r.name })),
        })
      } catch { /* menu not available (secondary window) */ }
    },

    /** Records a project open in the recent list and as the last project. */
    async touchRecent(dbPath: string, name: string) {
      const d = await this._db()
      await addRecent(d, dbPath, name)
      await trimRecent(d, this.recentLimit)
      await this.setLastProject(dbPath)
      await this.refreshRecents()
    },

    async removeRecentProject(dbPath: string) {
      const d = await this._db()
      await removeRecent(d, dbPath)
      await this.refreshRecents()
    },

    async setRecentLimit(limit: number) {
      this.recentLimit = Math.max(1, Math.min(100, Math.round(limit) || DEFAULT_RECENT_LIMIT))
      const d = await this._db()
      await saveSetting(d, 'recentLimit', String(this.recentLimit))
      await trimRecent(d, this.recentLimit)
      await this.refreshRecents()
    },

    async setNavbarSide(side: 'left' | 'right') {
      this.navbarSide = side
      this.applyNavbarSide()
      const d = await this._db()
      await saveSetting(d, 'navbarSide', side)
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
      const d = await this._db()
      await saveSetting(d, 'lastProjectPath', path ?? '')
    },

    async setLocale(locale: string | null) {
      this.locale = locale
      const d = await this._db()
      await saveSetting(d, 'locale', locale ?? '')
    },

    /** Relocates projects/themes folder. mode: 'copy' | 'blank'. */
    async relocate(kind: 'projects' | 'themes', target: string, mode: 'copy' | 'blank') {
      const oldPath = kind === 'projects' ? this.projectsPath : this.themesPath
      const newPath = await invoke<string>('relocate_data', { oldPath, target, mode })
      const d = await this._db()
      if (kind === 'projects') {
        this.projectsPath = newPath
        await saveSetting(d, 'projectsPath', newPath)
      } else {
        this.themesPath = newPath
        await saveSetting(d, 'themesPath', newPath)
      }
      return newPath
    },
  },
})

function joinPath(base: string, child: string): string {
  if (!base) return child
  const sep = base.includes('\\') ? '\\' : '/'
  return base.replace(/[\\/]+$/, '') + sep + child
}
