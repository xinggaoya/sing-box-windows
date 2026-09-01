import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { APP_EVENTS } from '@/constants/events'
import { kernelService, type KernelStatus } from '@/services/kernel-service'
import { useAppStore, type ProxyMode } from '../app/AppStore'
import { eventService } from '@/services/event-service'
import type { KernelFailurePayload, KernelReadinessSnapshot, StartupDiagnosis } from '@/types/events'

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
  const startupDiagnosis = ref<StartupDiagnosis | null>(null)
  const readiness = ref<KernelReadinessSnapshot>({
    config_validated: null,
    process_spawned: null,
    process_alive: false,
    api_ready: false,
    relay_ready: false,
  })
  const lastError = ref('')
  const isLoading = ref(false)
  const isKernelInstalled = ref(false)
  const stateVersion = ref(0)
  const lastOperationId = ref('')
  const healthStatus = ref<{ healthy: boolean; issues: string[]; lastChecked?: number }>({
    healthy: true,
    issues: [],
    lastChecked: undefined,
  })

  const latestAvailableVersion = ref('')
  const availableVersions = ref<string[]>([])
  let statusUnlisten: (() => void) | null = null
  let errorUnlisten: (() => void) | null = null
  let healthUnlisten: (() => void) | null = null
  let statusPollTimer: ReturnType<typeof setInterval> | null = null
  let lastEventTime = 0

  const applyStatus = (next: KernelStatus) => {
    // 防御:如果后端返回的 error 标识 "Rust binary 未更新"(旧 binary 残缺实现),
    // 不要用这个残缺响应覆盖当前 status,避免前端闪退到 stopped。
    if (next.error && next.error.includes('Rust binary 未更新')) {
      console.warn('[kernel-store] 跳过旧 binary 响应(保留当前 status):', next)
      return
    }
    if (typeof next.state_version === 'number') {
      // 避免旧状态覆盖新状态（事件乱序或请求返回较慢时尤其重要）。
      if (next.state_version < stateVersion.value) {
        return
      }
      stateVersion.value = next.state_version
    }
    if (next.op_id) {
      lastOperationId.value = next.op_id
    }

    status.value = { ...status.value, ...next }
    if (next.readiness) {
      readiness.value = { ...readiness.value, ...next.readiness }
    }
    if (next.startup_diagnosis !== undefined) {
      startupDiagnosis.value = next.startup_diagnosis || null
    }
    appStore.setRunningState(next.process_running)
    if (next.version) {
      isKernelInstalled.value = true
    }
    if (startupDiagnosis.value?.message) {
      lastError.value = startupDiagnosis.value.message
    } else if (next.startup_diagnosis === null || (next.process_running && !next.error)) {
      lastError.value = ''
    } else if (next.error) {
      lastError.value = next.error
    }
  }

  const handleKernelFailureEvent = (payload: KernelFailurePayload) => {
    if (payload.startup_diagnosis) {
      startupDiagnosis.value = payload.startup_diagnosis
      lastError.value = payload.startup_diagnosis.message
    } else if (payload.message || payload.error) {
      lastError.value = String(payload.message || payload.error || '')
    }
  }

  const refreshStatus = async () => {
    const startTime = Date.now()
    try {
      const latest = await kernelService.getKernelStatus()

      // 如果在请求期间收到了事件更新，优先信任事件（因为它通常更新）
      // 只要有新事件到来，就认为主动查询的结果可能已经过时，特别是当涉及 api_ready 等状态变化时
      if (lastEventTime > startTime) {
        return status.value
      }
      if (typeof latest.state_version === 'number' && latest.state_version < stateVersion.value) {
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
    if (!errorUnlisten) {
      errorUnlisten = await kernelService.onKernelError((payload) => {
        lastEventTime = Date.now()
        handleKernelFailureEvent(payload)
      })
    }
    if (!healthUnlisten) {
      healthUnlisten = await eventService.on(APP_EVENTS.kernelHealth, (payload) => {
        healthStatus.value = {
          healthy: !!payload.healthy,
          issues: payload.issues || [],
          lastChecked: Date.now(),
        }
      })
    }

    // 2. 先走"快照"初始化，再通过事件增量更新
    const snapshot = await kernelService.getKernelSnapshot()
    applyStatus(snapshot)

    // 3. 定时轮询 status:之前完全依赖后端 kernel-status-changed 事件推送,
    //    但该事件只在 kernel_start/stop/restart 状态点 emit,内核运行期间不 emit,
    //    导致 status 不会实时反映最新状态(用户反馈"内核状态不刷新")。
    //    改为 3 秒 polling 一次,事件 + polling 双管齐下。
    if (statusPollTimer == null) {
      statusPollTimer = setInterval(() => {
        // 窗口不可见时跳过,减少无意义请求
        if (typeof document !== 'undefined' && document.hidden) return
        refreshStatus().catch(() => {
          // 静默失败:polling 失败不应该刷错误,下次还会再试
        })
      }, 3000)
    }

    // 4. Eager load available versions for better UX (dropdown ready on open)
    if (availableVersions.value.length === 0) {
      fetchKernelReleases().catch((err) => {
        console.warn('Failed to eager load kernel versions:', err)
      })
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

  const applyProxySettings = async (options?: {
    system_proxy_enabled?: boolean
    tun_enabled?: boolean
  }) => {
    try {
      const result = await kernelService.applyProxySettings(options)
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
      lastError.value =
        error instanceof Error ? error.message : 'Failed to fetch latest kernel version'
      return ''
    }
  }

  const fetchKernelReleases = async () => {
    try {
      const versions = await kernelService.getKernelReleases()
      availableVersions.value = versions.map((v) => normalizeKernelVersion(v))
      return availableVersions.value
    } catch (error) {
      lastError.value =
        error instanceof Error ? error.message : 'Failed to fetch kernel version list'
      return []
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
  const startupDiagnosisSummary = computed(
    () => startupDiagnosis.value?.message || startupDiagnosis.value?.detail || '',
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

  const cleanupStore = () => {
    if (statusPollTimer != null) {
      clearInterval(statusPollTimer)
      statusPollTimer = null
    }
    if (statusUnlisten) {
      statusUnlisten()
      statusUnlisten = null
    }
    if (errorUnlisten) {
      errorUnlisten()
      errorUnlisten = null
    }
    if (healthUnlisten) {
      healthUnlisten()
      healthUnlisten = null
    }
  }

  return {
    status,
    startupDiagnosis,
    readiness,
    isLoading,
    lastError,
    isKernelInstalled,
    isRunning,
    isReady,
    startupDiagnosisSummary,
    isStarting,
    isStopping,
    uptime,
    healthStatus,
    initializeStore,
    cleanupStore,
    handleKernelFailureEvent,
    refreshStatus,
    restartKernel,
    stopKernel,
    switchProxyMode,
    applyProxySettings,
    toggleIpVersion,
    checkKernelInstallation,
    hasVersionInfo,
    getVersionString,
    fetchLatestKernelVersion,
    fetchKernelReleases,
    hasKernelUpdate,
    latestAvailableVersion,
    availableVersions,
    stateVersion,
    lastOperationId,
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
  const partsA = trimPrefix(a)
    .split('.')
    .map((n) => parseInt(n, 10) || 0)
  const partsB = trimPrefix(b)
    .split('.')
    .map((n) => parseInt(n, 10) || 0)
  const maxLen = Math.max(partsA.length, partsB.length)
  for (let i = 0; i < maxLen; i += 1) {
    const diff = (partsA[i] || 0) - (partsB[i] || 0)
    if (diff !== 0) return diff
  }
  return 0
}
