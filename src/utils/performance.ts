/**
 * 性能优化工具
 */

import { nextTick, onScopeDispose } from 'vue'
import type { Component } from 'vue'

// 内存信息接口
interface MemoryInfo {
  readonly usedJSHeapSize: number
  readonly totalJSHeapSize: number
  readonly jsHeapSizeLimit: number
}

// 扩展Performance接口
interface PerformanceWithMemory extends Performance {
  readonly memory?: MemoryInfo
}

// 事件监听器管理器
export class EventListenerManager {
  private listeners: Set<() => void> = new Set()

  /**
   * 添加事件监听器
   */
  add(cleanup: () => void) {
    this.listeners.add(cleanup)
  }

  /**
   * 清理所有事件监听器
   */
  cleanup() {
    this.listeners.forEach((cleanup) => cleanup())
    this.listeners.clear()
  }

  /**
   * 在组件卸载时自动清理
   */
  autoCleanup() {
    onScopeDispose(() => {
      this.cleanup()
    })
  }
}

// 组件预加载管理器
export class ComponentPreloader {
  private preloadedComponents = new Map<string, Promise<Component>>()
  private observer: IntersectionObserver | null = null

  constructor() {
    this.setupIntersectionObserver()
  }

  /**
   * 设置路口观察器用于懒加载
   */
  private setupIntersectionObserver() {
    if (typeof window !== 'undefined' && 'IntersectionObserver' in window) {
      this.observer = new IntersectionObserver(
        (entries) => {
          entries.forEach((entry) => {
            if (entry.isIntersecting) {
              const target = entry.target as HTMLElement
              const componentName = target.dataset.preload
              if (componentName && !this.preloadedComponents.has(componentName)) {
                this.preloadComponent(componentName)
              }
            }
          })
        },
        {
          rootMargin: '50px', // 提前50px开始预加载
        },
      )
    }
  }

  /**
   * 预加载组件
   */
  async preloadComponent(componentName: string): Promise<Component | null> {
    if (this.preloadedComponents.has(componentName)) {
      return this.preloadedComponents.get(componentName)!
    }

    try {
      let componentPromise: Promise<Component>

      switch (componentName) {
        case 'HomeView':
          componentPromise = import('@/views/HomeView.vue')
          break
        case 'ProxyView':
          componentPromise = import('@/views/ProxyView.vue')
          break
        case 'SubView':
          componentPromise = import('@/views/SubView.vue')
          break
        case 'LogView':
          componentPromise = import('@/views/LogView.vue')
          break
        case 'SettingView':
          componentPromise = import('@/views/SettingView.vue')
          break
        case 'RulesView':
          componentPromise = import('@/views/RulesView.vue')
          break
        case 'ConnectionsView':
          componentPromise = import('@/views/ConnectionsView.vue')
          break
        default:
          return null
      }

      this.preloadedComponents.set(componentName, componentPromise)
      await componentPromise
      console.log(`Component ${componentName} preloaded`)
      return componentPromise
    } catch (error) {
      console.error(`Failed to preload component ${componentName}:`, error)
      return null
    }
  }

  /**
   * 观察元素用于懒加载
   */
  observe(element: HTMLElement) {
    if (this.observer) {
      this.observer.observe(element)
    }
  }

  /**
   * 停止观察元素
   */
  unobserve(element: HTMLElement) {
    if (this.observer) {
      this.observer.unobserve(element)
    }
  }

  /**
   * 销毁预加载器
   */
  destroy() {
    if (this.observer) {
      this.observer.disconnect()
      this.observer = null
    }
    this.preloadedComponents.clear()
  }
}

// 内存监控器
export class MemoryMonitor {
  private monitoring = false
  private intervalId: number | null = null

  /**
   * 开始监控内存使用
   */
  startMonitoring(interval = 10000) {
    if (this.monitoring) return

    this.monitoring = true
    this.intervalId = window.setInterval(() => {
      this.logMemoryUsage()
    }, interval)
  }

  /**
   * 停止监控内存使用
   */
  stopMonitoring() {
    if (this.intervalId) {
      clearInterval(this.intervalId)
      this.intervalId = null
    }
    this.monitoring = false
  }

  /**
   * 记录内存使用情况
   */
  private logMemoryUsage() {
    const performanceWithMemory = performance as PerformanceWithMemory
    if (performanceWithMemory.memory) {
      const memory = performanceWithMemory.memory
      console.log('Memory Usage:', {
        used: `${Math.round(memory.usedJSHeapSize / 1024 / 1024)}MB`,
        total: `${Math.round(memory.totalJSHeapSize / 1024 / 1024)}MB`,
        limit: `${Math.round(memory.jsHeapSizeLimit / 1024 / 1024)}MB`,
      })
    }
  }

  /**
   * 强制垃圾回收（仅开发环境）
   */
  forceGC() {
    if (import.meta.env.DEV && 'gc' in window) {
      const windowWithGC = window as Window & { gc?: () => void }
      windowWithGC.gc?.()
    }
  }
}

// 防抖函数优化版
export function debounce<T extends (...args: unknown[]) => unknown>(
  func: T,
  wait: number,
  immediate = false,
): T & { cancel: () => void } {
  let timeout: number | null = null
  let result: ReturnType<T> | undefined

  const debounced = function (this: unknown, ...args: Parameters<T>) {
    const callNow = immediate && !timeout

    if (timeout) clearTimeout(timeout)

    timeout = window.setTimeout(() => {
      timeout = null
      if (!immediate) {
        result = func.apply(this, args) as ReturnType<T>
      }
    }, wait)

    if (callNow) {
      result = func.apply(this, args) as ReturnType<T>
    }

    return result as ReturnType<T>
  } as T & { cancel: () => void }

  debounced.cancel = () => {
    if (timeout) {
      clearTimeout(timeout)
      timeout = null
    }
  }

  return debounced
}

// 节流函数优化版
export function throttle<T extends (...args: unknown[]) => unknown>(
  func: T,
  limit: number,
): T & { cancel: () => void } {
  let inThrottle: boolean = false
  let lastFunc: number | null = null
  let lastRan: number = 0

  const throttled = function (this: unknown, ...args: Parameters<T>) {
    if (!inThrottle) {
      func.apply(this, args)
      lastRan = Date.now()
      inThrottle = true
    } else {
      if (lastFunc) clearTimeout(lastFunc)
      lastFunc = window.setTimeout(
        () => {
          if (Date.now() - lastRan >= limit) {
            func.apply(this, args)
            lastRan = Date.now()
          }
        },
        limit - (Date.now() - lastRan),
      )
    }
  } as T & { cancel: () => void }

  throttled.cancel = () => {
    if (lastFunc) {
      clearTimeout(lastFunc)
      lastFunc = null
    }
    inThrottle = false
  }

  return throttled
}

// 创建单例实例
export const eventListenerManager = new EventListenerManager()
export const componentPreloader = new ComponentPreloader()
export const memoryMonitor = new MemoryMonitor()

// 在开发环境启用内存监控
if (import.meta.env.DEV) {
  memoryMonitor.startMonitoring()
}
