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
            <div @contextmenu="handleContextMenu" class="app-container">
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
import { defineComponent, onMounted, onBeforeUnmount, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'
import { useMessage } from 'naive-ui'
import type { Router } from 'vue-router'
import { tauriApi } from '@/services/tauri-api'

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

// 生产环境下禁用右键菜单
const handleContextMenu = (event: MouseEvent) => {
  // 只在生产环境中禁用右键菜单
  if (import.meta.env.PROD) {
    event.preventDefault()
    return false
  }
}

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
    // 自动启动时需要加载更多Store以确保数据流正常
    requiredStores.push('kernel', 'connection', 'traffic', 'log')
    console.log('🔄 自动启动模式：预加载数据相关Store')
  }

  await storeManager.preloadStores(requiredStores)
}

// 应用初始化逻辑
async function initializeApp() {
  try {
    // 初始化托盘
    const trayStore = await storeManager.loadStore<TrayStore>('tray')
    await trayStore.initTray()

    // 如果启用了自动启动，启动内核（使用改进的启动逻辑）
    if (appStore?.autoStartKernel) {
      console.log('🚀 检测到自动启动内核设置，准备启动内核...')
      await startKernelWithRetry()
    }
  } catch (error) {
    console.error('应用初始化过程中出错:', error)
  }
}

// 简化的内核自动启动函数
async function startKernelWithRetry() {
  console.log('🚀 检测到自动启动内核设置，开始启动...')

  try {
    // 等待应用完全初始化
    await new Promise((resolve) => setTimeout(resolve, 3000))

    // 检查管理员权限和代理模式
    const isAdmin = await tauriApi.system.checkAdmin()
    const currentProxyMode = appStore?.proxyMode || 'system'

    console.log(`🔍 自启动检查 - 管理员权限: ${isAdmin}, 当前代理模式: ${currentProxyMode}`)

    // 如果不是管理员权限且当前模式是TUN，则切换为system模式
    if (!isAdmin && currentProxyMode === 'tun') {
      console.log('⚠️ 检测到非管理员权限运行且为TUN模式，自动切换为system模式')

      try {
        // 切换为system模式
        await tauriApi.proxy.setSystemProxy()
        if (appStore) {
          await appStore.switchProxyMode('system')
        }
        console.log('✅ 已自动切换为system模式')
      } catch (error) {
        console.error('❌ 切换为system模式失败:', error)
        // 即使切换失败也继续尝试启动内核
      }
    }

    // 加载内核Store
    const kernelStore = await storeManager.loadStore<KernelStore>('kernel')

    // 初始化事件监听器
    console.log('🎧 初始化事件监听器...')
    await kernelStore.initEventListeners()

    // 启动内核（后端已包含完整检查）
    console.log('🚀 启动内核，后端将进行完整就绪检查...')
    await kernelStore.startKernel()

    console.log('✅ 内核自动启动成功！')
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    console.error('❌ 内核自动启动失败:', errorMessage)

    // 设置应用状态为未运行
    if (appStore) {
      appStore.setRunningState(false)
    }

    // 提示用户可以手动启动
    console.log('💡 提示：您可以在主页手动启动内核')
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

.app-container {
  height: 100%;
  width: 100%;
}

/* 生产环境下禁用文本选择 */
@media (not (hover: hover)) {
  .app-container {
    -webkit-user-select: none;
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
  }
}

/* 在生产环境下的额外安全措施 */
.app-container {
  -webkit-touch-callout: none;
  -webkit-tap-highlight-color: transparent;
}
</style>
