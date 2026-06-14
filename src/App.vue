<template>
  <div data-tauri-drag-region class="titlebar">
    <div class="titlebar-button" id="titlebar-minimize" @click="minimize">
      <img src="https://api.iconify.design/mdi:window-minimize.svg" alt="minimize" />
    </div>
    <div class="titlebar-button" id="titlebar-maximize" @click="maximize">
      <img src="https://api.iconify.design/mdi:window-maximize.svg" alt="maximize" />
    </div>
    <div class="titlebar-button" id="titlebar-close" @click="close">
      <img src="https://api.iconify.design/mdi:close.svg" alt="close" />
    </div>
  </div>

  <div class="soundninja flex_c_h flex_space_between">
    <NavBar />
    <ErrorAlert />
    <NuxtPage v-slot="{ Component }">
      <transition name="fade" mode="out-in">
        <component :is="Component" />
      </transition>
    </NuxtPage>
  </div>
</template>

<script setup>
import { readTextFile, writeTextFile, createDir, BaseDirectory } from '@tauri-apps/api/fs'

const jsonStore = useJsonHandelingStore()

const hue = computed(() => jsonStore.configFile?.settings?.hue)

watch(hue, (hueval) => {
  if (hueval !== undefined && hueval !== null) {
    document.documentElement.style.setProperty(
      '--primary_color',
      `hsl(${hueval}, 100%, 58%)`
    )
  }
})

const defaultConfig = {
  tabList: [],
  settings: {
    hue: 189,
    outputSource: 'default',
  },
  files: [],
}

async function readConfigFile() {
  try {
    const contents = JSON.parse(
      await readTextFile('config.json', { dir: BaseDirectory.App })
    )
    jsonStore.updateConfigFile(contents)
    return contents
  } catch {
    await createDir('', { dir: BaseDirectory.App, recursive: true })
    await writeTextFile(
      { path: 'config.json', contents: JSON.stringify(defaultConfig, null, 2) },
      { dir: BaseDirectory.App }
    )
    jsonStore.updateConfigFile(defaultConfig)
    return defaultConfig
  }
}

const minimize = async () => {
  const { appWindow } = await import('@tauri-apps/api/window')
  appWindow.minimize()
}

const maximize = async () => {
  const { appWindow } = await import('@tauri-apps/api/window')
  appWindow.maximize()
}

const close = async () => {
  const { appWindow } = await import('@tauri-apps/api/window')
  appWindow.close()
}

onMounted(() => {
  readConfigFile()
})
</script>

