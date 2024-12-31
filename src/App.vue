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
import { onMounted, ref } from 'vue'
import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/AppStore'
import { useInfoStore } from '@/stores/infoStore'
import { invoke } from '@tauri-apps/api/core'
import { darkTheme, useMessage, type MessageApi } from 'naive-ui'

const appWindow = Window.getCurrent()
const appStore = useAppStore()
const infoStore = useInfoStore()
const theme = darkTheme
const message = ref<MessageApi>()

onMounted(async () => {
  message.value = useMessage()

  // 初始化托盘图标
  await initTray()
  
  // 如果不是开发环境，禁用右键菜单
  if (!import.meta.env.DEV) {
    document.oncontextmenu = () => false
  }

  // 如果开启了自动启动内核
  if (appStore.autoStartKernel && !appStore.isRunning) {
    try {
      // 检查是否有管理员权限
      const isAdmin = await invoke('check_admin')
      
      if (isAdmin) {
        // 如果是管理员权限，尝试启动内核
        await invoke('start_kernel')
        message.value?.success('内核已自动启动')
      } else {
        // 如果不是管理员权限，切换到系统代理模式并启动
        await invoke('set_system_proxy')
        appStore.mode = 'system'
        await invoke('start_kernel')
        message.value?.success('内核已以系统代理模式自动启动')
      }
      appStore.isRunning = true
    } catch (error) {
      message.value?.error('自动启动内核失败: ' + (error as string))
    }
  }
})

const initTray = async () => {
  const menu = await Menu.new({
    items: [
      {
        id: 'quit',
        text: '退出',
        action: async () => {
          if (appStore.isRunning) {
            await invoke('stop_kernel')
            appStore.isRunning = false
          }
          await appWindow.close()
        },
      },
    ],
  })

  const options = {
    icon: await defaultWindowIcon(),
    action: (event: any) => {
      switch (event.type) {
        case 'Click':
          appWindow.show()
          break
      }
    },
    menu,
    menuOnLeftClick: false,
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
