<template>
  <n-modal
    :show="localShow"
    :mask-closable="false"
    class="update-modal"
    preset="card"
    :title="t('notification.updateAvailable')"
    size="medium"
    :bordered="false"
    :segmented="true"
    :style="{ width: '560px', maxHeight: '80vh' }"
    @update:show="handleUpdateShow"
  >
    <n-scrollbar style="max-height: 60vh">
      <n-space vertical :size="20">
        <!-- 版本信息头部 -->
        <div class="update-header">
          <div class="update-title">
            <n-icon size="28" color="var(--primary-color)" class="update-icon">
              <download-outline />
            </n-icon>
            <div class="version-info">
              <h3>{{ t('setting.update.newVersion') }} {{ latestVersion }}</h3>
              <div class="version-meta">
                <n-tag size="small" type="info" round>
                  {{ t('setting.update.current') }}: {{ currentVersion }}
                </n-tag>
                <n-tag v-if="releaseDate" size="small" type="default" round>
                  {{ formattedDate }}
                </n-tag>
                <n-tag v-if="fileSize" size="small" type="success" round>
                  {{ formattedSize }}
                </n-tag>
              </div>
            </div>
          </div>
        </div>

        <!-- 更新说明 -->
        <div v-if="releaseNotes" class="release-notes">
          <n-divider title-placement="left">
            <n-icon><document-text-outline /></n-icon>
            {{ t('setting.update.releaseNotes') }}
          </n-divider>
          <div class="notes-content">
            <n-text class="notes-text">{{ formattedReleaseNotes }}</n-text>
          </div>
        </div>

        <!-- 确认信息 -->
        <div class="update-confirmation">
          <n-alert type="info" :show-icon="true">
            <template #header>{{ t('setting.update.updateNotice') }}</template>
            {{ t('setting.update.confirmUpdate') }}
          </n-alert>
        </div>

        <!-- 下载进度 -->
        <div v-if="isUpdating" class="progress-section">
          <n-progress
            type="line"
            :percentage="updateProgress"
            :processing="updateProgress < 100 && updateStatus !== 'error'"
            indicator-placement="inside"
            :height="28"
            :border-radius="14"
            :status="updateStatus === 'error' ? 'error' : 'info'"
          >
            <span class="progress-text">
              {{ updateStatus === 'installing' ? '安装中...' : `${updateProgress.toFixed(0)}%` }}
            </span>
          </n-progress>
          <div class="progress-message">
            <n-text :depth="2">{{ progressMessage }}</n-text>
          </div>
        </div>

        <!-- 错误信息 -->
        <div v-if="updateError" class="error-section">
          <n-alert type="error" :show-icon="true">
            <template #header>{{ t('setting.update.updateFailed') }}</template>
            {{ updateError }}
          </n-alert>
        </div>
      </n-space>
    </n-scrollbar>

    <!-- 操作按钮 -->
    <template #action>
      <n-space justify="space-between" style="width: 100%">
        <n-button
          size="medium"
          @click="onSkip"
          :disabled="isUpdating"
          class="action-button secondary"
          quaternary
        >
          {{ t('setting.update.skipVersion') }}
        </n-button>

        <n-space>
          <n-button size="medium" @click="onCancel" :disabled="isUpdating" class="action-button">
            {{ t('setting.update.later') }}
          </n-button>
          <n-button
            type="primary"
            size="medium"
            :loading="isUpdating"
            :disabled="isUpdating || updateStatus === 'installing'"
            @click="onUpdate"
            class="action-button primary"
          >
            <template #icon>
              <n-icon>
                <download-outline v-if="!isUpdating" />
                <sync-outline v-else />
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
        </n-space>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, defineProps, defineEmits, watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { useMessage } from 'naive-ui'
import { DownloadOutline, DocumentTextOutline, SyncOutline } from '@vicons/ionicons5'
import { listen } from '@tauri-apps/api/event'
import { useI18n } from 'vue-i18n'

const props = defineProps({
  show: {
    type: Boolean,
    default: false,
  },
  latestVersion: {
    type: String,
    required: true,
  },
  currentVersion: {
    type: String,
    required: true,
  },
  downloadUrl: {
    type: String,
    default: '',
  },
  releaseNotes: {
    type: String,
    default: '',
  },
  releaseDate: {
    type: String,
    default: '',
  },
  fileSize: {
    type: Number,
    default: 0,
  },
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

// 格式化文件大小
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

// 格式化发布日期
const formattedDate = computed(() => {
  if (!props.releaseDate) return ''
  try {
    const date = new Date(props.releaseDate)
    return date.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  } catch {
    return props.releaseDate
  }
})

// 格式化发布说明
const formattedReleaseNotes = computed(() => {
  if (!props.releaseNotes) return ''
  // 简单的 Markdown 转换：将 ## 转换为标题，- 转换为列表
  return (
    props.releaseNotes
      .replace(/^## (.+)$/gm, '◆ $1')
      .replace(/^- (.+)$/gm, '• $1')
      .replace(/\*\*(.+?)\*\*/g, '$1')
      .slice(0, 500) + (props.releaseNotes.length > 500 ? '...' : '')
  )
})

// 监听props.show的变化并更新本地状态
watch(
  () => props.show,
  (newVal) => {
    localShow.value = newVal
    if (newVal) {
      // 重置状态
      isUpdating.value = false
      updateProgress.value = 0
      updateStatus.value = 'downloading'
      progressMessage.value = ''
      updateError.value = ''
    }
  },
)

// 处理本地状态变化并发出事件
const handleUpdateShow = (value: boolean) => {
  localShow.value = value
  emits('update:show', value)
}

// 监听更新进度
const setupProgressListener = async () => {
  try {
    unlisten = await listen(
      'update-progress',
      (event: { payload: { status: string; progress: number; message: string } }) => {
        const { status, progress, message: msg } = event.payload

        updateProgress.value = progress
        progressMessage.value = msg

        if (status === 'downloading') {
          updateStatus.value = 'downloading'
        } else if (status === 'completed') {
          updateStatus.value = 'completed'
          isUpdating.value = false
          message.success(t('notification.updateDownloaded'))
          // 延迟关闭对话框
          setTimeout(() => {
            handleUpdateShow(false)
          }, 2000)
        } else if (status === 'error') {
          updateStatus.value = 'error'
          updateError.value = msg
          isUpdating.value = false
        } else if (status === 'installing') {
          updateStatus.value = 'installing'
          updateProgress.value = 100
        }
      },
    )
  } catch (error) {
    console.error('设置更新进度监听失败:', error)
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
    message.error(updateError.value)
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

// 清理监听器
const cleanup = () => {
  if (unlisten) {
    unlisten()
    unlisten = null
  }
}

// 监听本地状态变化来设置或清理监听器
watch(localShow, (newVal) => {
  if (newVal) {
    setupProgressListener()
  } else {
    if (!isUpdating.value) {
      cleanup()
    }
  }
})

onMounted(() => {
  localShow.value = props.show
  if (localShow.value) {
    setupProgressListener()
  }
})

onBeforeUnmount(() => {
  cleanup()
})
</script>

<style scoped>
.update-modal {
  border-radius: 16px;
  overflow: hidden;
}

.update-header {
  padding: 8px 0;
}

.update-title {
  display: flex;
  align-items: flex-start;
  gap: 16px;
}

.update-icon {
  margin-top: 4px;
  flex-shrink: 0;
}

.version-info {
  flex: 1;
}

.version-info h3 {
  margin: 0 0 8px 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.version-meta {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.release-notes {
  margin: 16px 0;
}

.notes-content {
  padding: 16px;
  background: var(--n-color-embedded);
  border-radius: 12px;
  border: 1px solid var(--n-border-color);
  max-height: 200px;
  overflow-y: auto;
}

.notes-text {
  white-space: pre-line;
  line-height: 1.6;
  color: var(--n-text-color-2);
}

.update-confirmation {
  margin: 16px 0;
}

.progress-section {
  margin: 16px 0;
}

.progress-text {
  font-weight: 500;
  color: #fff;
  font-size: 13px;
}

.progress-message {
  margin-top: 8px;
  text-align: center;
}

.error-section {
  margin: 16px 0;
}

.action-button {
  min-width: 100px;
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.25s ease;
}

.action-button.primary {
  box-shadow: 0 4px 12px rgba(24, 160, 88, 0.2);
}

.action-button.primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(24, 160, 88, 0.3);
}

.action-button.secondary {
  color: var(--n-text-color-3);
}

.action-button:hover:not(:disabled) {
  transform: translateY(-1px);
}
</style>
