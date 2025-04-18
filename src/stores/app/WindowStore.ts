import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Window } from '@tauri-apps/api/window'
import type { Router } from 'vue-router'
import mitt from '@/utils/mitt'

// 窗口状态类型
export interface WindowState {
  isVisible: boolean
  isFullscreen: boolean
  lastVisiblePath: string
}

export const useWindowStore = defineStore(
  'window',
  () => {
    // 窗口状态
    const windowState = ref<WindowState>({
      isVisible: true,
      isFullscreen: false,
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
      windowState,
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
    persist: true,
  }
)
