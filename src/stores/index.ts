import { createPinia } from 'pinia'
import type { App } from 'vue'

// 导出应用相关Store
export * from './app/AppStore'
export * from './app/ThemeStore'
export * from './app/LocaleStore'
export * from './app/WindowStore'
export * from './app/UpdateStore'
export * from './app/SudoStore'

// 导出内核相关Store
export * from './kernel/KernelStore'
export * from './kernel/ProxyStore'
export * from './kernel/ConnectionStore'
export * from './kernel/TrafficStore'
export * from './kernel/LogStore'

// 注：这里曾有一句“规则页 UI 已隐藏”的注释，但它与事实不符 —— `router/index.ts`
// 注册了 `/rules` 路由，`views/RulesView.vue` 存在且可达。
// 规则页直接通过 `invokeWithAppContext('get_rules')` 取数，不经 store；
// 若后续要做规则开关等写操作，再在此补 RulesStore。

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
