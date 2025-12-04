import { useAppStore } from '@/stores/app/AppStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { NotificationService } from './notification-service'
import { invokeWithAppContext } from './invoke-client'
import i18n from '@/locales'

export interface ProxyLatencyHistoryEntry {
  time: string
  delay: number
}

export interface ProxyData {
  type: string
  name: string
  now: string
  all: string[]
  history: ProxyLatencyHistoryEntry[]
  udp: boolean
}

export interface ProxiesData {
  proxies: Record<string, ProxyData>
}

export interface TunOptionsPayload {
  ipv4_address: string
  ipv6_address: string
  mtu: number
  auto_route: boolean
  strict_route: boolean
  stack: 'system' | 'gvisor' | 'mixed'
  enable_ipv6: boolean
}

export class ProxyService {
  private static instance: ProxyService
  private notificationService = NotificationService.getInstance()
  private t = i18n.global.t

  private get appStore() {
    return useAppStore()
  }

  private get kernelStore() {
    return useKernelStore()
  }

  private constructor() { }

  public static getInstance(): ProxyService {
    if (!ProxyService.instance) {
      ProxyService.instance = new ProxyService()
    }
    return ProxyService.instance
  }

  /**
   * 切换代理模式 - 简化版
   * 后端会从数据库读取配置，前端只需指定模式
   * @param mode 代理模式
   * @param messageCallback 消息回调
   * @returns 是否需要关闭应用（重启管理员）
   */
  public async switchMode(
    mode: 'system' | 'tun' | 'manual',
    messageCallback?: (type: 'success' | 'info' | 'error', content: string) => void,
  ): Promise<boolean> {
    try {
      // 根据模式同步独立开关，具体配置由后端从数据库读取
      if (mode === 'system') {
        await this.appStore.toggleSystemProxy(true)
        await this.appStore.toggleTun(false)
      } else if (mode === 'manual') {
        await this.appStore.toggleSystemProxy(false)
        await this.appStore.toggleTun(false)
      } else {
        await this.appStore.toggleSystemProxy(false)
        await this.appStore.toggleTun(true)
      }

      const applied = await this.kernelStore.applyProxySettings()
      if (!applied) {
        const errorText = '应用代理配置失败'
        if (messageCallback) messageCallback('error', errorText)
        return false
      }

      if (messageCallback) {
        const content =
          mode === 'system'
            ? '系统代理模式已启用'
            : mode === 'tun'
              ? 'TUN模式已启用'
              : '手动代理模式已启用，请手动设置系统代理'
        messageCallback(mode === 'manual' ? 'info' : 'success', content)
      }

      // 如果内核正在运行，需要重启
      if (this.appStore.isRunning) {
        try {
          if (messageCallback) messageCallback('info', this.t('home.status.restarting'))
          else this.notificationService.info(this.t('home.status.restarting'))

          await this.kernelStore.restartKernel()

          if (messageCallback) messageCallback('success', this.t('notification.kernelRestarted'))
          else this.notificationService.success(this.t('notification.kernelRestarted'))
        } catch (error) {
          const errorMsg = `${this.t('proxy.modeChangeFailed')}: ${error}`
          if (messageCallback) messageCallback('error', errorMsg)
          else this.notificationService.error(errorMsg)
        }
      }

      return false // 不需要关闭应用
    } catch (error) {
      if (messageCallback) {
        messageCallback('error', `切换代理模式失败: ${error}`)
      }
      console.error('切换代理模式失败:', error)
      return false
    }
  }

  // API Methods

  async setSystemProxy(systemProxyBypass?: string) {
    const args =
      typeof systemProxyBypass === 'string'
        ? { systemProxyBypass, system_proxy_bypass: systemProxyBypass }
        : undefined
    return invokeWithAppContext<void>('set_system_proxy', args, {
      withProxyPort: 'port'
    })
  }

  async setTunProxy(tunOptions?: TunOptionsPayload) {
    const args = tunOptions ? { tunOptions, tun_options: tunOptions } : undefined
    return invokeWithAppContext<void>('set_tun_proxy', args, {
      withProxyPort: 'port'
    })
  }

  async setManualProxy() {
    return invokeWithAppContext<void>('set_manual_proxy', undefined, {
      withProxyPort: 'port'
    })
  }

  async toggleIpVersion(preferIpv6: boolean) {
    return invokeWithAppContext<void>('toggle_ip_version', { preferIpv6 })
  }

  async toggleProxyMode(mode: string) {
    return invokeWithAppContext<string>('toggle_proxy_mode', { mode })
  }

  async getCurrentProxyMode() {
    return invokeWithAppContext<string>('get_current_proxy_mode')
  }

  async getProxies() {
    return invokeWithAppContext<ProxiesData>('get_proxies', undefined, {
      withApiPort: 'port'
    })
  }

  async changeProxy(group: string, proxy: string, server?: string, port?: number) {
    const args = { group, proxy, server, port }
    return invokeWithAppContext<void>(
      'change_proxy',
      port ? args : { ...args, port: undefined },
      { withApiPort: port ? undefined : 'port' }
    )
  }

  async testNodeDelay(proxy: string, server?: string, port?: number) {
    const args = { proxy, server, port }
    return invokeWithAppContext<void>(
      'test_node_delay',
      port ? args : { ...args, port: undefined },
      { withApiPort: port ? undefined : 'port' }
    )
  }

  async testGroupDelay(group: string, server?: string, port?: number) {
    const args = { group, server, port }
    return invokeWithAppContext<void>(
      'test_group_delay',
      port ? args : { ...args, port: undefined },
      { withApiPort: port ? undefined : 'port' }
    )
  }

  async getRules(port?: number) {
    const args = typeof port === 'number' ? { port } : undefined
    return invokeWithAppContext<unknown>('get_rules', args, {
      withApiPort: typeof port === 'number' ? undefined : 'port'
    })
  }
}

export const proxyService = ProxyService.getInstance()
