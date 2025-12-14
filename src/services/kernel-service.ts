/**
 * 内核管理服务
 * 职责：提供内核控制、状态查询、配置管理等功能
 */
import { APP_EVENTS } from '@/constants/events'
import { eventService } from './event-service'
import { invokeWithAppContext, withAppStore } from './invoke-client'

export interface KernelStatus {
  process_running: boolean
  api_ready: boolean
  websocket_ready: boolean
  uptime_ms?: number
  version?: string
  error?: string
}

export interface TunSettings {
  ipv4_address: string
  ipv6_address: string
  mtu: number
  auto_route: boolean
  strict_route: boolean
  stack: 'system' | 'gvisor' | 'mixed'
  enable_ipv6: boolean
}

export interface KernelConfig {
  proxy_mode: 'system' | 'tun' | 'manual'
  api_port: number
  proxy_port: number
  prefer_ipv6: boolean
  auto_start: boolean
  system_proxy_bypass: string
  tun: TunSettings
}

export interface KernelStartConfig {
  proxy_mode: string
  api_port: number
  proxy_port: number
  prefer_ipv6: boolean
  auto_start: boolean
  system_proxy_bypass: string
  tun: TunSettings
  system_proxy_enabled?: boolean
  tun_enabled?: boolean
}

export interface KernelStartOptions {
  config?: Partial<KernelStartConfig>
  forceRestart?: boolean
  timeoutMs?: number
  keepAlive?: boolean
}

export interface KernelStopOptions {
  force?: boolean
  timeoutMs?: number
}

export interface KernelAutoManageResult {
  state: 'missing_kernel' | 'missing_config' | 'invalid_config' | 'running' | 'error'
  message: string
  kernel_installed: boolean
  config_ready: boolean
  attempted_start: boolean
  last_start_message?: string
}

export interface KernelDownloadPayload {
  progress?: number
  message?: string
  status?: 'downloading' | 'completed' | 'error'
}

class KernelService {
  /**
   * 启动内核
   */
  async startKernel(options: KernelStartOptions = {}): Promise<{ success: boolean; message: string }> {
    return withAppStore(async store => {
      await store.waitForDataRestore()

      // 仅传递必要的参数覆盖，其他由后端从数据库读取
      const args: Record<string, any> = {}

      if (options.config?.proxy_mode) {
        args.proxy_mode = options.config.proxy_mode
        args.proxyMode = options.config.proxy_mode
      }

      if (options.config?.api_port) {
        args.api_port = options.config.api_port
        args.apiPort = options.config.api_port
      }

      if (options.config?.proxy_port) {
        args.proxy_port = options.config.proxy_port
        args.proxyPort = options.config.proxy_port
      }

      if (typeof options.config?.system_proxy_enabled === 'boolean') {
        args.system_proxy_enabled = options.config.system_proxy_enabled
      }

      if (typeof options.config?.tun_enabled === 'boolean') {
        args.tun_enabled = options.config.tun_enabled
      }

      if (options.keepAlive !== undefined) {
        args.keep_alive = options.keepAlive
        args.keepAlive = options.keepAlive
      }

      return invokeWithAppContext<{ success: boolean; message: string }>(
        'kernel_start_enhanced',
        Object.keys(args).length > 0 ? args : undefined,
        { skipDataRestore: true }
      )
    })
  }

  /**
   * 停止内核
   */
  async stopKernel(options: KernelStopOptions = {}): Promise<{ success: boolean; message: string }> {
    // 如果是强制停止，可能需要调用不同的后端API，或者传递参数
    // 目前后端 kernel_stop_enhanced 似乎不接受参数，但我们保留 options 接口以备将来扩展
    return invokeWithAppContext<{ success: boolean; message: string }>(
      'kernel_stop_enhanced',
      undefined,
      { skipDataRestore: true }
    )
  }

  async restartKernel(options: KernelStartOptions & KernelStopOptions = {}): Promise<{ success: boolean; message: string }> {
    const stopResult = await this.stopKernel(options)
    if (!stopResult.success) {
      return { success: false, message: `重启失败: ${stopResult.message}` }
    }

    await new Promise(resolve => setTimeout(resolve, 1000))

    return this.startKernel(options)
  }

  /**
   * 获取内核状态
   */
  async getKernelStatus(): Promise<KernelStatus> {
    try {
      return await invokeWithAppContext<KernelStatus>('kernel_get_status_enhanced', undefined, {
        withApiPort: true
      })
    } catch (error) {
      console.error('获取内核状态失败:', error)
      return {
        process_running: false,
        api_ready: false,
        websocket_ready: false,
        error: error instanceof Error ? error.message : '获取状态失败'
      }
    }
  }

  async isKernelRunning(): Promise<boolean> {
    try {
      return await invokeWithAppContext<boolean>('is_kernel_running', undefined, {
        skipDataRestore: true
      })
    } catch (error) {
      return false
    }
  }

  async getKernelVersion(): Promise<string> {
    try {
      return await invokeWithAppContext<string>('check_kernel_version', undefined, {
        skipDataRestore: true
      })
    } catch (error) {
      console.error('获取内核版本失败:', error)
      return ''
    }
  }

  async getLatestKernelVersion(): Promise<string> {
    try {
      return await invokeWithAppContext<string>('get_latest_kernel_version_cmd', undefined, {
        skipDataRestore: true
      })
    } catch (error) {
      console.error('获取最新内核版本失败:', error)
      return ''
    }
  }

  async getKernelReleases(): Promise<string[]> {
    try {
      return await invokeWithAppContext<string[]>('get_kernel_releases_cmd', undefined, {
        skipDataRestore: true
      })
    } catch (error) {
      console.error('获取内核版本列表失败:', error)
      return []
    }
  }

  async switchProxyMode(mode: 'system' | 'tun' | 'manual'): Promise<{ success: boolean; message: string }> {
    try {
      const overrides: Record<string, boolean> = {}
      if (mode === 'system') {
        overrides.system_proxy_enabled = true
        overrides.tun_enabled = false
      } else if (mode === 'tun') {
        overrides.system_proxy_enabled = false
        overrides.tun_enabled = true
      } else {
        overrides.system_proxy_enabled = false
        overrides.tun_enabled = false
      }

      await invokeWithAppContext<string | void>('apply_proxy_settings', overrides)
      return { success: true, message: `代理模式已切换到 ${mode}` }
    } catch (error) {
      console.error('切换代理模式失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '切换代理模式失败'
      }
    }
  }

  async applyProxySettings(options?: { system_proxy_enabled?: boolean; tun_enabled?: boolean }): Promise<{ success: boolean; message: string }> {
    try {
      const args: Record<string, unknown> = {}
      if (typeof options?.system_proxy_enabled === 'boolean') {
        args.system_proxy_enabled = options.system_proxy_enabled
      }
      if (typeof options?.tun_enabled === 'boolean') {
        args.tun_enabled = options.tun_enabled
      }

      await invokeWithAppContext<{ success: boolean; mode: string }>(
        'apply_proxy_settings',
        Object.keys(args).length ? args : undefined
      )
      return { success: true, message: '代理配置已应用' }
    } catch (error) {
      console.error('应用代理配置失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '应用代理配置失败'
      }
    }
  }

  /**
   * 切换节点代理模式 (Global/Rule)
   */
  async switchNodeProxyMode(mode: 'global' | 'rule'): Promise<string> {
    return invokeWithAppContext<string>('toggle_proxy_mode', { mode }, {
      skipDataRestore: true
    })
  }

  /**
   * 切换IP版本
   */
  async toggleIpVersion(preferIpv6: boolean): Promise<{ success: boolean; message: string }> {
    try {
      await invokeWithAppContext<void>('toggle_ip_version', { preferIpv6 }, {
        skipDataRestore: true
      })
      return { success: true, message: preferIpv6 ? '已切换到IPv6优先模式' : '已切换到IPv4优先模式' }
    } catch (error) {
      console.error('切换IP版本失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '切换IP版本失败'
      }
    }
  }

  /**
   * @deprecated 此方法未实现，仅为接口保留。
   * 如需使用，请先实现后端 `get_kernel_config` 命令。
   */
  async getKernelConfig(): Promise<KernelConfig> {
    console.warn('⚠️ getKernelConfig() 未实现，返回默认值')
    return {
      proxy_mode: 'manual',
      api_port: 12081,
      proxy_port: 12080,
      prefer_ipv6: false,
      auto_start: false,
      system_proxy_bypass: '',
      tun: {
        ipv4_address: '172.19.0.1/30',
        ipv6_address: 'fdfe:dcba:9876::1/126',
        mtu: 1500,
        auto_route: true,
        strict_route: true,
        stack: 'mixed',
        enable_ipv6: true,
      },
    }
  }

  /**
   * @deprecated 此方法未实现，仅为接口保留。
   * 如需使用，请先实现后端 `update_kernel_config` 命令。
   */
  async updateKernelConfig(_config: Partial<KernelConfig>): Promise<{ success: boolean; message: string }> {
    console.warn('⚠️ updateKernelConfig() 未实现')
    return { success: false, message: '此功能尚未实现' }
  }

  async checkKernelHealth(): Promise<{ healthy: boolean; issues: string[] }> {
    try {
      return await invokeWithAppContext<{ healthy: boolean; issues: string[] }>('kernel_check_health', undefined, {
        withApiPort: true
      })
    } catch (error) {
      console.error('检查内核健康状态失败:', error)
      return {
        healthy: false,
        issues: [error instanceof Error ? error.message : '健康检查失败']
      }
    }
  }

  autoManageKernel(options: KernelStartOptions & { forceRestart?: boolean } = {}): Promise<KernelAutoManageResult> {
    return withAppStore(async store => {
      await store.waitForDataRestore()

      // 仅传递覆盖参数
      const args: Record<string, any> = {}

      if (options.forceRestart !== undefined) {
        args.force_restart = options.forceRestart
        args.forceRestart = options.forceRestart
      }

      if (options.config) {
        if (typeof options.config.system_proxy_enabled === 'boolean') {
          args.system_proxy_enabled = options.config.system_proxy_enabled
        }
        if (typeof options.config.tun_enabled === 'boolean') {
          args.tun_enabled = options.config.tun_enabled
        }
      }

      return invokeWithAppContext<KernelAutoManageResult>(
        'kernel_auto_manage',
        Object.keys(args).length > 0 ? args : undefined,
        { skipDataRestore: true }
      )
    })
  }

  /**
   * 后台快速停止内核：仅发起请求，立即返回
   */
  async stopKernelFast(): Promise<{ success: boolean; message: string }> {
    try {
      return await invokeWithAppContext<{ success: boolean; message: string }>(
        'kernel_stop_background',
        undefined,
        { skipDataRestore: true }
      )
    } catch (error) {
      console.error('后台停止内核失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '后台停止内核失败'
      }
    }
  }

  /**
   * 强制停止内核并退出应用（后端后台执行，前端快速返回）
   */
  async forceStopAndExit(): Promise<{ success: boolean; message: string }> {
    try {
      return await invokeWithAppContext<{ success: boolean; message: string }>(
        'force_stop_and_exit',
        undefined,
        { skipDataRestore: true }
      )
    } catch (error) {
      console.error('强制停止并退出失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '强制停止并退出失败'
      }
    }
  }

  
  
  /**
   * 监听内核状态变化
   */
  async onKernelStatusChange(callback: (status: KernelStatus) => void): Promise<() => void> {
    return eventService.on(APP_EVENTS.kernelStatusChanged, (data) => {
      callback(data as KernelStatus)
    })
  }

  async onKernelReady(callback: () => void): Promise<() => void> {
    return eventService.on(APP_EVENTS.kernelReady, callback)
  }

  async onKernelError(callback: (error: any) => void): Promise<() => void> {
    return eventService.on(APP_EVENTS.kernelError, callback)
  }

  async onKernelStarting(callback: (data: any) => void): Promise<() => void> {
    return eventService.on(APP_EVENTS.kernelStarting, callback)
  }

  async onKernelStarted(callback: (data: any) => void): Promise<() => void> {
    return eventService.on(APP_EVENTS.kernelStarted, callback)
  }

  async onKernelStopped(callback: (data: any) => void): Promise<() => void> {
    return eventService.on(APP_EVENTS.kernelStopped, callback)
  }

  async updateSingboxPorts(proxyPort: number, apiPort: number): Promise<void> {
    return invokeWithAppContext<void>('update_singbox_ports', { proxyPort, apiPort })
  }
}

export const kernelService = new KernelService()
export default kernelService
