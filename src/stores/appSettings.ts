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
import {
  DEFAULT_APP_HOTKEYS,
  parseAppHotkeys,
  type AppHotkeyAction,
} from '~/utils/hotkeys'

const DEFAULT_RECENT_LIMIT = 30

export type TitlebarMode = 'styled' | 'system'

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
    /** 'styled' = custom HTML titlebar+menu; 'system' = OS decorations + native menu */
    titlebarMode: 'styled' as TitlebarMode,
    /** When true, hide titlebar/menu chrome entirely (no drag / window controls via GUI). */
    hideTitlebar: false,
    /** Skip the "hide title bar" warning dialog on future toggles. */
    hideTitlebarSkipWarn: false,
    /** Show slide-in tooltips next to sidebar (navbar) buttons. Default on. */
    navbarTooltips: true,
    /** Check GitHub Releases for a newer version on app start. Default on. */
    checkUpdatesOnStart: true,
    /** User-supplied Klipy GIF API key (app-wide, never stored in project files). */
    klipyApiKey: '',
    /** App-wide audio prefs (not stored in project files). */
    outputSource: 'default',
    outputHost: 'WASAPI',
    outputVolume: 1,
    asioLeftChannel: null as number | null,
    asioRightChannel: null as number | null,
    /** Capture device for the Record Editor (mic or PC-audio loopback). */
    inputSource: 'default',
    inputHost: 'WASAPI',
    /** Capture gain for recording (0–2, default 1). Applied in Rust while capturing. */
    inputVolume: 1,
    /** True when the selected inputSource is a WASAPI loopback (PC Audio) device. */
    inputLoopback: false,
    /** True once audio keys exist in app-config.db (or after one-time project migrate). */
    audioMigrated: false,
    loaded: false,
    /** Remappable in-app action combos. */
    hotkeys: { ...DEFAULT_APP_HOTKEYS } as Record<AppHotkeyAction, string>,
    /** When true, sound-trigger combos register as OS-global shortcuts. */
    soundTriggersGlobal: false,
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
      this.titlebarMode = s.titlebarMode === 'system' ? 'system' : 'styled'
      this.hideTitlebar = s.hideTitlebar === '1' || s.hideTitlebar === 'true'
      this.hideTitlebarSkipWarn = s.hideTitlebarSkipWarn === '1' || s.hideTitlebarSkipWarn === 'true'
      // Default enabled when unset (first launch / older configs).
      this.navbarTooltips = s.navbarTooltips !== '0' && s.navbarTooltips !== 'false'
      this.checkUpdatesOnStart = s.checkUpdatesOnStart !== '0' && s.checkUpdatesOnStart !== 'false'
      this.klipyApiKey = s.klipyApiKey || ''
      try {
        this.hotkeys = parseAppHotkeys(s.hotkeys ? JSON.parse(s.hotkeys) : null)
      } catch {
        this.hotkeys = { ...DEFAULT_APP_HOTKEYS }
      }
      this.soundTriggersGlobal = s.soundTriggersGlobal === '1' || s.soundTriggersGlobal === 'true'
      this.audioMigrated = s.audioMigrated === '1' || s.audioMigrated === 'true'
        || s.outputSource != null || s.outputHost != null || s.outputVolume != null
        || s.inputSource != null || s.inputHost != null
      if (this.audioMigrated) {
        this.outputSource = s.outputSource || 'default'
        this.outputHost = s.outputHost || 'WASAPI'
        const vol = s.outputVolume != null ? Number(s.outputVolume) : 1
        this.outputVolume = Number.isFinite(vol) ? Math.max(0, Math.min(1, vol)) : 1
        this.asioLeftChannel = s.asioLeftChannel != null && s.asioLeftChannel !== ''
          ? Number(s.asioLeftChannel) : null
        this.asioRightChannel = s.asioRightChannel != null && s.asioRightChannel !== ''
          ? Number(s.asioRightChannel) : null
        if (this.asioLeftChannel != null && !Number.isFinite(this.asioLeftChannel)) this.asioLeftChannel = null
        if (this.asioRightChannel != null && !Number.isFinite(this.asioRightChannel)) this.asioRightChannel = null
        this.inputSource = s.inputSource || 'default'
        this.inputHost = s.inputHost || 'WASAPI'
        const inVol = s.inputVolume != null ? Number(s.inputVolume) : 1
        this.inputVolume = Number.isFinite(inVol) ? Math.max(0, Math.min(2, inVol)) : 1
        this.inputLoopback = s.inputLoopback === '1' || s.inputLoopback === 'true'
      }

      await this.refreshRecents()
      this.loaded = true
      this.applyNavbarSide()
      await this.applyWindowChrome()
      await this.applyAudioVolume()
      await this.applyInputVolume()
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

    async setTitlebarMode(mode: TitlebarMode) {
      this.titlebarMode = mode === 'system' ? 'system' : 'styled'
      const d = await this._db()
      await saveSetting(d, 'titlebarMode', this.titlebarMode)
      await this.applyWindowChrome()
    },

    async setHideTitlebar(hidden: boolean) {
      this.hideTitlebar = !!hidden
      const d = await this._db()
      await saveSetting(d, 'hideTitlebar', this.hideTitlebar ? '1' : '0')
      await this.applyWindowChrome()
    },

    async setHideTitlebarSkipWarn(skip: boolean) {
      this.hideTitlebarSkipWarn = !!skip
      const d = await this._db()
      await saveSetting(d, 'hideTitlebarSkipWarn', this.hideTitlebarSkipWarn ? '1' : '0')
    },

    async setNavbarTooltips(enabled: boolean) {
      this.navbarTooltips = !!enabled
      const d = await this._db()
      await saveSetting(d, 'navbarTooltips', this.navbarTooltips ? '1' : '0')
    },

    async setCheckUpdatesOnStart(enabled: boolean) {
      this.checkUpdatesOnStart = !!enabled
      const d = await this._db()
      await saveSetting(d, 'checkUpdatesOnStart', this.checkUpdatesOnStart ? '1' : '0')
    },

    async setHotkeys(hotkeys: Record<AppHotkeyAction, string>) {
      this.hotkeys = { ...DEFAULT_APP_HOTKEYS, ...hotkeys }
      const d = await this._db()
      await saveSetting(d, 'hotkeys', JSON.stringify(this.hotkeys))
    },

    async setAppHotkey(action: AppHotkeyAction, combo: string) {
      this.hotkeys = { ...this.hotkeys, [action]: combo }
      const d = await this._db()
      await saveSetting(d, 'hotkeys', JSON.stringify(this.hotkeys))
    },

    async setSoundTriggersGlobal(enabled: boolean) {
      this.soundTriggersGlobal = !!enabled
      const d = await this._db()
      await saveSetting(d, 'soundTriggersGlobal', this.soundTriggersGlobal ? '1' : '0')
    },

    async setKlipyApiKey(key: string) {
      this.klipyApiKey = (key || '').trim()
      const d = await this._db()
      await saveSetting(d, 'klipyApiKey', this.klipyApiKey)
    },

    /** CSS --topbar_height only. Does not touch OS decorations or emit events. */
    async applyChromeCss() {
      if (typeof document === 'undefined') return
      const hidden = this.hideTitlebar
      const showStyled = !hidden && this.titlebarMode === 'styled'
      let topbar = '0px'
      if (showStyled) {
        let isMain = true
        try {
          const { getCurrentWindow } = await import('@tauri-apps/api/window')
          isMain = getCurrentWindow().label === 'main'
        } catch { /* non-tauri */ }
        topbar = isMain ? '5.6rem' : '3rem'
      }
      document.documentElement.style.setProperty('--topbar_height', topbar)
    },

    /** Apply chrome prefs received from another window (no OS re-invoke, no re-emit). */
    async applyChromeFromEvent(payload: { titlebarMode?: TitlebarMode; hideTitlebar?: boolean }) {
      if (payload.titlebarMode === 'system' || payload.titlebarMode === 'styled') {
        this.titlebarMode = payload.titlebarMode
      }
      if (typeof payload.hideTitlebar === 'boolean') {
        this.hideTitlebar = payload.hideTitlebar
      }
      await this.applyChromeCss()
    },

    /** Syncs OS decorations/native menu + CSS --topbar_height with stored chrome prefs. */
    async applyWindowChrome() {
      const nativeChrome = this.titlebarMode === 'system'
      const hidden = this.hideTitlebar
      try {
        await invoke('set_window_chrome', { nativeChrome, hidden })
      } catch (e) {
        console.error('set_window_chrome failed', e)
      }
      await this.applyChromeCss()
      try {
        const { emit } = await import('@tauri-apps/api/event')
        await emit('sn:chrome-changed', {
          titlebarMode: this.titlebarMode,
          hideTitlebar: this.hideTitlebar,
        })
      } catch { /* non-tauri */ }
    },

    /** Pushes master volume into the Rust audio engine. */
    async applyAudioVolume() {
      try {
        await invoke('set_output_volume', { volume: this.outputVolume })
      } catch (e) {
        console.error('set_output_volume failed', e)
      }
    },

    async applyInputVolume() {
      try {
        await invoke('set_input_volume', { volume: this.inputVolume })
      } catch (e) {
        console.error('set_input_volume failed', e)
      }
    },

    /**
     * One-time copy of audio prefs from a project settings object into app-config.db.
     * Call after the first project open when audioMigrated is still false.
     */
    async migrateAudioFromProject(projectSettings: {
      outputSource?: string
      outputHost?: string
      outputVolume?: number
      asioLeftChannel?: number
      asioRightChannel?: number
    } | null | undefined) {
      if (this.audioMigrated) return
      if (projectSettings) {
        if (projectSettings.outputSource) this.outputSource = projectSettings.outputSource
        if (projectSettings.outputHost) this.outputHost = projectSettings.outputHost
        if (typeof projectSettings.outputVolume === 'number' && Number.isFinite(projectSettings.outputVolume)) {
          this.outputVolume = Math.max(0, Math.min(1, projectSettings.outputVolume))
        }
        if (typeof projectSettings.asioLeftChannel === 'number') {
          this.asioLeftChannel = projectSettings.asioLeftChannel
        }
        if (typeof projectSettings.asioRightChannel === 'number') {
          this.asioRightChannel = projectSettings.asioRightChannel
        }
      }
      await this.persistAudioSettings()
      await this.applyAudioVolume()
    },

    async persistAudioSettings() {
      const d = await this._db()
      await saveSetting(d, 'outputSource', this.outputSource || 'default')
      await saveSetting(d, 'outputHost', this.outputHost || 'WASAPI')
      await saveSetting(d, 'outputVolume', String(this.outputVolume))
      await saveSetting(d, 'asioLeftChannel', this.asioLeftChannel != null ? String(this.asioLeftChannel) : '')
      await saveSetting(d, 'asioRightChannel', this.asioRightChannel != null ? String(this.asioRightChannel) : '')
      await saveSetting(d, 'inputSource', this.inputSource || 'default')
      await saveSetting(d, 'inputHost', this.inputHost || 'WASAPI')
      await saveSetting(d, 'inputVolume', String(this.inputVolume))
      await saveSetting(d, 'inputLoopback', this.inputLoopback ? '1' : '0')
      await saveSetting(d, 'audioMigrated', '1')
      this.audioMigrated = true
    },

    async setOutputSource(source: string) {
      this.outputSource = source || 'default'
      const d = await this._db()
      await saveSetting(d, 'outputSource', this.outputSource)
      if (!this.audioMigrated) {
        await saveSetting(d, 'audioMigrated', '1')
        this.audioMigrated = true
      }
    },

    async setOutputHost(host: string) {
      this.outputHost = host || 'WASAPI'
      const d = await this._db()
      await saveSetting(d, 'outputHost', this.outputHost)
      if (!this.audioMigrated) {
        await saveSetting(d, 'audioMigrated', '1')
        this.audioMigrated = true
      }
    },

    async setInputSource(source: string, loopback = false) {
      this.inputSource = source || 'default'
      this.inputLoopback = !!loopback
      const d = await this._db()
      await saveSetting(d, 'inputSource', this.inputSource)
      await saveSetting(d, 'inputLoopback', this.inputLoopback ? '1' : '0')
      if (!this.audioMigrated) {
        await saveSetting(d, 'audioMigrated', '1')
        this.audioMigrated = true
      }
    },

    async setInputHost(host: string) {
      this.inputHost = host || 'WASAPI'
      const d = await this._db()
      await saveSetting(d, 'inputHost', this.inputHost)
      if (!this.audioMigrated) {
        await saveSetting(d, 'audioMigrated', '1')
        this.audioMigrated = true
      }
    },

    async setInputVolume(volume: number) {
      const vol = Number.isFinite(volume) ? Math.max(0, Math.min(2, volume)) : 1
      this.inputVolume = vol
      const d = await this._db()
      await saveSetting(d, 'inputVolume', String(vol))
      if (!this.audioMigrated) {
        await saveSetting(d, 'audioMigrated', '1')
        this.audioMigrated = true
      }
      await this.applyInputVolume()
    },

    async setOutputVolume(volume: number) {
      const vol = Number.isFinite(volume) ? Math.max(0, Math.min(1, volume)) : 1
      this.outputVolume = vol
      const d = await this._db()
      await saveSetting(d, 'outputVolume', String(vol))
      if (!this.audioMigrated) {
        await saveSetting(d, 'audioMigrated', '1')
        this.audioMigrated = true
      }
      await this.applyAudioVolume()
    },

    async setAsioChannels(left: number | null, right: number | null) {
      this.asioLeftChannel = left
      this.asioRightChannel = right
      const d = await this._db()
      await saveSetting(d, 'asioLeftChannel', left != null ? String(left) : '')
      await saveSetting(d, 'asioRightChannel', right != null ? String(right) : '')
      if (!this.audioMigrated) {
        await saveSetting(d, 'audioMigrated', '1')
        this.audioMigrated = true
      }
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
