// TrayStore.ts - 管理应用程序托盘功能的Store
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'
import { TrayIcon, TrayIconEvent } from '@tauri-apps/api/tray'
import { defaultWindowIcon } from '@tauri-apps/api/app'
import { Menu } from '@tauri-apps/api/menu'
import { MenuItem } from '@tauri-apps/api/menu/menuItem'
import { Submenu } from '@tauri-apps/api/menu/submenu'
import { CheckMenuItem } from '@tauri-apps/api/menu/checkMenuItem'
import { useAppStore } from './AppStore'
import { useInfoStore } from './infoStore'
import { useSubStore } from './SubStore'
import { ProxyService } from '@/services/proxy-service'
import mitt from '@/utils/mitt'
import { Window } from '@tauri-apps/api/window'

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
    const proxyService = ProxyService.getInstance()

    // 托盘实例引用
    const trayInstance = ref<TrayIcon | null>(null)
    const trayInstanceId = ref<string | null>(null)

    /**
     * 更新托盘提示信息
     */
    const updateTrayTooltip = () => {
      if (trayInstance.value) {
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

        trayInstance.value.setTooltip(tooltipText)
      }
    }

    /**
     * 创建托盘菜单
     */
    const createTrayMenu = async () => {
      try {
        // 创建基本菜单项
        const showMenuItem = await MenuItem.new({
          id: 'show',
          text: '显示界面',
          enabled: true,
          action: async () => {
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
              appStore.isRunning = true
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
            appStore.isRunning = false
          },
        })

        const restartMenuItem = await MenuItem.new({
          id: 'restart',
          text: '重启内核',
          enabled: appStore.isRunning,
          action: async () => {
            await infoStore.restartKernel()
          },
        })

        // 创建内核控制子菜单
        const kernelSubmenu = await Submenu.new({
          id: 'kernel_control',
          text: '内核控制',
          items: [startMenuItem, stopMenuItem, restartMenuItem],
        })

        // 创建代理模式子菜单项
        const systemProxyMenuItem = await CheckMenuItem.new({
          id: 'system_proxy',
          text: '系统代理模式',
          checked: appStore.proxyMode === 'system',
          action: async () => {
            await proxyService.switchMode('system')
            appStore.proxyMode = 'system'
          },
        })

        const tunProxyMenuItem = await CheckMenuItem.new({
          id: 'tun_mode',
          text: 'TUN 模式',
          checked: appStore.proxyMode === 'tun',
          action: async () => {
            const needClose = await proxyService.switchMode('tun')
            appStore.proxyMode = 'tun'
            if (needClose) {
              const appWindow = Window.getCurrent()
              await appWindow.close()
            }
          },
        })

        // 创建代理模式子菜单
        const proxyModeSubmenu = await Submenu.new({
          id: 'proxy_mode',
          text: '代理模式',
          items: [systemProxyMenuItem, tunProxyMenuItem],
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
        const options = {
          icon: await defaultWindowIcon(),
          action: async (event: TrayIconEvent) => {
            switch (event.type) {
              case 'Click':
                // 如果点击的是左键，则显示界面
                if (event.button === 'Left') {
                  await appStore.showWindow()
                }
                break
            }
          },
          menu,
          menuOnLeftClick: false,
        }

        // @ts-expect-error TrayIcon API 可能不完全匹配，但实现是正确的
        trayInstance.value = await TrayIcon.new(options)

        // 保存托盘实例 ID
        trayInstanceId.value = trayInstance.value.id
        appStore.trayInstanceId = trayInstanceId.value

        // 初始化提示文本
        updateTrayTooltip()

        // 监听状态变化以更新提示
        watch(() => appStore.isRunning, updateTrayTooltip)
        watch(() => appStore.proxyMode, updateTrayTooltip)
        watch(() => [subStore.activeIndex, subStore.list.length], updateTrayTooltip)

        // 添加事件处理
        mitt.on('process-status', () => updateTrayTooltip())
        mitt.on('proxy-mode-changed', () => updateTrayTooltip())
        // @ts-expect-error mitt事件类型定义问题
        mitt.on('refresh-tray-menu', refreshTrayMenu)

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
      if (!trayInstance.value) return

      try {
        const menu = await createTrayMenu()
        await trayInstance.value.setMenu(menu)
        updateTrayTooltip()
      } catch (error) {
        console.error('刷新托盘菜单失败:', error)
      }
    }

    /**
     * 清理托盘资源
     */
    const destroyTray = async () => {
      if (trayInstance.value) {
        try {
          await TrayIcon.removeById(trayInstance.value.id)
          trayInstance.value = null
          trayInstanceId.value = null
        } catch (error) {
          console.error('销毁托盘失败:', error)
        }
      }

      // 移除事件监听
      mitt.off('process-status')
      mitt.off('proxy-mode-changed')
      // @ts-expect-error mitt事件类型定义问题
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
