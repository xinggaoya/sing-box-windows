import { createPinia } from 'pinia'
import type { App } from 'vue'
import { Store } from '@tauri-apps/plugin-store'
import { type PiniaPluginContext } from 'pinia'
import { storeManager } from './StoreManager'
import { memoryMonitor } from '@/utils/performance'

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

// 导出Store管理器
export { storeManager } from './StoreManager'

// 定义扩展Pinia选项的接口，类似pinia-plugin-persistedstate
interface PersistOptions {
  enabled?: boolean
  key?: string
  paths?: string[]
  excludeKeys?: string[]
}

// 扩展Pinia选项类型
declare module 'pinia' {
  interface DefineStoreOptionsBase<S, Store> {
    persist?: boolean | PersistOptions
  }
}

// Tauri Store 实例缓存
const storeCache = new Map<string, Store>()

// 获取或创建 Tauri Store 实例
async function getStore(storeName: string): Promise<Store> {
  if (!storeCache.has(storeName)) {
    try {
      // 使用Store.load静态方法加载或创建Store
      const store = await Store.load(`${storeName}.bin`)
      storeCache.set(storeName, store)
      return store
    } catch (error) {
      console.error(`获取 Tauri Store 失败:`, error)
      throw error
    }
  }
  return storeCache.get(storeName)!
}

// 创建 Tauri 持久化 Pinia 插件
function piniaTauriPersist(context: PiniaPluginContext) {
  const { store, options } = context

  // 检查store是否启用了持久化
  const persistOptions = options.persist
  if (persistOptions === false) {
    return // 如果明确禁用，则不进行持久化
  }

  // 确定存储的key
  const storeKey =
    typeof persistOptions === 'object' && persistOptions.key ? persistOptions.key : store.$id

  // 初始化时从 Tauri Store 恢复状态
  getStore(storeKey).then(async (tauriStore) => {
    try {
      const storedState = await tauriStore.get<Record<string, unknown>>(storeKey)
      if (storedState) {
        // 处理paths和excludeKeys选项
        if (typeof persistOptions === 'object') {
          const { paths, excludeKeys } = persistOptions
          let patchState: Record<string, unknown> = {}

          if (paths && paths.length > 0) {
            // 仅恢复指定路径
            paths.forEach((path) => {
              if (storedState[path] !== undefined) {
                patchState[path] = storedState[path]
              }
            })
          } else if (excludeKeys && excludeKeys.length > 0) {
            // 排除特定键
            patchState = { ...storedState }
            excludeKeys.forEach((key) => {
              delete patchState[key]
            })
          } else {
            patchState = storedState
          }

          store.$patch(patchState as any)
        } else {
          // 恢复全部状态
          store.$patch(storedState as any)
        }
      }
    } catch (error) {
      console.error(`从 Tauri Store 恢复状态失败:`, error)
    }
  })

  // 监听状态变化，保存到 Tauri Store
  store.$subscribe(async (mutation, state) => {
    try {
      const tauriStore = await getStore(storeKey)

      // 处理paths和excludeKeys选项
      let stateToStore: Record<string, unknown> = {}

      if (typeof persistOptions === 'object') {
        const { paths, excludeKeys } = persistOptions

        if (paths && paths.length > 0) {
          // 仅保存指定路径
          paths.forEach((path) => {
            if (state[path] !== undefined) {
              stateToStore[path] = state[path]
            }
          })
        } else if (excludeKeys && excludeKeys.length > 0) {
          // 排除特定键
          stateToStore = { ...JSON.parse(JSON.stringify(state)) }
          excludeKeys.forEach((key) => {
            delete stateToStore[key]
          })
        } else {
          stateToStore = JSON.parse(JSON.stringify(state))
        }
      } else {
        stateToStore = JSON.parse(JSON.stringify(state))
      }

      await tauriStore.set(storeKey, stateToStore)
      await tauriStore.save()
    } catch (error) {
      console.error(`保存状态到 Tauri Store 失败:`, error)
    }
  })
}

// 性能监控插件
function piniaPerformancePlugin(context: PiniaPluginContext) {
  const { store } = context

  // 只在开发环境监控
  if (import.meta.env.DEV) {
    // 监控store状态变化频率
    let changeCount = 0
    let lastLogTime = Date.now()

    store.$subscribe(() => {
      changeCount++
      const now = Date.now()

      // 每5秒记录一次统计
      if (now - lastLogTime > 5000) {
        if (changeCount > 0) {
          console.log(`Store "${store.$id}" changes: ${changeCount} in last 5s`)
        }
        changeCount = 0
        lastLogTime = now
      }
    })
  }
}

export function usePinia(app: App) {
  const pinia = createPinia()

  // 添加插件
  pinia.use(piniaTauriPersist)

  // 在开发环境添加性能监控
  if (import.meta.env.DEV) {
    pinia.use(piniaPerformancePlugin)
  }

  app.use(pinia)

  // 初始化Store管理器（异步）
  storeManager.initialize().catch((error) => {
    console.error('Store管理器初始化失败:', error)
  })
}
