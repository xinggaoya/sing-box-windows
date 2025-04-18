import { createPinia } from 'pinia'
import type { App } from 'vue'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

// 导出应用相关Store
export * from './app/AppStore'
export * from './app/ThemeStore'
export * from './app/LocaleStore'
export * from './app/WindowStore'
export * from './app/UpdateStore'

// 导出内核相关Store
export * from './kernel/KernelStore'
export * from './kernel/ProxyStore'
export * from './kernel/ConnectionStore'
export * from './kernel/TrafficStore'
export * from './kernel/LogStore'

// 导出订阅相关Store
export * from './subscription/SubStore'

// 导出系统托盘Store
export * from './tray/TrayStore'

export function usePinia(app: App) {
  const pinia = createPinia()
  pinia.use(piniaPluginPersistedstate)

  app.use(pinia)
}
