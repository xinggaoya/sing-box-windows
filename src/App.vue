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
import { useI18n } from 'vue-i18n'  // импортируем i18n

// Инициализируем i18n
const { t } = useI18n()

// Инициализация store
const appStore = useAppStore()
const infoStore = useInfoStore()
const trayStore = useTrayStore()
const router = useRouter()

onMounted(async () => {
  // Настройка обработчиков событий окна
  mitt.on('window-hide', () => {
    appStore.windowState.lastVisiblePath = router.currentRoute.value.path
    if (appStore.windowState.lastVisiblePath !== '/blank') {
      router.push('/blank')
    }
  })

  mitt.on('window-show', () => {
    if (router.currentRoute.value.path === '/blank' && appStore.windowState.lastVisiblePath) {
      router.push(appStore.windowState.lastVisiblePath)
    }
  })

  mitt.on('window-restore', () => {
    if (router.currentRoute.value.path === '/blank' && appStore.windowState.lastVisiblePath) {
      router.push(appStore.windowState.lastVisiblePath)
    }
  })

  const appWindow = Window.getCurrent()
  appWindow.isVisible().then((visible) => {
    appStore.windowState.isVisible = visible
    if (
      visible &&
      router.currentRoute.value.path === '/blank' &&
      appStore.windowState.lastVisiblePath
    ) {
      router.push(appStore.windowState.lastVisiblePath)
    }
  })

  await trayStore.initTray()

  if (!import.meta.env.DEV) {
    document.oncontextmenu = () => false
  }

  if (appStore.autoStartKernel) {
    try {
      await infoStore.startKernel()
      appStore.setRunningState(true)

      const appWindow = Window.getCurrent()
      if (!(await appWindow.isVisible())) {
        appStore.saveRouteAndGoBlank(router)
      }
    } catch (error) {
      // Локализуем сообщение об ошибке
      console.error('自动启动内核失败:', error)
    }
  }
  if (appStore.isRunning) {
    infoStore.initEventListeners()
  }
})

onUnmounted(async () => {
  mitt.off('window-minimize')
  mitt.off('window-hide')
  mitt.off('window-show')
  mitt.off('window-restore')

  await trayStore.destroyTray()
})
</script>
