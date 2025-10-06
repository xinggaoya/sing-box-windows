import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { getVersion } from '@tauri-apps/api/app'

// 定义更新信息类型
interface UpdateInfo {
  latest_version: string
  download_url: string
  has_update: boolean
  release_notes?: string
  release_date?: string
  file_size?: number
  is_prerelease?: boolean
}

// 定义更新状态类型
interface UpdateState {
  checking: boolean
  downloading: boolean
  progress: number
  status: 'idle' | 'checking' | 'downloading' | 'completed' | 'error' | 'installing'
  message: string
  error: string | null
}

export const useUpdateStore = defineStore(
  'update',
  () => {
    // 应用版本号状态
    const appVersion = ref('')

    // 更新相关状态
    const hasUpdate = ref(false)
    const latestVersion = ref('')
    const downloadUrl = ref('')
    const releaseNotes = ref('')
    const releaseDate = ref('')
    const fileSize = ref(0)

    // 更新状态
    const updateState = ref<UpdateState>({
      checking: false,
      downloading: false,
      progress: 0,
      status: 'idle',
      message: '',
      error: null,
    })

    // 用户设置
    const autoCheckUpdate = ref(true)
    const skipVersion = ref('')
    const acceptPrerelease = ref(false) // 是否接收预发布版本

    // 当前版本信息
    const isPrerelease = ref(false) // 当前检测到的版本是否为预发布版本

    // 获取应用版本
    const fetchAppVersion = async () => {
      try {
        appVersion.value = await getVersion()
        return appVersion.value
      } catch (error) {
        console.error('获取应用版本失败:', error)
        return ''
      }
    }

    // 格式化文件大小
    const formatFileSize = (bytes: number): string => {
      if (bytes === 0) return '0 B'
      const k = 1024
      const sizes = ['B', 'KB', 'MB', 'GB']
      const i = Math.floor(Math.log(bytes) / Math.log(k))
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
    }

    // 格式化发布日期
    const formatReleaseDate = (dateStr: string): string => {
      try {
        const date = new Date(dateStr)
        return date.toLocaleDateString('zh-CN', {
          year: 'numeric',
          month: 'long',
          day: 'numeric',
        })
      } catch {
        return dateStr
      }
    }

    // 检查更新
    const checkUpdate = async (silent: boolean = false): Promise<UpdateInfo | null> => {
      // 如果用户跳过了这个版本且是静默检查，则不进行检查
      if (silent && skipVersion.value && skipVersion.value === latestVersion.value) {
        return null
      }

      try {
        updateState.value.checking = true
        updateState.value.status = 'checking'
        updateState.value.error = null
        updateState.value.message = '正在检查更新...'

        // 确保当前版本已获取
        if (!appVersion.value) {
          await fetchAppVersion()
        }
        
        const updateInfo = await tauriApi.system.checkUpdate(
          appVersion.value,
          acceptPrerelease.value
        )

        if (updateInfo && updateInfo.has_update) {
          hasUpdate.value = true
          latestVersion.value = updateInfo.latest_version
          downloadUrl.value = updateInfo.download_url
          releaseNotes.value = updateInfo.release_notes || ''
          releaseDate.value = updateInfo.release_date || ''
          fileSize.value = updateInfo.file_size || 0
          isPrerelease.value = updateInfo.is_prerelease || false

          const versionType = updateInfo.is_prerelease ? '测试版本' : '正式版本'
          updateState.value.message = `发现新${versionType} ${updateInfo.latest_version}`

          // 更新可用事件现在通过Pinia响应式系统处理
          console.log('发现新版本:', updateInfo.latest_version)

          return updateInfo
        } else {
          hasUpdate.value = false
          updateState.value.message = '已是最新版本'
        }

        return null
      } catch (error) {
        console.error('检查更新失败:', error)
        updateState.value.error = error as string
        updateState.value.message = '检查更新失败'
        updateState.value.status = 'error'
        return null
      } finally {
        updateState.value.checking = false
        if (updateState.value.status === 'checking') {
          updateState.value.status = 'idle'
        }
      }
    }

    // 下载并安装更新
    const downloadAndInstallUpdate = async () => {
      if (!hasUpdate.value || !downloadUrl.value) return false

      try {
        updateState.value.downloading = true
        updateState.value.status = 'downloading'
        updateState.value.progress = 0
        updateState.value.error = null

        // 下载进度现在通过Pinia响应式系统处理
        console.log('开始下载更新:', {
          status: 'downloading',
          progress: 0,
          message: '准备下载更新...',
        })

        // 开始下载和安装
        const result = await tauriApi.system.downloadAndInstallUpdate(window)
        return result
      } catch (error) {
        console.error('下载更新失败:', error)
        updateState.value.error = error as string
        updateState.value.status = 'error'
        updateState.value.message = `下载更新失败: ${error}`
        return false
      }
    }

    // 跳过当前版本
    const skipCurrentVersion = () => {
      skipVersion.value = latestVersion.value
      hasUpdate.value = false
    }

    // 重置更新状态
    const resetUpdateState = () => {
      hasUpdate.value = false
      latestVersion.value = ''
      downloadUrl.value = ''
      releaseNotes.value = ''
      releaseDate.value = ''
      fileSize.value = 0
      updateState.value = {
        checking: false,
        downloading: false,
        progress: 0,
        status: 'idle',
        message: '',
        error: null,
      }
    }

    // 更新进度处理
    const updateProgress = (status: string, progress: number, message: string) => {
      const validStatuses: ReadonlyArray<UpdateState['status']> = [
        'idle',
        'checking',
        'downloading',
        'completed',
        'error',
        'installing',
      ]
      if (validStatuses.includes(status as UpdateState['status'])) {
        updateState.value.status = status as UpdateState['status']
      }
      updateState.value.progress = progress
      updateState.value.message = message

      if (status === 'completed') {
        updateState.value.downloading = false
      } else if (status === 'error') {
        updateState.value.downloading = false
        updateState.value.error = message
      } else if (status === 'installing') {
        updateState.value.downloading = false
        updateState.value.status = 'installing'
      }
    }

    return {
      // 状态
      appVersion,
      hasUpdate,
      latestVersion,
      downloadUrl,
      releaseNotes,
      releaseDate,
      fileSize,
      updateState,
      autoCheckUpdate,
      skipVersion,
      acceptPrerelease,
      isPrerelease,

      // 方法
      fetchAppVersion,
      checkUpdate,
      downloadAndInstallUpdate,
      skipCurrentVersion,
      resetUpdateState,
      updateProgress,

      // 计算属性
      formattedFileSize: () => formatFileSize(fileSize.value),
      formattedReleaseDate: () => (releaseDate.value ? formatReleaseDate(releaseDate.value) : ''),
    }
  },
  {
    persist: true,
  },
)
