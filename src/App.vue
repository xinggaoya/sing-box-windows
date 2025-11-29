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

            <!-- 更新通知组件 -->
            <UpdateNotification />
          </n-message-provider>
        </n-notification-provider>
      </n-modal-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, defineComponent, onMounted, onBeforeUnmount, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'
import { useMessage } from 'naive-ui'
import type { Router } from 'vue-router'

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

// 导入组件
import UpdateNotification from '@/components/UpdateNotification.vue'

// 消息消费组件
const MessageConsumer = defineComponent({
  name: 'MessageConsumer',
  setup() {
    const message = useMessage()

    // 注册消息实例到Store管理器（通过事件）
    onMounted(() => {
      mitt.emit('message-instance-ready', message)
    })

    return () => null
  },
})

const router = useRouter()
const { locale } = useI18n()

// Instantiate stores directly
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

// 生产环境下禁用右键菜单

// 清理函数数组
const cleanupFunctions: (() => void)[] = []

const handleBeforeUnload = () => {
  cleanup()
}

// 更新检查定时器ID
let updateIntervalId: number | undefined

// 自动检查更新
async function handleAutoUpdateCheck() {
  if (updateStore.autoCheckUpdate) {
    console.log('🚀 自动检查更新已启用，将在后台执行...')
    // 立即执行一次静默检查
    const updateResult = await updateStore.checkUpdate(true)
    if (updateResult && updateResult.has_update) {
      mitt.emit('update-available', updateResult)
    }

    // 设置定时检查，每4小时一次
    updateIntervalId = window.setInterval(async () => {
      console.log('⏰ 定时任务：执行后台更新检查...')
      const periodicResult = await updateStore.checkUpdate(true)
      if (periodicResult && periodicResult.has_update) {
        mitt.emit('update-available', periodicResult)
      }
    }, 4 * 60 * 60 * 1000) // 4 hours

    cleanupFunctions.push(() => {
      if (updateIntervalId) {
        clearInterval(updateIntervalId)
        console.log('🧹 清理了更新检查定时器')
      }
    })
  }
}

onMounted(async () => {
  window.addEventListener('beforeunload', handleBeforeUnload)
  cleanupFunctions.push(() => {
    window.removeEventListener('beforeunload', handleBeforeUnload)
  })

  try {
    await themeStore.initializeStore()
    // 0. 初始化 AppStore 以确保持久化数据已加载
    console.log('📋 初始化 AppStore...')
    await appStore.initializeStore()

    // 0.1 初始化订阅数据，确保跨会话持久化生效
    await subStore.initializeStore()

    // 0.2 初始化语言与更新配置，保证刷新后保留用户选择
    await localeStore.initializeStore()
    await updateStore.initializeStore()

    // 1. 注册消息实例
    const handleMessageReady = (message: unknown) => {
      appStore.setMessageInstance(message as ReturnType<typeof useMessage>)
    }
    mitt.on('message-instance-ready', handleMessageReady)
    cleanupFunctions.push(() => {
      mitt.off('message-instance-ready', handleMessageReady)
    })

    // 1.1 监听清理消息事件，托盘/空白页切换时强制销毁悬挂的提示
    const handleClearMessages = () => {
      appStore.clearMessages()
    }
    mitt.on('clear-ui-messages', handleClearMessages)
    cleanupFunctions.push(() => {
      mitt.off('clear-ui-messages', handleClearMessages)
    })

    // 2. 监听语言变化
    const stopWatchingLocale = watch(
      () => localeStore.currentLocale,
      (newLocale) => {
        if (newLocale) {
          locale.value = newLocale
        }
      },
      { immediate: true },
    )
    cleanupFunctions.push(stopWatchingLocale)

    // 2.1 当路由切到空白页时，再次清理消息，避免自动关闭定时器被清掉导致提示残留
    const stopWatchingRoute = watch(
      () => router.currentRoute.value.path,
      (newPath) => {
        if (newPath === '/blank') {
          appStore.clearMessages()
        }
      },
    )
    cleanupFunctions.push(stopWatchingRoute)

    // 3. 检查初始窗口状态和自启动情况
    await checkInitialWindowState()

    // 3.5 初始化内核状态监听
    await kernelStore.initializeStore()

    // 3.55 初始化日志监听，确保不打开日志页也能持续收集
    await logStore.initializeStore()
    cleanupFunctions.push(() => logStore.cleanupListeners())

    // 3.6 初始化内核事件数据（流量、连接）
    await Promise.allSettled([
      trafficStore.initializeStore(),
      connectionStore.initializeStore(),
    ])

    // 4. 初始化托盘
    const trayStore = useTrayStore()
    await trayStore.initTray()

    // 5. 执行自动更新检查
    await handleAutoUpdateCheck()

  } catch (error) {
    console.error('应用初始化失败:', error)
  }
})

// 检查初始窗口状态和自启动情况
async function checkInitialWindowState() {
  if (!windowStore) return

  const appWindow = Window.getCurrent()
  try {
    // 获取窗口状态
    const [visible, minimized] = await Promise.all([appWindow.isVisible(), appWindow.isMinimized()])

    windowStore.windowState.isVisible = visible

    console.log(`🔍 初始窗口状态检查: visible=${visible}, minimized=${minimized}`)

    // 如果窗口不可见或已最小化，说明可能是自启动到托盘
    if (!visible || minimized) {
      console.log('📱 检测到托盘模式启动，切换到空白页面')
      // 保存当前路径（如果不是空白页）并切换到空白页
      if (router.currentRoute.value.path !== '/blank') {
        windowStore.windowState.lastVisiblePath = router.currentRoute.value.path
        await router.push('/blank')
      }
    } else if (
      visible &&
      router.currentRoute.value.path === '/blank' &&
      windowStore.windowState.lastVisiblePath
    ) {
      // 如果窗口可见但当前在空白页，恢复到上次的页面
      console.log(`🔄 窗口可见，从空白页恢复到: ${windowStore.windowState.lastVisiblePath}`)
      await router.push(windowStore.windowState.lastVisiblePath)
    }
  } catch (error) {
    console.error('检查初始窗口状态失败:', error)
  }
}

// 清理所有监听器
function cleanup() {
  cleanupFunctions.forEach((fn) => fn())
  cleanupFunctions.length = 0
}

// 组件卸载时清理
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
