<template>
  <Transition name="overlay-fade">
    <div
      v-if="appStore.selectProjectActive"
      class="project-overlay"
      @click.self="appStore.setSelectProjectActive(false)"
    >
      <div class="project-dialog">
        <div class="project-dialog__header">
          <h2>{{ $t('project.title') }}</h2>
          <Icons icon="exit" custom-class="exit icon" @triggered="appStore.setSelectProjectActive(false)" />
        </div>

        <div class="project-dialog__body">
          <div v-if="projectsLoading" class="project-dialog__empty">{{ $t('project.loading') }}</div>
          <div v-else-if="projectsList.length === 0" class="project-dialog__empty">{{ $t('project.noProjects') }}</div>
          <ul v-else class="project-list">
            <li
              v-for="proj in projectsList"
              :key="proj"
              class="project-list__item"
              :class="{ active: jsonStore.currentProjectPath === proj }"
              @click="selectProject(proj)"
            >
              <span class="project-list__name">{{ proj.replace('.json', '') }}</span>
              <span v-if="jsonStore.currentProjectPath === proj" class="project-list__check">✓</span>
            </li>
          </ul>
        </div>

        <div class="project-dialog__footer">
          <div class="new-project-row">
            <input
              v-model="newProjectName"
              type="text"
              class="new-project-input settings-input"
              :placeholder="$t('project.newPlaceholder')"
              @keyup.enter="createProject"
            />
            <button
              class="settings-btn settings-btn--primary"
              :disabled="!newProjectName.trim()"
              @click="createProject"
            >
              {{ $t('project.create') }}
            </button>
          </div>
        </div>
      </div>
      <BlurBG />
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { readTextFile, writeTextFile, mkdir, readDir, BaseDirectory } from '@tauri-apps/plugin-fs'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()

const projectsList = ref<string[]>([])
const projectsLoading = ref(false)
const newProjectName = ref('')

async function refreshProjects() {
  projectsLoading.value = true
  try {
    await mkdir('projects', { baseDir: BaseDirectory.AppData, recursive: true })
    const entries = await readDir('projects', { baseDir: BaseDirectory.AppData })
    projectsList.value = entries
      .filter((e) => e.name?.endsWith('.json'))
      .map((e) => e.name as string)
      .sort()
  } catch {
    projectsList.value = []
  } finally {
    projectsLoading.value = false
  }
}

async function selectProject(filename: string) {
  try {
    const content = await readTextFile(`projects/${filename}`, { baseDir: BaseDirectory.AppData })
    const config = JSON.parse(content)
    jsonStore.updateConfigFile(config)
    jsonStore.setCurrentProjectPath(filename)
    appStore.setSelectProjectActive(false)
  } catch (e) {
    console.error('Failed to load project', e)
  }
}

async function createProject() {
  const name = newProjectName.value.trim()
  if (!name) return
  const safeName = name.replace(/[^a-z0-9_\- ]/gi, '_').replace(/\s+/g, '_')
  const filename = `${safeName}.json`
  const emptyConfig = {
    settings: { theme: 'dark-cyan', customCss: '', outputSource: 'default' },
    tabList: [],
    files: [],
  }
  try {
    await mkdir('projects', { baseDir: BaseDirectory.AppData, recursive: true })
    await writeTextFile(`projects/${filename}`, JSON.stringify(emptyConfig, null, 2), {
      baseDir: BaseDirectory.AppData,
    })
    jsonStore.updateConfigFile(emptyConfig)
    jsonStore.setCurrentProjectPath(filename)
    newProjectName.value = ''
    await refreshProjects()
    appStore.setSelectProjectActive(false)
  } catch (e) {
    console.error('Failed to create project', e)
  }
}

watch(() => appStore.selectProjectActive, (val) => {
  if (val) refreshProjects()
})
</script>
