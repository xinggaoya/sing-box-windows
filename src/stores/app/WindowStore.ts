import { defineStore } from 'pinia'
import { ref } from 'vue'
import { Window } from '@tauri-apps/api/window'
import type { Router } from 'vue-router'
import { storageService } from '@/services/backend-storage-service'

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

    // 从后端加载数据
    const loadFromBackend = async () => {
      try {
        console.log('🪟 从后端加载窗口配置...')
        const windowConfig = await storageService.getWindowConfig()
        
        // 更新响应式状态
        windowState.value = {
          isVisible: windowConfig.is_visible,
          isFullscreen: windowConfig.is_fullscreen,
          isMaximized: windowConfig.is_maximized,
          lastVisiblePath: windowConfig.last_visible_path,
        }
        
        console.log('🪟 窗口配置加载完成：', windowState.value)
      } catch (error) {
        console.error('从后端加载窗口配置失败:', error)
        // 加载失败时使用默认值
      }
    }

    // 保存配置到后端
    const saveToBackend = async () => {
      try {
        await storageService.updateWindowConfig({
          is_visible: windowState.value.isVisible,
          is_fullscreen: windowState.value.isFullscreen,
          is_maximized: windowState.value.isMaximized,
          last_visible_path: windowState.value.lastVisiblePath,
        })
        console.log('✅ 窗口配置已保存到后端')
      } catch (error) {
        console.error('保存窗口配置到后端失败:', error)
      }
    }

    // 获取应用窗口
    const getAppWindow = () => Window.getCurrent()

    // 最小化窗口
    const minimizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.minimize()
      // 窗口最小化事件现在通过Pinia响应式系统处理
      console.log('窗口已最小化')
    }

    // 隐藏窗口并保存路由状态
    const hideWindow = async (router?: Router) => {
      const appWindow = getAppWindow()
      await appWindow.hide()
      windowState.value.isVisible = false
      
      // 保存到后端
      await saveToBackend()

      // 如果提供了router，保存当前路由并切换到空白页
      if (router) {
        saveRouteAndGoBlank(router)

        // 延迟触发内存清理 - 现在通过Store方法处理
        setTimeout(() => {
          // 可以通过StoreManager触发内存清理
          console.log('请求内存清理')
        }, 1000)
      }
    }

    // 显示窗口
    const showWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.show()
      await appWindow.setFocus()
      windowState.value.isVisible = true
      
      // 保存到后端
      await saveToBackend()
      
      // 窗口显示事件现在通过Pinia响应式系统处理
      console.log('窗口已显示')
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
      
      // 保存到后端
      await saveToBackend()
    }

    // 最大化窗口
    const maximizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.maximize()
      windowState.value.isMaximized = true
      
      // 保存到后端
      await saveToBackend()
      
      // 窗口最大化事件现在通过Pinia响应式系统处理
      console.log('窗口已最大化')
    }

    // 还原窗口
    const unmaximizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.unmaximize()
      windowState.value.isMaximized = false
      
      // 保存到后端
      await saveToBackend()
      
      // 窗口还原事件现在通过Pinia响应式系统处理
      console.log('窗口已还原')
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
        
        // 保存到后端
        await saveToBackend()
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

    // 设置最后可见路径
    const setLastVisiblePath = async (path: string) => {
      windowState.value.lastVisiblePath = path
      await saveToBackend()
    }

    // 初始化方法
    const initializeStore = async () => {
      await loadFromBackend()
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
      setLastVisiblePath,
      initializeStore,
      loadFromBackend,
      saveToBackend,
    }
  },
  // 移除 persist 配置，现在使用后端存储
)