<template>
  <n-config-provider :theme="themeStore.theme" :theme-overrides="themeOverrides">
    <n-global-style />
    <n-dialog-provider>
      <n-modal-provider>
        <n-notification-provider>
          <n-message-provider>
            <message-consumer />
            <router-view />
          </n-message-provider>
        </n-notification-provider>
      </n-modal-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import themeOverrides from '@/assets/naive-ui-theme-overrides.json'
import { onMounted, onUnmounted, watch, defineComponent } from 'vue'
import { useRouter } from 'vue-router'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import type { Router } from 'vue-router'

// 导入性能优化工具
import { eventListenerManager, memoryMonitor } from '@/utils/performance'
import { storeManager, type StoreType } from '@/stores/StoreManager'

// 直接导入需要的Store
import { useThemeStore } from '@/stores/app/ThemeStore'

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
    eventListenerManager.add(() => {
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
    eventListenerManager.add(stopWatchingLocale)

    // 设置窗口事件处理器
    await setupWindowEventHandlers()

    // 按需加载其他Store
    await loadRequiredStores()

    // 启动初始化逻辑
    await initializeApp()
  } catch (error) {
    console.error('应用初始化失败:', error)
  }
})

// 设置窗口事件处理器
async function setupWindowEventHandlers() {
  const handleWindowHide = () => {
    if (windowStore) {
      windowStore.windowState.lastVisiblePath = router.currentRoute.value.path
      if (windowStore.windowState.lastVisiblePath !== '/blank') {
        router.push('/blank')
      }
    }
  }

  const handleWindowShow = () => {
    if (
      windowStore &&
      router.currentRoute.value.path === '/blank' &&
      windowStore.windowState.lastVisiblePath
    ) {
      router.push(windowStore.windowState.lastVisiblePath)
    }
  }

  const handleWindowRestore = () => {
    if (
      windowStore &&
      router.currentRoute.value.path === '/blank' &&
      windowStore.windowState.lastVisiblePath
    ) {
      router.push(windowStore.windowState.lastVisiblePath)
    }
  }

  mitt.on('window-hide', handleWindowHide)
  mitt.on('window-show', handleWindowShow)
  mitt.on('window-restore', handleWindowRestore)

  eventListenerManager.add(() => {
    mitt.off('window-hide', handleWindowHide)
    mitt.off('window-show', handleWindowShow)
    mitt.off('window-restore', handleWindowRestore)
  })

  // 检查当前窗口状态
  const appWindow = Window.getCurrent()
  try {
    const visible = await appWindow.isVisible()
    if (windowStore) {
      windowStore.windowState.isVisible = visible
      if (
        visible &&
        router.currentRoute.value.path === '/blank' &&
        windowStore.windowState.lastVisiblePath
      ) {
        router.push(windowStore.windowState.lastVisiblePath)
      }
    }
  } catch (error) {
    console.error('检查窗口状态失败:', error)
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
    // 重置订阅加载状态
    const subStore = await storeManager.loadStore<SubStore>('subscription')
    subStore?.resetLoadingState()

    // 初始化托盘图标
    const trayStore = await storeManager.loadStore<TrayStore>('tray')
    await trayStore?.initTray()

    // 禁用右键菜单（非开发环境）
    if (!import.meta.env.DEV) {
      document.oncontextmenu = () => false
    }

    // 自动启动内核
    if (appStore?.autoStartKernel) {
      await handleAutoStartKernel()
    }

    // 如果内核正在运行，初始化事件监听器
    if (appStore?.isRunning) {
      const kernelStore = await storeManager.loadStore<KernelStore>('kernel')
      kernelStore?.initEventListeners()
    }
  } catch (error) {
    console.error('应用初始化过程中出错:', error)
  }
}

// 处理自动启动内核
async function handleAutoStartKernel() {
  if (appStore?.proxyMode === 'tun') {
    await appStore.switchProxyMode('system')
  }

  try {
    const kernelStore = await storeManager.loadStore<KernelStore>('kernel')
    await kernelStore?.startKernel()
    appStore?.setRunningState(true)

    // 判断当前是否需要隐藏窗口
    const appWindow = Window.getCurrent()
    if (!(await appWindow.isVisible()) && windowStore) {
      windowStore.saveRouteAndGoBlank(router)
    }
  } catch (error) {
    console.error('自动启动内核失败:', error)
  }
}

onUnmounted(async () => {
  // 使用事件监听器管理器清理所有事件
  eventListenerManager.cleanup()

  // 销毁托盘
  try {
    const trayStore = storeManager.getLoadedStore<TrayStore>('tray')
    await trayStore?.destroyTray()
  } catch (error) {
    console.error('销毁托盘失败:', error)
  }

  // 在开发环境强制垃圾回收
  if (import.meta.env.DEV) {
    memoryMonitor.forceGC()
  }
})

// 使事件监听器在组件卸载时自动清理
eventListenerManager.autoCleanup()
</script>

<style>
#app {
  height: 100vh;
}
</style>
