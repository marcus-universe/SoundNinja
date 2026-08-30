<template>
  <div class="first-run">
    <div class="first-run__card">
      <h2 class="first-run__title">{{ $t('firstRun.title') }}</h2>
      <p class="first-run__message">{{ $t('firstRun.message') }}</p>
      <select v-model="selected" class="settings-select" @change="previewLocale">
        <option v-for="loc in availableLocales" :key="loc.code" :value="loc.code">{{ loc.name }}</option>
      </select>
      <div class="first-run__actions">
        <UIButton @click="confirm">{{ $t('firstRun.continue') }}</UIButton>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { isAppLocale, resolveAppLocale, type AppLocale } from '~/utils/locales'

const emit = defineEmits<{
  done: [locale: AppLocale]
}>()

const props = defineProps<{
  initial?: string
}>()

const { locale, locales: availableLocales, setLocale } = useI18n()

const selected = ref(resolveAppLocale(props.initial || locale.value))

async function previewLocale() {
  const next = isAppLocale(selected.value) ? selected.value : 'en'
  selected.value = next
  await setLocale(next)
}

async function confirm() {
  const next = isAppLocale(selected.value) ? selected.value : 'en'
  await setLocale(next)
  emit('done', next)
}

onMounted(previewLocale)
</script>
