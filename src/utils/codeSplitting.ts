/**
 * 代码分割工具
 * 帮助优化组件加载和Bundle分割
 */

import type { Component } from 'vue'
import { bundleAnalyzer } from './bundleAnalyzer'

// 组件加载优先级
export enum LoadPriority {
  IMMEDIATE = 'immediate', // 立即加载
  HIGH = 'high', // 高优先级
  NORMAL = 'normal', // 普通优先级
  LOW = 'low', // 低优先级
  IDLE = 'idle', // 空闲时加载
}

// 组件配置
interface ComponentConfig {
  priority: LoadPriority
  preload?: boolean
  retries?: number
  timeout?: number
  dependencies?: string[]
}

// 组件注册信息
interface ComponentRegistration {
  name: string
  loader: () => Promise<Component>
  config: ComponentConfig
  loaded: boolean
  loading: boolean
  error: Error | null
}

// 代码分割管理器
export class CodeSplittingManager {
  private components = new Map<string, ComponentRegistration>()
  private loadQueue: string[] = []
  private isProcessingQueue = false
  private loadConcurrency = 3 // 同时加载的组件数量

  /**
   * 注册组件
   */
  registerComponent(
    name: string,
    loader: () => Promise<Component>,
    config: ComponentConfig = { priority: LoadPriority.NORMAL },
  ) {
    this.components.set(name, {
      name,
      loader,
      config,
      loaded: false,
      loading: false,
      error: null,
    })

    // 如果是立即加载，直接加载
    if (config.priority === LoadPriority.IMMEDIATE) {
      this.loadComponent(name)
    }

    // 如果启用预加载，添加到队列
    if (config.preload) {
      this.addToQueue(name)
    }
  }

  /**
   * 批量注册组件
   */
  registerComponents(
    components: Record<
      string,
      {
        loader: () => Promise<Component>
        config?: ComponentConfig
      }
    >,
  ) {
    Object.entries(components).forEach(([name, { loader, config }]) => {
      this.registerComponent(name, loader, config)
    })
  }

  /**
   * 加载组件
   */
  async loadComponent(name: string): Promise<Component | null> {
    const registration = this.components.get(name)
    if (!registration) {
      console.warn(`组件 "${name}" 未注册`)
      return null
    }

    if (registration.loaded) {
      return registration.loader() // 直接返回已加载的组件
    }

    if (registration.loading) {
      // 等待正在加载的组件
      return this.waitForComponent(name)
    }

    registration.loading = true
    registration.error = null

    try {
      const startTime = performance.now()

      // 设置超时
      const timeoutPromise = new Promise<never>((_, reject) => {
        setTimeout(() => {
          reject(new Error(`组件 "${name}" 加载超时`))
        }, registration.config.timeout || 10000)
      })

      // 竞争加载和超时
      const component = await Promise.race([registration.loader(), timeoutPromise])

      const loadTime = performance.now() - startTime

      // 记录到Bundle分析器
      bundleAnalyzer.recordModuleLoad(
        name,
        0, // 组件大小需要实际测量
        registration.config.dependencies || [],
      )

      registration.loaded = true
      registration.loading = false

      console.log(`组件 "${name}" 加载成功，耗时 ${loadTime.toFixed(2)}ms`)
      return component
    } catch (error) {
      registration.loading = false
      registration.error = error instanceof Error ? error : new Error('组件加载失败')

      console.error(`组件 "${name}" 加载失败:`, error)

      // 重试逻辑
      const maxRetries = registration.config.retries || 0
      if (maxRetries > 0) {
        console.log(`尝试重新加载组件 "${name}"...`)
        registration.config.retries = maxRetries - 1
        return this.loadComponent(name)
      }

      throw registration.error
    }
  }

  /**
   * 等待组件加载完成
   */
  private async waitForComponent(name: string): Promise<Component | null> {
    return new Promise((resolve) => {
      const checkLoaded = () => {
        const registration = this.components.get(name)
        if (!registration) {
          resolve(null)
          return
        }

        if (registration.loaded) {
          registration
            .loader()
            .then(resolve)
            .catch(() => resolve(null))
        } else if (!registration.loading) {
          resolve(null)
        } else {
          setTimeout(checkLoaded, 100)
        }
      }
      checkLoaded()
    })
  }

  /**
   * 添加到加载队列
   */
  private addToQueue(name: string) {
    if (!this.loadQueue.includes(name)) {
      const registration = this.components.get(name)
      if (registration && !registration.loaded) {
        // 根据优先级插入队列
        const priority = registration.config.priority
        let insertIndex = this.loadQueue.length

        for (let i = 0; i < this.loadQueue.length; i++) {
          const queuedComponent = this.components.get(this.loadQueue[i])
          if (
            queuedComponent &&
            this.getPriorityValue(priority) > this.getPriorityValue(queuedComponent.config.priority)
          ) {
            insertIndex = i
            break
          }
        }

        this.loadQueue.splice(insertIndex, 0, name)
        this.processQueue()
      }
    }
  }

  /**
   * 获取优先级数值
   */
  private getPriorityValue(priority: LoadPriority): number {
    switch (priority) {
      case LoadPriority.IMMEDIATE:
        return 5
      case LoadPriority.HIGH:
        return 4
      case LoadPriority.NORMAL:
        return 3
      case LoadPriority.LOW:
        return 2
      case LoadPriority.IDLE:
        return 1
      default:
        return 0
    }
  }

  /**
   * 处理加载队列
   */
  private async processQueue() {
    if (this.isProcessingQueue) return

    this.isProcessingQueue = true

    while (this.loadQueue.length > 0) {
      // 取出最多loadConcurrency个组件并发加载
      const batch = this.loadQueue.splice(0, this.loadConcurrency)

      const loadPromises = batch.map(async (name) => {
        try {
          await this.loadComponent(name)
        } catch (error) {
          console.error(`队列中组件 "${name}" 加载失败:`, error)
        }
      })

      await Promise.allSettled(loadPromises)

      // 检查是否应该使用requestIdleCallback
      const hasIdlePriorityItems = this.loadQueue.some((name) => {
        const registration = this.components.get(name)
        return registration?.config.priority === LoadPriority.IDLE
      })

      if (hasIdlePriorityItems && 'requestIdleCallback' in window) {
        await new Promise((resolve) => {
          window.requestIdleCallback(resolve as IdleRequestCallback)
        })
      }
    }

    this.isProcessingQueue = false
  }

  /**
   * 预加载指定组件
   */
  async preloadComponents(names: string[]) {
    const promises = names.map((name) => {
      this.addToQueue(name)
      return this.loadComponent(name).catch((error) => {
        console.warn(`预加载组件 "${name}" 失败:`, error)
        return null
      })
    })

    await Promise.allSettled(promises)
  }

  /**
   * 预加载路由相关组件
   */
  async preloadForRoute(routeName: string) {
    const routeComponents = this.getComponentsForRoute(routeName)
    await this.preloadComponents(routeComponents)
  }

  /**
   * 获取路由相关组件
   */
  private getComponentsForRoute(routeName: string): string[] {
    // 这里可以根据路由名称返回相关组件
    // 实际实现需要根据具体的路由配置
    const routeComponentMap: Record<string, string[]> = {
      Home: ['HomeView', 'ProxyCard', 'StatusCard'],
      Proxy: ['ProxyView', 'ProxyList', 'ProxySettings'],
      Sub: ['SubView', 'SubList', 'SubEditor'],
      Log: ['LogView', 'LogViewer'],
      Setting: ['SettingView', 'ThemeSettings', 'NetworkSettings'],
      Rules: ['RulesView', 'RuleEditor'],
      Connections: ['ConnectionsView', 'ConnectionList'],
    }

    return routeComponentMap[routeName] || []
  }

  /**
   * 获取加载统计
   */
  getStats() {
    const total = this.components.size
    const loaded = Array.from(this.components.values()).filter((c) => c.loaded).length
    const loading = Array.from(this.components.values()).filter((c) => c.loading).length
    const errors = Array.from(this.components.values()).filter((c) => c.error).length

    return {
      total,
      loaded,
      loading,
      errors,
      loadedPercentage: total > 0 ? Math.round((loaded / total) * 100) : 0,
      queueLength: this.loadQueue.length,
    }
  }

  /**
   * 获取组件状态
   */
  getComponentStatus(name: string) {
    const registration = this.components.get(name)
    if (!registration) return null

    return {
      name: registration.name,
      loaded: registration.loaded,
      loading: registration.loading,
      error: registration.error?.message,
      priority: registration.config.priority,
    }
  }

  /**
   * 重置组件状态
   */
  resetComponent(name: string) {
    const registration = this.components.get(name)
    if (registration) {
      registration.loaded = false
      registration.loading = false
      registration.error = null
    }
  }

  /**
   * 清理所有组件
   */
  cleanup() {
    this.components.clear()
    this.loadQueue = []
    this.isProcessingQueue = false
  }
}

// 创建全局代码分割管理器
export const codeSplittingManager = new CodeSplittingManager()

// 便捷的组件懒加载函数
export function defineAsyncComponent(
  name: string,
  loader: () => Promise<Component>,
  config?: ComponentConfig,
) {
  codeSplittingManager.registerComponent(name, loader, config)

  return async () => {
    const component = await codeSplittingManager.loadComponent(name)
    if (!component) {
      throw new Error(`组件 "${name}" 加载失败`)
    }
    return component
  }
}

// Vue路由懒加载辅助函数
export function lazyRouteComponent(
  componentPath: string,
  priority: LoadPriority = LoadPriority.NORMAL,
) {
  const componentName = componentPath.split('/').pop()?.replace('.vue', '') || 'UnknownComponent'

  return defineAsyncComponent(componentName, () => import(/* @vite-ignore */ componentPath), {
    priority,
  })
}
