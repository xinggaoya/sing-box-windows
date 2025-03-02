import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { getVersion } from '@tauri-apps/api/app'
import { darkTheme } from 'naive-ui'
import { useOsTheme } from 'naive-ui'
import { Window } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import mitt from '@/utils/mitt'
import { useRouter } from 'vue-router'

// 定义更新信息类型
interface UpdateInfo {
  latest_version: string
  download_url: string
  has_update: boolean
}

// 窗口状态类型
export interface WindowState {
  isVisible: boolean
  isFullscreen: boolean
  lastVisiblePath: string
}

export const useAppStore = defineStore(
  'app',
  () => {
    // 应用运行状态
    const isRunning = ref(false)

    // 代理模式
    const proxyMode = ref<'system' | 'tun'>('system')

    // 自动启动设置
    const autoStartApp = ref(false)
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
    const theme = computed(() => (isDark.value ? darkTheme : null))

    // 窗口状态管理
    const windowState = ref<WindowState>({
      isVisible: true,
      isFullscreen: false,
      lastVisiblePath: '/',
    })

    // 切换主题
    const toggleTheme = () => {
      isDark.value = !isDark.value
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
          proxyMode.value = 'system'
        } else {
          const isAdmin = await tauriApi.proxy.checkAdmin()
          if (!isAdmin) {
            await tauriApi.proxy.restartAsAdmin()
            return false
          }
          await tauriApi.proxy.setTunProxy()
          proxyMode.value = 'tun'
        }
        return true
      } catch (error) {
        console.error('切换代理模式失败:', error)
        return false
      }
    }

    // 私有方法：获取当前窗口引用
    const getAppWindow = () => Window.getCurrent()

    // === 窗口操作方法 ===
    // 最小化窗口
    const minimizeWindow = async () => {
      const appWindow = getAppWindow()
      mitt.emit('window-minimize')
      await appWindow.minimize()
    }

    // 隐藏窗口
    const hideWindow = async () => {
      const appWindow = getAppWindow()
      mitt.emit('window-hide')
      await appWindow.hide()
    }

    // 显示窗口
    const showWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.show()
      mitt.emit('window-show')
    }

    // 切换全屏
    const toggleFullScreen = async () => {
      const appWindow = getAppWindow()
      const isFullscreen = await appWindow.isFullscreen()
      await appWindow.setFullscreen(!isFullscreen)
      windowState.value.isFullscreen = !isFullscreen
      return !isFullscreen
    }

    // 保存路由状态并切换到空白页
    const saveRouteAndGoBlank = (router: any) => {
      windowState.value.lastVisiblePath = router.currentRoute.value.path
      if (windowState.value.lastVisiblePath !== '/blank') {
        router.push('/blank')
      }
    }

    // 从空白页恢复到上次的路由
    const restoreFromBlank = (router: any) => {
      if (router.currentRoute.value.path === '/blank' && windowState.value.lastVisiblePath) {
        router.push(windowState.value.lastVisiblePath)
      }
    }

    // 设置窗口事件处理器
    const setupWindowEventHandlers = (router: any) => {
      // 窗口隐藏时切换到空白页
      mitt.on('window-hide', () => {
        saveRouteAndGoBlank(router)
      })

      // 窗口显示时恢复路由
      mitt.on('window-show', () => {
        restoreFromBlank(router)
      })

      // 窗口恢复时恢复路由
      mitt.on('window-restore', () => {
        restoreFromBlank(router)
      })

      // 检查当前窗口状态
      getAppWindow()
        .isVisible()
        .then((visible) => {
          windowState.value.isVisible = visible
          if (visible) {
            restoreFromBlank(router)
          }
        })
    }

    // 清理窗口事件监听
    const cleanupWindowEvents = () => {
      mitt.off('window-minimize')
      mitt.off('window-hide')
      mitt.off('window-show')
      mitt.off('window-restore')
    }

    return {
      isRunning,
      proxyMode,
      autoStartApp,
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
      // 窗口状态
      windowState,
      // 窗口操作
      minimizeWindow,
      hideWindow,
      showWindow,
      toggleFullScreen,
      saveRouteAndGoBlank,
      restoreFromBlank,
      setupWindowEventHandlers,
      cleanupWindowEvents,
    }
  },
  {
    persist: true, // 使用默认的持久化配置
  },
)
