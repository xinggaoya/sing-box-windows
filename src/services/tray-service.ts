import { invokeWithAppContext } from './invoke-client'

export interface TrayRuntimeStatePayload {
  kernelRunning: boolean
  proxyMode: 'system' | 'tun' | 'manual'
  activeSubscriptionName?: string | null
  locale: string
  windowVisible: boolean
}

function sanitizeStatePayload(payload: TrayRuntimeStatePayload): TrayRuntimeStatePayload {
  return {
    kernelRunning: payload.kernelRunning,
    proxyMode: payload.proxyMode,
    activeSubscriptionName: payload.activeSubscriptionName?.trim() || null,
    locale: payload.locale?.trim() || 'en-US',
    windowVisible: payload.windowVisible,
  }
}

export const trayService = {
  syncState(payload: TrayRuntimeStatePayload) {
    return invokeWithAppContext<void>(
      'tray_sync_state',
      { payload: sanitizeStatePayload(payload) },
      { skipDataRestore: true },
    )
  },

  setLastVisibleRoute(path: string) {
    return invokeWithAppContext<void>(
      'tray_set_last_visible_route',
      { path },
      { skipDataRestore: true },
    )
  },

  showMainWindow() {
    return invokeWithAppContext<void>('tray_show_main_window', undefined, {
      skipDataRestore: true,
    })
  },

  hideMainWindow() {
    return invokeWithAppContext<void>('tray_hide_main_window', undefined, {
      skipDataRestore: true,
    })
  },

  requestExit() {
    return invokeWithAppContext<void>('tray_request_app_exit', undefined, {
      skipDataRestore: true,
    })
  },
}
