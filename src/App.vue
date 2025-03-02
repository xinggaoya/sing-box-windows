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
import { onMounted, onUnmounted, ref } from 'vue'
import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/AppStore'
import { useInfoStore } from '@/stores/infoStore'
import { tauriApi } from '@/services/tauri-api'
import { ProxyService } from '@/services/proxy-service'
import { useRouter, useRoute } from 'vue-router'
import mitt from '@/utils/mitt'

const appWindow = Window.getCurrent()
const appStore = useAppStore()
const infoStore = useInfoStore()
const proxyService = ProxyService.getInstance()
const router = useRouter()
const route = useRoute()
const lastPath = ref('/')

// 记录最后一次正常路由路径
const isBlankRoute = () => route.path === '/blank'

// 保存当前路由并切换到空白页
const saveRouteAndGoBlank = () => {
  if (!isBlankRoute()) {
    lastPath.value = route.path
    router.push('/blank')
  }
}

// 从空白页恢复到之前的路由
const restoreFromBlank = () => {
  if (isBlankRoute() && lastPath.value) {
    router.push(lastPath.value)
  }
}

onMounted(async () => {
  // 初始化托盘图标
  await initTray()

  // 如果不是开发环境，禁用右键菜单
  if (!import.meta.env.DEV) {
    document.oncontextmenu = () => false
  }

  // 如果开启了自动启动内核
  if (appStore.autoStartKernel) {
    try {
      await startKernel()
      // 判断当前是否关闭窗口
      if (!await appWindow.isVisible()){
        saveRouteAndGoBlank()
      }
    } catch (error) {
      console.error('自动启动内核失败:', error)
    }
  }
  // 如果内核正在运行，初始化 WebSocket 连接
  if (appStore.isRunning) {
    infoStore.initWebSocket()
  }

  setupWindowEventHandlers()
})

onUnmounted(() => {
  // 清理事件监听
  mitt.off('window-minimize')
  mitt.off('window-hide')
  mitt.off('window-show')
  mitt.off('window-restore')
  // Tauri的事件监听器会自动清理
})

// 启动内核的统一处理函数
const startKernel = async () => {
  try {
    // 检查是否有管理员权限
    const isAdmin = await tauriApi.proxy.checkAdmin()

    if (!isAdmin) {
      // 如果不是管理员权限，切换到系统代理模式
      await appStore.switchProxyMode('system')
    }

    // 启动内核
    await infoStore.startKernel()
    appStore.isRunning = true
  } catch (error) {
    console.error('启动内核失败:', error)
    throw error
  }
}

const initTray = async () => {
  const menu = await Menu.new({
    items: [
      {
        id: 'show',
        text: '显示界面',
        action: async () => {
          // 先显示窗口
          await appWindow.show()
          // 触发window-show事件，确保从空白页恢复
          mitt.emit('window-show')
        },
      },
      {
        id: 'start',
        text: '启动内核',
        action: async () => {
          await startKernel()
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
          const showMessage = (type: 'success' | 'info' | 'error', content: string) => {
            console.log(`${type}: ${content}`)
          }
          await proxyService.switchMode('system', showMessage)
        },
      },
      {
        id: 'tun_mode',
        text: 'TUN 模式',
        action: async () => {
          const showMessage = (type: 'success' | 'info' | 'error', content: string) => {
            console.log(`${type}: ${content}`)
          }
          const needClose = await proxyService.switchMode('tun', showMessage)
          if (needClose) {
            await appWindow.close()
          }
        },
      },
      {
        id: 'quit',
        text: '退出',
        action: async () => {
          await infoStore.stopKernel()
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
          appWindow.show()
          // 触发window-show事件，确保从空白页恢复
          mitt.emit('window-show')
          break
      }
    },
    menu,
    menuOnLeftClick: true,
  }

  // 使用 @ts-expect-error 替代 @ts-ignore
  // @ts-expect-error TrayIcon API 可能不完全匹配，但实现是正确的
  await TrayIcon.new(options)
}

// 设置窗口事件监听
const setupWindowEventHandlers = () => {

  // 窗口隐藏时切换到空白页
  mitt.on('window-hide', () => {
    saveRouteAndGoBlank()
  })

  // 窗口显示时恢复路由
  mitt.on('window-show', () => {
    restoreFromBlank()
  })

  // 窗口恢复时恢复路由
  mitt.on('window-restore', () => {
    restoreFromBlank()
  })

  // 线程检查
  appWindow.isVisible().then((visible) => {
    if (visible) {
      restoreFromBlank()
    }
  })
}
</script>

<style>
#app {
  height: 100vh;
}
</style>
