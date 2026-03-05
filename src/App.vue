<template>
  <n-config-provider :theme="configProviderTheme" :theme-overrides="themeOverrides">
    <n-global-style />
    <n-dialog-provider>
      <n-modal-provider>
        <n-notification-provider>
          <n-message-provider>
            <!-- 消息消费组件 -->
            <MessageConsumer />

            <!-- 主路由视图 -->
            <div class="app-container">
              <router-view />
            </div>

            <!-- Linux/macOS：sudo 密码输入弹窗（全局，可从托盘/自动启动流程唤起） -->
            <SudoPasswordModal />

            <!-- 更新通知组件 -->
            <UpdateNotification />
          </n-message-provider>
        </n-notification-provider>
      </n-modal-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, defineComponent, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'
import { useMessage } from 'naive-ui'

import {
  useThemeStore,
  useAppStore,
  useLocaleStore,
  useWindowStore,
  useTrayStore,
  useKernelStore,
  useUpdateStore,
  useSubStore,
  useTrafficStore,
  useConnectionStore,
  useLogStore,
} from '@/stores'

import UpdateNotification from '@/components/UpdateNotification.vue'
import SudoPasswordModal from '@/components/system/SudoPasswordModal.vue'
import { useAppBootstrap } from '@/boot/useAppBootstrap'
import { eventService } from '@/services/event-service'
import { APP_EVENTS } from '@/constants/events'
import { systemService } from '@/services/system-service'
import { useSudoStore } from '@/stores'
import { kernelService } from '@/services/kernel-service'
import type { KernelFailurePayload, KernelOperationFailedPayload } from '@/types/events'

const MessageConsumer = defineComponent({
  name: 'MessageConsumer',
  setup() {
    const message = useMessage()

    onMounted(() => {
      mitt.emit('message-instance-ready', message)
    })

    return () => null
  },
})

const router = useRouter()
const { locale, t } = useI18n()

const themeStore = useThemeStore()
const appStore = useAppStore()
const localeStore = useLocaleStore()
const windowStore = useWindowStore()
const subStore = useSubStore()
const kernelStore = useKernelStore()
const updateStore = useUpdateStore()
const trafficStore = useTrafficStore()
const connectionStore = useConnectionStore()
const logStore = useLogStore()
const configProviderTheme = computed(() => themeStore.naiveTheme)
const themeOverrides = computed(() => themeStore.themeOverrides)

const cleanupFunctions: (() => void)[] = []
let sudoPromptRunning = false
let eventServiceDestroyed = false
const FAILURE_DEDUP_WINDOW_MS = 10_000
const recentKernelFailureMap = new Map<string, number>()

interface NormalizedKernelFailure {
  code: string
  userMessage: string
  details: string
  source: string
  recoverable: boolean
}

const extractKernelErrorMessage = (raw: unknown) => {
  if (typeof raw === 'string') return raw
  if (raw && typeof raw === 'object' && 'message' in raw) {
    const msg = (raw as { message?: unknown }).message
    if (typeof msg === 'string') return msg
  }
  if (raw && typeof raw === 'object' && 'error' in raw) {
    const err = (raw as { error?: unknown }).error
    return typeof err === 'string' ? err : String(err ?? '')
  }
  return String(raw ?? '')
}

const shouldNotifyKernelFailure = (key: string) => {
  const now = Date.now()
  const lastShown = recentKernelFailureMap.get(key) ?? 0
  if (now - lastShown < FAILURE_DEDUP_WINDOW_MS) {
    return false
  }

  recentKernelFailureMap.set(key, now)
  if (recentKernelFailureMap.size > 200) {
    // 轻量清理老记录，避免长期运行内存增长
    recentKernelFailureMap.forEach((timestamp, id) => {
      if (now - timestamp > FAILURE_DEDUP_WINDOW_MS * 3) {
        recentKernelFailureMap.delete(id)
      }
    })
  }

  return true
}

const mapKernelFailureCodeToMessage = (code: string, fallback: string) => {
  switch (code) {
    case 'KERNEL_CONFIG_INVALID':
      return '内核配置无效，请检查配置后重试'
    case 'KERNEL_BINARY_MISSING':
      return '未检测到内核文件，请先安装内核'
    case 'KERNEL_START_UNSTABLE':
      return '内核启动后快速退出，请检查配置或端口占用'
    case 'KERNEL_GUARD_RESTART_FAILED':
      return '内核异常停止且自动重启失败'
    case 'KERNEL_GUARD_SELF_HEAL_FAILED':
      return '内核自愈重启失败'
    case 'KERNEL_STOP_FAILED':
      return '内核停止失败'
    case 'KERNEL_AUTO_MANAGE_FAILED':
      return '内核自动管理失败'
    case 'KERNEL_CONFLICT_DETECTED':
      return '检测到旧内核正在运行，正在尝试强制停止后继续'
    case 'KERNEL_CONFLICT_FORCE_STOP_FAILED':
      return '旧内核进程强制停止失败，请手动结束进程后重试'
    default:
      return fallback
  }
}

const normalizeKernelFailurePayload = (payload: KernelFailurePayload | unknown): NormalizedKernelFailure => {
  const typed = payload && typeof payload === 'object' ? (payload as KernelFailurePayload) : {}
  const code = typed.code || 'KERNEL_RUNTIME_ERROR'
  const rawMessage = extractKernelErrorMessage(payload)
  const details = typed.details || rawMessage
  const baseMessage = typed.message || rawMessage || '内核运行异常'
  const userMessage = mapKernelFailureCodeToMessage(code, baseMessage)
  const source = typed.source || 'kernel'
  const recoverable = typed.recoverable === true

  return { code, userMessage, details, source, recoverable }
}

const normalizeKernelOperationFailedPayload = (
  payload: KernelOperationFailedPayload | unknown,
): NormalizedKernelFailure => {
  const typed = payload && typeof payload === 'object' ? (payload as KernelOperationFailedPayload) : {}
  const details = extractKernelErrorMessage(payload) || '未知错误'
  const operation = typed.operation || 'kernel.operation'
  const userMessage = `内核操作失败：${operation}`
  return {
    code: 'KERNEL_OPERATION_FAILED',
    userMessage,
    details,
    source: operation,
    recoverable: false,
  }
}

const notifyKernelFailure = (failure: NormalizedKernelFailure) => {
  const dedupKey = `${failure.code}|${failure.userMessage}`
  if (!shouldNotifyKernelFailure(dedupKey)) return

  if (failure.recoverable) {
    appStore.showWarningMessage?.(failure.userMessage)
  } else {
    appStore.showErrorMessage?.(failure.userMessage)
  }
  if (failure.details && failure.details !== failure.userMessage) {
    appStore.showInfoMessage?.(`详情：${failure.details}`)
  }
}

const parseSudoCode = (raw: unknown) => {
  const msg = extractKernelErrorMessage(raw)
  if (msg.includes('SUDO_PASSWORD_REQUIRED')) return 'required'
  if (msg.includes('SUDO_PASSWORD_INVALID')) return 'invalid'
  return null
}

const handleBeforeUnload = () => {
  cleanup()
}

onMounted(async () => {
  window.addEventListener('beforeunload', handleBeforeUnload)
  cleanupFunctions.push(() => {
    window.removeEventListener('beforeunload', handleBeforeUnload)
  })

  try {
    const { initialize, cleanup: cleanupBootstrap } = useAppBootstrap({
      router,
      localeRef: locale,
      stores: {
        themeStore,
        appStore,
        localeStore,
        windowStore,
        subStore,
        kernelStore,
        updateStore,
        trafficStore,
        connectionStore,
        logStore,
        trayStore: useTrayStore(),
      },
    })

    await initialize()
    cleanupFunctions.push(() => cleanupBootstrap())

    // 全局监听 kernel-error：所有页面统一提示失败信息，并保留 Linux/macOS 的 sudo 提示流程。
    const sudoStore = useSudoStore()
    const unlistenKernelError = await eventService.on(
      APP_EVENTS.kernelError,
      async (payload: KernelFailurePayload) => {
        const failure = normalizeKernelFailurePayload(payload)
        kernelStore.lastError = failure.details || failure.userMessage
        notifyKernelFailure(failure)

        if (sudoPromptRunning) return
        const code = parseSudoCode(payload)
        if (!code) return

        // 仅在 TUN 开启时处理（避免手动模式下误触发）
        if (!appStore.tunEnabled) return

        try {
          sudoPromptRunning = true
          const platform = await systemService.getPlatformInfo().catch(() => 'unknown')
          if (platform !== 'linux' && platform !== 'macos') return

          // 提示原因：密码缺失/失效
          appStore.showWarningMessage?.(
            code === 'invalid' ? t('home.sudoPassword.invalid') : t('home.sudoPassword.required')
          )

          const ok = await sudoStore.requestPassword()
          if (!ok) return

          // 用户保存后，重新触发一次自动管理（尊重用户当前配置）
          await kernelService.autoManageKernel({ forceRestart: true })
        } finally {
          sudoPromptRunning = false
        }
      },
    )
    cleanupFunctions.push(unlistenKernelError)

    const unlistenKernelOperationFailed = await eventService.on(
      APP_EVENTS.kernelOperationFailed,
      (payload: KernelOperationFailedPayload) => {
        const failure = normalizeKernelOperationFailedPayload(payload)
        kernelStore.lastError = failure.details || failure.userMessage
        notifyKernelFailure(failure)
      },
    )
    cleanupFunctions.push(unlistenKernelOperationFailed)

    const unlistenUpgradeRefreshFailed = await eventService.on(
      APP_EVENTS.upgradeSubscriptionRefreshFailed,
      (payload: unknown) => {
        const messageText =
          payload && typeof payload === 'object' && 'message' in payload
            ? String((payload as { message?: unknown }).message ?? '')
            : ''
        const fallback =
          '应用升级后自动刷新当前订阅失败，请在订阅页手动点击“立即更新配置”。'
        appStore.showWarningMessage?.(messageText || fallback)
      },
    )
    cleanupFunctions.push(unlistenUpgradeRefreshFailed)
  } catch (error) {
    console.error('应用初始化失败:', error)
  }
})

function cleanup() {
  cleanupFunctions.forEach((fn) => fn())
  cleanupFunctions.length = 0

  // 事件服务仅需销毁一次，避免 beforeunload 与组件卸载双触发时重复执行。
  if (!eventServiceDestroyed) {
    eventServiceDestroyed = true
    eventService.destroy()
  }
}

onBeforeUnmount(() => {
  cleanup()
})

</script>

<style>
/* 应用容器基础样式 */
#app {
  height: 100vh;
}

.app-container {
  height: 100%;
  width: 100%;
}
</style>
