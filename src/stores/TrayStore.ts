import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { TrayIcon, TrayIconEvent } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { MenuItem } from '@tauri-apps/api/menu/menuItem'
import { Submenu } from '@tauri-apps/api/menu/submenu'
import { useAppStore } from './AppStore'
import { useInfoStore } from './infoStore'
import { useSubStore } from './SubStore'
import { ProxyService } from '@/services/proxy-service'
import mitt from '@/utils/mitt'
import { Window } from '@tauri-apps/api/window'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'

export const useTrayStore = defineStore('tray', () => {
  const appStore = useAppStore()
  const infoStore = useInfoStore()
  const subStore = useSubStore()
  const router = useRouter()
  const proxyService = ProxyService.getInstance()
  const { t } = useI18n()

  const lastProxyMode = ref<'system' | 'tun'>(appStore.proxyMode)
  const trayInstanceId = ref<string | null>(null)

  const updateTrayTooltip = async () => {
    if (!trayInstanceId.value) return
    const status = appStore.isRunning ? t('status.running') : t('status.stopped')
    const mode = appStore.proxyMode === 'system' ? t('proxy.system') : t('proxy.tun')

    let configName = ''
    if (subStore.activeIndex !== null && subStore.list.length > 0) {
      configName = subStore.list[subStore.activeIndex].name
    }

    let tooltipText = `sing-box-window - ${t('message.kernel_' + (appStore.isRunning ? 'started' : 'stopped'))}, ${mode}`
    if (configName) tooltipText += `, ${t('proxy.current_node', { node: configName })}`

    try {
      const tray = await TrayIcon.getById(trayInstanceId.value)
      if (tray) await tray.setTooltip(tooltipText)
    } catch (e) {
      console.error('Failed to update tray tooltip:', e)
    }
  }

  const createTrayMenu = async () => {
    try {
      const currentProxyMode = appStore.proxyMode
      lastProxyMode.value = currentProxyMode

      const showMenuItem = await MenuItem.new({
        id: 'show',
        text: t('tray.menu.show_window'),
        enabled: true,
        action: async () => {
          await appStore.restoreFromBlank(router)
          await appStore.showWindow()
        },
      })

      const startMenuItem = await MenuItem.new({
        id: 'start',
        text: t('action.start'),
        enabled: !appStore.isRunning,
        action: async () => {
          try {
            await infoStore.startKernel()
            appStore.setRunningState(true)
            await refreshTrayMenu()
          } catch (error) {
            console.error('Failed to start kernel:', error)
          }
        },
      })

      const stopMenuItem = await MenuItem.new({
        id: 'stop',
        text: t('action.stop'),
        enabled: appStore.isRunning,
        action: async () => {
          await infoStore.stopKernel()
          appStore.setRunningState(false)
          await refreshTrayMenu()
        },
      })

      const restartMenuItem = await MenuItem.new({
        id: 'restart',
        text: t('tray.menu.restart_kernel') || 'Restart',
        enabled: appStore.isRunning,
        action: async () => {
          await infoStore.restartKernel()
          await refreshTrayMenu()
        },
      })

      const kernelSubmenu = await Submenu.new({
        id: 'kernel_control',
        text: t('settings.kernel_management'),
        items: [startMenuItem, stopMenuItem, restartMenuItem],
      })

      const systemProxyMenuItem = await MenuItem.new({
        id: 'system_proxy',
        text: t('proxy.system'),
        enabled: currentProxyMode !== 'system',
        action: async () => {
          try {
            await proxyService.switchMode('system')
            appStore.proxyMode = 'system'
            await refreshTrayMenu()
          } catch (error) {
            console.error('Switch to system mode failed:', error)
          }
        },
      })

      const tunProxyMenuItem = await MenuItem.new({
        id: 'tun_mode',
        text: t('proxy.tun'),
        enabled: currentProxyMode !== 'tun',
        action: async () => {
          try {
            const needClose = await proxyService.switchMode('tun')
            appStore.proxyMode = 'tun'
            await refreshTrayMenu()
            if (needClose) await Window.getCurrent().close()
          } catch (error) {
            console.error('Switch to TUN mode failed:', error)
          }
        },
      })

      const currentModeMenuItem = await MenuItem.new({
        id: 'current_mode',
        text: `${t('tray.menu.current_mode') || t('tray.menu.current_mode.unknown')}: ${t(
          'proxy.' + currentProxyMode
        )}`,
        enabled: false,
      })

      const proxyModeSubmenu = await Submenu.new({
        id: 'proxy_mode',
        text: t('proxy.switch'),
        items: [currentModeMenuItem, systemProxyMenuItem, tunProxyMenuItem],
      })

      const separator1 = await MenuItem.new({ id: 'sep1', text: '-', enabled: false })
      const separator2 = await MenuItem.new({ id: 'sep2', text: '-', enabled: false })

      const quitMenuItem = await MenuItem.new({
        id: 'quit',
        text: t('tray.menu.quit') || 'Quit',
        action: async () => {
          await infoStore.stopKernel()
          await Window.getCurrent().close()
        },
      })

      return await Menu.new({
        items: [showMenuItem, separator1, kernelSubmenu, proxyModeSubmenu, separator2, quitMenuItem],
      })
    } catch (error) {
      console.error('Failed to create tray menu:', error)
      return await Menu.new({ items: [] })
    }
  }

  const initTray = async () => {
    try {
      if (appStore.trayInstanceId) {
        try {
          await TrayIcon.removeById(appStore.trayInstanceId)
        } catch (_) {}
      }

      const icon = await defaultWindowIcon()
      if (!icon) throw new Error('Missing default icon')

      const menu = await createTrayMenu()
      const tray = await TrayIcon.new({
        icon,
        tooltip: 'sing-box-window',
        menu,
        action: async (event: TrayIconEvent) => {
          if (event.type === 'Click' && event.button === 'Left') {
            await appStore.restoreFromBlank(router)
            await appStore.showWindow()
          }
        },
      })

      trayInstanceId.value = tray.id
      appStore.trayInstanceId = tray.id

      await updateTrayTooltip()

      watch(() => appStore.isRunning, () => {
        updateTrayTooltip()
        refreshTrayMenu()
      })

      watch(() => appStore.proxyMode, (newMode) => {
        updateTrayTooltip()
        if (newMode !== lastProxyMode.value) refreshTrayMenu()
      })

      watch(() => [subStore.activeIndex, subStore.list.length], updateTrayTooltip)

      mitt.on('process-status', () => {
        updateTrayTooltip()
        refreshTrayMenu()
      })

      mitt.on('proxy-mode-changed', () => {
        updateTrayTooltip()
        refreshTrayMenu()
      })

      mitt.on('refresh-tray-menu', refreshTrayMenu)

      return true
    } catch (error) {
      console.error('Failed to initialize tray:', error)
      return false
    }
  }

  const refreshTrayMenu = async () => {
    if (!trayInstanceId.value) return

    try {
      const tray = await TrayIcon.getById(trayInstanceId.value)
      const menu = await createTrayMenu()
      if (tray) {
        await tray.setMenu(menu)
        await updateTrayTooltip()
      } else {
        throw new Error('Tray instance not found')
      }
    } catch (error) {
      console.error('Failed to refresh tray menu:', error)
      await destroyTray()
      await initTray()
    }
  }

  const destroyTray = async () => {
    if (trayInstanceId.value) {
      try {
        await TrayIcon.removeById(trayInstanceId.value)
      } catch (error) {
        console.error('Failed to destroy tray:', error)
      }
      trayInstanceId.value = null
      appStore.trayInstanceId = null
    }

    mitt.off('process-status')
    mitt.off('proxy-mode-changed')
    mitt.off('refresh-tray-menu')
  }

  return {
    trayInstanceId,
    initTray,
    updateTrayTooltip,
    refreshTrayMenu,
    destroyTray,
  }
})
