import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { getVersion } from '@tauri-apps/api/app'
import { darkTheme } from 'naive-ui'
import { useOsTheme } from 'naive-ui'

// 定义更新信息类型
interface UpdateInfo {
  latest_version: string
  download_url: string
  has_update: boolean
}

export const useAppStore = defineStore(
  'app',
  () => {
    // 应用运行状态
    const isRunning = ref(false)

    // 代理模式
    const mode = ref<'system' | 'tun'>('system')

    // 自动启动设置
    const autoStart = ref(false)
    const autoStartKernel = ref(false)

    // IP版本设置
    const preferIpv6 = ref(false)

    // 添加版本号状态
    const appVersion = ref('')

    // 更新相关状态
    const hasUpdate = ref(false)
    const latestVersion = ref('')
    const downloadUrl = ref('')

    // 主题相关状态
    const osThemeRef = useOsTheme()
    const isDark = ref(osThemeRef.value === 'dark')
    const theme = ref(isDark.value ? darkTheme : null)

    // 切换主题
    const toggleTheme = () => {
      isDark.value = !isDark.value
      theme.value = isDark.value ? darkTheme : null
    }

    // 获取应用版本号
    const fetchAppVersion = async () => {
      try {
        appVersion.value = await getVersion()
      } catch (error) {
        console.error('获取版本号失败:', error)
      }
    }

    // 检查更新
    const checkUpdate = async (silent: boolean = false): Promise<UpdateInfo | null> => {
      try {
        await fetchAppVersion() // 确保有最新版本号
        const result = await tauriApi.update.checkUpdate(appVersion.value)
        if (result.has_update) {
          hasUpdate.value = true
          latestVersion.value = result.latest_version
          downloadUrl.value = result.download_url
          return result
        }
        if (!silent) {
          return {
            has_update: false,
            latest_version: appVersion.value,
            download_url: '',
          }
        }
      } catch (error) {
        console.error('检查更新失败:', error)
        if (!silent) {
          throw error
        }
      }
      return null
    }

    // 下载并安装更新
    const downloadAndInstallUpdate = async () => {
      try {
        await tauriApi.update.downloadAndInstallUpdate(downloadUrl.value)
      } catch (error) {
        console.error('更新失败:', error)
        throw error
      }
    }

    // 切换代理模式
    const switchProxyMode = async (targetMode: 'system' | 'tun') => {
      try {
        if (targetMode === 'system') {
          await tauriApi.proxy.setSystemProxy()
          mode.value = 'system'
        } else {
          const isAdmin = await tauriApi.proxy.checkAdmin()
          if (!isAdmin) {
            await tauriApi.proxy.restartAsAdmin()
            return false
          }
          await tauriApi.proxy.setTunProxy()
          mode.value = 'tun'
        }
        return true
      } catch (error) {
        console.error('切换代理模式失败:', error)
        return false
      }
    }

    return {
      isRunning,
      mode,
      autoStart,
      autoStartKernel,
      preferIpv6,
      switchProxyMode,
      appVersion,
      fetchAppVersion,
      // 导出更新相关方法和状态
      hasUpdate,
      latestVersion,
      downloadUrl,
      checkUpdate,
      downloadAndInstallUpdate,
      // 导出主题相关状态和方法
      isDark,
      theme,
      toggleTheme,
    }
  },
  {
    persist: true, // 使用默认的持久化配置
  },
)
