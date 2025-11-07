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
import { defineComponent, onMounted, onBeforeUnmount, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'
import { useMessage } from 'naive-ui'
import type { Router } from 'vue-router'
import { tauriApi } from '@/services/tauri'

// 导入主题配置
import themeOverrides from '@/assets/naive-ui-theme-overrides.json'

import { useThemeStore, useAppStore, useLocaleStore, useWindowStore, useTrayStore, useKernelStore, useUpdateStore } from '@/stores'

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
  const updateStore = useUpdateStore()
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
    // 1. 注册消息实例
    const handleMessageReady = (message: unknown) => {
      appStore.setMessageInstance(message as ReturnType<typeof useMessage>)
    }
    mitt.on('message-instance-ready', handleMessageReady)
    cleanupFunctions.push(() => {
      mitt.off('message-instance-ready', handleMessageReady)
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

    // 3. 检查初始窗口状态和自启动情况
    await checkInitialWindowState()

    // 4. 初始化托盘
    const trayStore = useTrayStore()
    await trayStore.initTray()

    // 5. 如果启用了自动启动，启动内核
    if (appStore.autoStartKernel) {
      console.log('🚀 检测到自动启动内核设置，准备启动内核...')
      await startKernelWithRetry()
    }

    // 6. 执行自动更新检查
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

// 简化的内核自动启动函数（增强版本，支持开机自启动检测）
async function startKernelWithRetry() {
  console.log('🚀 检测到自动启动内核设置，开始启动...')

  try {
    // 检测是否是开机自启动场景
    if (appStore.isAutostartScenario) {
      console.log('🕐 检测到开机自启动场景，使用延迟启动策略')
      
      // 开机自启动场景：延迟10-15秒让系统完全就绪
      const delay = 10000 + Math.random() * 5000 // 10-15秒随机延迟
      const success = await appStore.delayedKernelStart(delay)
      
      if (success) {
        console.log('✅ 开机自启动延迟启动内核成功！')
        return
      } else {
        console.warn('⚠️ 开机自启动延迟启动失败，尝试正常启动流程')
      }
    }

    // 正常启动流程
    await normalKernelStart()
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    console.error('❌ 内核自动启动失败:', errorMessage)

    // 设置应用状态为未运行
    appStore.setRunningState(false)

    // 提示用户可以手动启动
    console.log('💡 提示：您可以在主页手动启动内核')
  }
}

// 正常内核启动流程
async function normalKernelStart() {
  // 等待应用完全初始化
  await new Promise((resolve) => setTimeout(resolve, 3000))

  // 检查管理员权限和代理模式
  const isAdmin = await tauriApi.system.checkAdmin()
  const currentProxyMode = appStore.proxyMode || 'system'

  console.log(`🔍 自启动检查 - 管理员权限: ${isAdmin}, 当前代理模式: ${currentProxyMode}`)

  // 如果不是管理员权限且当前模式是TUN，则切换为system模式
  if (!isAdmin && currentProxyMode === 'tun') {
    console.log('⚠️ 检测到非管理员权限运行且为TUN模式，自动切换为system模式')

    try {
      // 切换为system模式
      await tauriApi.proxy.setSystemProxy()
      await appStore.switchProxyMode('system')
      console.log('✅ 已自动切换为system模式')
    } catch (error) {
      console.error('❌ 切换为system模式失败:', error)
      // 即使切换失败也继续尝试启动内核
    }
  }

  // 获取内核Store实例
  const kernelStore = useKernelStore()

  // 初始化事件监听器（现在由各个Store自动管理）
  console.log('🎧 事件监听器将由各个Store自动初始化...')

  // 启动内核（后端已包含完整检查）
  console.log('🚀 启动内核，后端将进行完整就绪检查...')
  await kernelStore.startKernel()

  console.log('✅ 内核自动启动成功！')
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
