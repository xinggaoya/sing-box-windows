/**
 * 实时内存监控工具
 * 用于监控和控制应用内存使用情况
 */

import mitt from '@/utils/mitt'

interface MemoryInfo {
  jsHeapSizeLimit: number
  totalJSHeapSize: number
  usedJSHeapSize: number
  timestamp: number
}

interface PerformanceWithMemory extends Performance {
  memory?: {
    jsHeapSizeLimit: number
    totalJSHeapSize: number
    usedJSHeapSize: number
  }
}

export class MemoryMonitor {
  private static instance: MemoryMonitor
  private monitorInterval: number | null = null
  private isMonitoring = false
  private memoryHistory: MemoryInfo[] = []
  private readonly MAX_HISTORY = 10
  private readonly DANGER_THRESHOLD = 100 * 1024 * 1024 // 100MB
  private readonly CRITICAL_THRESHOLD = 200 * 1024 * 1024 // 200MB

  static getInstance(): MemoryMonitor {
    if (!MemoryMonitor.instance) {
      MemoryMonitor.instance = new MemoryMonitor()
    }
    return MemoryMonitor.instance
  }

  /**
   * 开始内存监控
   */
  startMonitoring(intervalMs = 15000) {
    // 每15秒检查一次
    if (this.isMonitoring) return

    this.isMonitoring = true
    console.log('🔍 开始内存监控，检查间隔:', intervalMs / 1000, '秒')

    this.monitorInterval = window.setInterval(() => {
      this.checkMemory()
    }, intervalMs)

    // 立即执行一次检查
    this.checkMemory()
  }

  /**
   * 停止内存监控
   */
  stopMonitoring() {
    if (this.monitorInterval) {
      clearInterval(this.monitorInterval)
      this.monitorInterval = null
    }
    this.isMonitoring = false
    console.log('✅ 停止内存监控')
  }

  /**
   * 检查当前内存使用情况
   */
  private checkMemory() {
    const performanceWithMemory = performance as PerformanceWithMemory
    const memory = performanceWithMemory.memory

    if (!memory) {
      console.warn('当前浏览器不支持内存监控')
      return
    }

    const memoryInfo: MemoryInfo = {
      jsHeapSizeLimit: memory.jsHeapSizeLimit,
      totalJSHeapSize: memory.totalJSHeapSize,
      usedJSHeapSize: memory.usedJSHeapSize,
      timestamp: Date.now(),
    }

    // 添加到历史记录
    this.memoryHistory.push(memoryInfo)
    if (this.memoryHistory.length > this.MAX_HISTORY) {
      this.memoryHistory.shift()
    }

    // 检查内存使用情况
    this.analyzeMemoryUsage(memoryInfo)
  }

  /**
   * 分析内存使用情况
   */
  private analyzeMemoryUsage(current: MemoryInfo) {
    const usedMB = current.usedJSHeapSize / 1024 / 1024
    const limitMB = current.jsHeapSizeLimit / 1024 / 1024

    console.log(
      `📊 内存使用: ${usedMB.toFixed(1)}MB / ${limitMB.toFixed(1)}MB (${((usedMB / limitMB) * 100).toFixed(1)}%)`,
    )

    // 检查是否超过危险阈值
    if (current.usedJSHeapSize > this.CRITICAL_THRESHOLD) {
      console.error(`🚨 内存使用达到严重级别: ${usedMB.toFixed(1)}MB`)
      this.triggerCriticalCleanup()
    } else if (current.usedJSHeapSize > this.DANGER_THRESHOLD) {
      console.warn(`⚠️ 内存使用较高: ${usedMB.toFixed(1)}MB`)
      this.triggerPreventiveCleanup()
    }

    // 检查内存增长趋势
    this.checkMemoryTrend()
  }

  /**
   * 检查内存增长趋势
   */
  private checkMemoryTrend() {
    if (this.memoryHistory.length < 3) return

    const recent = this.memoryHistory.slice(-3)
    const isIncreasing = recent.every((info, index) => {
      if (index === 0) return true
      return info.usedJSHeapSize > recent[index - 1].usedJSHeapSize
    })

    if (isIncreasing) {
      const growth = recent[recent.length - 1].usedJSHeapSize - recent[0].usedJSHeapSize
      const growthMB = growth / 1024 / 1024

      if (growthMB > 20) {
        // 如果短时间内增长超过20MB
        console.warn(`📈 检测到内存快速增长: ${growthMB.toFixed(1)}MB`)
        this.triggerPreventiveCleanup()
      }
    }
  }

  /**
   * 触发预防性清理
   */
  private triggerPreventiveCleanup() {
    console.log('🧹 触发预防性内存清理')
    mitt.emit('memory-cleanup-requested')

    // 请求垃圾回收
    this.requestGarbageCollection()
  }

  /**
   * 触发紧急清理
   */
  private triggerCriticalCleanup() {
    console.log('🚨 触发紧急内存清理')

    // 清理所有可能的缓存和临时数据
    // mitt.emit('memory-critical-cleanup')
    mitt.emit('memory-cleanup-requested')

    // 强制垃圾回收
    this.requestGarbageCollection()

    // 通知用户 - 使用控制台警告而不是事件
    const usedMB = this.memoryHistory[this.memoryHistory.length - 1]?.usedJSHeapSize / 1024 / 1024
    console.warn(`🚨 内存使用危险: ${usedMB?.toFixed(1)}MB`)
  }

  /**
   * 请求垃圾回收
   */
  private requestGarbageCollection() {
    // 检查是否有gc函数可用（通常在开发环境中）
    const windowWithGc = window as Window & { gc?: () => void }
    if (windowWithGc.gc && typeof windowWithGc.gc === 'function') {
      try {
        windowWithGc.gc()
        console.log('✅ 已请求垃圾回收')
      } catch (error) {
        console.warn('垃圾回收请求失败:', error)
      }
    }
  }

  /**
   * 获取当前内存状态
   */
  getCurrentMemoryInfo(): MemoryInfo | null {
    if (this.memoryHistory.length === 0) return null
    return this.memoryHistory[this.memoryHistory.length - 1]
  }

  /**
   * 获取内存使用趋势
   */
  getMemoryTrend(): 'increasing' | 'decreasing' | 'stable' | 'unknown' {
    if (this.memoryHistory.length < 2) return 'unknown'

    const recent = this.memoryHistory.slice(-2)
    const diff = recent[1].usedJSHeapSize - recent[0].usedJSHeapSize
    const threshold = 5 * 1024 * 1024 // 5MB threshold

    if (diff > threshold) return 'increasing'
    if (diff < -threshold) return 'decreasing'
    return 'stable'
  }

  /**
   * 强制检查内存
   */
  forceCheck() {
    this.checkMemory()
  }
}

// 导出单例实例
export const memoryMonitor = MemoryMonitor.getInstance()
