import { createPinia } from 'pinia'
import type { App } from 'vue'
import { storeManager } from './StoreManager'

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

// 导出Store管理器
export { storeManager } from './StoreManager'

export function usePinia(app: App) {
  const pinia = createPinia()
  app.use(pinia)

  // 初始化Store管理器（异步）
  storeManager.initialize().catch((error) => {
    console.error('Store管理器初始化失败:', error)
  })
}