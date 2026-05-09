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
  sniffHost?: string
  network: string
  process?: string
  processPath: string
  remoteDestination?: string
  sourceIP: string
  sourcePort: string
  type: string
  inboundName?: string
  inboundIP?: string
  inboundPort?: string
  inboundUser?: string
  uid?: number
  specialProxy?: string
  specialRules?: string
}

export interface ConnectionItem {
  chains: string[]
  download: number
  downloadSpeed?: number
  id: string
  metadata: ConnectionMetadata
  rule: string
  rulePayload: string
  start: string
  upload: number
  uploadSpeed?: number
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

export type StartupDiagnosisKind =
  | 'config_invalid'
  | 'config_missing'
  | 'binary_missing'
  | 'permission_denied'
  | 'sudo_required'
  | 'sudo_invalid'
  | 'port_conflict'
  | 'process_exited_early'
  | 'api_http_error'
  | 'api_timeout'
  | 'conflict_cleanup_failed'
  | 'guard_restart_failed'
  | 'unknown'

export type StartupDiagnosisStage = 'preflight' | 'spawn' | 'readiness' | 'guard' | 'auto_manage'

export interface KernelReadinessSnapshot {
  config_validated?: boolean | null
  process_spawned?: boolean | null
  process_alive: boolean
  api_ready: boolean
  relay_ready: boolean
}

export interface StartupDiagnosis {
  attempt_id: string
  stage: StartupDiagnosisStage
  code: string
  kind: StartupDiagnosisKind
  message: string
  detail: string
  source: string
  recoverable: boolean
  config_path?: string | null
  http_status?: number | null
  suggested_actions?: string[] | null
  timestamp_ms: number
}

export interface KernelLifecyclePayload {
  process_running: boolean
  api_ready: boolean
  websocket_ready: boolean
  readiness?: KernelReadinessSnapshot
  startup_diagnosis?: StartupDiagnosis | null
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
  startup_diagnosis?: StartupDiagnosis
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
  release_page_url: string
  has_update: boolean
  release_notes?: string
  release_date?: string
  file_size?: number
  is_prerelease?: boolean
  supports_in_app_update: boolean
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

export interface TrayToggleProxyFeaturePayload {
  feature: 'systemProxy' | 'tun'
  enabled: boolean
}
