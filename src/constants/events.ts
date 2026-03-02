import type { KernelStatus, KernelDownloadPayload } from '@/services/kernel-service'
import type {
  TrafficDataPayload,
  ConnectionsDataPayload,
  LogEventPayload,
  MemoryStatsPayload,
  KernelHealthPayload,
  UpdateAvailablePayload,
  UpgradeSubscriptionRefreshFailedPayload,
  KernelFailurePayload,
  KernelOperationFailedPayload,
  TrayNavigatePayload,
  TraySwitchProxyModePayload,
} from '@/types/events'

export const APP_EVENTS = {
  trafficData: 'traffic-data',
  memoryData: 'memory-data',
  logData: 'log-data',
  connectionsData: 'connections-data',
  kernelHealth: 'kernel-health',
  kernelStatusChanged: 'kernel-status-changed',
  kernelReady: 'kernel-ready',
  kernelError: 'kernel-error',
  kernelStarting: 'kernel-starting',
  kernelStarted: 'kernel-started',
  kernelStopped: 'kernel-stopped',
  kernelOperationStarted: 'kernel-operation-started',
  kernelOperationFinished: 'kernel-operation-finished',
  kernelOperationFailed: 'kernel-operation-failed',
  kernelDownloadProgress: 'kernel-download-progress',
  updateProgress: 'update-progress',
  updateAvailable: 'update-available',
  subscriptionUpdated: 'subscription-updated',
  upgradeSubscriptionRefreshFailed: 'upgrade-subscription-refresh-failed',
  trayActionShowWindow: 'tray-action-show-window',
  trayActionHideWindow: 'tray-action-hide-window',
  trayActionNavigateLastRoute: 'tray-action-navigate-last-route',
  trayActionExitRequested: 'tray-action-exit-requested',
  trayActionRestartKernel: 'tray-action-restart-kernel',
  trayActionSwitchProxyMode: 'tray-action-switch-proxy-mode',
} as const

export type AppEventName = typeof APP_EVENTS[keyof typeof APP_EVENTS]

export type AppEventPayloads = {
  [APP_EVENTS.trafficData]: TrafficDataPayload
  [APP_EVENTS.memoryData]: MemoryStatsPayload
  [APP_EVENTS.logData]: LogEventPayload
  [APP_EVENTS.connectionsData]: ConnectionsDataPayload
  [APP_EVENTS.kernelHealth]: KernelHealthPayload
  [APP_EVENTS.kernelStatusChanged]: KernelStatus
  [APP_EVENTS.kernelReady]: void
  [APP_EVENTS.kernelError]: KernelFailurePayload
  [APP_EVENTS.kernelStarting]: unknown
  [APP_EVENTS.kernelStarted]: unknown
  [APP_EVENTS.kernelStopped]: unknown
  [APP_EVENTS.kernelOperationStarted]: unknown
  [APP_EVENTS.kernelOperationFinished]: unknown
  [APP_EVENTS.kernelOperationFailed]: KernelOperationFailedPayload
  [APP_EVENTS.kernelDownloadProgress]: KernelDownloadPayload
  [APP_EVENTS.updateProgress]: {
    status: 'downloading' | 'completed' | 'error' | 'installing'
    progress: number
    message?: string
  }
  [APP_EVENTS.updateAvailable]: UpdateAvailablePayload
  [APP_EVENTS.subscriptionUpdated]: unknown
  [APP_EVENTS.upgradeSubscriptionRefreshFailed]: UpgradeSubscriptionRefreshFailedPayload
  [APP_EVENTS.trayActionShowWindow]: unknown
  [APP_EVENTS.trayActionHideWindow]: unknown
  [APP_EVENTS.trayActionNavigateLastRoute]: TrayNavigatePayload
  [APP_EVENTS.trayActionExitRequested]: unknown
  [APP_EVENTS.trayActionRestartKernel]: unknown
  [APP_EVENTS.trayActionSwitchProxyMode]: TraySwitchProxyModePayload
}
