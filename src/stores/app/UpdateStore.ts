import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { getVersion } from '@tauri-apps/api/app'
import { DatabaseService } from '@/services/database-service'
import type { UpdateConfig } from '@/types/database'

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
    const acceptPrerelease = ref(false) // 是否接收预发布版本（保留，但不同步到数据库）

    // 当前版本信息
    const isPrerelease = ref(false) // 当前检测到的版本是否为预发布版本

    // 从数据库加载数据
    const loadFromBackend = async () => {
      try {
        console.log('🔄 从数据库加载更新配置...')
        const updateConfig = await DatabaseService.getUpdateConfig()
        
        // 更新响应式状态
        autoCheckUpdate.value = updateConfig.auto_check
        skipVersion.value = updateConfig.skip_version || ''
        
        // 获取当前版本
        await fetchAppVersion()
        
        console.log('🔄 更新配置加载完成：', {
          appVersion: appVersion.value,
          autoCheckUpdate: autoCheckUpdate.value,
          skipVersion: skipVersion.value,
        })
      } catch (error) {
        console.error('从数据库加载更新配置失败:', error)
        // 加载失败时获取当前版本
        await fetchAppVersion()
      }
    }

    // 保存配置到数据库
    const saveToBackend = async () => {
      try {
        const config: UpdateConfig = {
          auto_check: autoCheckUpdate.value,
          last_check: Math.floor(Date.now() / 1000),
          last_version: appVersion.value,
          skip_version: skipVersion.value || null,
        }
        await DatabaseService.saveUpdateConfig(config)
        console.log('✅ 更新配置已保存到数据库')
      } catch (error) {
        console.error('保存更新配置到数据库失败:', error)
      }
    }

    // 获取应用版本
    const fetchAppVersion = async () => {
      try {
        appVersion.value = await getVersion()
        // 保存版本到后端
        await saveToBackend()
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
    const skipCurrentVersion = async () => {
      skipVersion.value = latestVersion.value
      hasUpdate.value = false
      await saveToBackend()
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

    // 设置自动检查更新
    const setAutoCheckUpdate = async (enabled: boolean) => {
      autoCheckUpdate.value = enabled
      // 保存会在 watch 中自动处理
    }

    // 设置接受预发布版本
    const setAcceptPrerelease = async (accept: boolean) => {
      acceptPrerelease.value = accept
      // 注意：这个字段不同步到数据库，所以不需要自动保存
    }

    // 标记是否正在初始化
    let isInitializing = false
    
    // 监听更新配置变化并自动保存到数据库
    watch(
      [autoCheckUpdate, skipVersion],
      async () => {
        // 初始化期间不保存
        if (isInitializing) return
        await saveToBackend()
      },
      { deep: true }
    )

    // 初始化方法
    const initializeStore = async () => {
      isInitializing = true
      await loadFromBackend()
      // 等待一下确保数据加载完成
      await new Promise(resolve => setTimeout(resolve, 100))
      isInitializing = false
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
      setAutoCheckUpdate,
      setAcceptPrerelease,

      // 计算属性
      formattedFileSize: () => formatFileSize(fileSize.value),
      formattedReleaseDate: () => (releaseDate.value ? formatReleaseDate(releaseDate.value) : ''),

      // 初始化和持久化
      initializeStore,
      loadFromBackend,
      saveToBackend,
    }
  },
  // 移除 persist 配置，现在使用后端存储
)