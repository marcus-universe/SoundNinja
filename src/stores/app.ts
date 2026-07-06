import { defineStore } from 'pinia'
import { useJsonHandelingStore } from './jsonHandeling'

interface ContextMenuState {
  visible: boolean
  x: number
  y: number
  type: 'tab' | 'sound' | 'separator' | null
  targetName: string
  targetIndex: number
}

export const useAppStore = defineStore('app', {
  state: () => ({
    navbar: ['upload', 'folder', 'settings', 'about'] as string[],
    currentTab: 'All',
    activeOverlay: null as 'settings' | 'about' | null,
    pendingSettingsTab: null as string | null,
    PopupActive: { active: false, type: 'addTab' } as { active: boolean; type: string },
    RenameContent: '',
    ErrorMessage: '',
    ErrorActive: false,
    Searchbar: {
      SearchbarActive: false,
      SearchbarContent: '',
    },
    importFoldersActive: false,
    selectProjectActive: false,
    draggingSoundIndex: null as number | null,
    draggingTabName: null as string | null,
    contextMenu: {
      visible: false,
      x: 0,
      y: 0,
      type: null,
      targetName: '',
      targetIndex: -1,
    } as ContextMenuState,
  }),

  actions: {
    setPopupActive({ active, type }: { active: boolean; type: string }) {
      this.PopupActive.active = active
      this.PopupActive.type = type
    },

    setRenameContent({ name }: { name: string }) {
      if (name === '') {
        this.ErrorMessage = 'Field is empty'
      } else {
        this.RenameContent = name
        this.ErrorMessage = ''
        const jsonStore = useJsonHandelingStore()
        if (this.PopupActive.type === 'addTab') {
          jsonStore.addTab(name)
        } else if (this.PopupActive.type === 'renameTab') {
          jsonStore.renameTab(this.contextMenu.targetName, name)
          if (this.currentTab === this.contextMenu.targetName) {
            this.currentTab = name
          }
        } else if (this.PopupActive.type === 'renameSound') {
          jsonStore.renameSound(this.contextMenu.targetIndex, name)
        }
        jsonStore.writeConfig()
      }
    },

    setErrorActive(val: string | boolean) {
      if (val === false || val === '') {
        this.ErrorActive = false
        this.ErrorMessage = ''
      } else {
        this.ErrorActive = true
        this.ErrorMessage = String(val)
      }
    },

    setSearchOpen(val: boolean) {
      this.Searchbar.SearchbarActive = val
    },

    setSearchContent(val: string) {
      this.Searchbar.SearchbarContent = val
    },

    setSelectProjectActive(val: boolean) {
      this.selectProjectActive = val
    },

    setCurrentTab(val: string) {
      this.currentTab = val
    },

    setActiveOverlay(val: 'settings' | 'about' | null) {
      this.activeOverlay = val
    },

    openSettingsTab(tab: string) {
      this.activeOverlay = 'settings'
      this.pendingSettingsTab = tab
    },

    consumePendingSettingsTab(): string | null {
      const t = this.pendingSettingsTab
      this.pendingSettingsTab = null
      return t
    },

    setImportFoldersActive(val: boolean) {
      this.importFoldersActive = val
    },

    setDraggingSoundIndex(idx: number | null) {
      this.draggingSoundIndex = idx
    },

    setDraggingTabName(name: string | null) {
      this.draggingTabName = name
    },

    openContextMenu({
      x, y, type, targetName, targetIndex,
    }: { x: number; y: number; type: 'tab' | 'sound'; targetName: string; targetIndex: number }) {
      this.contextMenu = { visible: true, x, y, type, targetName, targetIndex }
    },

    closeContextMenu() {
      this.contextMenu.visible = false
    },
  },
})
