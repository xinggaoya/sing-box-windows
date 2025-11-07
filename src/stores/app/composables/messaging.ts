import { ref } from 'vue'
import type { MessageApiInjection } from 'naive-ui'

type MessageMethod = 'success' | 'error' | 'warning' | 'info'

export function useAppMessaging() {
  const instanceRef = ref<MessageApiInjection | null>(null)

  const setMessageInstance = (instance: MessageApiInjection) => {
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
