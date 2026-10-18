<template>
  <p v-if="banner" class="remote-banner" role="status">{{ banner }}</p>
</template>

<script setup lang="ts">
import { remoteLink } from '~/utils/webuiHost'

const { t } = useI18n()
const jsonStore = useJsonHandelingStore()

const banner = computed(() => {
  if (remoteLink.state === 'rejected') return t('remoteUi.rejected')
  if (remoteLink.state === 'hostDown') return t('remoteUi.hostDown')
  if (remoteLink.state === 'connecting') return t('remoteUi.connecting')
  if (remoteLink.state === 'closed') return t('remoteUi.closed')
  if (!jsonStore.currentProjectPath) return t('remoteUi.waiting')
  return ''
})
</script>
