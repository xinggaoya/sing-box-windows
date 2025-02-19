<template>
  <n-config-provider :theme="theme">
    <n-dialog-provider>
      <n-message-provider>
        <router-view />
      </n-message-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { TrayIcon } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { onMounted } from 'vue'
import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/AppStore'
import { useInfoStore } from '@/stores/infoStore'
import { tauriApi } from '@/services/tauri-api'
import { darkTheme } from 'naive-ui'
import { ProxyService } from '@/services/proxy-service'

const appWindow = Window.getCurrent()
const appStore = useAppStore()
const infoStore = useInfoStore()
const proxyService = ProxyService.getInstance()
const theme = darkTheme

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
    } catch (error) {
      console.error('自动启动内核失败:', error)
    }
  }
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
          appWindow.show()
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
    icon: await defaultWindowIcon(),
    action: (event: any) => {
      switch (event.type) {
        case 'DoubleClick':
          appWindow.show()
          break
      }
    },
    menu,
    menuOnLeftClick: true,
  }

  //@ts-ignore
  await TrayIcon.new(options)
}
</script>

<style>
#app {
  height: 100vh;
}
</style>
