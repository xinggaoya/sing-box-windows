<template>
  <router-view />
</template>

<script setup lang="ts">
import { TrayIcon } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { onMounted } from 'vue'
import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/AppStore'
import { invoke } from '@tauri-apps/api/core'
import { useInfoStore } from '@/stores/infoStore'

const appWindow = Window.getCurrent()
const appStore = useAppStore()
const infoStore = useInfoStore()

onMounted(async () => {
  initTray()
  // 如果dev
  if (!import.meta.env.DEV) {
    document.oncontextmenu = () => false
  }
  if (appStore.autoStartKernel && !appStore.isRunning) {
    await invoke('set_system_proxy')
    appStore.mode = 'system'
    await infoStore.startKernel()
  }
})

const initTray = async () => {
  const menu = await Menu.new({
    items: [
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
