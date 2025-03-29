// TrayStore.ts - 管理应用程序托盘功能的Store
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { TrayIcon, TrayIconEvent } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { MenuItem, type MenuItemOptions } from '@tauri-apps/api/menu/menuItem'
import { Submenu, type SubmenuOptions } from '@tauri-apps/api/menu/submenu'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from './AppStore'
import { useInfoStore } from './infoStore'
import { useSubStore } from './SubStore'
import { ProxyService } from '@/services/proxy-service'
import mitt from '@/utils/mitt'
import { Window } from '@tauri-apps/api/window'
import { useRouter } from 'vue-router'

// 声明mitt事件类型
declare module '@/utils/mitt' {
  interface Events {
    'refresh-tray-menu': void
    'process-status': void
    'proxy-mode-changed': void
  }
}

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

// 定义Tauri菜单项类型
interface TauriMenuItem {
  id?: string
  text?: string
  type?: string
  checked?: boolean
  enabled?: boolean
  action?: () => Promise<void>
  submenu?: TauriMenuItem[]
}

export const useTrayStore = defineStore(
  'tray',
  () => {
    // 引用其他Store
    const appStore = useAppStore()
    const infoStore = useInfoStore()
    const subStore = useSubStore()
    const router = useRouter()
    const proxyService = ProxyService.getInstance()

    // 添加一个内部状态，记录上次菜单刷新时的代理模式
    const lastProxyMode = ref<'system' | 'tun'>(appStore.proxyMode)

    // 只存储托盘ID，不存储实例
    const trayInstanceId = ref<string | null>(null)

    /**
     * 更新托盘提示信息
     */
    const updateTrayTooltip = async () => {
      if (trayInstanceId.value) {
        const status = appStore.isRunning ? '运行中' : '已停止'
        const mode = appStore.proxyMode === 'system' ? '系统代理' : 'TUN模式'

        // 获取当前使用的配置名称
        let configName = ''
        if (subStore.activeIndex !== null && subStore.list.length > 0) {
          configName = subStore.list[subStore.activeIndex].name
        }

        // 构建提示文本
        let tooltipText = `sing-box-window - 内核${status}, ${mode}`

        // 如果有配置名称，则显示
        if (configName) {
          tooltipText += `, 配置: ${configName}`
        }
        try {
          const tray = await TrayIcon.getById(trayInstanceId.value)
          if (tray) {
            await tray.setTooltip(tooltipText)
          }
        } catch (e) {
          console.error('备用方法更新托盘提示也失败:', e)
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
          text: '显示界面',
          enabled: true,
          action: async () => {
            await appStore.restoreFromBlank(router)
            await appStore.showWindow()
          },
        })

        // 创建内核控制子菜单项
        const startMenuItem = await MenuItem.new({
          id: 'start',
          text: '启动内核',
          enabled: !appStore.isRunning,
          action: async () => {
            try {
              await infoStore.startKernel()
              appStore.setRunningState(true)
              await refreshTrayMenu() // 刷新菜单以更新状态
            } catch (error) {
              console.error('启动内核失败:', error)
            }
          },
        })

        const stopMenuItem = await MenuItem.new({
          id: 'stop',
          text: '停止内核',
          enabled: appStore.isRunning,
          action: async () => {
            await infoStore.stopKernel()
            appStore.setRunningState(false)
            await refreshTrayMenu() // 刷新菜单以更新状态
          },
        })

        const restartMenuItem = await MenuItem.new({
          id: 'restart',
          text: '重启内核',
          enabled: appStore.isRunning,
          action: async () => {
            await infoStore.restartKernel()
            await refreshTrayMenu() // 刷新菜单以更新状态
          },
        })

        // 创建内核控制子菜单
        const kernelSubmenu = await Submenu.new({
          id: 'kernel_control',
          text: '内核控制',
          items: [startMenuItem, stopMenuItem, restartMenuItem],
        })

        // 创建代理模式子菜单项 - 使用普通MenuItem而不是CheckMenuItem
        const systemProxyMenuItem = await MenuItem.new({
          id: 'system_proxy',
          text: '系统代理模式',
          // 当前为系统代理模式时禁用按钮
          enabled: currentProxyMode !== 'system',
          action: async () => {
            try {
              console.log('切换到系统代理模式')
              await proxyService.switchMode('system')
              appStore.proxyMode = 'system'
              // 强制立即刷新菜单
              await refreshTrayMenu()
            } catch (error) {
              console.error('切换到系统代理模式失败:', error)
            }
          },
        })

        const tunProxyMenuItem = await MenuItem.new({
          id: 'tun_mode',
          text: 'TUN 模式',
          // 当前为TUN模式时禁用按钮
          enabled: currentProxyMode !== 'tun',
          action: async () => {
            try {
              console.log('切换到TUN模式')
              const needClose = await proxyService.switchMode('tun')
              appStore.proxyMode = 'tun'
              // 强制立即刷新菜单
              await refreshTrayMenu()
              if (needClose) {
                const appWindow = Window.getCurrent()
                await appWindow.close()
              }
            } catch (error) {
              console.error('切换到TUN模式失败:', error)
            }
          },
        })

        // 当前模式指示器菜单项（仅作为标签，不可点击）
        const currentModeMenuItem = await MenuItem.new({
          id: 'current_mode',
          text: `当前模式: ${currentProxyMode === 'system' ? '系统代理' : 'TUN模式'}`,
          enabled: false,
        })

        // 创建代理模式子菜单
        const proxyModeSubmenu = await Submenu.new({
          id: 'proxy_mode',
          text: '代理模式',
          items: [currentModeMenuItem, systemProxyMenuItem, tunProxyMenuItem],
        })

        // 创建分隔符菜单项
        const separator1 = await MenuItem.new({
          id: 'separator1',
          text: '-',
          enabled: false,
        })

        const separator2 = await MenuItem.new({
          id: 'separator2',
          text: '-',
          enabled: false,
        })

        // 创建退出菜单项
        const quitMenuItem = await MenuItem.new({
          id: 'quit',
          text: '退出',
          action: async () => {
            await infoStore.stopKernel()
            const appWindow = Window.getCurrent()
            await appWindow.close()
          },
        })

        // 创建主菜单
        return await Menu.new({
          items: [
            showMenuItem,
            separator1,
            kernelSubmenu,
            proxyModeSubmenu,
            separator2,
            quitMenuItem,
          ],
        })
      } catch (error) {
        console.error('创建菜单失败:', error)
        // 返回一个空菜单
        return await Menu.new({ items: [] })
      }
    }

    /**
     * 初始化托盘菜单
     */
    const initTray = async () => {
      try {
        // 清理之前的托盘实例（如果存在）
        if (appStore.trayInstanceId) {
          try {
            await TrayIcon.removeById(appStore.trayInstanceId)
          } catch (error) {
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
                  await appStore.restoreFromBlank(router)
                  await appStore.showWindow()
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

        watch(() => [subStore.activeIndex, subStore.list.length], updateTrayTooltip)

        // 添加事件处理
        mitt.on('process-status', () => {
          updateTrayTooltip()
          refreshTrayMenu() // 当进程状态变化时刷新菜单
        })

        mitt.on('proxy-mode-changed', () => {
          console.log('收到代理模式变更事件，刷新托盘菜单')
          updateTrayTooltip()
          refreshTrayMenu() // 当代理模式变化时刷新菜单
        })

        // 监听菜单刷新事件
        mitt.on('refresh-tray-menu', () => {
          console.log('收到刷新托盘菜单事件')
          refreshTrayMenu()
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
     * 清理托盘资源
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

      // 移除事件监听
      mitt.off('process-status')
      mitt.off('proxy-mode-changed')
      mitt.off('refresh-tray-menu')
    }

    return {
      trayInstanceId,
      initTray,
      updateTrayTooltip,
      refreshTrayMenu,
      destroyTray,
    }
  },
  {
    persist: true,
  },
)
