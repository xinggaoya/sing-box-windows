import { defineStore } from 'pinia'
import { ref } from 'vue'

// 仅负责“弹窗状态 + Promise 协调”，不直接做 i18n/message，避免在非组件上下文使用。
export const useSudoStore = defineStore('sudo', () => {
  const visible = ref(false)
  const password = ref('')
  const saving = ref(false)
  const errorCode = ref<'' | 'empty' | 'invalid' | 'failed'>('')
  const errorDetail = ref('')

  let resolver: ((ok: boolean) => void) | null = null

  const requestPassword = () => {
    visible.value = true
    password.value = ''
    saving.value = false
    errorCode.value = ''
    errorDetail.value = ''

    return new Promise<boolean>((resolve) => {
      resolver = resolve
    })
  }

  const close = (ok: boolean) => {
    visible.value = false
    password.value = ''
    saving.value = false
    errorCode.value = ''
    errorDetail.value = ''

    if (resolver) resolver(ok)
    resolver = null
  }

  const submit = async () => {
    const trimmed = password.value.trim()
    if (!trimmed) {
      errorCode.value = 'empty'
      return false
    }

    saving.value = true
    errorCode.value = ''
    errorDetail.value = ''
    try {
      const { sudoService } = await import('@/services/sudo-service')
      await sudoService.setPassword(trimmed)
      close(true)
      return true
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error || '')
      if (message.includes('SUDO_PASSWORD_INVALID')) {
        errorCode.value = 'invalid'
        return false
      }
      errorCode.value = 'failed'
      errorDetail.value = message
      return false
    } finally {
      saving.value = false
    }
  }

  return {
    visible,
    password,
    saving,
    errorCode,
    errorDetail,
    requestPassword,
    close,
    submit,
  }
})

