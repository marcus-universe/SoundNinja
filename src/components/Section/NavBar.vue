<template>
  <div class="navbar flex_c_v">
    <div class="iconContainer flex_c_v flex_space_evenly gap1">
      <Icons
        :icon="'search'"
        :customClass="[
          'icon searchButton',
          { active: appStore.Searchbar.SearchbarActive },
        ]"
        @triggered="IconClicked"
      />
      <transition name="slideIn">
        <div
          v-if="appStore.Searchbar.SearchbarActive"
          class="searchBar flex_c_h align_c flex_start gap1"
        >
          <input
            type="text"
            :placeholder="$t('navbar.search')"
            v-model="appStore.Searchbar.SearchbarContent"
            @input="jsonStore.filterSounds(appStore.Searchbar.SearchbarContent)"
          />
        </div>
      </transition>

      <Icons
        v-for="(navelm, index) in appStore.navbar"
        :key="navelm"
        :icon="appStore.navbar[index]"
        @triggered="IconClicked"
      />
    </div>
  </div>
</template>

<script setup>
import { open } from '@tauri-apps/plugin-dialog'

const { t } = useI18n()
const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

async function uploadFiles() {
  const selected = await open({
    multiple: true,
    title: 'Select files to upload',
    filters: [
      {
        name: 'Add Sounds',
        extensions: ['mp3', 'wav', 'ogg'],
      },
    ],
  })

  if (Array.isArray(selected)) {
        //   const tabs = ["All"];
        //   if (self.currentTab !== "All") {
        //     tabs.push(self.currentTab);
        //   }
        //   soundlist.push({
        //     name: file
        //       .replace(/^.*[\\]/, "")
        //       .replace(".wav", "")
        //       .replace(".mp3", "")
        //       .replace(".ogg", "")
        //       .replaceAll("_", " ")
        //       .replace(/([A-Z])/g, " $1")
        //       .trim(),
        //     path: file,
        //     volume: 0.4,
        //     tabs: tabs,
        //     active: false,
        //   });
        // });
    const indexLength = jsonStore.configFile.files.length
    const soundlist = selected.map((file, index) => {
      const tabs = ['All']
      if (appStore.currentTab !== 'All') {
        tabs.push(appStore.currentTab)
      }
      return {
        name: file
          .replace(/^.*[\\]/, '')
          .replace('.wav', '')
          .replace('.mp3', '')
          .replace('.ogg', '')
          .replaceAll('_', ' ')
          .replace(/([A-Z])/g, ' $1')
          .trim(),
        path: file,
        volume: 0.4,
        tabs: tabs,
        active: false,
        index: index + indexLength,
      }
    })

    try {
      jsonStore.addFiles(soundlist)
    } catch (err) {
      console.log(err)
    }
  }
}

function OpenSearch() {
  appStore.setSearchOpen(!appStore.Searchbar.SearchbarActive)
}

function IconClicked(icon) {
  if (icon === 'upload') {
    uploadFiles()
  } else if (icon === 'search') {
    OpenSearch()
  } else if (icon === 'folder') {
    appStore.setSelectProjectActive(true)
  } else if (icon === 'settings') {
    appStore.setActiveOverlay(appStore.activeOverlay === 'settings' ? null : 'settings')
  } else if (icon === 'about') {
    appStore.openSettingsTab('about')
  }
}
</script>
