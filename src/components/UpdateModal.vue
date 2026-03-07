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

      <div v-if="releaseNotes" class="release-notes-container">
        <div class="notes-header">
          <n-icon><DocumentTextOutline /></n-icon>
          <span>{{ t('setting.update.releaseNotes') }}</span>
        </div>
        <div class="notes-content custom-scrollbar">
          <div class="markdown-body">{{ formattedReleaseNotes }}</div>
        </div>
      </div>

      <div v-if="!supportsInAppUpdate" class="platform-hint">
        <n-icon><OpenOutline /></n-icon>
        <span>{{ t('setting.update.externalUpdateHint') }}</span>
      </div>

      <div v-if="supportsInAppUpdate && isUpdating" class="progress-section">
        <div class="progress-info">
          <span class="status-text">
            {{
              updateStatus === 'installing'
                ? t('setting.update.installing')
                : t('setting.update.downloading')
            }}
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
        <div v-if="progressMessage" class="progress-message">
          {{ progressMessage }}
        </div>
      </div>

      <div v-if="supportsInAppUpdate && updateError" class="error-message">
        <n-icon><AlertCircleOutline /></n-icon>
        <span>{{ updateError }}</span>
      </div>
    </div>

    <template #action>
      <div class="modal-actions">
        <n-button
          secondary
          @click="onSkip"
          :disabled="supportsInAppUpdate && isUpdating"
          class="skip-btn"
        >
          {{ t('setting.update.skipVersion') }}
        </n-button>

        <div class="right-actions">
          <n-button @click="onCancel" :disabled="supportsInAppUpdate && isUpdating">
            {{ t('setting.update.later') }}
          </n-button>
          <n-button
            type="primary"
            :loading="supportsInAppUpdate && isUpdating"
            :disabled="actionDisabled"
            @click="onUpdate"
          >
            <template #icon>
              <n-icon>
                <OpenOutline v-if="!supportsInAppUpdate" />
                <DownloadOutline v-else-if="!isUpdating" />
                <SyncOutline v-else />
              </n-icon>
            </template>
            {{ actionLabel }}
          </n-button>
        </div>
      </div>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { useMessage } from 'naive-ui'
import {
  DownloadOutline,
  DocumentTextOutline,
  SyncOutline,
  ArrowForwardOutline,
  CalendarOutline,
  ServerOutline,
  AlertCircleOutline,
  OpenOutline,
} from '@vicons/ionicons5'
import { listen, type Event } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'

interface UpdateProgressPayload {
  status: 'checking' | 'downloading' | 'completed' | 'error' | 'installing'
  progress: number
  message: string
}

interface UpdateModalProps {
  show?: boolean
  latestVersion: string
  currentVersion: string
  downloadUrl?: string
  releasePageUrl?: string
  releaseNotes?: string
  releaseDate?: string
  fileSize?: number
  supportsInAppUpdate?: boolean
}

const props = withDefaults(defineProps<UpdateModalProps>(), {
  show: false,
  downloadUrl: '',
  releasePageUrl: '',
  releaseNotes: '',
  releaseDate: '',
  fileSize: 0,
  supportsInAppUpdate: false,
})

const emits = defineEmits<{
  'update:show': [value: boolean]
  update: []
  cancel: []
  skip: []
}>()

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
  return props.releaseNotes.replace(/^## (.+)$/gm, '### $1').replace(/\r\n/g, '\n')
})

const actionLabel = computed(() => {
  if (!props.supportsInAppUpdate) {
    return t('setting.update.openReleasePage')
  }

  if (updateStatus.value === 'installing') {
    return t('setting.update.installing')
  }

  if (isUpdating.value) {
    return t('setting.update.downloading')
  }

  return t('setting.update.updateNow')
})

const actionDisabled = computed(() => {
  if (!props.supportsInAppUpdate) {
    return !props.releasePageUrl
  }
  return isUpdating.value || updateStatus.value === 'installing'
})

const resetModalState = () => {
  isUpdating.value = false
  updateProgress.value = 0
  updateStatus.value = 'downloading'
  progressMessage.value = ''
  updateError.value = ''
}

watch(
  () => props.show,
  (newVal) => {
    localShow.value = newVal
    if (newVal) {
      resetModalState()
    }
  },
)

watch([() => localShow.value, () => props.supportsInAppUpdate], ([isOpen, supports]) => {
  // 非 Windows 不监听安装进度，避免展示无效状态。
  if (isOpen && supports) {
    void setupProgressListener()
    return
  }

  if (!isUpdating.value) {
    cleanup()
  }
})

const handleUpdateShow = (value: boolean) => {
  localShow.value = value
  emits('update:show', value)
}

const setupProgressListener = async () => {
  if (unlisten || !props.supportsInAppUpdate) return

  try {
    unlisten = await listen<UpdateProgressPayload>(
      'update-progress',
      (event: Event<UpdateProgressPayload>) => {
        const { status, progress, message: rawMessage } = event.payload
        updateProgress.value = progress
        progressMessage.value =
          status === 'installing' ? t('setting.update.installStarted') : rawMessage

        if (status === 'downloading') {
          updateStatus.value = 'downloading'
        } else if (status === 'completed') {
          updateStatus.value = 'completed'
          isUpdating.value = false
          message.success(t('notification.updateDownloaded'))
          setTimeout(() => handleUpdateShow(false), 2000)
        } else if (status === 'error') {
          updateStatus.value = 'error'
          updateError.value = rawMessage
          isUpdating.value = false
        } else if (status === 'installing') {
          updateStatus.value = 'installing'
          updateProgress.value = 100
        }
      },
    )
  } catch (error) {
    console.error('Failed to setup progress listener:', error)
  }
}

const onUpdate = () => {
  if (props.supportsInAppUpdate) {
    isUpdating.value = true
    updateError.value = ''
    updateStatus.value = 'downloading'
    progressMessage.value = t('setting.update.preparingDownload')
  }
  emits('update')
}

const onCancel = () => {
  if (props.supportsInAppUpdate && isUpdating.value) return
  handleUpdateShow(false)
  emits('cancel')
}

const onSkip = () => {
  if (props.supportsInAppUpdate && isUpdating.value) return
  handleUpdateShow(false)
  emits('skip')
}

const cleanup = () => {
  if (unlisten) {
    unlisten()
    unlisten = null
  }
}

onMounted(() => {
  localShow.value = props.show
  if (localShow.value && props.supportsInAppUpdate) {
    void setupProgressListener()
  }
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
  gap: 18px;
  padding: 4px 0;
}

.update-header {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
}

.version-badge {
  flex: 1;
  min-width: 0;
  padding: 14px 16px;
  border-radius: 14px;
  border: 1px solid var(--border-color);
  background: var(--bg-secondary);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.version-badge.new {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.16), rgba(59, 130, 246, 0.12));
  border-color: rgba(99, 102, 241, 0.24);
}

.version-badge.current {
  background: var(--bg-secondary);
}

.version-badge .label {
  font-size: 12px;
  color: var(--text-secondary);
}

.version-badge .value {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
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

.platform-hint {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 14px 16px;
  border-radius: 12px;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(99, 102, 241, 0.1));
  border: 1px solid rgba(99, 102, 241, 0.18);
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.6;
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

.markdown-body {
  white-space: pre-wrap;
}
</style>
