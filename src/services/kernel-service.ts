/**
 * 内核管理服务
 * 负责协调 Tauri 命令、事件服务与状态缓存
 */
import { eventService } from './event-service'
import { kernelApi } from './tauri'
import type { KernelStartOptions, KernelStopOptions } from './tauri/kernel'
import { StatusCache } from './kernel/status-cache'
import { KernelLifecycleController } from './kernel/lifecycle-controller'

export interface KernelStatus {
  process_running: boolean
  api_ready: boolean
  websocket_ready: boolean
  uptime_ms?: number
  version?: string
  error?: string
}

export interface KernelConfig {
  proxy_mode: 'system' | 'tun' | 'manual'
  api_port: number
  proxy_port: number
  prefer_ipv6: boolean
  auto_start: boolean
}

class KernelService {
  private readonly lifecycle = new KernelLifecycleController()
  private readonly statusCache = new StatusCache<KernelStatus>(2000)

  async startKernel(options: KernelStartOptions = {}): Promise<{ success: boolean; message: string }> {
    return this.lifecycle.run(
      'start',
      active => ({ success: false, message: active === 'start' ? '内核正在启动中，请稍候' : '内核正在停止中，请稍候' }),
      async () => {
        const result = await kernelApi.startKernel(options)
        this.clearStatusCache()
        return result
      }
    )
  }

  async stopKernel(options: KernelStopOptions = {}): Promise<{ success: boolean; message: string }> {
    return this.lifecycle.run(
      'stop',
      active => ({ success: false, message: active === 'stop' ? '内核正在停止中，请稍候' : '内核正在启动中，请稍候' }),
      async () => {
        const result = await kernelApi.stopKernel(options)
        this.clearStatusCache()
        return result
      }
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

  async getKernelStatus(): Promise<KernelStatus> {
    const cached = this.statusCache.get('kernel_status')
    if (cached) {
      return cached
    }

    try {
      const status = await kernelApi.getKernelStatus<KernelStatus>()
      this.statusCache.set('kernel_status', status)
      return status
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
    const status = await this.getKernelStatus()
    return status.process_running
  }

  async getKernelVersion(): Promise<string> {
    try {
      return await kernelApi.getKernelVersion()
    } catch (error) {
      console.error('获取内核版本失败:', error)
      return ''
    }
  }

  async switchProxyMode(mode: 'system' | 'tun' | 'manual'): Promise<{ success: boolean; message: string }> {
    try {
      await kernelApi.switchProxyMode(mode)
      this.clearStatusCache()
      return { success: true, message: `代理模式已切换到 ${mode}` }
    } catch (error) {
      console.error('切换代理模式失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '切换代理模式失败'
      }
    }
  }

  async toggleIpVersion(preferIpv6: boolean): Promise<{ success: boolean; message: string }> {
    try {
      await kernelApi.toggleIpVersion(preferIpv6)
      this.clearStatusCache()
      return { success: true, message: preferIpv6 ? '已切换到IPv6优先模式' : '已切换到IPv4优先模式' }
    } catch (error) {
      console.error('切换IP版本失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '切换IP版本失败'
      }
    }
  }

  async getKernelConfig(): Promise<KernelConfig> {
    return {
      proxy_mode: 'manual',
      api_port: 9090,
      proxy_port: 7890,
      prefer_ipv6: false,
      auto_start: false
    }
  }

  async updateKernelConfig(config: Partial<KernelConfig>): Promise<{ success: boolean; message: string }> {
    try {
      console.log('🔧 更新内核配置:', config)
      return { success: true, message: '配置更新成功' }
    } catch (error) {
      console.error('更新内核配置失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '更新内核配置失败'
      }
    }
  }

  async checkKernelHealth(): Promise<{ healthy: boolean; issues: string[] }> {
    try {
      return await kernelApi.checkKernelHealth()
    } catch (error) {
      console.error('检查内核健康状态失败:', error)
      return {
        healthy: false,
        issues: [error instanceof Error ? error.message : '健康检查失败']
      }
    }
  }

  private clearStatusCache(): void {
    this.statusCache.clear()
  }

  async forceRefreshStatus(): Promise<KernelStatus> {
    this.clearStatusCache()
    return this.getKernelStatus()
  }

  async onKernelStatusChange(callback: (status: KernelStatus) => void): Promise<() => void> {
    return eventService.on('kernel-status-changed', (data: unknown) => {
      callback(data as KernelStatus)
    })
  }

  async onKernelReady(callback: () => void): Promise<() => void> {
    return eventService.on('kernel-ready', callback)
  }

  async onKernelError(callback: (error: string) => void): Promise<() => void> {
    return eventService.on('kernel-error', (data: unknown) => {
      callback(data as string)
    })
  }
}

export const kernelService = new KernelService()
export default kernelService
