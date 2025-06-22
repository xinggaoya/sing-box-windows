/**
 * WebSocket监控工具 - 用于诊断频繁请求问题
 */

interface RequestLog {
  timestamp: number
  type: string
  url?: string
  data?: unknown
}

class WebSocketMonitor {
  private static instance: WebSocketMonitor
  private requestLogs: RequestLog[] = []
  private readonly MAX_LOGS = 100
  private monitoring = false

  private constructor() {}

  public static getInstance(): WebSocketMonitor {
    if (!WebSocketMonitor.instance) {
      WebSocketMonitor.instance = new WebSocketMonitor()
    }
    return WebSocketMonitor.instance
  }

  /**
   * 开始监控
   */
  public startMonitoring() {
    if (this.monitoring) return

    this.monitoring = true
    console.log('🔍 WebSocket监控已启动')

    // 清空之前的日志
    this.requestLogs = []

    // 监控开始时间
    const startTime = Date.now()

    // 每10秒输出统计信息
    const statsInterval = setInterval(() => {
      if (!this.monitoring) {
        clearInterval(statsInterval)
        return
      }

      const now = Date.now()
      const last10Seconds = now - 10000
      const recentRequests = this.requestLogs.filter((log) => log.timestamp > last10Seconds)

      if (recentRequests.length > 50) {
        console.warn(`⚠️ 过去10秒内有 ${recentRequests.length} 个WebSocket相关请求`)

        // 分类统计
        const typeStats: Record<string, number> = {}
        recentRequests.forEach((req) => {
          typeStats[req.type] = (typeStats[req.type] || 0) + 1
        })

        console.log('📊 请求类型统计:', typeStats)

        // 如果请求过于频繁，输出详细信息
        if (recentRequests.length > 100) {
          console.error('🚨 检测到异常频繁的请求！最近的请求:')
          recentRequests.slice(-10).forEach((req) => {
            console.log(`  - ${new Date(req.timestamp).toLocaleTimeString()}: ${req.type}`)
          })
        }
      }
    }, 10000)

    // 5分钟后自动停止监控
    setTimeout(
      () => {
        this.stopMonitoring()
      },
      5 * 60 * 1000,
    )
  }

  /**
   * 停止监控
   */
  public stopMonitoring() {
    if (!this.monitoring) return

    this.monitoring = false
    console.log('🔍 WebSocket监控已停止')

    // 输出总结
    const totalRequests = this.requestLogs.length
    if (totalRequests > 0) {
      const timespan =
        this.requestLogs[this.requestLogs.length - 1].timestamp - this.requestLogs[0].timestamp
      const avgRequestsPerSecond = (totalRequests / (timespan / 1000)).toFixed(2)

      console.log(`📊 监控总结: ${totalRequests} 个请求，平均每秒 ${avgRequestsPerSecond} 个`)

      // 类型统计
      const typeStats: Record<string, number> = {}
      this.requestLogs.forEach((req) => {
        typeStats[req.type] = (typeStats[req.type] || 0) + 1
      })
      console.log('📊 总体类型统计:', typeStats)
    }
  }

  /**
   * 记录请求
   */
  public logRequest(type: string, url?: string, data?: unknown) {
    if (!this.monitoring) return

    const log: RequestLog = {
      timestamp: Date.now(),
      type,
      url,
      data: data
        ? typeof data === 'string'
          ? data.substring(0, 100)
          : JSON.stringify(data).substring(0, 100)
        : undefined,
    }

    this.requestLogs.push(log)

    // 保持日志数量在限制内
    if (this.requestLogs.length > this.MAX_LOGS) {
      this.requestLogs = this.requestLogs.slice(-this.MAX_LOGS)
    }
  }

  /**
   * 获取最近的请求日志
   */
  public getRecentLogs(seconds: number = 60): RequestLog[] {
    const cutoff = Date.now() - seconds * 1000
    return this.requestLogs.filter((log) => log.timestamp > cutoff)
  }
}

export const wsMonitor = WebSocketMonitor.getInstance()

// 在开发环境下自动启动监控
if (import.meta.env.DEV) {
  // 延迟启动，给应用时间初始化
  setTimeout(() => {
    wsMonitor.startMonitoring()
    console.log('🔍 开发环境下自动启动WebSocket监控')
  }, 3000)
}
