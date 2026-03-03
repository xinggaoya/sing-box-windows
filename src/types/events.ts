// 事件载荷类型定义，供前端监听使用

export interface TrafficDataPayload {
  up: number
  down: number
  totalUp?: number
  totalDown?: number
}

export interface MemoryStatsPayload {
  inuse: number
  oslimit: number
}

export interface ConnectionMetadata {
  destinationIP: string
  destinationPort: string
  dnsMode: string
  host: string
  network: string
  processPath: string
  sourceIP: string
  sourcePort: string
  type: string
}

export interface ConnectionItem {
  chains: string[]
  download: number
  id: string
  metadata: ConnectionMetadata
  rule: string
  rulePayload: string
  start: string
  upload: number
}

export interface ConnectionsDataPayload {
  connections: ConnectionItem[]
  downloadTotal: number
  uploadTotal: number
  memory: number
}

export interface LogEventPayload {
  type: string
  payload: string
  timestamp: number
}

export interface KernelHealthPayload {
  healthy: boolean
  issues: string[]
}

export interface KernelLifecyclePayload {
  process_running: boolean
  api_ready: boolean
  websocket_ready: boolean
  kernel_state?: 'stopped' | 'starting' | 'running' | 'stopping' | 'failed' | 'crashed'
  state_version?: number
  proxy_mode?: 'system' | 'tun' | 'manual'
  api_port?: number
  proxy_port?: number
  auto_restarted?: boolean
}

export interface KernelFailurePayload {
  code?: string
  message?: string
  details?: string
  source?: string
  recoverable?: boolean
  timestamp?: number
  // 兼容旧后端/旧前端
  error?: string
}

export interface KernelOperationEventPayload {
  op_id?: string
  operation?: string
  state_version?: number
  timestamp?: number
  error?: string | null
}

export type KernelOperationFailedPayload = KernelOperationEventPayload

export interface UpdateAvailablePayload {
  latest_version: string
  download_url: string
  has_update: boolean
  release_notes?: string
  release_date?: string
  file_size?: number
  is_prerelease?: boolean
}

export interface UpgradeSubscriptionRefreshFailedPayload {
  message: string
  version: string
  active_config_path: string
  attempts: number
  last_error: string
}

export interface TrayNavigatePayload {
  path: string
}

export interface TraySwitchProxyModePayload {
  mode: 'system' | 'tun' | 'manual'
}
