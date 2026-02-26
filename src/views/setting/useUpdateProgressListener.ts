import { APP_EVENTS } from '@/constants/events'
import { eventService } from '@/services/event-service'

interface MessageApiLike {
  success: (content: string) => void
  error: (content: string) => void
}

interface UpdateStoreLike {
  updateState: {
    status?: string
    progress?: number
  }
  updateProgress: (status: string, progress: number, message: string) => void
}

interface UseUpdateProgressListenerOptions {
  message: MessageApiLike
  updateStore: UpdateStoreLike
  t: (key: string) => string
}

export const useUpdateProgressListener = (options: UseUpdateProgressListenerOptions) => {
  let updateProgressListener: (() => void) | null = null

  const setupUpdateProgressListener = async () => {
    try {
      updateProgressListener = await eventService.on(APP_EVENTS.updateProgress, (payload) => {
        const data = payload as { status?: string; progress?: number; message?: string }
        const progress =
          typeof data.progress === 'number'
            ? data.progress
            : (options.updateStore.updateState.progress ?? 0)
        const status = data.status || options.updateStore.updateState.status || 'idle'
        const rawMessage = data.message || ''
        const localizedMessage =
          status === 'installing' ? options.t('setting.update.installStarted') : rawMessage

        options.updateStore.updateProgress(status, progress, localizedMessage)

        if (status === 'completed') {
          options.message.success(options.t('notification.updateDownloaded'))
        } else if (status === 'error') {
          options.message.error(localizedMessage || options.t('setting.update.updateFailed'))
        }
      })
    } catch (error) {
      console.error('监听更新进度失败:', error)
    }
  }

  const cleanupUpdateProgressListener = () => {
    if (updateProgressListener) {
      updateProgressListener()
      updateProgressListener = null
    }
  }

  return {
    setupUpdateProgressListener,
    cleanupUpdateProgressListener,
  }
}
