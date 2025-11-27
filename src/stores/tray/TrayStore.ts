// TrayStore.ts - 管理应用程序托盘功能的Store
import { defineStore } from 'pinia'
import { ref, watch, type WatchStopHandle } from 'vue'
import { TrayIcon, TrayIconEvent } from '@tauri-apps/api/tray'
import { Window } from '@tauri-apps/api/window'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { MenuItem } from '@tauri-apps/api/menu/menuItem'
import { Submenu } from '@tauri-apps/api/menu/submenu'
import { useAppStore } from '@/stores'
import { useSubStore } from '@/stores'
import { useKernelStore } from '@/stores'
import { useWindowStore } from '@/stores/app/WindowStore'
import i18n from '@/locales'
import type { ProxyMode } from '@/types'
import { useRouter } from 'vue-router'
import { systemService } from '@/services/system-service'

// 自定义菜单项类型定义
export interface TrayMenuOptions {
  type?: 'normal' | 'separator' | 'checkbox'
  id: string
  text: string
  checked?: boolean
  enabled?: boolean
  action?: () => Promise<void>
  children?: TrayMenuOptions[]
}

export const useTrayStore = defineStore('tray', () => {
  // 引用其他Store
  const appStore = useAppStore()
  const subStore = useSubStore()
  const router = useRouter()
  const kernelStore = useKernelStore()
  const windowStore = useWindowStore()

  // 添加一个内部状态，记录上次菜单刷新时的代理模式
  const lastProxyMode = ref<ProxyMode>(appStore.proxyMode)

  // 只存储托盘ID，不存储实例（与 AppStore 持久化字段保持同步）
  const trayInstanceId = ref<string | null>(appStore.trayInstanceId)
  const setTrayInstanceId = (id: string | null) => {
    trayInstanceId.value = id
    appStore.trayInstanceId = id
  }

  // watcher 清理集合，避免重复注册
  const trayWatchers: WatchStopHandle[] = []
  const registerWatcher = (...args: Parameters<typeof watch>) => {
    const stop = watch(...args)
    trayWatchers.push(stop)
    return stop
  }
  const cleanupWatchers = () => {
    while (trayWatchers.length) {
      const stop = trayWatchers.pop()
      stop?.()
    }
  }

  // 获取翻译函数
  const t = i18n.global.t

  /**
   * 处理路由恢复（直接方法，不依赖事件系统）
   */
  const handleRouteRestore = async () => {
    try {
      const currentPath = router.currentRoute.value.path

      // 如果当前在空白页面，尝试恢复
      if (currentPath === '/blank') {
        const savedPath = windowStore.windowState.lastVisiblePath

        if (savedPath && savedPath !== '/blank') {
          await router.push(savedPath)
        } else {
          await router.push('/')
        }
      }
    } catch (error) {
      console.error('路由恢复失败:', error)
      // 如果恢复失败，至少回到首页
      try {
        await router.push('/')
      } catch (fallbackError) {
        console.error('恢复到首页失败:', fallbackError)
      }
    }
  }

  /**
   * 更新托盘提示信息
   */
  const updateTrayTooltip = async () => {
    const currentId = trayInstanceId.value ?? appStore.trayInstanceId
    if (currentId) {
      try {
        const status = appStore.isRunning ? t('status.running') : t('status.stopped')
        const mode =
          appStore.proxyMode === 'system'
            ? t('home.proxyMode.system')
            : appStore.proxyMode === 'manual'
              ? t('home.proxyMode.manual')
              : t('home.proxyMode.tun')

        // 获取当前使用的配置名称
        let configName = ''
        if (subStore.activeIndex !== null && subStore.list.length > 0) {
          configName = subStore.list[subStore.activeIndex].name
        }

        // 构建提示文本
        let tooltipText = `sing-box-window - ${t('tray.kernel')}${status}, ${mode}`

        // 如果有配置名称，则显示
        if (configName) {
          tooltipText += `, ${t('sub.title')}: ${configName}`
        }

        // 获取托盘实例并更新提示
        const trayIcon = await TrayIcon.getById(currentId)
        if (trayIcon) {
          await trayIcon.setTooltip(tooltipText)
          console.log('更新托盘提示成功:', tooltipText)
        } else {
          console.warn('托盘实例不存在，无法更新提示')
        }
      } catch (e) {
        console.error('更新托盘提示失败:', e)
      }
    }
  }

  /**
   * 托盘代理模式切换
   */
  const switchProxyModeFromTray = async (targetMode: ProxyMode) => {
    const previousMode = appStore.proxyMode as ProxyMode
    if (previousMode === targetMode) {
      return
    }

    // TUN模式特殊处理
    if (targetMode === 'tun') {
      const isAdmin = await systemService.checkAdmin()
      if (!isAdmin) {
        try {
          // 保存TUN启用状态
          await appStore.toggleTun(true)
          await appStore.saveToBackend()

          if (appStore.isRunning) {
            await kernelStore.stopKernel({ force: true })
          }
          await systemService.restartAsAdmin()
          return
        } catch (error) {
          console.error('以管理员身份重启以启用TUN失败:', error)
          await appStore.toggleTun(false)
          await refreshTrayMenu()
          return
        }
      } else {
        // 已是管理员，直接启用TUN
        try {
          await appStore.toggleTun(true)
          await appStore.toggleSystemProxy(false) // 互斥：关闭系统代理

          const success = await kernelStore.restartKernel()
          if (!success) {
            throw new Error(kernelStore.lastError || '内核重启失败')
          }
        } catch (error) {
          console.error('启用TUN模式失败:', error)
          await appStore.toggleTun(false)
          await refreshTrayMenu()
        }
        return
      }
    }

    // 其他模式（System/Manual）
    try {
      if (targetMode === 'system') {
        await appStore.toggleSystemProxy(true)
        await appStore.toggleTun(false)
      } else if (targetMode === 'manual') {
        await appStore.toggleSystemProxy(false)
        await appStore.toggleTun(false)
      }

      // 切换非TUN模式不需要重启内核，直接调用后端API
      const success = await kernelStore.switchProxyMode(targetMode)
      if (!success) {
        throw new Error(kernelStore.lastError || '代理模式切换失败')
      }
      await kernelStore.refreshStatus()
    } catch (error) {
      console.error('托盘切换代理模式失败:', error)
      // 恢复之前的状态
      if (previousMode === 'tun') {
        await appStore.toggleTun(true)
      } else if (previousMode === 'system') {
        await appStore.toggleSystemProxy(true)
      } else {
        await appStore.toggleSystemProxy(false)
        await appStore.toggleTun(false)
      }
    } finally {
      await refreshTrayMenu()
    }
  }

  /**
   * 创建托盘菜单
   */
  const createTrayMenu = async () => {
    try {
      // 同步当前代理模式，确保菜单使用最新的状态
      const currentProxyMode = appStore.proxyMode

      // 更新上次菜单刷新时的代理模式
      lastProxyMode.value = currentProxyMode

      console.log(`创建托盘菜单, 当前代理模式: ${currentProxyMode}`)

      // 创建基本菜单项
      const showMenuItem = await MenuItem.new({
        id: 'show',
        text: t('tray.show'),
        enabled: true,
        action: async () => {
          const appWindow = Window.getCurrent()
          await appWindow.show()
          await appWindow.setFocus()

          // 直接处理路由恢复，不依赖事件系统
          await handleRouteRestore()
        },
      })

      // 创建内核控制子菜单项
      const restartMenuItem = await MenuItem.new({
        id: 'restart',
        text: t('home.restart'),
        enabled: appStore.isRunning,
        action: async () => {
          await kernelStore.restartKernel()
          await refreshTrayMenu() // 刷新菜单以更新状态
        },
      })

      const statusMenuItem = await MenuItem.new({
        id: 'kernel_status',
        text: appStore.isRunning ? t('status.running') : t('status.stopped'),
        enabled: false,
      })

      // 创建内核控制子菜单
      const kernelSubmenu = await Submenu.new({
        id: 'kernel_control',
        text: t('setting.kernel.title'),
        items: [statusMenuItem, restartMenuItem],
      })

      const createProxyMenuItem = async (mode: ProxyMode, label: string) => {
        const isActive = currentProxyMode === mode
        return MenuItem.new({
          id: `proxy_mode_${mode}`,
          text: `${isActive ? '✓ ' : ''}${label}`,
          enabled: !isActive,
          action: async () => {
            await switchProxyModeFromTray(mode)
          },
        })
      }

      const systemProxyMenuItem = await createProxyMenuItem(
        'system',
        t('home.proxyMode.system'),
      )
      const tunProxyMenuItem = await createProxyMenuItem('tun', t('home.proxyMode.tun'))
      const manualProxyMenuItem = await createProxyMenuItem(
        'manual',
        t('home.proxyMode.manual'),
      )

      // 当前模式指示器菜单项（仅作为标签，不可点击）
      const currentModeMenuItem = await MenuItem.new({
        id: 'current_mode',
        text: `${t('proxy.currentMode')} ${currentProxyMode === 'system'
          ? t('home.proxyMode.system')
          : currentProxyMode === 'manual'
            ? t('home.proxyMode.manual')
            : t('home.proxyMode.tun')
          }`,
        enabled: false,
      })

      // 创建代理模式子菜单
      const proxyModeSubmenu = await Submenu.new({
        id: 'proxy_mode',
        text: t('home.switchMode'),
        items: [currentModeMenuItem, systemProxyMenuItem, tunProxyMenuItem, manualProxyMenuItem],
      })

      // 创建退出菜单项
      const quitMenuItem = await MenuItem.new({
        id: 'quit',
        text: t('tray.quit'),
        action: async () => {
          try {
            // 交给后端强制停止并退出，前端快速返回
            kernelStore.forceStopAndExit().catch((e) => {
              console.warn('强制停止内核并退出失败（已忽略以便退出）：', e)
            })
          } catch (e) {
            console.error('退出应用失败:', e)
          }
        },
      })

      // 创建主菜单
      return await Menu.new({
        items: [showMenuItem, kernelSubmenu, proxyModeSubmenu, quitMenuItem],
      })
    } catch (error) {
      console.error('创建菜单失败:', error)
      // 返回一个空菜单
      return await Menu.new({ items: [] })
    }
  }

  /**
   * 初始化托盘
   */
  const initTray = async () => {
    try {
      // 清理之前的托盘实例（如果存在），解决热刷新重复托盘
      const existingId = trayInstanceId.value ?? appStore.trayInstanceId
      if (existingId) {
        try {
          await TrayIcon.removeById(existingId)
        } catch (removeError) {
          console.warn('移除旧托盘失败（可能已被系统清理）:', removeError)
        } finally {
          setTrayInstanceId(null)
        }
      }

      // 创建菜单
      const menu = await createTrayMenu()

      // 设置托盘图标
      const icon = await defaultWindowIcon()

      // 确保图标不为null
      if (!icon) {
        throw new Error('无法获取默认窗口图标')
      }

      const options = {
        icon, // 直接使用非null的图标
        tooltip: 'sing-box-window', // 初始化工具提示
        action: async (event: TrayIconEvent) => {
          switch (event.type) {
            case 'Click':
              // 如果点击的是左键，则显示界面
              if (event.button === 'Left') {
                const appWindow = Window.getCurrent()
                await appWindow.show()
                await appWindow.setFocus()

                // 直接处理路由恢复，不依赖事件系统
                await handleRouteRestore()
              }
              break
          }
        },
        menu,
        menuOnLeftClick: false,
      }

      try {
        // 创建托盘实例，但不存储引用，只存储ID
        const trayInstance = await TrayIcon.new(options)
        setTrayInstanceId(trayInstance.id)
      } catch (error) {
        console.error('创建托盘实例失败:', error)
        throw error
      }

      // 初始化提示文本
      await updateTrayTooltip()

      // 注册状态监听，确保每次初始化前先清理旧watcher
      cleanupWatchers()

      registerWatcher(
        () => appStore.isRunning,
        () => {
          updateTrayTooltip()
          refreshTrayMenu()
        },
      )

      registerWatcher(
        () => appStore.proxyMode as ProxyMode,
        (newMode) => {
          const mode = newMode as unknown as ProxyMode
          console.log(`代理模式变更为: ${mode}, 上次菜单模式: ${lastProxyMode.value}`)
          updateTrayTooltip()
          if (mode !== lastProxyMode.value) {
            console.log('模式已变化，强制刷新托盘菜单')
            refreshTrayMenu()
          }
        },
      )

      registerWatcher(
        () => i18n.global.locale.value,
        () => {
          console.log('语言已变更，刷新托盘菜单')
          refreshTrayMenu()
          updateTrayTooltip()
        },
      )

      registerWatcher(
        () => [subStore.activeIndex, subStore.list.length],
        () => {
          updateTrayTooltip()
        },
      )

      const { useLocaleStore } = await import('@/stores')
      const localeStore = useLocaleStore()
      registerWatcher(
        () => localeStore.currentLocale,
        () => {
          console.log('LocaleStore 语言变更，刷新托盘菜单')
          refreshTrayMenu()
          updateTrayTooltip()
        },
      )

      return true
    } catch (error) {
      console.error('初始化托盘失败:', error)
      return false
    }
  }

  /**
   * 刷新托盘菜单
   */
  const refreshTrayMenu = async () => {
    const currentId = trayInstanceId.value ?? appStore.trayInstanceId
    if (!currentId) return

    // 接使用TrayIcon实例的方法
    try {
      const tray = await TrayIcon.getById(currentId)
      const menu = await createTrayMenu()
      if (tray) {
        await tray.setMenu(menu)
        await updateTrayTooltip()
        console.log('使用TrayIcon实例刷新菜单成功')
      } else {
        throw new Error('无法获取托盘实例')
      }
    } catch (trayError) {
      console.error('使用托盘实例设置菜单也失败:', trayError)

      // 如果还是失败，最后的办法是重新创建托盘
      await destroyTray()
      await initTray()
    }
  }

  /**
   * 销毁托盘
   */
  const destroyTray = async () => {
    const currentId = trayInstanceId.value ?? appStore.trayInstanceId
    if (currentId) {
      try {
        // 使用静态方法关闭托盘
        await TrayIcon.removeById(currentId)
        setTrayInstanceId(null)
      } catch (error) {
        console.error('销毁托盘失败:', error)
      }
    }

    cleanupWatchers()
    console.log('托盘事件监听器已清理')
  }

  return {
    trayInstanceId,
    initTray,
    destroyTray,
    updateTrayTooltip,
    refreshTrayMenu,
  }


})
