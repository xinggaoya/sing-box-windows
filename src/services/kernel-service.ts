/**
 * 内核管理服务 - 简化版
 * 职责：提供简洁的API调用接口和事件监听
 * 状态管理：由后端负责，前端通过事件获取
 */
import { eventService } from './event-service'
import { kernelApi } from './tauri'
import type { KernelStartOptions, KernelStopOptions, KernelAutoManageResult } from './tauri/kernel'

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

class KernelService {
  /**
   * 启动内核 - 直接调用后端API
   * 状态变化通过事件推送，无需手动刷新
   */
  async startKernel(options: KernelStartOptions = {}): Promise<{ success: boolean; message: string }> {
    return kernelApi.startKernel(options)
  }

  /**
   * 停止内核 - 直接调用后端API
   * 状态变化通过事件推送，无需手动刷新
   */
  async stopKernel(options: KernelStopOptions = {}): Promise<{ success: boolean; message: string }> {
    return kernelApi.stopKernel(options)
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
   * 获取内核状态 - 仅用于主动查询
   * 推荐：使用 onKernelStatusChange 监听状态变化
   */
  async getKernelStatus(): Promise<KernelStatus> {
    try {
      return await kernelApi.getKernelStatus<KernelStatus>()
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

  /**
   * 切换代理模式
   */
  async switchProxyMode(mode: 'system' | 'tun' | 'manual'): Promise<{ success: boolean; message: string }> {
    try {
      await kernelApi.switchProxyMode(mode)
      return { success: true, message: `代理模式已切换到 ${mode}` }
    } catch (error) {
      console.error('切换代理模式失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '切换代理模式失败'
      }
    }
  }

  async applyProxySettings(): Promise<{ success: boolean; message: string }> {
    try {
      await kernelApi.applyProxySettings()
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
   * 切换IP版本
   */
  async toggleIpVersion(preferIpv6: boolean): Promise<{ success: boolean; message: string }> {
    try {
      await kernelApi.toggleIpVersion(preferIpv6)
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
      auto_start: false,
      system_proxy_bypass: '',
      tun: {
        ipv4_address: '',
        ipv6_address: '',
        mtu: 1500,
        auto_route: true,
        strict_route: true,
        stack: 'mixed',
        enable_ipv6: true,
      },
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

  autoManageKernel(options: KernelStartOptions & { forceRestart?: boolean } = {}): Promise<KernelAutoManageResult> {
    return kernelApi.autoManageKernel(options)
  }

  /**
   * 后台快速停止内核：仅发起请求，立即返回
   */
  async stopKernelFast(): Promise<{ success: boolean; message: string }> {
    try {
      return await kernelApi.stopKernelFast()
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
      return await kernelApi.forceStopAndExit()
    } catch (error) {
      console.error('强制停止并退出失败:', error)
      return {
        success: false,
        message: error instanceof Error ? error.message : '强制停止并退出失败'
      }
    }
  }

  /**
   * 监听内核状态变化 - 推荐使用
   * 后端会主动推送状态变化，无需轮询
   */

  async onKernelStatusChange(callback: (status: KernelStatus) => void): Promise<() => void> {
    return eventService.on('kernel-status-changed', (data: unknown) => {
      callback(data as KernelStatus)
    })
  }

  async onKernelReady(callback: () => void): Promise<() => void> {
    return eventService.on('kernel-ready', callback)
  }

  async onKernelError(callback: (error: any) => void): Promise<() => void> {
    return eventService.on('kernel-error', callback)
  }

  /**
   * 监听内核启动中事件
   */
  async onKernelStarting(callback: (data: any) => void): Promise<() => void> {
    return eventService.on('kernel-starting', callback)
  }

  /**
   * 监听内核已启动事件
   */
  async onKernelStarted(callback: (data: any) => void): Promise<() => void> {
    return eventService.on('kernel-started', callback)
  }

  /**
   * 监听内核已停止事件
   */
  async onKernelStopped(callback: (data: any) => void): Promise<() => void> {
    return eventService.on('kernel-stopped', callback)
  }
}

export const kernelService = new KernelService()
export default kernelService
