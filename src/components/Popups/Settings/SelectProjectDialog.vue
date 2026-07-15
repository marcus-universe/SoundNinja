<template>
  <Transition name="overlay-fade">
    <div v-if="appStore.selectProjectActive" class="project-overlay" @click.self="closeDialog">
      <div class="project-dialog">
        <div class="project-dialog__header">
          <h2>{{ $t('project.title') }}</h2>
          <DialogCloseButton :title="$t('settings.close')" @close="closeDialog" />
        </div>

        <div class="project-dialog__body">
          <div v-if="projects.length === 0" class="project-dialog__empty">{{ $t('project.noProjects') }}</div>
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
                :title="$t('project.removeProject')"
                @click.stop="removeProject(proj)"
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
import type { ProjectInfo } from '~/utils/projects'

const { t } = useI18n()

const appStore = useAppStore()
const jsonStore = useJsonHandelingStore()
const appSettings = useAppSettingsStore()

const newProjectName = ref('')
const projects = ref<ProjectInfo[]>([])
const errorMessage = ref('')

function closeDialog() {
  errorMessage.value = ''
  newProjectName.value = ''
  appStore.setSelectProjectActive(false)
}

function parentDir(path: string) {
  const i = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  return i >= 0 ? path.slice(0, i) : path
}

async function loadProjects() {
  try {
    projects.value = await listProjects(appSettings.projectsPath)
    errorMessage.value = ''
  } catch (e) {
    console.error('Failed to load projects', e)
    errorMessage.value = String(e)
  }
}

async function selectProject(proj: ProjectInfo) {
  try {
    const stillExists = await invoke('path_exists_abs', { path: proj.dbPath })
    if (!stillExists) {
      await appSettings.removeRecentProject(proj.dbPath)
      await loadProjects()
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

async function removeProject(proj: ProjectInfo) {
  if (jsonStore.currentProjectPath === proj.dbPath) {
    errorMessage.value = t('project.deleteCurrentError')
    return
  }
  if (!window.confirm(t('project.deleteConfirm', { name: proj.name }))) return
  try {
    await deleteProjectFolder(proj.dbPath)
    await appSettings.removeRecentProject(proj.dbPath)
    errorMessage.value = ''
    await loadProjects()
  } catch (e) {
    console.error('Failed to delete project', e)
    errorMessage.value = String(e)
  }
}

async function openExisting() {
  const selected = await openDialog({
    title: 'Open Project',
    filters: [{ name: 'Sound Ninja Project', extensions: ['db'] }],
    multiple: false,
  })
  if (!selected || Array.isArray(selected)) return
  try {
    await jsonStore.openProject(selected)
    await appSettings.touchRecent(selected, projectNameFromDbPath(selected))
    await loadProjects()
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
    const safeName = safeProjectName(name)
    const existing = projects.value.some((p) => safeProjectName(p.name).toLowerCase() === safeName.toLowerCase())
    if (existing) {
      errorMessage.value = t('project.duplicateNameError')
      return
    }
    const dbPath = await createProjectFolder(appSettings.projectsPath, name)
    await jsonStore.openProject(dbPath)
    await appSettings.touchRecent(dbPath, projectNameFromDbPath(dbPath))
    await loadProjects()
    errorMessage.value = ''
    newProjectName.value = ''
    closeDialog()
  } catch (e) {
    console.error('Failed to create project', e)
    errorMessage.value = String(e)
  }
}

async function deleteProjectFolder(dbPath: string) {
  await invoke('delete_file_abs', { path: dbPath })
  await invoke('delete_dir_abs', { path: parentDir(dbPath) })
}

watch(
  () => appStore.selectProjectActive,
  (active) => {
    if (active) loadProjects()
  },
  { immediate: true }
)
</script>
