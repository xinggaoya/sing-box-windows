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
import type { TrayNavigatePayload, TrayToggleProxyFeaturePayload } from '@/types/events'
import { useAppStore, useKernelStore, useLocaleStore, useSubStore, useSudoStore } from '@/stores'
import { useWindowStore } from '@/stores/app/WindowStore'
import { prepareTunAndRestartAsAdmin, useTunToggle } from '@/composables/useTunToggle'

const TRAY_SYNC_DEBOUNCE_MS = 150

const isTrayTogglePayload = (value: unknown): value is TrayToggleProxyFeaturePayload => {
  if (!value || typeof value !== 'object') {
    return false
  }

  const payload = value as Partial<TrayToggleProxyFeaturePayload>
  return (
    (payload.feature === 'systemProxy' || payload.feature === 'tun') &&
    typeof payload.enabled === 'boolean'
  )
}

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
      systemProxyEnabled: appStore.systemProxyEnabled,
      tunEnabled: appStore.tunEnabled,
      activeSubscriptionName,
      locale: localeStore.currentLocale || i18n.global.locale.value || 'en-US',
      windowVisible: windowStore.windowState.isVisible,
      closeBehavior: appStore.trayCloseBehavior,
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
        : windowStore.windowState.lastVisiblePath &&
            windowStore.windowState.lastVisiblePath !== '/blank'
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

  const applySystemProxyToggleFromTray = async (enabled: boolean) => {
    const previousSystemProxyEnabled = appStore.systemProxyEnabled
    if (previousSystemProxyEnabled === enabled) {
      return
    }

    try {
      await appStore.toggleSystemProxy(enabled)

      const success = await kernelStore.applyProxySettings({
        system_proxy_enabled: enabled,
      })
      if (!success) {
        throw new Error(kernelStore.lastError || i18n.global.t('notification.applyProxyFailed'))
      }
    } catch (error) {
      console.error('托盘切换系统代理失败:', error)
      await appStore.toggleSystemProxy(previousSystemProxyEnabled)
    } finally {
      scheduleSync(true)
    }
  }

  // 与首页共用同一个 TUN 切换状态机（回滚顺序、sudo 重试次数等细节只有一份实现）。
  const { applyTunSwitch } = useTunToggle({
    messages: {
      applyFailed: i18n.global.t('notification.applyProxyFailed'),
      restartFailed: i18n.global.t('notification.kernelRestartFailed'),
    },
    ensurePrivilege: async (target) => {
      if (!target) return 'proceed'
      const platform = await systemService.getPlatformInfo().catch(() => 'unknown')

      if (platform === 'windows') {
        const isAdmin = await systemService.checkAdmin()
        if (isAdmin) return 'proceed'
        // 无管理员权限：保存配置并以管理员身份重启，由重启接手
        const escalated = await prepareTunAndRestartAsAdmin()
        if (!escalated) {
          console.error('以管理员身份重启以启用TUN失败')
        }
        return 'escalated'
      }

      if (platform === 'linux' || platform === 'macos') {
        const status = await sudoService.getStatus()
        if (!status.supported) {
          appStore.showErrorMessage?.(i18n.global.t('home.sudoPassword.unsupported'))
          return 'proceed'
        }
        if (!status.has_saved) {
          // 托盘没有输入框上下文，先把用户引导到首页
          await windowStore.showWindow()
          await router.push('/').catch(() => {})
          const ok = await useSudoStore().requestPassword()
          if (!ok) {
            appStore.showErrorMessage?.(i18n.global.t('home.sudoPassword.required'))
          }
        }
        return 'proceed'
      }

      appStore.showErrorMessage?.(i18n.global.t('home.sudoPassword.unsupported'))
      return 'proceed'
    },
    handleSudoFailure: async (code) => {
      appStore.showWarningMessage?.(
        code === 'invalid'
          ? i18n.global.t('home.sudoPassword.invalid')
          : i18n.global.t('home.sudoPassword.required'),
      )
      await windowStore.showWindow()
      await router.push('/').catch(() => {})
      return useSudoStore().requestPassword()
    },
    applyOverrides: (target) => ({ tun_enabled: target }),
  })

  const applyTunToggleFromTray = async (enabled: boolean) => {
    if (appStore.tunEnabled === enabled) return

    try {
      const outcome = await applyTunSwitch(enabled)
      if (outcome.status === 'failed') {
        console.error(`托盘切换TUN(${enabled})失败:`, outcome.failure.text)
        appStore.showErrorMessage?.(outcome.failure.text)
      }
    } finally {
      scheduleSync(true)
    }
  }

  const toggleProxyFeatureFromTray = async (payload: TrayToggleProxyFeaturePayload) => {
    if (payload.feature === 'systemProxy') {
      await applySystemProxyToggleFromTray(payload.enabled)
      return
    }

    await applyTunToggleFromTray(payload.enabled)
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
      async (payload: TrayToggleProxyFeaturePayload) => {
        if (!isTrayTogglePayload(payload)) {
          return
        }
        await toggleProxyFeatureFromTray(payload)
      },
    )

    await register(APP_EVENTS.trayRuntimeStateUpdated, async () => {
      await appStore.loadFromBackend()
      scheduleSync(true)
    })

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
      () => [appStore.systemProxyEnabled, appStore.tunEnabled],
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
      () => appStore.trayCloseBehavior,
      () => scheduleSync(true),
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
    const pendingRoute = await trayService.consumePendingRestoreRoute().catch((error) => {
      console.error('获取待恢复路由失败:', error)
      return null
    })
    if (pendingRoute?.path) {
      await handleRouteRestore(pendingRoute.path)
      scheduleSync(true)
    }
    const pendingProxyToggle = await trayService.consumePendingProxyToggle().catch((error) => {
      console.error('获取待处理托盘代理切换失败:', error)
      return null
    })
    if (pendingProxyToggle && isTrayTogglePayload(pendingProxyToggle)) {
      await toggleProxyFeatureFromTray(pendingProxyToggle)
    }
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
