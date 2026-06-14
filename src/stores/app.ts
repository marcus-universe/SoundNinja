import { defineStore } from 'pinia'
import { useJsonHandelingStore } from './jsonHandeling'

export const useAppStore = defineStore('app', {
  state: () => ({
    navbar: ['upload', 'folder', 'reset', 'settings', 'about'] as string[],
    currentTab: 'All',
    PopupActive: { active: false, type: 'addTab' } as { active: boolean; type: string },
    RenameContent: '',
    ErrorMessage: '',
    ErrorActive: false,
    Searchbar: {
      SearchbarActive: false,
      SearchbarContent: '',
    },
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
        if (this.PopupActive.type === 'addTab') {
          const jsonStore = useJsonHandelingStore()
          jsonStore.configFile.tabList.push(this.RenameContent)
        }
        const jsonStore = useJsonHandelingStore()
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

    setCurrentTab(val: string) {
      this.currentTab = val
    },
  },
})
