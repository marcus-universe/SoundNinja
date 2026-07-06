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
              :key="proj.dbPath"
              class="project-list__item"
              :class="{ active: jsonStore.currentProjectPath === proj.dbPath }"
              @click="selectProject(proj)"
            >
              <span class="project-list__name">{{ proj.name }}</span>
              <span v-if="jsonStore.currentProjectPath === proj.dbPath" class="project-list__check">✓</span>
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
    </div>
  </Transition>
</template>

<script setup lang="ts">
import type { ProjectInfo } from '~/utils/projects'

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const projectsList = ref<ProjectInfo[]>([])
const projectsLoading = ref(false)
const newProjectName = ref('')

async function refreshProjects() {
  projectsLoading.value = true
  try {
    projectsList.value = await listProjects(appSettings.projectsPath)
  } catch {
    projectsList.value = []
  } finally {
    projectsLoading.value = false
  }
}

async function selectProject(proj: ProjectInfo) {
  try {
    await jsonStore.openProject(proj.dbPath)
    await appSettings.setLastProject(proj.dbPath)
    appStore.setSelectProjectActive(false)
  } catch (e) {
    console.error('Failed to load project', e)
  }
}

async function createProject() {
  const name = newProjectName.value.trim()
  if (!name) return
  try {
    const dbPath = await createProjectFolder(appSettings.projectsPath, name)
    await jsonStore.openProject(dbPath)
    await appSettings.setLastProject(dbPath)
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
