// TrayStore.ts - 管理应用程序托盘功能的Store
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
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
import { ProxyService } from '@/services/proxy-service'
import i18n from '@/locales'
import { useRouter } from 'vue-router'
import { tauriApi } from '@/services/tauri-api'

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
  const proxyService = ProxyService.getInstance()

  // 添加一个内部状态，记录上次菜单刷新时的代理模式
  const lastProxyMode = ref<'system' | 'tun' | 'manual'>(appStore.proxyMode)

  // 只存储托盘ID，不存储实例
  const trayInstanceId = ref<string | null>(null)

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
    if (trayInstanceId.value) {
      try {
        const status = appStore.isRunning ? t('home.status.running') : t('home.status.stopped')
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
        const trayIcon = await TrayIcon.getById(trayInstanceId.value)
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
      const startMenuItem = await MenuItem.new({
        id: 'start',
        text: t('tray.start'),
        enabled: !appStore.isRunning,
        action: async () => {
          try {
            await kernelStore.startKernel()
            appStore.setRunningState(true)
            await refreshTrayMenu() // 刷新菜单以更新状态
          } catch (error) {
            console.error('启动内核失败:', error)
          }
        },
      })

      const stopMenuItem = await MenuItem.new({
        id: 'stop',
        text: t('tray.stop'),
        enabled: appStore.isRunning,
        action: async () => {
          await kernelStore.stopKernel()
          appStore.setRunningState(false)
          await refreshTrayMenu() // 刷新菜单以更新状态
        },
      })

      const restartMenuItem = await MenuItem.new({
        id: 'restart',
        text: t('home.restart'),
        enabled: appStore.isRunning,
        action: async () => {
          await kernelStore.restartKernel()
          await refreshTrayMenu() // 刷新菜单以更新状态
        },
      })

      // 创建内核控制子菜单
      const kernelSubmenu = await Submenu.new({
        id: 'kernel_control',
        text: t('setting.kernel.title'),
        items: [startMenuItem, stopMenuItem, restartMenuItem],
      })

      // 创建代理模式子菜单项 - 使用普通MenuItem而不是CheckMenuItem
      const systemProxyMenuItem = await MenuItem.new({
        id: 'system_proxy',
        text: t('home.proxyMode.system'),
        // 当前为系统代理模式时禁用按钮
        enabled: currentProxyMode !== 'system',
        action: async () => {
          try {
            await proxyService.switchMode('system')
            appStore.proxyMode = 'system'
            // 强制立即刷新菜单
            await refreshTrayMenu()
          } catch (error) {
            console.error('切换到系统代理模式失败:', error)
          }
        },
      })

      // TUN模式菜单项
      const tunProxyMenuItem = await MenuItem.new({
        id: 'tun_mode',
        text: t('home.proxyMode.tun'),
        // 当前为TUN模式时禁用按钮
        enabled: currentProxyMode !== 'tun',
        action: async () => {
          try {
            // 检查是否有管理员权限
            const isAdmin = await tauriApi.system.checkAdmin()
            console.log('托盘切换TUN模式 - 当前管理员权限状态:', isAdmin)

            if (!isAdmin) {
              // 没有管理员权限，先设置模式然后以管理员重启
              console.log('没有管理员权限，准备以管理员身份重启')
              try {
                // 先设置应用状态，以便重启后保持选择
                appStore.setProxyMode('tun')
                console.log('已设置应用状态为TUN模式，准备重启')

                // 以管理员重启
                await tauriApi.system.restartAsAdmin()
                console.log('已发送管理员重启请求')
              } catch (restartError) {
                console.error('以管理员身份重启失败:', restartError)
                // 重启失败时恢复之前的模式
                appStore.proxyMode = currentProxyMode
                await refreshTrayMenu()
              }
            } else {
              // 有管理员权限，正常切换
              console.log('有管理员权限，正常切换TUN模式')
              const needClose = await proxyService.switchMode('tun')
              appStore.proxyMode = 'tun'
              // 强制立即刷新菜单
              await refreshTrayMenu()
              if (needClose) {
                const appWindow = Window.getCurrent()
                await appWindow.close()
              }
            }
          } catch (error) {
            console.error('切换到TUN模式失败:', error)
            // 切换失败时恢复之前的模式
            appStore.proxyMode = currentProxyMode
            await refreshTrayMenu()
          }
        },
      })

      // 手动模式菜单项
      const manualProxyMenuItem = await MenuItem.new({
        id: 'manual_mode',
        text: t('home.proxyMode.manual'),
        // 当前为手动模式时禁用按钮
        enabled: currentProxyMode !== 'manual',
        action: async () => {
          try {
            await proxyService.switchMode('manual')
            appStore.proxyMode = 'manual'
            // 强制立即刷新菜单
            await refreshTrayMenu()
          } catch (error) {
            console.error('切换到手动模式失败:', error)
          }
        },
      })

      // 当前模式指示器菜单项（仅作为标签，不可点击）
      const currentModeMenuItem = await MenuItem.new({
        id: 'current_mode',
        text: `${t('proxy.currentMode')} ${
          currentProxyMode === 'system'
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
            await kernelStore.stopKernel()
            const appWindow = Window.getCurrent()
            await appWindow.destroy()
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
      // 清理之前的托盘实例（如果存在）
      if (appStore.trayInstanceId) {
        try {
          await destroyTray()
        } catch {
          // 忽略可能的错误
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
        trayInstanceId.value = trayInstance.id
        appStore.trayInstanceId = trayInstance.id
      } catch (error) {
        console.error('创建托盘实例失败:', error)
        throw error
      }

      // 初始化提示文本
      await updateTrayTooltip()

      // 监听状态变化以更新提示和菜单
      watch(
        () => appStore.isRunning,
        () => {
          updateTrayTooltip()
          refreshTrayMenu() // 当运行状态变化时刷新菜单
        },
      )

      // 直接监听代理模式变化并强制刷新菜单
      watch(
        () => appStore.proxyMode,
        (newMode) => {
          console.log(`代理模式变更为: ${newMode}, 上次菜单模式: ${lastProxyMode.value}`)
          updateTrayTooltip()
          // 如果模式确实发生了变化，则强制刷新菜单
          if (newMode !== lastProxyMode.value) {
            console.log('模式已变化，强制刷新托盘菜单')
            refreshTrayMenu()
          }
        },
      )

      // 监听语言变更
      watch(
        () => i18n.global.locale.value,
        () => {
          console.log('语言已变更，刷新托盘菜单')
          refreshTrayMenu()
          updateTrayTooltip()
        },
      )

      watch(() => [subStore.activeIndex, subStore.list.length], updateTrayTooltip)

      // 使用Pinia的watch监听状态变化，替代mitt事件
      watch(() => appStore.isRunning, () => {
        updateTrayTooltip()
        refreshTrayMenu() // 当进程状态变化时刷新菜单
      })

      watch(() => appStore.proxyMode, () => {
        console.log('代理模式已变更，刷新托盘菜单')
        updateTrayTooltip()
        refreshTrayMenu() // 当代理模式变化时刷新菜单
      })

      // 监听语言变更 - 需要导入LocaleStore
      const { useLocaleStore } = await import('@/stores')
      const localeStore = useLocaleStore()
      watch(() => localeStore.currentLocale, () => {
        console.log('语言已变更，刷新托盘菜单')
        refreshTrayMenu()
        updateTrayTooltip()
      })

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
    if (!trayInstanceId.value) return

    // 接使用TrayIcon实例的方法
    try {
      const tray = await TrayIcon.getById(trayInstanceId.value)
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
    if (trayInstanceId.value) {
      try {
        // 使用静态方法关闭托盘
        await TrayIcon.removeById(trayInstanceId.value)
        trayInstanceId.value = null
        appStore.trayInstanceId = null
      } catch (error) {
        console.error('销毁托盘失败:', error)
      }
    }

    // 移除事件监听 - 现在使用Pinia的watch，无需手动清理
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
