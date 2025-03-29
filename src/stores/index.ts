import { createPinia } from 'pinia'
import type { App } from 'vue'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'

// 导出所有Store
export * from './AppStore'
export * from './infoStore'
export * from './SubStore'
export * from './TrayStore'

export function usePinia(app: App) {
  const pinia = createPinia()
  pinia.use(piniaPluginPersistedstate)

  app.use(pinia)
}
