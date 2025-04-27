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
}

// 定义事件类型
export type Events = {
  // 窗口相关事件
  'window-minimize': void
  'window-hide': void
  'window-show': void
  'window-restore': void
  
  // 进程相关事件
  'process-status': void
  'download-progress': DownloadProgress
  'proxy-mode-changed': void
  'refresh-tray-menu': void
  'update-available': UpdateInfo
  'language-changed': void
  'kernel-started': void
  'kernel-stopped': void
  'tray-clicked': void
  'error': string
  
  // WebSocket 连接状态事件
  'traffic-connection-state': ConnectionState
  'memory-connection-state': ConnectionState
  'connections-connection-state': ConnectionState
  'logs-connection-state': ConnectionState
  
  // WebSocket 数据事件
  'traffic-data': any
  'memory-data': any
  'connections-data': any
  'log-data': any
}

// 创建 mitt 实例
const emitter = mitt<Events>()

export default emitter
