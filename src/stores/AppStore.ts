import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { tauriApi } from '@/services/tauri-api'
import { getVersion } from '@tauri-apps/api/app'
import { darkTheme } from 'naive-ui'
import { useOsTheme } from 'naive-ui'
import { Window } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import mitt from '@/utils/mitt'
import { useRouter, Router } from 'vue-router'
import { supportedLocales } from '@/locales'

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

// 语言类型
export type Locale = 'zh-CN' | 'en-US' | 'auto'

export const useAppStore = defineStore(
  'app',
  () => {
    // 应用运行状态
    const isRunning = ref(false)

    // 托盘实例ID - 由TrayStore使用
    const trayInstanceId = ref<string | null>(null)

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

    // 添加语言设置
    const locale = ref<Locale>('auto')

    // 窗口状态
    const windowState = ref<WindowState>({
      isVisible: true,
      isFullscreen: false,
      lastVisiblePath: '/',
    })

    // 主题相关状态
    const osTheme = useOsTheme()
    const isDark = ref(osTheme.value === 'dark')
    const theme = computed(() => (isDark.value ? darkTheme : null))

    // 计算实际使用的语言
    const currentLocale = computed(() => {
      if (locale.value === 'auto') {
        // 获取浏览器语言
        const browserLang = navigator.language
        // 检查是否支持这个语言
        const isSupported = supportedLocales.some((loc) => loc.code === browserLang)
        return isSupported ? browserLang : 'zh-CN'
      }
      return locale.value
    })

    // 语言切换
    const setLocale = (newLocale: Locale) => {
      locale.value = newLocale
    }

    // 主题切换
    const toggleTheme = () => {
      isDark.value = !isDark.value
    }

    // 获取应用版本
    const fetchAppVersion = async () => {
      try {
        appVersion.value = await getVersion()
      } catch (error) {
        console.error('获取应用版本失败:', error)
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

    // 应用运行状态变更
    const setRunningState = (state: boolean) => {
      if (isRunning.value !== state) {
        isRunning.value = state
        // 发送进程状态变更事件
        mitt.emit('process-status')
      }
    }

    // 代理模式切换
    const switchProxyMode = async (targetMode: 'system' | 'tun') => {
      // 如果当前模式与目标模式相同，则不需要切换
      if (proxyMode.value === targetMode) return

      // 根据模式调用对应服务
      try {
        if (targetMode === 'system') {
          await tauriApi.proxy.setSystemProxy()
        } else {
          // TUN模式可能需要管理员权限，检查并处理
          const isAdmin = await tauriApi.proxy.checkAdmin()
          if (!isAdmin) {
            // 需要管理员权限，实现重启
            await tauriApi.proxy.restartAsAdmin()
            return
          }
          await tauriApi.proxy.setTunProxy()
        }

        // 切换成功后更新状态
        proxyMode.value = targetMode

        // 发出代理模式变更事件，通知其他组件
        mitt.emit('proxy-mode-changed')
      } catch (error) {
        console.error('切换代理模式失败:', error)
        throw error
      }
    }

    // 获取应用窗口
    const getAppWindow = () => Window.getCurrent()

    // 最小化窗口
    const minimizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.minimize()
      // 触发最小化事件
      mitt.emit('window-minimize')
    }

    // 隐藏窗口
    const hideWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.hide()
      windowState.value.isVisible = false
      // 触发隐藏事件
      mitt.emit('window-hide')
    }

    // 显示窗口
    const showWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.show()
      await appWindow.setFocus()
      windowState.value.isVisible = true
      // 触发显示事件
      mitt.emit('window-show')
    }

    // 设置窗口置顶
    const setWindowAlwaysOnTop = async () => {
      const appWindow = getAppWindow()
      await appWindow.setAlwaysOnTop(true)
    }

    // 获取窗口可见状态
    const getWindowVisible = async () => {
      const appWindow = getAppWindow()
      return await appWindow.isVisible()
    }

    // 切换全屏模式
    const toggleFullScreen = async () => {
      const appWindow = getAppWindow()
      const isFullscreen = await appWindow.isFullscreen()

      if (isFullscreen) {
        await appWindow.setFullscreen(false)
      } else {
        await appWindow.setFullscreen(true)
      }

      windowState.value.isFullscreen = !isFullscreen
    }

    // 保存路由状态并切换到空白页
    const saveRouteAndGoBlank = (router: Router) => {
      windowState.value.lastVisiblePath = router.currentRoute.value.path
      if (windowState.value.lastVisiblePath !== '/blank') {
        router.push('/blank')
      }
    }

    // 从空白页恢复到上次的路由
    const restoreFromBlank = (router: Router) => {
      if (router.currentRoute.value.path === '/blank' && windowState.value.lastVisiblePath) {
        router.push(windowState.value.lastVisiblePath)
      }
    }

    // 设置窗口事件处理器
    const setupWindowEventHandlers = (router: Router) => {
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
      trayInstanceId,
      isRunning,
      proxyMode,
      autoStartApp,
      autoStartKernel,
      preferIpv6,
      appVersion,
      hasUpdate,
      latestVersion,
      downloadUrl,
      windowState,
      isDark,
      theme,
      locale,
      currentLocale,
      toggleTheme,
      setLocale,
      fetchAppVersion,
      checkUpdate,
      downloadAndInstallUpdate,
      setRunningState,
      switchProxyMode,
      getAppWindow,
      minimizeWindow,
      hideWindow,
      showWindow,
      setWindowAlwaysOnTop,
      getWindowVisible,
      toggleFullScreen,
      saveRouteAndGoBlank,
      restoreFromBlank,
      setupWindowEventHandlers,
      cleanupWindowEvents,
    }
  },
  {
    persist: {
      paths: ['isDark', 'proxyMode', 'autoStartApp', 'autoStartKernel', 'preferIpv6', 'locale'],
    },
  },
)
