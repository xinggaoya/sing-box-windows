import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { getVersion } from '@tauri-apps/api/app'
import mitt from '@/utils/mitt'

// 定义更新信息类型
interface UpdateInfo {
  latest_version: string
  download_url: string
  has_update: boolean
}

export const useUpdateStore = defineStore(
  'update',
  () => {
    // 添加版本号状态
    const appVersion = ref('')

    // 更新相关状态
    const hasUpdate = ref(false)
    const latestVersion = ref('')
    const downloadUrl = ref('')
    
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

    // 检查更新
    const checkUpdate = async (silent: boolean = false): Promise<UpdateInfo | null> => {
      try {
        const updateInfo = await tauriApi.update.checkUpdate(appVersion.value)

        if (updateInfo && updateInfo.has_update) {
          hasUpdate.value = true
          latestVersion.value = updateInfo.latest_version
          downloadUrl.value = updateInfo.download_url

          // 只有在非静默模式下才通知
          if (!silent) {
            mitt.emit('update-available', updateInfo)
          }

          return updateInfo
        }

        return null
      } catch (error) {
        console.error('检查更新失败:', error)
        return null
      }
    }

    // 下载并安装更新
    const downloadAndInstallUpdate = async () => {
      if (!hasUpdate.value || !downloadUrl.value) return false

      try {
        // 通知下载开始
        mitt.emit('download-progress', {
          status: 'checking',
          progress: 0,
          message: '准备下载更新...',
        })

        // 开始下载和安装
        const result = await tauriApi.update.downloadAndInstallUpdate(downloadUrl.value)
        return result
      } catch (error) {
        console.error('下载更新失败:', error)
        return false
      }
    }

    return {
      appVersion,
      hasUpdate,
      latestVersion,
      downloadUrl,
      fetchAppVersion,
      checkUpdate,
      downloadAndInstallUpdate
    }
  },
  {
    persist: true,
  }
)
