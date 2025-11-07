export * from './kernel'
export * from './proxy'
export * from './subscription'
export * from './system'
export * from './config'
export { invokeWithAppContext, withAppStore } from './invoke-client'

import { kernelApi } from './kernel'
import { proxyApi } from './proxy'
import { subscriptionApi } from './subscription'
import { systemApi } from './system'
import { configApi } from './config'

export const tauriApi = {
  kernel: kernelApi,
  proxy: proxyApi,
  subscription: subscriptionApi,
  system: systemApi,
  config: configApi,
  update: systemApi,
  downloadLatestKernel: systemApi.downloadLatestKernel,
  isKernelRunning: kernelApi.isKernelRunning,
  getRules: proxyApi.getRules,
  addManualSubscription: subscriptionApi.addManualSubscription,
  getCurrentConfig: subscriptionApi.getCurrentConfig,
  openDevtools: systemApi.openDevtools,
  downloadAndInstallUpdate: systemApi.downloadAndInstallUpdate
}

export const kernel = kernelApi
export const proxy = proxyApi
export const subscription = subscriptionApi
export const system = systemApi
export const config = configApi
