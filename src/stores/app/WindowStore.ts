import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { Window } from '@tauri-apps/api/window'
import type { Router } from 'vue-router'
import { DatabaseService } from '@/services/database-service'
import type { WindowConfig } from '@/types/database'
import { useAppStore } from './AppStore'

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
    const appStore = useAppStore()

    // 窗口状态
    const windowState = ref<WindowState>({
      isVisible: true,
      isFullscreen: false,
      isMaximized: false,
      lastVisiblePath: '/',
    })

    // 从数据库加载数据
    const loadFromBackend = async () => {
      try {
        console.log('?? 从数据库加载窗口配置...')
        const windowConfig = await DatabaseService.getWindowConfig()
        
        // 更新响应式状态
        windowState.value = {
          isVisible: true, // 数据库没有is_visible，默认为true
          isFullscreen: false, // 数据库没有is_fullscreen，默认为false
          isMaximized: windowConfig.is_maximized,
          lastVisiblePath: '/', // 数据库没有last_visible_path，使用默认值
        }
        
        console.log('?? 窗口配置加载完成：', windowState.value)
      } catch (error) {
        console.error('从数据库加载窗口配置失败:', error)
        // 加载失败时使用默认值
      }
    }

    const resolveWindowSize = async (): Promise<{ width: number, height: number }> => {
      try {
        const size = await getAppWindow().innerSize()
        return { width: size.width, height: size.height }
      } catch (error) {
        console.warn('读取窗口大小失败，使用默认值', error)
        return { width: 1000, height: 700 }
      }
    }

    // 保存配置到数据库
    const saveToBackend = async () => {
      try {
        const { width, height } = await resolveWindowSize()
        const config: WindowConfig = {
          is_maximized: windowState.value.isMaximized,
          width,
          height,
        }
        await DatabaseService.saveWindowConfig(config)
        console.log('? 窗口配置已保存到数据库')
      } catch (error) {
        console.error('保存窗口配置到数据库失败:', error)
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

      // 隐藏到托盘时立即清理所有消息，避免进入空白页后消息无法自行消失
      appStore.clearMessages()
      
      // 保存到数据库
      await saveToBackend()

      // 如果提供了router，保存当前路由并切换到空白页
      if (router) {
        saveRouteAndGoBlank(router)
      }
    }

    // 显示窗口
    const showWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.show()
      await appWindow.setFocus()
      windowState.value.isVisible = true
      
      // 保存到数据库
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
      
      // 保存到数据库
      await saveToBackend()
    }

    // 最大化窗口
    const maximizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.maximize()
      windowState.value.isMaximized = true
      
      // 保存到数据库
      await saveToBackend()
      
      // 窗口最大化事件现在通过Pinia响应式系统处理
      console.log('窗口已最大化')
    }

    // 还原窗口
    const unmaximizeWindow = async () => {
      const appWindow = getAppWindow()
      await appWindow.unmaximize()
      windowState.value.isMaximized = false
      
      // 保存到数据库
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
        
        // 保存到数据库
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

    // 标记是否正在初始化
    let isInitializing = false
    
    // 监听窗口状态变化并自动保存到数据库
    watch(
      () => windowState.value.isMaximized,
      async () => {
        // 初始化期间不保存
        if (isInitializing) return
        await saveToBackend()
      }
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
