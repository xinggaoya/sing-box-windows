// 全局类型定义
export * from './api'
export * from './store'
export * from './models'

// 全局常量类型
export type ProxyMode = 'system' | 'tun' | 'manual'

// WebSocket消息类型
export interface WebSocketMessage<T = any> {
  type: string
  data: T
  timestamp?: number
}

// 通用响应类型
export interface ApiResponse<T = any> {
  success: boolean
  data?: T
  error?: string
  message?: string
}
