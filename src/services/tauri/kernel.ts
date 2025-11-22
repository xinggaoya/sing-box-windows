import { invokeWithAppContext, withAppStore } from './invoke-client'

export interface TunSettings {
  ipv4_address: string
  ipv6_address: string
  mtu: number
  auto_route: boolean
  strict_route: boolean
  stack: 'system' | 'gvisor' | 'mixed'
  enable_ipv6: boolean
}

export interface KernelStartConfig {
  proxy_mode: string
  api_port: number
  proxy_port: number
  prefer_ipv6: boolean
  auto_start: boolean
  system_proxy_bypass: string
  tun: TunSettings
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

export const kernelApi = {
  /**
   * 启动内核 - 简化版
   * 后端会自动从数据库读取配置，前端无需传递
   */
  async startKernel(options: KernelStartOptions = {}) {
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
  },

  /**
   * 自动管理内核 - 简化版
   * 后端会自动从数据库读取配置
   */
  autoManageKernel(options: KernelStartOptions & { forceRestart?: boolean } = {}) {
    return withAppStore(async store => {
      await store.waitForDataRestore()

      // 仅传递覆盖参数
      const args: Record<string, any> = {}

      if (options.forceRestart !== undefined) {
        args.force_restart = options.forceRestart
        args.forceRestart = options.forceRestart
      }

      return invokeWithAppContext<KernelAutoManageResult>(
        'kernel_auto_manage',
        Object.keys(args).length > 0 ? args : undefined,
        { skipDataRestore: true }
      )
    })
  },

  stopKernel(_options: KernelStopOptions = {}) {
    return invokeWithAppContext<{ success: boolean; message: string }>(
      'kernel_stop_enhanced',
      undefined,
      { skipDataRestore: true }
    )
  },

  async restartKernel(options: KernelStartOptions & KernelStopOptions = {}) {
    const stopResult = await kernelApi.stopKernel(options)
    if (!stopResult.success) {
      return stopResult
    }

    await new Promise(resolve => setTimeout(resolve, 1000))

    return kernelApi.startKernel(options)
  },

  getKernelStatus<T = unknown>() {
    return invokeWithAppContext<T>('kernel_get_status_enhanced', undefined, {
      withApiPort: true
    })
  },

  getKernelVersion() {
    return invokeWithAppContext<string>('check_kernel_version', undefined, {
      skipDataRestore: true
    })
  },

  switchProxyMode(mode: 'system' | 'tun' | 'manual') {
    const command =
      mode === 'system' ? 'set_system_proxy' : mode === 'tun' ? 'set_tun_proxy' : 'set_manual_proxy'

    return withAppStore(async store => {
      await store.waitForDataRestore()
      const args: Record<string, unknown> = {}

      if (mode === 'system') {
        args.systemProxyBypass = store.systemProxyBypass
        args.system_proxy_bypass = store.systemProxyBypass
      } else if (mode === 'tun') {
        const tunOptions = {
          ipv4_address: store.tunIpv4,
          ipv6_address: store.tunIpv6,
          mtu: store.tunMtu,
          auto_route: store.tunAutoRoute,
          strict_route: store.tunStrictRoute,
          stack: store.tunStack as 'system' | 'gvisor' | 'mixed',
          enable_ipv6: store.tunEnableIpv6,
        }
        args.tunOptions = tunOptions
        args.tun_options = tunOptions
      }

      return invokeWithAppContext<string | void>(command, args, {
        withProxyPort: 'port',
      })
    })
  },

  switchNodeProxyMode(mode: 'global' | 'rule') {
    return invokeWithAppContext<string>('toggle_proxy_mode', { mode }, {
      skipDataRestore: true
    })
  },

  toggleIpVersion(preferIpv6: boolean) {
    return invokeWithAppContext<void>('toggle_ip_version', { preferIpv6 }, {
      skipDataRestore: true
    })
  },

  getKernelConfig() {
    return Promise.resolve({})
  },

  updateKernelConfig(_config: unknown) {
    return Promise.resolve({ success: true, message: '配置更新功能暂未实现' })
  },

  checkKernelHealth() {
    return invokeWithAppContext<{ healthy: boolean; issues: string[] }>('kernel_check_health', undefined, {
      withApiPort: true
    })
  },

  isKernelRunning() {
    return invokeWithAppContext<boolean>('is_kernel_running', undefined, {
      skipDataRestore: true
    })
  },

  checkKernelVersion() {
    return kernelApi.getKernelVersion()
  },

  checkKernelStatus<T = unknown>(apiPort?: number) {
    return invokeWithAppContext<T>(
      'check_kernel_status',
      typeof apiPort === 'number' ? { api_port: apiPort } : undefined,
      { withApiPort: typeof apiPort !== 'number' ? 'api_port' : undefined }
    )
  },

  getKernelRunningState() {
    return kernelApi.isKernelRunning()
  },

  getApiToken() {
    return invokeWithAppContext<string>('get_api_token')
  },

  downloadKernel() {
    return invokeWithAppContext<void>('download_kernel', undefined, {
      skipDataRestore: true
    })
  },

  installKernel() {
    return invokeWithAppContext<void>('install_kernel', undefined, {
      skipDataRestore: true
    })
  }
}
