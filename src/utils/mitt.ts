import type { Emitter } from 'mitt'
import mitt from 'mitt'
import { ConnectionState } from '../services/websocket-service'

// 定义下载进度接口
export interface DownloadProgress {
  status: 'checking' | 'downloading' | 'extracting' | 'completed' | 'error'
  progress: number
  message: string
}

// 定义更新信息接口
export interface UpdateInfo {
  latest_version: string
  download_url: string
  has_update: boolean
  release_date?: string
  file_size?: number
}

// 定义更新弹窗数据接口
export interface UpdateModalData {
  show: boolean
  latestVersion: string
  currentVersion: string
  downloadUrl: string
  releaseNotes: string
  releaseDate: string
  fileSize: number
}

// 定义规则数据接口
export interface RulesData {
  rules: Array<{
    type: string
    payload: string
    proxy: string
  }>
}

// 定义事件类型
export type Events = {
  // 窗口相关事件
  'window-minimize': void
  'window-hide': void
  'window-show': void
  'window-restore': void
  'window-maximize': void
  'window-unmaximize': void

  // 应用初始化事件
  'message-instance-ready': unknown

  // 进程相关事件
  'process-status': void
  'download-progress': DownloadProgress
  'proxy-mode-changed': void
  'refresh-tray-menu': void
  'update-available': UpdateInfo
  'show-update-modal': UpdateModalData
  'language-changed': void
  'kernel-started': void
  'kernel-stopped': void
  'kernel-start-failed': { error: string }
  'connecting-status-changed': boolean
  'tray-clicked': void
  error: string

  // 内存管理事件
  'memory-cleanup-requested': void

  // WebSocket 连接状态事件
  'traffic-connection-state': ConnectionState
  'memory-connection-state': ConnectionState
  'connections-connection-state': ConnectionState
  'logs-connection-state': ConnectionState
  'ws-connected': void // 添加WebSocket连接成功事件
  'ws-disconnected': void // 添加WebSocket连接断开事件

  // WebSocket 数据事件
  'traffic-data': Record<string, unknown>
  'memory-data': Record<string, unknown>
  'connections-data': Record<string, unknown>
  'log-data': { type: string; payload: string }
  'rules-data': RulesData
}

// 创建 mitt 实例
const emitter = mitt<Events>()

export default emitter
