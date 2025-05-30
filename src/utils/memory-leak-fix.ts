/**
 * 内存泄露修复工具
 * 用于检测和修复应用中的内存泄露问题
 */

import mitt from '@/utils/mitt'

// 扩展Performance接口以包含内存信息
interface PerformanceWithMemory extends Performance {
  memory?: {
    jsHeapSizeLimit: number
    totalJSHeapSize: number
    usedJSHeapSize: number
  }
}

// WebSocket连接接口
interface WebSocketLike {
  close?: () => void
  disconnect?: () => void
  [key: string]: unknown
}

// 内存监控接口
interface MemoryStats {
  jsHeapSizeLimit: number
  totalJSHeapSize: number
  usedJSHeapSize: number
  timestamp: number
}

// 内存泄露检测器
export class MemoryLeakDetector {
  private static instance: MemoryLeakDetector
  private memoryHistory: MemoryStats[] = []
  private maxHistorySize = 20 // 保存最近20次记录
  private monitorInterval: number | null = null
  private isMonitoring = false
  private leakThreshold = 50 * 1024 * 1024 // 50MB阈值

  static getInstance(): MemoryLeakDetector {
    if (!MemoryLeakDetector.instance) {
      MemoryLeakDetector.instance = new MemoryLeakDetector()
    }
    return MemoryLeakDetector.instance
  }

  /**
   * 开始内存监控
   */
  startMonitoring(intervalMs = 30000) {
    if (this.isMonitoring) return

    this.isMonitoring = true
    console.log('🔍 开始内存泄露监控')

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
    console.log('✅ 停止内存泄露监控')
  }

  /**
   * 检查内存使用情况
   */
  private checkMemory() {
    const performanceWithMemory = performance as PerformanceWithMemory
    const memory = performanceWithMemory.memory
    if (!memory) {
      console.warn('当前浏览器不支持内存监控')
      return
    }

    const stats: MemoryStats = {
      jsHeapSizeLimit: memory.jsHeapSizeLimit,
      totalJSHeapSize: memory.totalJSHeapSize,
      usedJSHeapSize: memory.usedJSHeapSize,
      timestamp: Date.now(),
    }

    this.memoryHistory.push(stats)

    // 保持历史记录在限定大小内
    if (this.memoryHistory.length > this.maxHistorySize) {
      this.memoryHistory.shift()
    }

    // 检测内存泄露
    this.detectLeak()
  }

  /**
   * 检测内存泄露
   */
  private detectLeak() {
    if (this.memoryHistory.length < 5) return

    const recent = this.memoryHistory.slice(-5)
    const first = recent[0]
    const last = recent[recent.length - 1]

    // 计算内存增长
    const memoryIncrease = last.usedJSHeapSize - first.usedJSHeapSize
    const timeSpan = last.timestamp - first.timestamp

    // 如果5次检查中内存持续增长且超过阈值
    const isIncreasing = recent.every((stat, index) => {
      if (index === 0) return true
      return stat.usedJSHeapSize >= recent[index - 1].usedJSHeapSize
    })

    if (isIncreasing && memoryIncrease > this.leakThreshold) {
      console.warn(`🚨 检测到可能的内存泄露:`)
      console.warn(`  - 内存增长: ${(memoryIncrease / 1024 / 1024).toFixed(2)} MB`)
      console.warn(`  - 时间跨度: ${(timeSpan / 1000).toFixed(1)} 秒`)
      console.warn(`  - 当前使用: ${(last.usedJSHeapSize / 1024 / 1024).toFixed(2)} MB`)

      // 尝试清理
      this.attemptCleanup()
    }
  }

  /**
   * 尝试清理内存
   */
  private attemptCleanup() {
    console.log('🧹 开始内存清理...')

    // 触发垃圾回收（如果可能）
    if ('gc' in window) {
      // @ts-expect-error - Chrome开发者工具专用API
      window.gc()
      console.log('✅ 触发了垃圾回收')
    }

    // 清理事件监听器
    this.cleanupEventListeners()

    // 通知其他模块进行清理
    mitt.emit('memory-cleanup-requested')
  }

  /**
   * 清理事件监听器
   */
  private cleanupEventListeners() {
    // 这里可以添加更多清理逻辑
    console.log('🧹 清理事件监听器')
  }

  /**
   * 获取内存统计
   */
  getMemoryStats(): MemoryStats | null {
    if (this.memoryHistory.length === 0) return null
    return this.memoryHistory[this.memoryHistory.length - 1]
  }

  /**
   * 强制检查内存
   */
  forceCheck() {
    this.checkMemory()
  }
}

// WebSocket 连接清理器
export class WebSocketCleaner {
  private static instance: WebSocketCleaner
  private connections: Set<WebSocketLike> = new Set()
  private timers: Set<number> = new Set()

  static getInstance(): WebSocketCleaner {
    if (!WebSocketCleaner.instance) {
      WebSocketCleaner.instance = new WebSocketCleaner()
    }
    return WebSocketCleaner.instance
  }

  /**
   * 注册WebSocket连接
   */
  registerConnection(connection: WebSocketLike) {
    this.connections.add(connection)
  }

  /**
   * 注销WebSocket连接
   */
  unregisterConnection(connection: WebSocketLike) {
    this.connections.delete(connection)
  }

  /**
   * 注册定时器
   */
  registerTimer(timerId: number) {
    this.timers.add(timerId)
  }

  /**
   * 注销定时器
   */
  unregisterTimer(timerId: number) {
    this.timers.delete(timerId)
    clearInterval(timerId)
    clearTimeout(timerId)
  }

  /**
   * 清理所有连接和定时器
   */
  cleanupAll() {
    console.log('🧹 清理所有WebSocket连接和定时器')

    // 清理定时器
    this.timers.forEach((timerId) => {
      clearInterval(timerId)
      clearTimeout(timerId)
    })
    this.timers.clear()

    // 清理WebSocket连接
    this.connections.forEach((connection) => {
      try {
        if (connection && typeof connection.close === 'function') {
          connection.close()
        } else if (connection && typeof connection.disconnect === 'function') {
          connection.disconnect()
        }
      } catch (error) {
        console.warn('清理WebSocket连接时出错:', error)
      }
    })
    this.connections.clear()
  }
}

// Store 清理器
export class StoreCleaner {
  private static cleanupFunctions: Set<() => void> = new Set()

  /**
   * 注册清理函数
   */
  static registerCleanup(cleanupFn: () => void) {
    this.cleanupFunctions.add(cleanupFn)
  }

  /**
   * 执行所有清理函数
   */
  static cleanupAll() {
    console.log('🧹 清理所有Store')
    this.cleanupFunctions.forEach((fn) => {
      try {
        fn()
      } catch (error) {
        console.warn('Store清理时出错:', error)
      }
    })
  }
}

// 导出单例实例
export const memoryLeakDetector = MemoryLeakDetector.getInstance()
export const webSocketCleaner = WebSocketCleaner.getInstance()
