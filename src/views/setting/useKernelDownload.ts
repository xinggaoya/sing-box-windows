import { ref, type Ref } from 'vue'
import { APP_EVENTS } from '@/constants/events'
import { eventService } from '@/services/event-service'
import type { KernelDownloadPayload } from '@/services/kernel-service'
import { systemService } from '@/services/system-service'

interface MessageApiLike {
  success: (content: string) => void
  error: (content: string) => void
}

interface UseKernelDownloadOptions {
  selectedVersion: Ref<string | undefined>
  message: MessageApiLike
  t: (key: string) => string
  checkKernelInstallation: () => Promise<unknown> | unknown
}

export const useKernelDownload = (options: UseKernelDownloadOptions) => {
  const loading = ref(false)
  const downloading = ref(false)
  const downloadProgress = ref(0)
  const downloadMessage = ref('')
  const downloadError = ref('')

  let downloadListener: (() => void) | null = null

  const cleanupDownloadListener = () => {
    if (downloadListener) {
      downloadListener()
      downloadListener = null
    }
  }

  const runKernelInstallationCheck = async () => {
    await Promise.resolve(options.checkKernelInstallation())
  }

  const downloadTheKernel = async () => {
    if (downloading.value) return

    let downloadCompleted = false
    loading.value = true
    downloading.value = true
    downloadProgress.value = 0
    downloadMessage.value = options.t('setting.kernel.preparingDownload')
    downloadError.value = ''

    // 新一轮下载前先移除旧监听，避免重复订阅导致进度状态冲突。
    cleanupDownloadListener()
    downloadListener = await eventService.on(APP_EVENTS.kernelDownloadProgress, (payload) => {
      const data = payload as KernelDownloadPayload
      if (typeof data.progress === 'number') {
        downloadProgress.value = Math.min(100, Math.max(0, data.progress))
      }
      if (data.message) {
        downloadMessage.value = data.message
      }

      if (data.status === 'completed') {
        downloadCompleted = true
        downloading.value = false
        loading.value = false
        options.message.success(options.t('setting.kernel.downloadSuccess'))
        void runKernelInstallationCheck()
        cleanupDownloadListener()
      } else if (data.status === 'error') {
        downloading.value = false
        loading.value = false
        downloadError.value = data.message || options.t('setting.kernel.downloadFailed')
        options.message.error(downloadError.value)
        cleanupDownloadListener()
      } else {
        downloadMessage.value ||= options.t('setting.kernel.downloadingDescription')
      }
    })

    try {
      await systemService.downloadKernel(options.selectedVersion.value)

      // 后端若未推送 completed，也做一次兜底检查。
      if (!downloadCompleted) {
        await runKernelInstallationCheck()
      }
    } catch (error) {
      console.error('下载内核失败:', error)
      downloadError.value =
        error instanceof Error ? error.message : options.t('setting.kernel.downloadFailed')
      options.message.error(downloadError.value)
      downloading.value = false
      loading.value = false
      cleanupDownloadListener()
    } finally {
      // 理论上应由事件驱动结束态，这里只做兜底避免按钮卡死。
      if (downloading.value) {
        loading.value = false
        downloading.value = false
      }
    }
  }

  return {
    loading,
    downloading,
    downloadProgress,
    downloadMessage,
    downloadError,
    downloadTheKernel,
    cleanupDownloadListener,
  }
}
