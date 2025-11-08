import { ref } from 'vue'
import type { MessageApi } from 'naive-ui/es/message'

type MessageMethod = 'success' | 'error' | 'warning' | 'info'

export function useAppMessaging() {
  const instanceRef = ref<MessageApi | null>(null)

  const setMessageInstance = (instance: MessageApi) => {
    instanceRef.value = instance
  }

  const dispatch = (method: MessageMethod) => (content: string) => {
    instanceRef.value?.[method](content)
  }

  return {
    setMessageInstance,
    showSuccessMessage: dispatch('success'),
    showErrorMessage: dispatch('error'),
    showWarningMessage: dispatch('warning'),
    showInfoMessage: dispatch('info')
  }
}
