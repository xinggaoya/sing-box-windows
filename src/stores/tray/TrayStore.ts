// TrayStore.ts - 托盘桥接层（后端主导托盘，前端只做状态同步与动作执行）
import { defineStore } from 'pinia'
import { ref, watch, type WatchStopHandle } from 'vue'
import { useRouter } from 'vue-router'
import i18n from '@/locales'
import { APP_EVENTS } from '@/constants/events'
import { eventService } from '@/services/event-service'
import { trayService, type TrayRuntimeStatePayload } from '@/services/tray-service'
import { systemService } from '@/services/system-service'
import { sudoService } from '@/services/sudo-service'
import type { TrayNavigatePayload, TraySwitchProxyModePayload } from '@/types/events'
import { useAppStore, useKernelStore, useLocaleStore, useSubStore, useSudoStore } from '@/stores'
import { useWindowStore } from '@/stores/app/WindowStore'
import type { ProxyMode } from '@/stores/app/AppStore'

const TRAY_SYNC_DEBOUNCE_MS = 150

const isProxyMode = (value: unknown): value is ProxyMode =>
  value === 'system' || value === 'tun' || value === 'manual'

export const useTrayStore = defineStore('tray', () => {
  const appStore = useAppStore()
  const subStore = useSubStore()
  const localeStore = useLocaleStore()
  const kernelStore = useKernelStore()
  const windowStore = useWindowStore()
  const router = useRouter()

  const initialized = ref(false)
  const syncing = ref(false)

  const trayWatchers: WatchStopHandle[] = []
  const eventUnlisteners: Array<() => void> = []
  let lastSyncedPayloadKey = ''
  let lastSyncedRoute = ''
  let syncTimer: number | null = null

  const registerWatcher = (...args: Parameters<typeof watch>) => {
    const stop = watch(...args)
    trayWatchers.push(stop)
    return stop
  }

  const cleanupWatchers = () => {
    while (trayWatchers.length) {
      const stop = trayWatchers.pop()
      stop?.()
    }
  }

  const cleanupEvents = () => {
    while (eventUnlisteners.length) {
      const unlisten = eventUnlisteners.pop()
      try {
        unlisten?.()
      } catch (error) {
        console.error('清理托盘事件监听失败:', error)
      }
    }
  }

  const cleanupTimer = () => {
    if (syncTimer !== null) {
      clearTimeout(syncTimer)
      syncTimer = null
    }
  }

  const buildRuntimePayload = (): TrayRuntimeStatePayload => {
    let activeSubscriptionName: string | null = null
    if (
      subStore.activeIndex !== null &&
      subStore.activeIndex >= 0 &&
      subStore.activeIndex < subStore.list.length
    ) {
      activeSubscriptionName = subStore.list[subStore.activeIndex].name || null
    }

    return {
      kernelRunning: appStore.isRunning,
      proxyMode: appStore.proxyMode as ProxyMode,
      activeSubscriptionName,
      locale: localeStore.currentLocale || i18n.global.locale.value || 'en-US',
      windowVisible: windowStore.windowState.isVisible,
    }
  }

  const syncStateToBackend = async (force = false) => {
    if (!initialized.value) return
    if (syncing.value) return

    const payload = buildRuntimePayload()
    const payloadKey = JSON.stringify(payload)
    if (!force && payloadKey === lastSyncedPayloadKey) {
      return
    }

    syncing.value = true
    try {
      await trayService.syncState(payload)
      lastSyncedPayloadKey = payloadKey
    } catch (error) {
      console.error('同步托盘状态到后端失败:', error)
    } finally {
      syncing.value = false
    }
  }

  const scheduleSync = (force = false) => {
    if (!initialized.value) return

    if (force) {
      cleanupTimer()
      void syncStateToBackend(true)
      return
    }

    if (syncTimer !== null) return
    syncTimer = window.setTimeout(() => {
      syncTimer = null
      void syncStateToBackend(false)
    }, TRAY_SYNC_DEBOUNCE_MS)
  }

  const syncLastVisibleRoute = async (path: string) => {
    if (!path || path === '/blank') return
    if (path === lastSyncedRoute) return

    lastSyncedRoute = path
    try {
      await trayService.setLastVisibleRoute(path)
    } catch (error) {
      console.error('同步最后可见路由失败:', error)
    }
  }

  const handleRouteRestore = async (preferredPath?: string) => {
    const targetPath =
      preferredPath && preferredPath !== '/blank'
        ? preferredPath
        : windowStore.windowState.lastVisiblePath && windowStore.windowState.lastVisiblePath !== '/blank'
          ? windowStore.windowState.lastVisiblePath
          : '/'

    try {
      if (router.currentRoute.value.path !== targetPath) {
        await router.push(targetPath)
      }
    } catch (error) {
      console.error('托盘恢复路由失败，回退首页:', error)
      try {
        if (router.currentRoute.value.path !== '/') {
          await router.push('/')
        }
      } catch (fallbackError) {
        console.error('托盘恢复首页失败:', fallbackError)
      }
    }
  }

  // 复用原先托盘代理切换逻辑，避免 TUN 提权行为回归
  const switchProxyModeFromTray = async (targetMode: ProxyMode) => {
    const previousMode = appStore.proxyMode as ProxyMode
    if (previousMode === targetMode) {
      return
    }

    if (targetMode === 'tun') {
      const platform = await systemService.getPlatformInfo().catch(() => 'unknown')

      if (platform === 'windows') {
        const isAdmin = await systemService.checkAdmin()
        if (!isAdmin) {
          try {
            await appStore.toggleTun(true)
            await appStore.saveToBackend()

            if (appStore.isRunning) {
              await kernelStore.stopKernel({ force: true })
            }
            await systemService.restartAsAdmin()
            return
          } catch (error) {
            console.error('以管理员身份重启以启用TUN失败:', error)
            await appStore.toggleTun(false)
            scheduleSync(true)
            return
          }
        } else {
          try {
            await appStore.toggleTun(true)
            await appStore.toggleSystemProxy(false)

            const applied = await kernelStore.applyProxySettings()
            if (!applied) throw new Error(kernelStore.lastError || '应用代理配置失败')

            const success = await kernelStore.restartKernel()
            if (!success) {
              throw new Error(kernelStore.lastError || '内核重启失败')
            }
          } catch (error) {
            console.error('启用TUN模式失败:', error)
            await appStore.toggleTun(false)
            scheduleSync(true)
          }
          return
        }
      }

      if (platform === 'linux' || platform === 'macos') {
        const parseSudoCode = (raw: unknown) => {
          const msg = raw instanceof Error ? raw.message : String(raw || '')
          if (msg.includes('SUDO_PASSWORD_REQUIRED')) return 'required'
          if (msg.includes('SUDO_PASSWORD_INVALID')) return 'invalid'
          return null
        }

        try {
          const status = await sudoService.getStatus()
          if (!status.supported) {
            appStore.showErrorMessage?.(i18n.global.t('home.sudoPassword.unsupported'))
            scheduleSync(true)
            return
          }

          if (!status.has_saved) {
            await windowStore.showWindow()
            await router.push('/').catch(() => {})
            const ok = await useSudoStore().requestPassword()
            if (!ok) {
              scheduleSync(true)
              return
            }
          }

          await appStore.toggleTun(true)
          await appStore.toggleSystemProxy(false)

          const applied = await kernelStore.applyProxySettings()
          if (!applied) throw new Error(kernelStore.lastError || '应用代理配置失败')

          let success = await kernelStore.restartKernel()
          if (!success) {
            const code = parseSudoCode(kernelStore.lastError)
            if (code === 'required' || code === 'invalid') {
              appStore.showWarningMessage?.(
                code === 'invalid'
                  ? i18n.global.t('home.sudoPassword.invalid')
                  : i18n.global.t('home.sudoPassword.required'),
              )

              await windowStore.showWindow()
              await router.push('/').catch(() => {})
              const ok = await useSudoStore().requestPassword()
              if (ok) {
                const appliedRetry = await kernelStore.applyProxySettings()
                if (!appliedRetry) throw new Error(kernelStore.lastError || '应用代理配置失败')
                success = await kernelStore.restartKernel()
              }
            }
          }

          if (!success) {
            throw new Error(kernelStore.lastError || '内核重启失败')
          }
        } catch (error) {
          console.error('启用TUN模式失败:', error)
          await appStore.toggleTun(false)
          scheduleSync(true)
        }
        return
      }

      appStore.showErrorMessage?.(i18n.global.t('home.sudoPassword.unsupported'))
      scheduleSync(true)
      return
    }

    try {
      if (targetMode === 'system') {
        await appStore.toggleSystemProxy(true)
        await appStore.toggleTun(false)
      } else if (targetMode === 'manual') {
        await appStore.toggleSystemProxy(false)
        await appStore.toggleTun(false)
      }

      const success = await kernelStore.switchProxyMode(targetMode)
      if (!success) {
        throw new Error(kernelStore.lastError || '代理模式切换失败')
      }
    } catch (error) {
      console.error('托盘切换代理模式失败:', error)
      if (previousMode === 'tun') {
        await appStore.toggleTun(true)
      } else if (previousMode === 'system') {
        await appStore.toggleSystemProxy(true)
      } else {
        await appStore.toggleSystemProxy(false)
        await appStore.toggleTun(false)
      }
    } finally {
      scheduleSync(true)
    }
  }

  const registerBackendEvents = async () => {
    const register = async <T = unknown>(
      eventName: string,
      handler: (payload: T) => void | Promise<void>,
    ) => {
      try {
        const unlisten = await eventService.on(eventName, handler as (payload: unknown) => void)
        eventUnlisteners.push(unlisten)
      } catch (error) {
        console.error(`注册托盘事件失败: ${eventName}`, error)
      }
    }

    await register(APP_EVENTS.trayActionShowWindow, async () => {
      await windowStore.showWindow()
    })

    await register(APP_EVENTS.trayActionNavigateLastRoute, async (payload: TrayNavigatePayload) => {
      await handleRouteRestore(payload?.path)
      scheduleSync(true)
    })

    await register(APP_EVENTS.trayActionHideWindow, async () => {
      await windowStore.hideWindow(router)
      scheduleSync(true)
    })

    await register(APP_EVENTS.trayActionRestartKernel, async () => {
      await kernelStore.restartKernel()
      scheduleSync(true)
    })

    await register(
      APP_EVENTS.trayActionSwitchProxyMode,
      async (payload: TraySwitchProxyModePayload) => {
        if (!isProxyMode(payload?.mode)) {
          return
        }
        await switchProxyModeFromTray(payload.mode)
      },
    )

    await register(APP_EVENTS.trayActionExitRequested, () => {
      appStore.clearMessages()
    })
  }

  const initTray = async () => {
    cleanupWatchers()
    cleanupEvents()
    cleanupTimer()

    lastSyncedPayloadKey = ''
    lastSyncedRoute = ''
    initialized.value = true

    await registerBackendEvents()

    registerWatcher(
      () => appStore.isRunning,
      () => scheduleSync(false),
    )

    registerWatcher(
      () => appStore.proxyMode,
      () => scheduleSync(false),
    )

    registerWatcher(
      () => [subStore.activeIndex, subStore.list.length],
      () => scheduleSync(false),
    )

    registerWatcher(
      () => localeStore.currentLocale,
      () => scheduleSync(false),
    )

    registerWatcher(
      () => windowStore.windowState.isVisible,
      () => scheduleSync(false),
    )

    registerWatcher(
      () => router.currentRoute.value.path,
      (path) => {
        const nextPath = typeof path === 'string' ? path : String(path || '')
        void syncLastVisibleRoute(nextPath)
      },
      { immediate: true },
    )

    await syncStateToBackend(true)
    return true
  }

  const destroyTray = async () => {
    cleanupWatchers()
    cleanupEvents()
    cleanupTimer()
    initialized.value = false
  }

  // 兼容旧调用方
  const refreshTrayMenu = async () => {
    scheduleSync(true)
  }

  // 兼容旧调用方
  const updateTrayTooltip = async () => {
    scheduleSync(true)
  }

  return {
    initialized,
    initTray,
    destroyTray,
    refreshTrayMenu,
    updateTrayTooltip,
  }
})
