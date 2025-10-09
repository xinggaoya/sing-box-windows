/**
 * 内核管理服务
 * 重构后的内核管理，提供高可用性和更好的用户体验
 */
import { invoke } from '@tauri-apps/api/core'
import { eventService } from './event-service'

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

export interface StartOptions {
  config?: Partial<KernelConfig>
  force_restart?: boolean
  timeout_ms?: number
}

export interface StopOptions {
  force?: boolean
  timeout_ms?: number
}

class KernelService {
  private isStarting = false
  private isStopping = false
  private statusCache = new Map<string, { status: KernelStatus; timestamp: number }>()
  private readonly CACHE_TTL = 2000 // 2秒缓存

  /**
   * 启动内核 - 简化版本，大部分逻辑在后端处理
   */
  async startKernel(options: StartOptions = {}): Promise<{ success: boolean; message: string }> {
    if (this.isStarting) {
      return { success: false, message: '内核正在启动中，请稍候' }
    }

    if (this.isStopping) {
      return { success: false, message: '内核正在停止中，请稍候' }
    }

    this.isStarting = true

    try {
      console.log('🚀 开始启动内核...', options)
      
      // 调用后端启动命令，后端会处理所有复杂逻辑
      const result = await invoke<{ success: boolean; message: string }>('kernel_start', {
        options: {
          config: options.config,
          force_restart: options.force_restart || false,
          timeout_ms: options.timeout_ms || 30000,
        },
      })

      console.log('✅ 内核启动结果:', result)
      
      // 清除状态缓存
      this.clearStatusCache()
      
      return result
    } catch (error) {
      console.error('❌ 内核启动失败:', error)
      return { 
        success: false, 
        message: error instanceof Error ? error.message : '启动内核失败' 
      }
    } finally {
      this.isStarting = false
    }
  }

  /**
   * 停止内核
   */
  async stopKernel(options: StopOptions = {}): Promise<{ success: boolean; message: string }> {
    if (this.isStopping) {
      return { success: false, message: '内核正在停止中，请稍候' }
    }

    if (this.isStarting) {
      return { success: false, message: '内核正在启动中，请稍候' }
    }

    this.isStopping = true

    try {
      console.log('🛑 开始停止内核...', options)
      
      const result = await invoke<{ success: boolean; message: string }>('kernel_stop', {
        options: {
          force: options.force || false,
          timeout_ms: options.timeout_ms || 10000,
        },
      })

      console.log('✅ 内核停止结果:', result)
      
      // 清除状态缓存
      this.clearStatusCache()
      
      return result
    } catch (error) {
      console.error('❌ 内核停止失败:', error)
      return { 
        success: false, 
        message: error instanceof Error ? error.message : '停止内核失败' 
      }
    } finally {
      this.isStopping = false
    }
  }

  /**
   * 重启内核
   */
  async restartKernel(options: StartOptions & StopOptions = {}): Promise<{ success: boolean; message: string }> {
    console.log('🔄 开始重启内核...')
    
    // 先停止
    const stopResult = await this.stopKernel({ 
      force: options.force, 
      timeout_ms: options.timeout_ms 
    })
    
    if (!stopResult.success) {
      return { success: false, message: `重启失败: ${stopResult.message}` }
    }
    
    // 短暂等待
    await new Promise(resolve => setTimeout(resolve, 1000))
    
    // 再启动
    return this.startKernel(options)
  }

  /**
   * 获取内核状态 - 带缓存优化
   */
  async getKernelStatus(): Promise<KernelStatus> {
    const now = Date.now()
    const cacheKey = 'kernel_status'
    const cached = this.statusCache.get(cacheKey)
    
    // 如果缓存未过期，直接返回
    if (cached && (now - cached.timestamp) < this.CACHE_TTL) {
      return cached.status
    }

    try {
      const status = await invoke<KernelStatus>('kernel_get_status')
      
      // 更新缓存
      this.statusCache.set(cacheKey, { status, timestamp: now })
      
      return status
    } catch (error) {
      console.error('获取内核状态失败:', error)
      
      // 返回默认状态
      const defaultStatus: KernelStatus = {
        process_running: false,
        api_ready: false,
        websocket_ready: false,
        error: error instanceof Error ? error.message : '获取状态失败'
      }
      
      return defaultStatus
    }
  }

  /**
   * 检查内核是否正在运行
   */
  async isKernelRunning(): Promise<boolean> {
    const status = await this.getKernelStatus()
    return status.process_running
  }

  /**
   * 获取内核版本
   */
  async getKernelVersion(): Promise<string> {
    try {
      return await invoke<string>('kernel_get_version')
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
      console.log('🔄 切换代理模式:', mode)
      
      const result = await invoke<{ success: boolean; message: string }>('kernel_switch_proxy_mode', {
        mode,
      })
      
      // 清除状态缓存
      this.clearStatusCache()
      
      return result
    } catch (error) {
      console.error('切换代理模式失败:', error)
      return { 
        success: false, 
        message: error instanceof Error ? error.message : '切换代理模式失败' 
      }
    }
  }

  /**
   * 切换IP版本偏好
   */
  async toggleIpVersion(preferIpv6: boolean): Promise<{ success: boolean; message: string }> {
    try {
      console.log('🔄 切换IP版本偏好:', preferIpv6)
      
      const result = await invoke<{ success: boolean; message: string }>('kernel_toggle_ip_version', {
        prefer_ipv6: preferIpv6,
      })
      
      // 清除状态缓存
      this.clearStatusCache()
      
      return result
    } catch (error) {
      console.error('切换IP版本失败:', error)
      return { 
        success: false, 
        message: error instanceof Error ? error.message : '切换IP版本失败' 
      }
    }
  }

  /**
   * 获取内核配置
   */
  async getKernelConfig(): Promise<KernelConfig> {
    try {
      return await invoke<KernelConfig>('kernel_get_config')
    } catch (error) {
      console.error('获取内核配置失败:', error)
      throw error
    }
  }

  /**
   * 更新内核配置
   */
  async updateKernelConfig(config: Partial<KernelConfig>): Promise<{ success: boolean; message: string }> {
    try {
      console.log('🔧 更新内核配置:', config)
      
      const result = await invoke<{ success: boolean; message: string }>('kernel_update_config', {
        config,
      })
      
      // 清除状态缓存
      this.clearStatusCache()
      
      return result
    } catch (error) {
      console.error('更新内核配置失败:', error)
      return { 
        success: false, 
        message: error instanceof Error ? error.message : '更新内核配置失败' 
      }
    }
  }

  /**
   * 检查内核健康状态
   */
  async checkKernelHealth(): Promise<{ healthy: boolean; issues: string[] }> {
    try {
      return await invoke<{ healthy: boolean; issues: string[] }>('kernel_check_health')
    } catch (error) {
      console.error('检查内核健康状态失败:', error)
      return { 
        healthy: false, 
        issues: [error instanceof Error ? error.message : '健康检查失败'] 
      }
    }
  }

  /**
   * 清除状态缓存
   */
  private clearStatusCache(): void {
    this.statusCache.clear()
  }

  /**
   * 监听内核事件
   */
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

// 导出单例
export const kernelService = new KernelService()
export default kernelService