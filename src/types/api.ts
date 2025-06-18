// API 请求和响应类型定义

// 内核相关
export interface KernelVersion {
  version: string
  buildTime?: string
  goVersion?: string
}

// 代理节点
export interface ProxyNode {
  name: string
  type: string
  delay?: number
  history?: number[]
  now?: string
}

// 代理组
export interface ProxyGroup {
  name: string
  type: string
  all: string[]
  now: string
  testUrl?: string
}

// 流量统计
export interface TrafficStats {
  upload: number
  download: number
  uploadSpeed: number
  downloadSpeed: number
}

// 连接信息
export interface Connection {
  id: string
  metadata: ConnectionMetadata
  upload: number
  download: number
  start: number
  chains: string[]
  rule: string
  rulePayload: string
}

export interface ConnectionMetadata {
  network: string
  type: string
  sourceIP: string
  destinationIP: string
  sourcePort: string
  destinationPort: string
  host: string
  dnsMode: string
  process: string
  processPath?: string
  uid?: number
}

// 日志
export interface LogEntry {
  type: 'info' | 'warning' | 'error' | 'debug'
  payload: string
  timestamp?: number
}

// 规则
export interface Rule {
  type: string
  payload: string
  proxy: string
  size?: number
}

// 订阅
export interface Subscription {
  id: string
  name: string
  url: string
  enabled: boolean
  updateInterval?: number
  lastUpdate?: number
  nodes?: ProxyNode[]
}
