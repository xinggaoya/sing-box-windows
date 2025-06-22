/**
 * Store 管理器 - 实现按需加载和内存优化
 */

import type { App } from 'vue'
import { createPinia } from 'pinia'

// Store类型定义
export type StoreType =
  | 'app'
  | 'theme'
  | 'locale'
  | 'window'
  | 'update'
  | 'kernel'
  | 'kernel-runtime'
  | 'proxy'
  | 'connection'
  | 'traffic'
  | 'log'
  | 'subscription'
  | 'tray'

// Store实例缓存
const storeInstances = new Map<StoreType, unknown>()
const storeInitializers = new Map<StoreType, () => Promise<unknown>>()

// 注册Store初始化器
function registerStoreInitializers() {
  // 应用相关Store
  storeInitializers.set('app', async () => {
    const { useAppStore } = await import('./app/AppStore')
    const store = useAppStore()
    storeInstances.set('app', store)
    // 延迟初始化Store，避免循环依赖
    setTimeout(() => {
      if ('initializeStore' in store && typeof store.initializeStore === 'function') {
        store.initializeStore().catch(console.error)
      }
    }, 0)
    return store
  })

  storeInitializers.set('theme', async () => {
    const { useThemeStore } = await import('./app/ThemeStore')
    const store = useThemeStore()
    storeInstances.set('theme', store)
    return store
  })

  storeInitializers.set('locale', async () => {
    const { useLocaleStore } = await import('./app/LocaleStore')
    const store = useLocaleStore()
    storeInstances.set('locale', store)
    return store
  })

  storeInitializers.set('window', async () => {
    const { useWindowStore } = await import('./app/WindowStore')
    const store = useWindowStore()
    storeInstances.set('window', store)
    return store
  })

  storeInitializers.set('update', async () => {
    const { useUpdateStore } = await import('./app/UpdateStore')
    const store = useUpdateStore()
    storeInstances.set('update', store)
    return store
  })

  // 内核相关Store
  storeInitializers.set('kernel', async () => {
    const { useKernelStore } = await import('./kernel/KernelStore')
    const store = useKernelStore()
    storeInstances.set('kernel', store)

    // 延迟初始化Store，重置临时数据
    setTimeout(() => {
      if ('initializeStore' in store && typeof store.initializeStore === 'function') {
        store.initializeStore().catch(console.error)
      }
    }, 0)

    return store
  })

  storeInitializers.set('kernel-runtime', async () => {
    const { useKernelRuntimeStore } = await import('./kernel/KernelRuntimeStore')
    const store = useKernelRuntimeStore()
    storeInstances.set('kernel-runtime', store)

    // 延迟初始化Store
    setTimeout(() => {
      if ('initializeStore' in store && typeof store.initializeStore === 'function') {
        store.initializeStore()
      }
    }, 0)

    return store
  })

  storeInitializers.set('proxy', async () => {
    const { useProxyStore } = await import('./kernel/ProxyStore')
    const store = useProxyStore()
    storeInstances.set('proxy', store)
    return store
  })

  storeInitializers.set('connection', async () => {
    const { useConnectionStore } = await import('./kernel/ConnectionStore')
    const store = useConnectionStore()
    storeInstances.set('connection', store)
    // 延迟初始化Store，避免循环依赖
    setTimeout(() => {
      if ('initializeStore' in store && typeof store.initializeStore === 'function') {
        store.initializeStore()
      }
    }, 0)
    return store
  })

  storeInitializers.set('traffic', async () => {
    const { useTrafficStore } = await import('./kernel/TrafficStore')
    const store = useTrafficStore()
    storeInstances.set('traffic', store)
    // 延迟初始化Store，避免循环依赖
    setTimeout(() => {
      if ('initializeStore' in store && typeof store.initializeStore === 'function') {
        store.initializeStore()
      }
    }, 0)
    return store
  })

  storeInitializers.set('log', async () => {
    const { useLogStore } = await import('./kernel/LogStore')
    const store = useLogStore()
    storeInstances.set('log', store)
    // 延迟初始化Store，避免循环依赖
    setTimeout(() => {
      if ('initializeStore' in store && typeof store.initializeStore === 'function') {
        store.initializeStore()
      }
    }, 0)
    return store
  })

  // 订阅相关Store
  storeInitializers.set('subscription', async () => {
    const { useSubStore } = await import('./subscription/SubStore')
    const store = useSubStore()
    storeInstances.set('subscription', store)
    return store
  })

  // 系统托盘Store
  storeInitializers.set('tray', async () => {
    const { useTrayStore } = await import('./tray/TrayStore')
    const store = useTrayStore()
    storeInstances.set('tray', store)
    return store
  })
}

export class StoreManager {
  private static instance: StoreManager
  private coreStores: Set<StoreType> = new Set(['app', 'theme', 'locale'])
  private loadedStores: Set<StoreType> = new Set()

  static getInstance(): StoreManager {
    if (!StoreManager.instance) {
      StoreManager.instance = new StoreManager()
    }
    return StoreManager.instance
  }

  /**
   * 初始化Store管理器
   */
  async initialize() {
    registerStoreInitializers()

    // 预加载核心Store
    await this.loadCoreStores()

    // 清理事件会在应用卸载时被调用
  }

  /**
   * 加载核心Store（应用启动时必需的）
   */
  private async loadCoreStores() {
    const promises = Array.from(this.coreStores).map((storeType) => this.loadStore(storeType))
    await Promise.all(promises)
  }

  /**
   * 按需加载Store
   */
  async loadStore<T = unknown>(storeType: StoreType): Promise<T> {
    if (storeInstances.has(storeType)) {
      return storeInstances.get(storeType) as T
    }

    const initializer = storeInitializers.get(storeType)
    if (!initializer) {
      throw new Error(`Store type "${storeType}" not found`)
    }

    const store = await initializer()
    this.loadedStores.add(storeType)
    console.log(`Store "${storeType}" loaded`)
    return store as T
  }

  /**
   * 预加载指定Store（用于路由切换前预加载）
   */
  async preloadStores(storeTypes: StoreType[]) {
    const promises = storeTypes
      .filter((type) => !this.loadedStores.has(type))
      .map((type) => this.loadStore(type))

    await Promise.allSettled(promises)
  }

  /**
   * 获取已加载的Store
   */
  getLoadedStore<T = unknown>(storeType: StoreType): T | null {
    return (storeInstances.get(storeType) as T) || null
  }

  /**
   * 检查Store是否已加载
   */
  isStoreLoaded(storeType: StoreType): boolean {
    return this.loadedStores.has(storeType)
  }

  /**
   * 清理未使用的Store（谨慎使用）
   */
  cleanup() {
    // 保留核心Store，清理其他Store
    for (const [storeType] of storeInstances) {
      if (!this.coreStores.has(storeType)) {
        // 调用Store的清理方法
        const store = storeInstances.get(storeType)
        if (
          store &&
          typeof store === 'object' &&
          'cleanupStore' in store &&
          typeof store.cleanupStore === 'function'
        ) {
          store.cleanupStore()
        }
        storeInstances.delete(storeType)
        this.loadedStores.delete(storeType)
      }
    }
  }

  /**
   * 清理所有Store（应用卸载时使用）
   */
  cleanupAll() {
    for (const [storeType] of storeInstances) {
      // 调用Store的清理方法
      const store = storeInstances.get(storeType)
      if (
        store &&
        typeof store === 'object' &&
        'cleanupStore' in store &&
        typeof store.cleanupStore === 'function'
      ) {
        store.cleanupStore()
      }
    }
    storeInstances.clear()
    this.loadedStores.clear()
  }

  /**
   * 获取内存使用统计
   */
  getStats() {
    return {
      totalStores: storeInitializers.size,
      loadedStores: this.loadedStores.size,
      coreStores: this.coreStores.size,
      loadedStoreTypes: Array.from(this.loadedStores),
    }
  }
}

// 路由到Store的映射关系
export const routeStoreMap: Record<string, StoreType[]> = {
  '/': ['app', 'kernel', 'traffic'],
  '/proxy': ['proxy', 'kernel'],
  '/sub': ['subscription'],
  '/log': ['log', 'kernel'],
  '/setting': ['app', 'theme', 'locale', 'update'],
  '/rules': ['proxy', 'kernel'],
  '/connections': ['connection', 'traffic'],
}

// 导出单例实例
export const storeManager = StoreManager.getInstance()
