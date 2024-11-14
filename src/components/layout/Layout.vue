<template>
  <MainLayout />
</template>

<script setup lang="ts">
import MainLayout from '@/components/layout/main/MainLayout.vue'
import { TrayIcon } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { onBeforeMount } from 'vue'
import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/AppStore'
import { invoke } from '@tauri-apps/api/core'
import { useInfoStore } from '@/stores/infoStore'

const appWindow = Window.getCurrent()
const appStore = useAppStore()
const infoStore = useInfoStore()

onBeforeMount(async () => {
  initTray()
  // 如果dev
  if (!import.meta.env.DEV) {
    document.oncontextmenu = () => false
  }
  if (appStore.autoStartKernel && !infoStore.isRunning) {
    await invoke('set_system_proxy')
    appStore.mode = 'system'
    await invoke('start_kernel')
  }
})

const initTray = async () => {
  const menu = await Menu.new({
    items: [
      {
        id: 'quit',
        text: '退出',
        action: async () => {
          await invoke('stop_kernel')
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
          console.log('click')
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

<style scoped></style>
