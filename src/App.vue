<template>
  <n-config-provider :theme="themeStore.theme" :theme-overrides="themeOverrides">
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
import { useAppStore } from '@/stores/app/AppStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { useLocaleStore } from '@/stores/app/LocaleStore'
import { useWindowStore } from '@/stores/app/WindowStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrayStore } from '@/stores/tray/TrayStore'
import { useRouter } from 'vue-router'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'

// 消息消费组件
const MessageConsumer = defineComponent({
  name: 'MessageConsumer',
  setup() {
    const message = useMessage()
    const appStore = useAppStore()
    
    // 注册消息实例
    onMounted(() => {
      appStore.setMessageInstance(message)
    })
    
    return () => null
  }
})

// 初始化 store
const appStore = useAppStore()
const themeStore = useThemeStore()
const localeStore = useLocaleStore()
const windowStore = useWindowStore()
const kernelStore = useKernelStore()
const trayStore = useTrayStore()
const router = useRouter()
const { locale } = useI18n()

// 监听语言变化
watch(
  () => localeStore.currentLocale,
  (newLocale) => {
    locale.value = newLocale
  },
  { immediate: true },
)

onMounted(async () => {
  // 设置窗口事件处理器
  // windowStore.setupWindowEventHandlers(router)

  // 自己实现窗口事件处理器
  // 窗口隐藏时切换到空白页
  mitt.on('window-hide', () => {
    windowStore.windowState.lastVisiblePath = router.currentRoute.value.path
    if (windowStore.windowState.lastVisiblePath !== '/blank') {
      router.push('/blank')
    }
  })

  // 窗口显示时恢复路由
  mitt.on('window-show', () => {
    if (router.currentRoute.value.path === '/blank' && windowStore.windowState.lastVisiblePath) {
      router.push(windowStore.windowState.lastVisiblePath)
    }
  })

  // 窗口恢复时恢复路由
  mitt.on('window-restore', () => {
    if (router.currentRoute.value.path === '/blank' && windowStore.windowState.lastVisiblePath) {
      router.push(windowStore.windowState.lastVisiblePath)
    }
  })

  // 检查当前窗口状态
  const appWindow = Window.getCurrent()
  appWindow.isVisible().then((visible) => {
    windowStore.windowState.isVisible = visible
    if (
      visible &&
      router.currentRoute.value.path === '/blank' &&
      windowStore.windowState.lastVisiblePath
    ) {
      router.push(windowStore.windowState.lastVisiblePath)
    }
  })

  // 初始化托盘图标
  await trayStore.initTray()

  // 如果不是开发环境，禁用右键菜单
  if (!import.meta.env.DEV) {
    document.oncontextmenu = () => false
  }

  // 如果开启了自动启动内核
  if (appStore.autoStartKernel) {
    if (appStore.proxyMode === 'tun') {
      await appStore.switchProxyMode('system')
    }
    try {
      await kernelStore.startKernel()
      appStore.setRunningState(true)

      // 判断当前是否需要隐藏窗口
      const appWindow = Window.getCurrent()
      if (!(await appWindow.isVisible())) {
        windowStore.saveRouteAndGoBlank(router)
      }
    } catch (error) {
      console.error('自动启动内核失败:', error)
    }
  }
  // 如果内核正在运行，初始化事件监听器
  if (appStore.isRunning) {
    kernelStore.initEventListeners()
  }
})

onUnmounted(async () => {
  // 清理事件监听
  // windowStore.cleanupWindowEvents()
  mitt.off('window-minimize')
  mitt.off('window-hide')
  mitt.off('window-show')
  mitt.off('window-restore')

  // 销毁托盘
  await trayStore.destroyTray()
})
</script>

<style>
#app {
  height: 100vh;
}
</style>
