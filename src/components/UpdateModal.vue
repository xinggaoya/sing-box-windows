<template>
  <n-modal
    :show="localShow"
    :mask-closable="false"
    class="update-modal"
    preset="card"
    :title="t('notification.updateAvailable')"
    size="small"
    :bordered="false"
    :segmented="true"
    :style="{ width: '460px' }"
    @update:show="handleUpdateShow"
  >
    <n-space vertical :size="16">
      <div>
        <div class="update-title">
          <n-icon size="24" color="var(--primary-color)" class="update-icon">
            <download-outline />
          </n-icon>
          <span>{{ t('setting.update.newVersion') }} {{ latestVersion }}</span>
        </div>
        <div class="update-description">
          <p>{{ t('setting.update.confirmUpdate') }}</p>
          <p class="current-version">{{ t('setting.update.current') }}: {{ currentVersion }}</p>
        </div>
      </div>

      <n-progress
        v-if="isUpdating"
        type="line"
        :percentage="updateProgress"
        :processing="updateProgress < 100"
        indicator-placement="inside"
        :height="24"
        :border-radius="12"
      >
        <span class="progress-text">{{ updateProgress.toFixed(0) }}%</span>
      </n-progress>

      <n-space justify="end" :size="16">
        <n-button size="medium" @click="onCancel" :disabled="isUpdating" class="update-button">
          {{ t('setting.update.later') }}
        </n-button>
        <n-button
          type="primary"
          size="medium"
          :loading="isUpdating"
          :disabled="isUpdating"
          @click="onUpdate"
          class="update-button"
        >
          {{ isUpdating ? t('setting.update.downloading') : t('setting.update.updateNow') }}
        </n-button>
      </n-space>
    </n-space>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, defineProps, defineEmits, watch, onMounted, onBeforeUnmount } from 'vue'
import { useMessage } from 'naive-ui'
import { DownloadOutline } from '@vicons/ionicons5'
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
})

const emits = defineEmits(['update:show', 'update', 'cancel'])

const message = useMessage()
const { t } = useI18n()
const isUpdating = ref(false)
const updateProgress = ref(0)
const localShow = ref(false)
let unlisten: (() => void) | null = null

// 监听props.show的变化并更新本地状态
watch(
  () => props.show,
  (newVal) => {
    localShow.value = newVal
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
        const { status, progress } = event.payload

        if (status === 'downloading') {
          updateProgress.value = progress
        } else if (status === 'completed') {
          isUpdating.value = false
          message.success(t('notification.updateDownloaded'))
          // 关闭对话框
          handleUpdateShow(false)
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
    emits('update', props.downloadUrl)
  } catch (error) {
    isUpdating.value = false
    message.error(`${t('common.error')}: ${error}`)
  }
}

const onCancel = () => {
  if (isUpdating.value) return
  handleUpdateShow(false)
  emits('cancel')
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

.update-title {
  font-size: 18px;
  font-weight: 600;
  margin-bottom: 8px;
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--n-text-color-1);
}

.update-icon {
  margin-right: 4px;
}

.update-description {
  margin-top: 8px;
  color: var(--n-text-color-2);
  line-height: 1.5;
}

.current-version {
  margin-top: 4px;
  font-size: 13px;
  color: var(--n-text-color-3);
}

.progress-text {
  font-weight: 500;
  color: #fff;
}

.update-button {
  min-width: 100px;
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.25s ease;
}

.update-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: var(--shadow-light);
}
</style>
