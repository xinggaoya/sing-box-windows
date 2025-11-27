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
  const latestAvailableVersion = ref('')
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

      // 如果在请求期间收到了事件更新，优先信任事件（因为它通常更新）
      // 只要有新事件到来，就认为主动查询的结果可能已经过时，特别是当涉及 api_ready 等状态变化时
      if (lastEventTime > startTime) {
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

    // 3. 移除主动轮询，完全依赖后端事件推送
    // 用户反馈：直接选用推送即可，无需主动定时查询
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
      // await refreshStatus() // 移除主动刷新，依赖事件推送
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
      // await refreshStatus() // 移除主动刷新，依赖事件推送
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
      // await refreshStatus() // 移除主动刷新，依赖事件推送
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
      // await refreshStatus() // 移除主动刷新，依赖事件推送
      return true
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '应用代理配置失败'
      return false
    }
  }

  const toggleIpVersion = async (preferIpv6: boolean) => {
    try {
      const result = await kernelService.toggleIpVersion(preferIpv6)
      if (!result.success) {
        lastError.value = result.message
        return false
      }
      return true
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '切换IP版本失败'
      return false
    }
  }

  const checkKernelInstallation = async () => {
    try {
      const version = await kernelService.getKernelVersion()
      const cleaned = normalizeKernelVersion(version)
      isKernelInstalled.value = Boolean(cleaned)
      if (cleaned) {
        status.value.version = cleaned
      }
      return isKernelInstalled.value
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '检查内核安装失败'
      return false
    }
  }

  const hasVersionInfo = () => Boolean(status.value.version)
  const getVersionString = () => normalizeKernelVersion(status.value.version || '')

  const fetchLatestKernelVersion = async () => {
    try {
      const latest = await kernelService.getLatestKernelVersion()
      latestAvailableVersion.value = normalizeKernelVersion(latest)
      return latestAvailableVersion.value
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : '获取最新内核版本失败'
      return ''
    }
  }

  const hasKernelUpdate = computed(() => {
    if (!latestAvailableVersion.value) return false
    if (!status.value.version) return true
    return compareVersion(latestAvailableVersion.value, status.value.version) > 0
  })

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
    toggleIpVersion,
    checkKernelInstallation,
    hasVersionInfo,
    getVersionString,
    fetchLatestKernelVersion,
    hasKernelUpdate,
    latestAvailableVersion,
  }
})

// 将后端返回的版本字符串进行裁剪，避免携带多余 JSON 或前缀
function normalizeKernelVersion(raw: string): string {
  const input = (raw || '').trim()
  if (!input) return ''

  try {
    const parsed = JSON.parse(input) as { version?: string }
    if (parsed?.version) {
      return trimPrefix(parsed.version)
    }
  } catch {
    // 非 JSON 格式按字符串处理
  }

  return trimPrefix(input)
}

function trimPrefix(version: string): string {
  let v = version.trim()
  if (v.toLowerCase().startsWith('sing-box')) {
    v = v.slice('sing-box'.length).trim()
  }
  return v.replace(/^v/, '')
}

// 简单的语义版本比较：>0 表示 a > b，0 表示相等，<0 表示 a < b
function compareVersion(a: string, b: string): number {
  const partsA = trimPrefix(a).split('.').map(n => parseInt(n, 10) || 0)
  const partsB = trimPrefix(b).split('.').map(n => parseInt(n, 10) || 0)
  const maxLen = Math.max(partsA.length, partsB.length)
  for (let i = 0; i < maxLen; i += 1) {
    const diff = (partsA[i] || 0) - (partsB[i] || 0)
    if (diff !== 0) return diff
  }
  return 0
}
