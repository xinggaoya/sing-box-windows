import mitt from 'mitt'

// 定义连接状态接口（移除了对WebSocket服务的依赖）
export interface ConnectionState {
  connected: boolean
  connecting: boolean
  error: Error | null
}

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

// 注意：mitt现在已弃用，改为使用Tauri事件系统和Pinia响应式系统
// 保留此文件仅为向后兼容，建议新代码使用Tauri事件或Pinia

// 创建 mitt 实例（向后兼容）
const emitter = mitt<any>()

export default emitter
export { mitt } // 重新导出mitt，如果某些地方还需要使用
