import { useAppStore } from '@/stores/app/AppStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { NotificationService } from './notification-service'
import { invokeWithAppContext } from './invoke-client'
import i18n from '@/locales'

// === gRPC API 返回类型（对应后端 singbox_api::types） ===

/** 节点项（对应 proto GroupItem） */
export interface GroupItem {
  tag: string
  type: string
  urlTestTime: number
  urlTestDelay: number
}

/** 节点组（对应 proto Group） */
export interface ProxyGroup {
  tag: string
  type: string
  selectable: boolean
  selected: string
  isExpand: boolean
  items: GroupItem[]
}

/** 完整节点组集合（对应 proto Groups） */
export interface GroupsData {
  group: ProxyGroup[]
}

/** Clash 模式状态（对应 proto ClashModeStatus） */
export interface ClashModeStatus {
  modeList: string[]
  currentMode: string
}

/** 节点测速结果（前端封装，对应 /SelectOutbound + /URLTest 调用结果） */
export interface UrlTestResult {
  ok: boolean
  message?: string
}

export interface TunOptionsPayload {
  ipv4_address: string
  ipv6_address: string
  mtu: number
  auto_route: boolean
  strict_route: boolean
  stack: 'system' | 'gvisor' | 'mixed'
  enable_ipv6: boolean
  route_exclude_address?: string[]
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

  private constructor() {}

  public static getInstance(): ProxyService {
    if (!ProxyService.instance) {
      ProxyService.instance = new ProxyService()
    }
    return ProxyService.instance
  }

  /**
   * 切换代理模式 - 简化版
   * 后端会从数据库读取配置，前端只需指定模式
   */
  public async switchMode(
    mode: 'system' | 'tun' | 'manual',
    messageCallback?: (type: 'success' | 'info' | 'error', content: string) => void,
  ): Promise<boolean> {
    try {
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
        const errorText = this.t('notification.applyProxyFailed')
        if (messageCallback) messageCallback('error', errorText)
        return false
      }

      if (messageCallback) {
        const content =
          mode === 'system'
            ? this.t('notification.systemProxyEnabled')
            : mode === 'tun'
              ? this.t('notification.tunEnabled')
              : this.t('notification.manualProxyEnabled')
        messageCallback(mode === 'manual' ? 'info' : 'success', content)
      }

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

      return false
    } catch (error) {
      if (messageCallback) {
        messageCallback('error', `${this.t('notification.proxySwitchFailed')}: ${error}`)
      }
      console.error('proxySwitchFailed:', error)
      return false
    }
  }

  // === Inbound 写入（不依赖 gRPC） ===

  async setSystemProxy(systemProxyBypass?: string) {
    const args =
      typeof systemProxyBypass === 'string'
        ? { systemProxyBypass, system_proxy_bypass: systemProxyBypass }
        : undefined
    return invokeWithAppContext<void>('set_system_proxy', args, {
      withProxyPort: 'port',
    })
  }

  async setTunProxy(tunOptions?: TunOptionsPayload) {
    const args = tunOptions ? { tunOptions, tun_options: tunOptions } : undefined
    return invokeWithAppContext<void>('set_tun_proxy', args, {
      withProxyPort: 'port',
    })
  }

  async setManualProxy() {
    return invokeWithAppContext<void>('set_manual_proxy', undefined, {
      withProxyPort: 'port',
    })
  }

  async toggleIpVersion(preferIpv6: boolean) {
    return invokeWithAppContext<void>('toggle_ip_version', { preferIpv6 })
  }

  // === gRPC API 交互（sing-box 1.14+ 官方 type: api） ===

  /** 获取内核版本号（"1.14.0" 等） */
  async getKernelVersion(): Promise<string> {
    return invokeWithAppContext<string>('get_kernel_version')
  }

  /** 获取所有节点组（一次快照） */
  async getGroups(): Promise<GroupsData> {
    return invokeWithAppContext<GroupsData>('get_groups')
  }

  /** 切换代理节点（gRPC SelectOutbound） */
  async selectOutbound(group: string, proxy: string): Promise<void> {
    return invokeWithAppContext<void>('select_outbound', { group, proxy })
  }

  /** 整组测速（gRPC URLTest） */
  async urlTest(group: string): Promise<UrlTestResult> {
    try {
      await invokeWithAppContext<void>('url_test', { group })
      return { ok: true }
    } catch (e) {
      return { ok: false, message: String(e) }
    }
  }

  /** 关闭所有连接 */
  async closeAllConnections(): Promise<void> {
    return invokeWithAppContext<void>('close_all_connections')
  }

  /** 获取 Clash 模式状态 */
  async getClashModeStatus(): Promise<ClashModeStatus> {
    return invokeWithAppContext<ClashModeStatus>('get_clash_mode_status')
  }

  /** 运行时切换 Clash 模式（gRPC SetClashMode） */
  async setClashMode(mode: string): Promise<void> {
    return invokeWithAppContext<void>('set_clash_mode', { mode })
  }

  /** 持久化组展开状态（gRPC SetGroupExpand） */
  async setGroupExpand(group: string, isExpand: boolean): Promise<void> {
    return invokeWithAppContext<void>('set_group_expand', {
      group,
      isExpand,
      is_expand: isExpand,
    })
  }

  /** 获取内核启动时间戳（秒） */
  async getStartedAt(): Promise<number> {
    return invokeWithAppContext<number>('get_started_at')
  }
}

export const proxyService = ProxyService.getInstance()