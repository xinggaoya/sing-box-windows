import { onUnmounted } from 'vue'

/**
 * 简化的清理管理器
 * 替代过度复杂的内存管理系统
 */
export const useCleanup = () => {
  const disposables: (() => void)[] = []
  
  const addCleanup = (fn: () => void) => {
    disposables.push(fn)
    return () => {
      const index = disposables.indexOf(fn)
      if (index > -1) {
        disposables.splice(index, 1)
      }
    }
  }
  
  const cleanup = () => {
    disposables.forEach(fn => {
      try {
        fn()
      } catch (error) {
        console.warn('Cleanup function failed:', error)
      }
    })
    disposables.length = 0
  }
  
  onUnmounted(cleanup)
  
  return {
    addCleanup,
    cleanup,
    get count() {
      return disposables.length
    }
  }
}

/**
 * 简单的内存监控
 */
export const useMemoryMonitor = () => {
  const logMemoryUsage = (label: string) => {
    // 扩展Performance接口以包含内存信息
    interface PerformanceWithMemory extends Performance {
      memory?: {
        jsHeapSizeLimit: number
        totalJSHeapSize: number
        usedJSHeapSize: number
      }
    }
    
    const perf = performance as PerformanceWithMemory
    if (perf.memory) {
      const { usedJSHeapSize, totalJSHeapSize } = perf.memory
      const usage = ((usedJSHeapSize / totalJSHeapSize) * 100).toFixed(2)
      console.log(`[${label}] Memory: ${usage}% (${(usedJSHeapSize / 1024 / 1024).toFixed(2)}MB)`)
    }
  }
  
  return {
    logMemoryUsage
  }
}