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

const extractKernelErrorMessage = (raw: unknown) => {
  if (typeof raw === 'string') return raw
  if (raw && typeof raw === 'object' && 'error' in raw) {
    const err = (raw as { error?: unknown }).error
    return typeof err === 'string' ? err : String(err ?? '')
  }
  return String(raw ?? '')
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

    // 监听后端 kernel-error：在 Linux/macOS 且启用了 TUN 时，如果提示需要 sudo 密码，则弹窗引导用户设置。
    const sudoStore = useSudoStore()
    const unlistenKernelError = await eventService.on(APP_EVENTS.kernelError, async (payload: unknown) => {
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
    })
    cleanupFunctions.push(unlistenKernelError)
  } catch (error) {
    console.error('应用初始化失败:', error)
  }
})

function cleanup() {
  cleanupFunctions.forEach((fn) => fn())
  cleanupFunctions.length = 0
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
