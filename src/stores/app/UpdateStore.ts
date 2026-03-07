import { openUrl } from '@tauri-apps/plugin-opener'
import { defineStore } from 'pinia'
import { ref, watch, computed } from 'vue'
import { systemService, type AppUpdateInfo } from '@/services/system-service'
import { getVersion } from '@tauri-apps/api/app'
import { DatabaseService } from '@/services/database-service'
import type { UpdateConfig } from '@/types/database'

export type UpdateChannel = 'stable' | 'prerelease' | 'autobuild'
type PlatformOs = 'windows' | 'linux' | 'macos' | 'unknown'

interface UpdateState {
  checking: boolean
  downloading: boolean
  progress: number
  status: 'idle' | 'checking' | 'downloading' | 'completed' | 'error' | 'installing'
  message: string
  error: string | null
}

const normalizeUpdateChannel = (value: string | null | undefined): UpdateChannel | null => {
  if (value === 'stable' || value === 'prerelease' || value === 'autobuild') {
    return value
  }
  return null
}

const normalizePlatformOs = (value: string | null | undefined): PlatformOs => {
  if (value === 'windows' || value === 'linux' || value === 'macos') {
    return value
  }
  return 'unknown'
}

export const useUpdateStore = defineStore('update', () => {
  const appVersion = ref('')

  const hasUpdate = ref(false)
  const latestVersion = ref('')
  const downloadUrl = ref('')
  const releasePageUrl = ref('')
  const releaseNotes = ref('')
  const releaseDate = ref('')
  const fileSize = ref(0)

  const platformOs = ref<PlatformOs>('unknown')
  const supportsInAppUpdate = ref(false)

  const updateState = ref<UpdateState>({
    checking: false,
    downloading: false,
    progress: 0,
    status: 'idle',
    message: '',
    error: null,
  })

  const autoCheckUpdate = ref(true)
  const skipVersion = ref('')
  const updateChannel = ref<UpdateChannel>('stable')
  const acceptPrerelease = ref(false)
  const lastCheck = ref(0)
  const isPrerelease = ref(false)

  const syncPlatformCapability = (os: string | null | undefined, supports?: boolean) => {
    platformOs.value = normalizePlatformOs(os)
    if (typeof supports === 'boolean') {
      supportsInAppUpdate.value = supports
      return
    }
    supportsInAppUpdate.value = platformOs.value === 'windows'
  }

  const loadPlatformInfo = async () => {
    try {
      const os = await systemService.getPlatformInfo()
      syncPlatformCapability(os)
    } catch (error) {
      console.error('获取更新平台信息失败:', error)
    }
  }

  const loadFromBackend = async () => {
    try {
      console.log('?? 从数据库加载更新配置...')
      const updateConfig = await DatabaseService.getUpdateConfig()

      autoCheckUpdate.value = updateConfig.auto_check
      skipVersion.value = updateConfig.skip_version || ''
      const persistedChannel = normalizeUpdateChannel(updateConfig.update_channel)
      updateChannel.value =
        persistedChannel || (updateConfig.accept_prerelease ? 'prerelease' : 'stable')
      acceptPrerelease.value = updateChannel.value !== 'stable'
      lastCheck.value = updateConfig.last_check ?? 0

      await fetchAppVersion(false)

      console.log('?? 更新配置加载完成：', {
        appVersion: appVersion.value,
        autoCheckUpdate: autoCheckUpdate.value,
        skipVersion: skipVersion.value,
        lastCheck: lastCheck.value,
      })
    } catch (error) {
      console.error('从数据库加载更新配置失败:', error)
      await fetchAppVersion(false)
    }
  }

  const saveToBackend = async (options?: { touchLastCheck?: boolean }) => {
    try {
      const nextLastCheck = options?.touchLastCheck
        ? Math.floor(Date.now() / 1000)
        : lastCheck.value
      lastCheck.value = nextLastCheck
      const config: UpdateConfig = {
        auto_check: autoCheckUpdate.value,
        last_check: nextLastCheck,
        last_version: appVersion.value || null,
        skip_version: skipVersion.value || null,
        accept_prerelease: acceptPrerelease.value,
        update_channel: updateChannel.value,
      }
      await DatabaseService.saveUpdateConfig(config)
      console.log('? 更新配置已保存到数据库')
    } catch (error) {
      console.error('保存更新配置到数据库失败:', error)
    }
  }

  const fetchAppVersion = async (persist: boolean = false) => {
    try {
      appVersion.value = await getVersion()
      if (persist) {
        await saveToBackend({ touchLastCheck: false })
      }
      return appVersion.value
    } catch (error) {
      console.error('获取应用版本失败:', error)
      return ''
    }
  }

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

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

  const applyUpdateInfo = (info: AppUpdateInfo) => {
    hasUpdate.value = info.has_update
    latestVersion.value = info.latest_version
    downloadUrl.value = info.download_url || ''
    releasePageUrl.value = info.release_page_url || ''
    releaseNotes.value = info.release_notes || ''
    releaseDate.value = info.release_date || ''
    fileSize.value = info.file_size || 0
    isPrerelease.value = info.is_prerelease || false
    syncPlatformCapability(platformOs.value, info.supports_in_app_update)
  }

  const checkUpdate = async (silent: boolean = false): Promise<AppUpdateInfo | null> => {
    if (silent && skipVersion.value && skipVersion.value === latestVersion.value) {
      return null
    }

    try {
      updateState.value.checking = true
      updateState.value.status = 'checking'
      updateState.value.error = null
      updateState.value.message = '正在检查更新...'

      if (!appVersion.value) {
        await fetchAppVersion(false)
      }

      const updateInfo = await systemService.checkUpdate(
        appVersion.value,
        acceptPrerelease.value,
        updateChannel.value,
      )

      if (updateInfo) {
        applyUpdateInfo(updateInfo)
      }

      if (updateInfo && updateInfo.has_update) {
        const versionType = updateInfo.is_prerelease ? '测试版本' : '正式版本'
        updateState.value.message = `发现新${versionType} ${updateInfo.latest_version}`
        console.log('发现新版本:', updateInfo.latest_version)
        return updateInfo
      }

      hasUpdate.value = false
      updateState.value.message = '已是最新版本'
      return null
    } catch (error) {
      console.error('检查更新失败:', error)
      updateState.value.error = error instanceof Error ? error.message : String(error)
      updateState.value.message = '检查更新失败'
      updateState.value.status = 'error'
      return null
    } finally {
      updateState.value.checking = false
      if (updateState.value.status === 'checking') {
        updateState.value.status = 'idle'
      }
      await saveToBackend({ touchLastCheck: true })
    }
  }

  const downloadAndInstallUpdate = async () => {
    if (!supportsInAppUpdate.value) {
      const message = '当前平台暂不支持应用内更新，请前往版本页面下载最新版本'
      updateState.value.downloading = false
      updateState.value.error = message
      updateState.value.status = 'error'
      updateState.value.message = message
      return false
    }

    if (!hasUpdate.value || !downloadUrl.value) return false

    try {
      updateState.value.downloading = true
      updateState.value.status = 'downloading'
      updateState.value.progress = 0
      updateState.value.error = null

      console.log('开始下载更新:', {
        status: 'downloading',
        progress: 0,
        message: '准备下载更新...',
      })

      const result = await systemService.downloadAndInstallUpdate(downloadUrl.value)
      return result
    } catch (error) {
      console.error('下载更新失败:', error)
      const message = error instanceof Error ? error.message : String(error)
      updateState.value.downloading = false
      updateState.value.error = message
      updateState.value.status = 'error'
      updateState.value.message = `下载更新失败: ${message}`
      return false
    }
  }

  const openReleasePage = async () => {
    if (!releasePageUrl.value) {
      throw new Error('未获取到版本页面链接')
    }

    await openUrl(releasePageUrl.value)
    return true
  }

  const skipCurrentVersion = async () => {
    skipVersion.value = latestVersion.value
    hasUpdate.value = false
    await saveToBackend()
  }

  const resetUpdateState = () => {
    hasUpdate.value = false
    latestVersion.value = ''
    downloadUrl.value = ''
    releasePageUrl.value = ''
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

  const setAutoCheckUpdate = async (enabled: boolean) => {
    autoCheckUpdate.value = enabled
  }

  const setAcceptPrerelease = async (accept: boolean) => {
    acceptPrerelease.value = accept
    updateChannel.value = accept ? 'prerelease' : 'stable'
    await saveToBackend({ touchLastCheck: false })
  }

  const setUpdateChannel = async (channel: UpdateChannel) => {
    updateChannel.value = channel
    acceptPrerelease.value = channel !== 'stable'
    await saveToBackend({ touchLastCheck: false })
  }

  let isInitializing = false

  watch(
    [autoCheckUpdate, skipVersion, acceptPrerelease, updateChannel],
    async () => {
      if (isInitializing) return
      await saveToBackend()
    },
    { deep: true },
  )

  const initializeStore = async () => {
    isInitializing = true
    await Promise.all([loadFromBackend(), loadPlatformInfo()])
    await new Promise((resolve) => setTimeout(resolve, 100))
    isInitializing = false
  }

  return {
    appVersion,
    hasUpdate,
    latestVersion,
    downloadUrl,
    releasePageUrl,
    releaseNotes,
    releaseDate,
    fileSize,
    platformOs,
    supportsInAppUpdate,
    updateState,
    autoCheckUpdate,
    skipVersion,
    updateChannel,
    acceptPrerelease,
    isPrerelease,

    fetchAppVersion,
    checkUpdate,
    applyUpdateInfo,
    downloadAndInstallUpdate,
    openReleasePage,
    skipCurrentVersion,
    resetUpdateState,
    updateProgress,
    setAutoCheckUpdate,
    setAcceptPrerelease,
    setUpdateChannel,
    loadPlatformInfo,

    formattedFileSize: computed(() => formatFileSize(fileSize.value)),
    formattedReleaseDate: computed(() =>
      releaseDate.value ? formatReleaseDate(releaseDate.value) : '',
    ),
    isChecking: computed(() => updateState.value.checking),
    canOpenReleasePage: computed(() => releasePageUrl.value.length > 0),

    initializeStore,
    loadFromBackend,
    saveToBackend,
  }
})
