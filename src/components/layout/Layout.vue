<template>
  <MainLayout />
</template>

<script setup lang="ts">
import MainLayout from '@/components/layout/main/MainLayout.vue'
import { TrayIcon } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { onBeforeMount, onUnmounted } from 'vue'
import { Window } from '@tauri-apps/api/window'

let tray: any = null
const appWindow = Window.getCurrent()

onBeforeMount(() => {
  initTray()
  // 如果dev
  if (!import.meta.env.DEV) {
    document.oncontextmenu = () => false
  }
})
// 停止前
onUnmounted(() => {
  tray.destroy()
})
const initTray = async () => {
  const menu = await Menu.new({
    items: [
      {
        id: 'quit',
        text: '退出',
        action: () => {
          appWindow.close()
        }
      }
    ]
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
    menuOnLeftClick: false
  }

  //@ts-ignore
  tray = await TrayIcon.new(options)
}

</script>

<style scoped>

</style>
