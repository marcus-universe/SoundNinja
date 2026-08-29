<template>
  <div class="playing-list-window">
    <header class="playing-list-window__head">
      <h2 class="playing-list-window__title">{{ $t('player.playlist') }}</h2>
    </header>
    <PlayingList window-mode :items="playing" @changed="refreshPlaying" />
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import type { PlayingInfo } from '~/components/Section/PlayingList.vue'

const playing = ref<PlayingInfo[]>([])
let unlisten: UnlistenFn | null = null

async function refreshPlaying() {
  try {
    playing.value = (await invoke<PlayingInfo[]>('get_playing_sounds')) ?? []
  } catch {
    playing.value = []
  }
}

onMounted(async () => {
  await refreshPlaying()
  unlisten = await listen<PlayingInfo[]>('playing_changed', (e) => {
    playing.value = e.payload ?? []
  })
  getCurrentWindow()
    .onCloseRequested((event) => {
      event.preventDefault()
      getCurrentWindow().hide().catch(() => {})
    })
    .catch(() => {})
})

onUnmounted(() => {
  if (unlisten) unlisten()
})
</script>
