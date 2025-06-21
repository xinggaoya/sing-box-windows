<template>
  <n-config-provider :theme="themeStore.theme" :theme-overrides="themeOverrides">
    <n-global-style />
    <n-dialog-provider>
      <n-modal-provider>
        <n-notification-provider>
          <n-message-provider>
            <!-- 消息消费组件 -->
            <MessageConsumer />

            <!-- 主路由视图 -->
            <router-view />

            <!-- 更新通知组件 -->
            <UpdateNotification />
          </n-message-provider>
        </n-notification-provider>
      </n-modal-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { defineComponent, onMounted, onBeforeUnmount, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'
import { useMessage } from 'naive-ui'
import type { Router } from 'vue-router'

// 导入主题配置
import themeOverrides from '@/assets/naive-ui-theme-overrides.json'

import { storeManager, type StoreType } from '@/stores/StoreManager'

// 直接导入需要的Store
import { useThemeStore } from '@/stores/app/ThemeStore'

// 导入组件
import UpdateNotification from '@/components/UpdateNotification.vue'

// Store类型定义
interface AppStore {
  setMessageInstance: (instance: ReturnType<typeof useMessage>) => void
  autoStartKernel: boolean
  proxyMode: string
  isRunning: boolean
  switchProxyMode: (mode: string) => Promise<void>
  setRunningState: (state: boolean) => void
}

interface LocaleStore {
  currentLocale: string
}

interface WindowStore {
  windowState: {
    lastVisiblePath: string
    isVisible: boolean
  }
  saveRouteAndGoBlank: (router: Router) => void
}

interface SubStore {
  resetLoadingState: () => void
}

interface TrayStore {
  initTray: () => Promise<void>
  destroyTray: () => Promise<void>
}

interface KernelStore {
  startKernel: () => Promise<void>
  initEventListeners: () => void
}

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

// 只初始化核心Store
const router = useRouter()
const { locale } = useI18n()

// 直接使用主题Store（保证与MainLayout.vue使用同一个实例）
const themeStore = useThemeStore()

// 核心Store（按需懒加载）
let appStore: AppStore | null = null
let localeStore: LocaleStore | null = null
let windowStore: WindowStore | null = null

// 清理函数数组
const cleanupFunctions: (() => void)[] = []

onMounted(async () => {
  try {
    // 初始化Store管理器
    await storeManager.initialize()

    // 加载核心Store（不包括theme，因为已经直接导入了）
    appStore = await storeManager.loadStore<AppStore>('app')
    localeStore = await storeManager.loadStore<LocaleStore>('locale')
    windowStore = await storeManager.loadStore<WindowStore>('window')

    // 监听消息实例准备事件
    const handleMessageReady = (message: unknown) => {
      appStore?.setMessageInstance(message as ReturnType<typeof useMessage>)
    }
    mitt.on('message-instance-ready', handleMessageReady)
    cleanupFunctions.push(() => {
      mitt.off('message-instance-ready', handleMessageReady)
    })

    // 监听语言变化
    const stopWatchingLocale = watch(
      () => localeStore?.currentLocale,
      (newLocale) => {
        if (newLocale) {
          locale.value = newLocale
        }
      },
      { immediate: true },
    )
    cleanupFunctions.push(stopWatchingLocale)

    // 检查初始窗口状态和自启动情况
    await checkInitialWindowState()

    // 按需加载其他Store
    await loadRequiredStores()

    // 启动初始化逻辑
    await initializeApp()
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

      // 延迟触发内存清理
      setTimeout(() => {
        console.log('🧹 自启动模式下触发内存清理')
        mitt.emit('memory-cleanup-requested')
      }, 1000)
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

// 按需加载必需的Store
async function loadRequiredStores() {
  // 根据应用配置决定需要加载的Store
  const requiredStores: StoreType[] = ['tray'] // 托盘是必需的

  if (appStore?.autoStartKernel) {
    requiredStores.push('kernel')
  }

  await storeManager.preloadStores(requiredStores)
}

// 应用初始化逻辑
async function initializeApp() {
  try {
    // 初始化托盘
    const trayStore = await storeManager.loadStore<TrayStore>('tray')
    await trayStore.initTray()

    // 如果启用了自动启动，启动内核
    if (appStore?.autoStartKernel) {
      const kernelStore = await storeManager.loadStore<KernelStore>('kernel')
      kernelStore.initEventListeners()
      await kernelStore.startKernel()
    }
  } catch (error) {
    console.error('应用初始化过程中出错:', error)
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

// 应用关闭前清理
window.addEventListener('beforeunload', cleanup)
</script>

<style>
#app {
  height: 100vh;
}
</style>
