// 此文件已被弃用，请使用 @/composables/useCleanup.ts 替代
// 保留此文件仅为向后兼容
export { useCleanup, useMemoryMonitor } from '@/composables/useCleanup'

// 向后兼容的假导出
export const globalMemoryManager = {
  cleanup: () => console.warn('globalMemoryManager 已弃用，请使用 useCleanup'),
  logMemoryUsage: (_label?: string) => console.warn('logMemoryUsage 已弃用，请使用 useMemoryMonitor'),
  initialize: () => console.warn('initialize 已弃用，请使用 useCleanup'),
  startGlobalMemoryMonitoring: () => console.warn('startGlobalMemoryMonitoring 已弃用，请使用 useMemoryMonitor'),
  stopGlobalMemoryMonitoring: () => console.warn('stopGlobalMemoryMonitoring 已弃用'),
  cleanupAll: () => console.warn('cleanupAll 已弃用，请使用 useCleanup'),
  cleanupAllStores: () => console.warn('cleanupAllStores 已弃用，请使用 storeManager'),
  start: () => console.warn('start 已弃用'),
  stop: () => console.warn('stop 已弃用')
}

export const webSocketCleaner = {
  cleanup: () => console.warn('webSocketCleaner 已弃用，请使用 useCleanup'),
  addCleanup: (_fn: () => void) => console.warn('addCleanup 已弃用，请使用 useCleanup')
}

export const temporaryStoreManager = {
  cleanup: () => console.warn('temporaryStoreManager 已弃用，请使用 useCleanup'),
  addCleanup: (_fn: () => void) => console.warn('addCleanup 已弃用，请使用 useCleanup'),
  registerStore: (name: string, _store?: any) => {
    console.warn(`registerStore 已弃用: ${name}`)
  },
  unregisterStore: (_store?: any) => console.warn('unregisterStore 已弃用'),
  registerCleanup: (_fn: () => void) => console.warn('registerCleanup 已弃用，请使用 useCleanup'),
  initialize: () => console.warn('initialize 已弃用'),
  destroy: () => console.warn('destroy 已弃用')
}

export const StoreCleaner = {
  cleanup: () => console.warn('StoreCleaner 已弃用，请使用 useCleanup'),
  addCleanup: (_fn: () => void) => console.warn('addCleanup 已弃用，请使用 useCleanup')
}