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
import themeOverrides from '@/assets/naive-ui-theme-overrides.json'
import { onMounted, onUnmounted } from 'vue'
import { useAppStore } from '@/stores/AppStore'
import { useInfoStore } from '@/stores/infoStore'
import { useTrayStore } from '@/stores/TrayStore'
import { useRouter } from 'vue-router'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'

// 初始化 store
const appStore = useAppStore()
const infoStore = useInfoStore()
const trayStore = useTrayStore()
const router = useRouter()

onMounted(async () => {
  // 设置窗口事件处理器
  appStore.setupWindowEventHandlers(router)

  // 初始化托盘图标
  await trayStore.initTray()

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

onUnmounted(async () => {
  // 清理事件监听
  appStore.cleanupWindowEvents()
  mitt.off('window-minimize')
  mitt.off('window-hide')
  mitt.off('window-show')
  mitt.off('window-restore')

  // 清理托盘
  await trayStore.destroyTray()
  // Tauri的事件监听器会自动清理
})
</script>

<style>
#app {
  height: 100vh;
}
</style>
