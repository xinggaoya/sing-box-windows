import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { kernelService, type KernelStatus } from '@/services/kernel-service'
import { useAppStore, type ProxyMode } from '../app/AppStore'

const DEFAULT_STATUS: KernelStatus = {
  process_running: false,
  api_ready: false,
  websocket_ready: false,
  uptime_ms: 0,
  version: '',
  error: undefined,
}

export const useKernelStore = defineStore('kernel', () => {
  const appStore = useAppStore()
  const status = ref<KernelStatus>({ ...DEFAULT_STATUS })
  const lastError = ref('')
  const isLoading = ref(false)
  const isKernelInstalled = ref(false)
  let statusUnlisten: (() => void) | null = null
  let lastEventTime = 0

  const applyStatus = (next: KernelStatus) => {
    status.value = next
    appStore.setRunningState(next.process_running)
    if (next.version) {
      isKernelInstalled.value = true
    }
    if (next.error) {
      lastError.value = next.error
    }
  }

  const refreshStatus = async () => {
    const startTime = Date.now()
    try {
      const latest = await kernelService.getKernelStatus()

      // 如果在请求期间收到了事件更新，且状态不一致，优先信任事件（因为它通常更新）
      if (lastEventTime > startTime && latest.process_running !== status.value.process_running) {
        console.log('⚠️ 忽略过期的状态刷新，因为已收到更新的事件')
        return status.value
      }

      applyStatus(latest)
      return latest
    } catch (error) {
      const message = error instanceof Error ? error.message : '获取内核状态失败'
      lastError.value = message
      throw error
    }
  }

  const initializeStore = async () => {
    // 1. 先设置监听器，防止漏掉启动时的事件
    if (!statusUnlisten) {
      statusUnlisten = await kernelService.onKernelStatusChange((nextStatus) => {
        lastEventTime = Date.now()
        applyStatus(nextStatus)
      })
    }

    // 2. 获取当前状态
    await refreshStatus()

    // 3. 如果当前状态是停止，但可能正在自动启动中，进行短时间的轮询检查
    if (!status.value.process_running) {
      let checkCount = 0
      const maxChecks = 5
      const intervalId = setInterval(async () => {
        checkCount++
        if (checkCount > maxChecks || status.value.process_running) {
          clearInterval(intervalId)
          return
        }
        await refreshStatus()
      }, 1000)
    }
  }

  const restartKernel = async () => {
    if (isLoading.value) return false
    isLoading.value = true
    try {
      const result = await kernelService.restartKernel()
      if (!result.success) {
        lastError.value = result.message
        return false
      }
      await refreshStatus()
      return true
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '内核重启失败'
      return false
    } finally {
      isLoading.value = false
    }
  }

  const stopKernel = async (options?: { force?: boolean }) => {
    try {
      const result = await kernelService.stopKernel({ force: options?.force ?? false })
      if (!result.success) {
        lastError.value = result.message
        return false
      }
      await refreshStatus()
      return true
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '内核停止失败'
      return false
    }
  }

  // 后台快速停止内核，不阻塞前端退出流程
  const stopKernelFast = async () => {
    try {
      const result = await kernelService.stopKernelFast()
      if (!result.success) {
        lastError.value = result.message
      }
      return result.success
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '内核停止失败'
      return false
    }
  }

  // 强制停止内核并退出应用（后端处理退出）
  const forceStopAndExit = async () => {
    try {
      const result = await kernelService.forceStopAndExit()
      if (!result.success) {
        lastError.value = result.message
      }
      return result.success
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '退出时停止内核失败'
      return false
    }
  }

  const switchProxyMode = async (mode: ProxyMode) => {
    try {
      const result = await kernelService.switchProxyMode(mode)
      if (!result.success) {
        lastError.value = result.message
        return false
      }
      await refreshStatus()
      return true
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '切换代理模式失败'
      return false
    }
  }

  const applyProxySettings = async () => {
    try {
      const result = await kernelService.applyProxySettings()
      if (!result.success) {
        lastError.value = result.message
        return false
      }
      await refreshStatus()
      return true
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '应用代理配置失败'
      return false
    }
  }

  const checkKernelInstallation = async () => {
    try {
      const version = await kernelService.getKernelVersion()
      isKernelInstalled.value = Boolean(version)
      if (version) {
        status.value.version = version
      }
      return isKernelInstalled.value
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '检查内核安装失败'
      return false
    }
  }

  const hasVersionInfo = () => Boolean(status.value.version)
  const getVersionString = () => status.value.version || ''

  const isRunning = computed(() => status.value.process_running)
  const isReady = computed(
    () => status.value.process_running && status.value.api_ready && status.value.websocket_ready,
  )
  const isStarting = computed(() => isLoading.value && !isRunning.value)
  const isStopping = computed(() => isLoading.value && isRunning.value)
  const uptime = computed(() => {
    const ms = status.value.uptime_ms || 0
    const seconds = Math.floor(ms / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)

    if (hours > 0) {
      return `${hours}小时${minutes % 60}分钟`
    }
    if (minutes > 0) {
      return `${minutes}分钟${seconds % 60}秒`
    }
    return `${seconds}秒`
  })

  return {
    status,
    isLoading,
    lastError,
    isKernelInstalled,
    isRunning,
    isReady,
    isStarting,
    isStopping,
    uptime,
    initializeStore,
    refreshStatus,
    restartKernel,
    stopKernel,
    stopKernelFast,
    forceStopAndExit,
    switchProxyMode,
    applyProxySettings,
    checkKernelInstallation,
    hasVersionInfo,
    getVersionString,
  }
})
