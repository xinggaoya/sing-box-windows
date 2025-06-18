/**
 * 内存优化工具
 * 当应用隐藏到托盘时触发内存清理操作
 */

import mitt from '@/utils/mitt'
import { memoryMonitor } from '@/utils/memory-monitor'

export class MemoryOptimizer {
  private static instance: MemoryOptimizer
  private isListenerSetup = false
  private cleanupTasks: Array<() => void> = []

  static getInstance(): MemoryOptimizer {
    if (!MemoryOptimizer.instance) {
      MemoryOptimizer.instance = new MemoryOptimizer()
    }
    return MemoryOptimizer.instance
  }

  /**
   * 初始化内存优化器
   */
  initialize() {
    if (this.isListenerSetup) return

    // 监听内存清理请求事件
    mitt.on('memory-cleanup-requested', () => {
      this.performMemoryCleanup()
    })

    this.isListenerSetup = true
    console.log('🧹 内存优化器已初始化')
  }

  /**
   * 注册清理任务
   */
  registerCleanupTask(task: () => void) {
    this.cleanupTasks.push(task)
  }

  /**
   * 执行内存清理
   */
  private async performMemoryCleanup() {
    console.log('🧹 开始执行内存清理...')

    try {
      // 1. 清理DOM事件监听器和定时器
      this.cleanupDOMResources()

      // 2. 清理浏览器缓存
      this.cleanupBrowserCaches()

      // 3. 执行注册的自定义清理任务
      this.executeCustomCleanupTasks()

      // 4. 强制垃圾回收
      this.requestGarbageCollection()

      // 5. 检查清理效果
      setTimeout(() => {
        memoryMonitor.forceCheck()
      }, 2000)

      console.log('✅ 内存清理完成')
    } catch (error) {
      console.error('❌ 内存清理过程中出错:', error)
    }
  }

  /**
   * 清理DOM相关资源
   */
  private cleanupDOMResources() {
    try {
      // 清理未使用的图片缓存
      const images = document.querySelectorAll('img')
      images.forEach((img) => {
        const imageElement = img as HTMLImageElement
        // 对于不在视窗内的图片，清理其src
        const rect = imageElement.getBoundingClientRect()
        if (rect.bottom < 0 || rect.top > window.innerHeight) {
          const originalSrc = imageElement.src
          imageElement.src = ''
          // 标记以便后续恢复
          imageElement.setAttribute('data-original-src', originalSrc)
        }
      })

      console.log('🖼️ DOM资源清理完成')
    } catch (error) {
      console.error('DOM资源清理失败:', error)
    }
  }

  /**
   * 清理浏览器缓存
   */
  private cleanupBrowserCaches() {
    try {
      // 清理sessionStorage中的临时数据
      const keysToKeep = ['theme', 'locale', 'windowState']
      const allKeys = Object.keys(sessionStorage)

      allKeys.forEach((key) => {
        if (!keysToKeep.some((keepKey) => key.includes(keepKey))) {
          sessionStorage.removeItem(key)
        }
      })

      console.log('🗄️ 浏览器缓存清理完成')
    } catch (error) {
      console.error('浏览器缓存清理失败:', error)
    }
  }

  /**
   * 执行自定义清理任务
   */
  private executeCustomCleanupTasks() {
    try {
      this.cleanupTasks.forEach((task, index) => {
        try {
          task()
        } catch (error) {
          console.error(`自定义清理任务 ${index} 执行失败:`, error)
        }
      })

      console.log(`🔧 执行了 ${this.cleanupTasks.length} 个自定义清理任务`)
    } catch (error) {
      console.error('自定义清理任务执行失败:', error)
    }
  }

  /**
   * 请求垃圾回收
   */
  private requestGarbageCollection() {
    try {
      // 检查是否有gc函数可用
      const windowWithGc = window as Window & { gc?: () => void }
      if (windowWithGc.gc && typeof windowWithGc.gc === 'function') {
        windowWithGc.gc()
        console.log('🗑️ 已请求垃圾回收')
      }
    } catch (error) {
      console.error('垃圾回收请求失败:', error)
    }
  }

  /**
   * 恢复图片资源（当窗口重新显示时）
   */
  restoreImageResources() {
    try {
      const images = document.querySelectorAll('img[data-original-src]')
      images.forEach((img) => {
        const imageElement = img as HTMLImageElement
        const originalSrc = imageElement.getAttribute('data-original-src')
        if (originalSrc) {
          imageElement.src = originalSrc
          imageElement.removeAttribute('data-original-src')
        }
      })

      console.log('🖼️ 图片资源已恢复')
    } catch (error) {
      console.error('图片资源恢复失败:', error)
    }
  }

  /**
   * 清理事件监听
   */
  cleanup() {
    if (this.isListenerSetup) {
      mitt.off('memory-cleanup-requested')
      this.isListenerSetup = false
      this.cleanupTasks = []
      console.log('🧹 内存优化器已清理')
    }
  }
}

// 导出单例实例
export const memoryOptimizer = MemoryOptimizer.getInstance()
