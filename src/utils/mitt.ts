import type { Emitter } from 'mitt'
import mitt from 'mitt'

// 定义下载进度接口
interface DownloadProgress {
  status: 'checking' | 'found' | 'downloading' | 'extracting' | 'completed'
  progress: number
  message: string
}

// 定义更新信息接口
interface UpdateInfo {
  latest_version: string
  download_url: string
  has_update: boolean
}

// 定义事件类型
type Events = {
  'process-status': void
  'download-progress': DownloadProgress
  'proxy-mode-changed': void
  'window-minimize': void
  'window-hide': void
  'window-show': void
  'window-restore': void
  'refresh-tray-menu': void
  'update-available': UpdateInfo
  'language-changed': void
  error: string
}

// 创建事件总线实例
const emitter: Emitter<Events> = mitt<Events>()

export default emitter
