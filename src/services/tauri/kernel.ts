import { invokeWithAppContext, withAppStore } from './invoke-client'

export interface KernelStartConfig {
  proxy_mode: string
  api_port: number
  proxy_port: number
  prefer_ipv6: boolean
  auto_start: boolean
}

export interface KernelStartOptions {
  config?: Partial<KernelStartConfig>
  forceRestart?: boolean
  timeoutMs?: number
}

export interface KernelStopOptions {
  force?: boolean
  timeoutMs?: number
}

export const kernelApi = {
  async startKernel(options: KernelStartOptions = {}) {
    return withAppStore(async store => {
      await store.waitForDataRestore()
      return invokeWithAppContext<{ success: boolean; message: string }>(
        'kernel_start_enhanced',
        {
          proxyMode: options.config?.proxy_mode ?? store.proxyMode,
          apiPort: options.config?.api_port ?? store.apiPort
        },
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

    return invokeWithAppContext<string | void>(command, undefined, {
      withProxyPort: 'port'
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
