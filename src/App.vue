<template>
  <n-config-provider :theme="appStore.theme" :theme-overrides="themeOverrides">
    <n-dialog-provider>
      <n-modal-provider>
        <n-notification-provider>
          <n-message-provider>
            <router-view />
          </n-message-provider>
        </n-notification-provider>
      </n-modal-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { TrayIcon } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import themeOverrides from '@/assets/naive-ui-theme-overrides.json'
import { onMounted, onUnmounted, inject } from 'vue'
import { useAppStore } from '@/stores/AppStore'
import { useInfoStore } from '@/stores/infoStore'
import { useRouter } from 'vue-router'
import { NotificationService } from '@/services/notification-service'
import { Window } from '@tauri-apps/api/window'
import { ProxyService } from '@/services/proxy-service'
import mitt from '@/utils/mitt'

// 初始化 store
const appStore = useAppStore()
const infoStore = useInfoStore()
const router = useRouter()
const notificationService = NotificationService.getInstance()
const proxyService = ProxyService.getInstance()

// 初始化通知服务
const initNotificationService = () => {
  // 这里的具体实现会在组件中通过provide/inject进行处理
  // 这样可以在其他组件中获取Naive UI的消息提示函数
}

onMounted(async () => {
  // 设置窗口事件处理器
  appStore.setupWindowEventHandlers(router)

  // 初始化托盘图标
  await initTray()

  // 如果不是开发环境，禁用右键菜单
  if (!import.meta.env.DEV) {
    document.oncontextmenu = () => false
  }

  // 如果开启了自动启动内核
  if (appStore.autoStartKernel) {
    try {
      await infoStore.startKernel()
      appStore.isRunning = true

      // 判断当前是否需要隐藏窗口
      const appWindow = Window.getCurrent()
      if (!(await appWindow.isVisible())) {
        appStore.saveRouteAndGoBlank(router)
      }
    } catch (error) {
      console.error('自动启动内核失败:', error)
    }
  }
  // 如果内核正在运行，初始化 WebSocket 连接
  if (appStore.isRunning) {
    infoStore.initWebSocket()
  }
})

onUnmounted(() => {
  // 清理事件监听
  appStore.cleanupWindowEvents()
  mitt.off('window-minimize')
  mitt.off('window-hide')
  mitt.off('window-show')
  mitt.off('window-restore')
  // Tauri的事件监听器会自动清理
})

// 初始化托盘菜单
const initTray = async () => {
  const menu = await Menu.new({
    items: [
      {
        id: 'show',
        text: '显示界面',
        action: async () => {
          await appStore.showWindow()
        },
      },
      {
        id: 'start',
        text: '启动内核',
        action: async () => {
          try {
            await infoStore.startKernel()
            appStore.isRunning = true
          } catch (error) {
            console.error('启动内核失败:', error)
          }
        },
      },
      {
        id: 'stop',
        text: '停止内核',
        action: async () => {
          await infoStore.stopKernel()
          appStore.isRunning = false
        },
      },
      {
        id: 'restart',
        text: '重启内核',
        action: async () => {
          await infoStore.restartKernel()
        },
      },
      {
        id: 'system_proxy',
        text: '系统代理模式',
        action: async () => {
          await proxyService.switchMode('system')
          appStore.proxyMode = 'system'
        },
      },
      {
        id: 'tun_mode',
        text: 'TUN 模式',
        action: async () => {
          const needClose = await proxyService.switchMode('tun')
          appStore.proxyMode = 'tun'
          if (needClose) {
            const appWindow = Window.getCurrent()
            await appWindow.close()
          }
        },
      },
      {
        id: 'quit',
        text: '退出',
        action: async () => {
          await infoStore.stopKernel()
          const appWindow = Window.getCurrent()
          await appWindow.close()
        },
      },
    ],
  })

  const options = {
    tooltip: 'sing-box-window',
    icon: await defaultWindowIcon(),
    action: (event: { type: string }) => {
      switch (event.type) {
        case 'DoubleClick':
          appStore.showWindow()
          break
      }
    },
    menu,
    menuOnLeftClick: true,
  }

  // @ts-expect-error TrayIcon API 可能不完全匹配，但实现是正确的
  await TrayIcon.new(options)
}
</script>

<style>
#app {
  height: 100vh;
}
</style>
