import type { Emitter } from 'mitt'
import mitt from 'mitt'

// 定义下载进度接口
interface DownloadProgress {
  status: 'checking' | 'found' | 'downloading' | 'extracting' | 'completed'
  progress: number
  message: string
}

// 定义事件类型
type Events = {
  'process-status': string
  'download-progress': DownloadProgress
  'proxy-mode-changed': string
  'window-minimize': void
  'window-hide': void
  'window-show': void
  'window-restore': void
  error: string
}

// 创建事件总线实例
const emitter: Emitter<Events> = mitt<Events>()

export default emitter
