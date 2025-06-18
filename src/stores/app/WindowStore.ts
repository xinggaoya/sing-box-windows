import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Window } from '@tauri-apps/api/window'
import type { Router } from 'vue-router'
import mitt from '@/utils/mitt'
import { memoryOptimizer } from '@/utils/memory-optimization'

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
      windowState.value.lastVisiblePath = router.currentRoute.value.path
      if (windowState.value.lastVisiblePath !== '/blank') {
        router.push('/blank')
      }
    }

    // 从空白页恢复到上次的路由
    const restoreFromBlank = (router: Router) => {
      if (router.currentRoute.value.path === '/blank' && windowState.value.lastVisiblePath) {
        console.log(`从空白页恢复到之前路径: ${windowState.value.lastVisiblePath}`)
        router.push(windowState.value.lastVisiblePath)
      } else {
        console.log(`当前路径非空白页或没有保存的路径: ${router.currentRoute.value.path}`)
      }
    }

    // 设置窗口事件处理器
    const setupWindowEventHandlers = (router: Router) => {
      // 窗口隐藏时切换到空白页并触发内存清理
      mitt.on('window-hide', () => {
        console.log(`保存当前路径并切换到空白页: ${router.currentRoute.value.path}`)
        saveRouteAndGoBlank(router)

        // 延迟触发内存清理，给页面切换一些时间
        setTimeout(() => {
          console.log('🧹 窗口隐藏，触发内存清理')
          mitt.emit('memory-cleanup-requested')
        }, 1000)
      })

      // 窗口显示时恢复路由并恢复图片资源
      mitt.on('window-show', () => {
        console.log('接收到窗口显示事件，准备恢复路由')
        restoreFromBlank(router)

        // 恢复图片资源
        setTimeout(() => {
          memoryOptimizer.restoreImageResources()
        }, 500)
      })

      // 窗口恢复时恢复路由
      mitt.on('window-restore', () => {
        console.log('接收到窗口恢复事件，准备恢复路由')
        restoreFromBlank(router)
      })

      // 窗口最大化事件
      mitt.on('window-maximize', () => {
        console.log('窗口已最大化')
        updateWindowState()
      })

      // 窗口还原事件
      mitt.on('window-unmaximize', () => {
        console.log('窗口已还原')
        updateWindowState()
      })

      // 检查当前窗口状态
      updateWindowState().then(() => {
        if (windowState.value.isVisible) {
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
      mitt.off('window-maximize')
      mitt.off('window-unmaximize')
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
      restoreFromBlank,
      setupWindowEventHandlers,
      cleanupWindowEvents,
    }
  },
  {
    persist: true,
  },
)
