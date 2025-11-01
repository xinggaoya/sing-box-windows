import { createPinia } from 'pinia'
import type { App } from 'vue'

// 导出应用相关Store
export * from './app/AppStore'
export * from './app/ThemeStore'
export * from './app/LocaleStore'
export * from './app/WindowStore'
export * from './app/UpdateStore'

// 导出内核相关Store
export * from './kernel/KernelStore'
export * from './kernel/KernelRuntimeStore'
export * from './kernel/ProxyStore'
export * from './kernel/ConnectionStore'
export * from './kernel/TrafficStore'
export * from './kernel/LogStore'

// 导出订阅相关Store
export * from './subscription/SubStore'

// 导出系统托盘Store
export * from './tray/TrayStore'

/**
 * Sets up and installs the Pinia store.
 * @param app The Vue application instance.
 */
export function usePinia(app: App) {
  const pinia = createPinia()
  app.use(pinia)
}
