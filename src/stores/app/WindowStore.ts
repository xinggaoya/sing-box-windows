import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Window } from '@tauri-apps/api/window'
import type { Router } from 'vue-router'
import mitt from '@/utils/mitt'

// 窗口状态类型
export interface WindowState {
  isVisible: boolean
  isFullscreen: boolean
  isMaximized: boolean
  lastVisiblePath: string
}

export const useWindowStore = defineStore(
  'window',
  () => {
    // 窗口状态
    const windowState = ref<WindowState>({
      isVisible: true,
      isFullscreen: false,
      isMaximized: false,
      lastVisiblePath: '/',
    })

    // 获取应用窗口
    const getAppWindow = () => Window.getCurrent()

    // 最小化窗口
    const minimizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.minimize()
      // 触发最小化事件
      mitt.emit('window-minimize')
    }

    // 隐藏窗口并保存路由状态
    const hideWindow = async (router?: Router) => {
      const appWindow = getAppWindow()
      await appWindow.hide()
      windowState.value.isVisible = false

      // 如果提供了router，保存当前路由并切换到空白页
      if (router) {
        saveRouteAndGoBlank(router)

        // 延迟触发内存清理
        setTimeout(() => {
          mitt.emit('memory-cleanup-requested')
        }, 1000)
      }
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

    // 最大化窗口
    const maximizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.maximize()
      windowState.value.isMaximized = true
      // 触发最大化事件
      mitt.emit('window-maximize')
    }

    // 还原窗口
    const unmaximizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.unmaximize()
      windowState.value.isMaximized = false
      // 触发还原事件
      mitt.emit('window-unmaximize')
    }

    // 切换最大化状态
    const toggleMaximize = async () => {
      const appWindow = getAppWindow()
      const isMaximized = await appWindow.isMaximized()

      if (isMaximized) {
        await unmaximizeWindow()
      } else {
        await maximizeWindow()
      }
    }

    // 检查窗口状态
    const updateWindowState = async () => {
      const appWindow = getAppWindow()
      try {
        const [isVisible, isFullscreen, isMaximized] = await Promise.all([
          appWindow.isVisible(),
          appWindow.isFullscreen(),
          appWindow.isMaximized(),
        ])

        windowState.value.isVisible = isVisible
        windowState.value.isFullscreen = isFullscreen
        windowState.value.isMaximized = isMaximized
      } catch (error) {
        console.error('更新窗口状态失败:', error)
      }
    }

    // 保存路由状态并切换到空白页
    const saveRouteAndGoBlank = (router: Router) => {
      const currentPath = router.currentRoute.value.path

      // 只有当前路径不是空白页时才保存
      if (currentPath !== '/blank') {
        windowState.value.lastVisiblePath = currentPath
        router.push('/blank').catch((error) => {
          console.error('切换到空白页面失败:', error)
        })
      }
    }

    return {
      windowState,
      getAppWindow,
      minimizeWindow,
      maximizeWindow,
      unmaximizeWindow,
      toggleMaximize,
      updateWindowState,
      hideWindow,
      showWindow,
      setWindowAlwaysOnTop,
      getWindowVisible,
      toggleFullScreen,
      saveRouteAndGoBlank,
    }
  },
  {
    persist: true,
  },
)
