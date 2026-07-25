<template>
  <Transition name="overlay-fade">
    <div v-if="appStore.selectProjectActive" class="project-overlay" @click.self="closeDialog">
      <div class="project-dialog">
        <div class="project-dialog__header">
          <h2>{{ $t('project.title') }}</h2>
          <DialogCloseButton :title="$t('settings.close')" @close="closeDialog" />
        </div>

        <div class="project-dialog__body">
          <div v-if="projects.length === 0" class="project-dialog__empty">{{ $t('project.noRecent') }}</div>
          <ul v-else class="project-list">
            <li
              v-for="proj in projects"
              :key="proj.dbPath"
              class="project-list__item"
              :class="{ active: jsonStore.currentProjectPath === proj.dbPath }"
              @click="selectProject(proj)"
            >
              <span class="project-list__name">{{ proj.name }}</span>
              <span v-if="jsonStore.currentProjectPath === proj.dbPath" class="project-list__check">✓</span>
              <button
                class="project-list__remove"
                :title="$t('project.removeRecent')"
                @click.stop="removeFromRecent(proj)"
              >
                <Icons icon="exit" custom-class="dialog-close project-list__remove-icon" />
              </button>
            </li>
          </ul>
        </div>

        <p v-if="errorMessage" class="project-dialog__error">{{ errorMessage }}</p>

        <div class="project-dialog__footer">
          <div class="new-project-row">
            <button class="settings-btn" @click="openExisting">{{ $t('project.openExisting') }}</button>
          </div>
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
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import {
  PROJECT_FILE_FILTER,
  createProjectFolder,
  projectDbPath,
  projectNameFromDbPath,
  type ProjectInfo,
} from '~/utils/projects'

const { t } = useI18n()

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const newProjectName = ref('')
const errorMessage = ref('')

const projects = computed(() => appSettings.recentProjects ?? [])

function closeDialog() {
  errorMessage.value = ''
  newProjectName.value = ''
  appStore.setSelectProjectActive(false)
}

async function refreshList() {
  try {
    await appSettings.refreshRecents()
    errorMessage.value = ''
  } catch (e) {
    console.error('Failed to load recent projects', e)
    errorMessage.value = String(e)
  }
}

async function selectProject(proj: ProjectInfo) {
  try {
    const stillExists = await invoke('path_exists_abs', { path: proj.dbPath })
    if (!stillExists) {
      await appSettings.removeRecentProject(proj.dbPath)
      return
    }
    await jsonStore.openProject(proj.dbPath)
    await appSettings.touchRecent(proj.dbPath, proj.name)
    closeDialog()
  } catch (e) {
    console.error('Failed to load project', e)
    errorMessage.value = String(e)
  }
}

async function removeFromRecent(proj: ProjectInfo) {
  try {
    await appSettings.removeRecentProject(proj.dbPath)
    errorMessage.value = ''
  } catch (e) {
    console.error('Failed to remove recent project', e)
    errorMessage.value = String(e)
  }
}

async function openExisting() {
  const selected = await openDialog({
    title: 'Open Project',
    filters: [PROJECT_FILE_FILTER],
    multiple: false,
  })
  if (!selected || Array.isArray(selected)) return
  try {
    await jsonStore.openProject(selected)
    await appSettings.touchRecent(selected, projectNameFromDbPath(selected))
    closeDialog()
  } catch (e) {
    console.error('Failed to open project', e)
    errorMessage.value = String(e)
  }
}

async function createProject() {
  const name = newProjectName.value.trim()
  if (!name) return
  try {
    const dbPath = projectDbPath(appSettings.projectsPath, name)
    const alreadyExists = await invoke<boolean>('path_exists_abs', { path: dbPath })
    if (alreadyExists) {
      errorMessage.value = t('project.duplicateNameError')
      return
    }
    const createdPath = await createProjectFolder(appSettings.projectsPath, name)
    await jsonStore.openProject(createdPath)
    await appSettings.touchRecent(createdPath, projectNameFromDbPath(createdPath))
    errorMessage.value = ''
    newProjectName.value = ''
    closeDialog()
  } catch (e) {
    console.error('Failed to create project', e)
    errorMessage.value = String(e)
  }
}

watch(
  () => appStore.selectProjectActive,
  (active) => {
    if (active) refreshList()
  },
  { immediate: true }
)
</script>
