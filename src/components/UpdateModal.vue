<template>
  <n-modal
    :show="localShow"
    :mask-closable="false"
    class="modern-modal update-modal"
    preset="dialog"
    :title="t('notification.updateAvailable')"
    :style="{ width: '500px' }"
    @update:show="handleUpdateShow"
  >
    <div class="modal-content">
      <!-- Header Info -->
      <div class="update-header">
        <div class="version-badge new">
          <span class="label">{{ t('setting.update.newVersion') }}</span>
          <span class="value">{{ latestVersion }}</span>
        </div>
        <n-icon size="24" class="arrow-icon"><ArrowForwardOutline /></n-icon>
        <div class="version-badge current">
          <span class="label">{{ t('setting.update.current') }}</span>
          <span class="value">{{ currentVersion }}</span>
        </div>
      </div>

      <!-- Meta Info -->
      <div class="update-meta">
        <div v-if="releaseDate" class="meta-item">
          <n-icon><CalendarOutline /></n-icon>
          <span>{{ formattedDate }}</span>
        </div>
        <div v-if="fileSize" class="meta-item">
          <n-icon><ServerOutline /></n-icon>
          <span>{{ formattedSize }}</span>
        </div>
      </div>

      <!-- Release Notes -->
      <div v-if="releaseNotes" class="release-notes-container">
        <div class="notes-header">
          <n-icon><DocumentTextOutline /></n-icon>
          <span>{{ t('setting.update.releaseNotes') }}</span>
        </div>
        <div class="notes-content custom-scrollbar">
          <div class="markdown-body">{{ formattedReleaseNotes }}</div>
        </div>
      </div>

      <!-- Progress -->
      <div v-if="isUpdating" class="progress-section">
        <div class="progress-info">
          <span class="status-text">
            {{ updateStatus === 'installing' ? t('setting.update.installing') : t('setting.update.downloading') }}
          </span>
          <span class="percentage">{{ updateProgress.toFixed(0) }}%</span>
        </div>
        <n-progress
          type="line"
          :percentage="updateProgress"
          :processing="updateProgress < 100 && updateStatus !== 'error'"
          :show-indicator="false"
          :status="updateStatus === 'error' ? 'error' : 'default'"
          class="custom-progress"
        />
        <div class="progress-message" v-if="progressMessage">
          {{ progressMessage }}
        </div>
      </div>

      <!-- Error -->
      <div v-if="updateError" class="error-message">
        <n-icon><AlertCircleOutline /></n-icon>
        <span>{{ updateError }}</span>
      </div>
    </div>

    <!-- Actions -->
    <template #action>
      <div class="modal-actions">
        <n-button
          secondary
          @click="onSkip"
          :disabled="isUpdating"
          class="skip-btn"
        >
          {{ t('setting.update.skipVersion') }}
        </n-button>
        
        <div class="right-actions">
          <n-button @click="onCancel" :disabled="isUpdating">
            {{ t('setting.update.later') }}
          </n-button>
          <n-button
            type="primary"
            :loading="isUpdating"
            :disabled="isUpdating || updateStatus === 'installing'"
            @click="onUpdate"
          >
            <template #icon>
              <n-icon>
                <DownloadOutline v-if="!isUpdating" />
                <SyncOutline v-else />
              </n-icon>
            </template>
            {{
              updateStatus === 'installing'
                ? t('setting.update.installing')
                : isUpdating
                  ? t('setting.update.downloading')
                  : t('setting.update.updateNow')
            }}
          </n-button>
        </div>
      </div>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, defineProps, defineEmits, watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { useMessage } from 'naive-ui'
import {
  DownloadOutline,
  DocumentTextOutline,
  SyncOutline,
  ArrowForwardOutline,
  CalendarOutline,
  ServerOutline,
  AlertCircleOutline
} from '@vicons/ionicons5'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'

const props = defineProps({
  show: { type: Boolean, default: false },
  latestVersion: { type: String, required: true },
  currentVersion: { type: String, required: true },
  downloadUrl: { type: String, default: '' },
  releaseNotes: { type: String, default: '' },
  releaseDate: { type: String, default: '' },
  fileSize: { type: Number, default: 0 },
})

const emits = defineEmits(['update:show', 'update', 'cancel', 'skip'])

const message = useMessage()
const { t } = useI18n()
const isUpdating = ref(false)
const updateProgress = ref(0)
const updateStatus = ref<'downloading' | 'completed' | 'error' | 'installing'>('downloading')
const progressMessage = ref('')
const updateError = ref('')
const localShow = ref(false)
let unlisten: (() => void) | null = null

const formattedSize = computed(() => {
  if (!props.fileSize) return ''
  const units = ['B', 'KB', 'MB', 'GB']
  let size = props.fileSize
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex++
  }
  return `${size.toFixed(1)} ${units[unitIndex]}`
})

const formattedDate = computed(() => {
  if (!props.releaseDate) return ''
  try {
    return new Date(props.releaseDate).toLocaleDateString()
  } catch {
    return props.releaseDate
  }
})

const formattedReleaseNotes = computed(() => {
  if (!props.releaseNotes) return ''
  return props.releaseNotes
    .replace(/^## (.+)$/gm, '### $1') // Downgrade headers
    .replace(/\r\n/g, '\n')
})

watch(() => props.show, (newVal) => {
  localShow.value = newVal
  if (newVal) {
    isUpdating.value = false
    updateProgress.value = 0
    updateStatus.value = 'downloading'
    progressMessage.value = ''
    updateError.value = ''
  }
})

const handleUpdateShow = (value: boolean) => {
  localShow.value = value
  emits('update:show', value)
}

const setupProgressListener = async () => {
  try {
    unlisten = await listen('update-progress', (event: any) => {
      const { status, progress, message: msg } = event.payload
      updateProgress.value = progress
      progressMessage.value = msg

      if (status === 'downloading') updateStatus.value = 'downloading'
      else if (status === 'completed') {
        updateStatus.value = 'completed'
        isUpdating.value = false
        message.success(t('notification.updateDownloaded'))
        setTimeout(() => handleUpdateShow(false), 2000)
      } else if (status === 'error') {
        updateStatus.value = 'error'
        updateError.value = msg
        isUpdating.value = false
      } else if (status === 'installing') {
        updateStatus.value = 'installing'
        updateProgress.value = 100
      }
    })
  } catch (error) {
    console.error('Failed to setup progress listener:', error)
  }
}

const onUpdate = async () => {
  try {
    isUpdating.value = true
    updateError.value = ''
    updateStatus.value = 'downloading'
    progressMessage.value = t('setting.update.preparingDownload')
    emits('update', props.downloadUrl)
  } catch (error) {
    isUpdating.value = false
    updateError.value = `${t('common.error')}: ${error}`
  }
}

const onCancel = () => {
  if (isUpdating.value) return
  handleUpdateShow(false)
  emits('cancel')
}

const onSkip = () => {
  if (isUpdating.value) return
  handleUpdateShow(false)
  emits('skip')
}

const cleanup = () => {
  if (unlisten) {
    unlisten()
    unlisten = null
  }
}

watch(localShow, (newVal) => {
  if (newVal) setupProgressListener()
  else if (!isUpdating.value) cleanup()
})

onMounted(() => {
  localShow.value = props.show
  if (localShow.value) setupProgressListener()
})

onBeforeUnmount(() => cleanup())
</script>

<style scoped>
.update-modal {
  border-radius: 16px;
}

.modal-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.update-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding: 12px 0;
}

.version-badge {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.version-badge .label {
  font-size: 12px;
  color: var(--text-tertiary);
  text-transform: uppercase;
}

.version-badge .value {
  font-size: 18px;
  font-weight: 700;
  font-family: monospace;
}

.version-badge.new .value {
  color: var(--primary-color);
}

.version-badge.current .value {
  color: var(--text-secondary);
}

.arrow-icon {
  color: var(--text-tertiary);
  opacity: 0.5;
}

.update-meta {
  display: flex;
  justify-content: center;
  gap: 24px;
  color: var(--text-secondary);
  font-size: 13px;
}

.meta-item {
  display: flex;
  align-items: center;
  gap: 6px;
}

.release-notes-container {
  background: var(--bg-secondary);
  border-radius: 12px;
  border: 1px solid var(--border-color);
  overflow: hidden;
}

.notes-header {
  padding: 10px 16px;
  background: var(--bg-tertiary);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.notes-content {
  padding: 16px;
  max-height: 200px;
  overflow-y: auto;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
}

.progress-section {
  background: var(--bg-tertiary);
  padding: 16px;
  border-radius: 12px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 13px;
  font-weight: 500;
}

.progress-message {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-tertiary);
  text-align: center;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border-radius: 8px;
  font-size: 13px;
}

.modal-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}

.right-actions {
  display: flex;
  gap: 12px;
}

/* Markdown Styles */
.markdown-body {
  white-space: pre-wrap;
}
</style>
