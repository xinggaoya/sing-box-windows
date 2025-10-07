/**
 * 内核管理服务 - 重构版本
 * 提供高可用性和更好的用户体验
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
  forceRestart?: boolean
  timeoutMs?: number
}

export interface StopOptions {
  force?: boolean
  timeoutMs?: number
}

class KernelService {
  private isStarting = false
  private isStopping = false
  private statusCache = new Map<string, { status: KernelStatus; timestamp: number }>()
  private readonly CACHE_TTL = 2000 // 2秒缓存

  /**
   * 启动内核 - 使用新的增强命令
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
      
      // 获取当前端口配置
      const { useAppStore } = await import('@/stores/app/AppStore')
      const appStore = useAppStore()
      await appStore.waitForDataRestore(5000) // 等待端口配置加载完成
      
      const proxyMode = options.config?.proxy_mode || 'manual'
      const apiPort = options.config?.api_port || appStore.apiPort
      
      console.log('🔌 使用端口配置:', { apiPort, proxyMode })
      
      // 使用新的增强启动命令
      const result = await invoke<string>('kernel_start_enhanced', { 
        proxyMode,
        apiPort 
      })
      
      console.log('✅ 内核启动结果:', result)
      
      // 清除状态缓存
      this.clearStatusCache()
      
      return { success: true, message: result }
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
   * 停止内核 - 使用新的增强命令
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
      
      // 使用新的增强停止命令
      const result = await invoke<string>('kernel_stop_enhanced')
      
      console.log('✅ 内核停止结果:', result)
      
      // 清除状态缓存
      this.clearStatusCache()
      
      return { success: true, message: result }
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
      timeoutMs: options.timeoutMs 
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
   * 获取内核状态 - 使用增强命令
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
      // 获取当前端口配置
      const { useAppStore } = await import('@/stores/app/AppStore')
      const appStore = useAppStore()
      await appStore.waitForDataRestore(3000) // 等待端口配置加载完成
      
      const apiPort = appStore.apiPort
      console.log('📊 查询内核状态，使用API端口:', apiPort)
      
      const status = await invoke<any>('kernel_get_status_enhanced', { 
        apiPort 
      })
      
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
      return await invoke<string>('check_kernel_version')
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
      
      // 暂时使用现有的切换命令
      const { proxyApi } = await import('./tauri-api')
      await proxyApi.toggleProxyMode(mode)
      
      // 清除状态缓存
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

  /**
   * 切换IP版本偏好
   */
  async toggleIpVersion(preferIpv6: boolean): Promise<{ success: boolean; message: string }> {
    try {
      console.log('🔄 切换IP版本偏好:', preferIpv6)
      
      // 暂时使用现有的切换命令
      const { proxyApi } = await import('./tauri-api')
      await proxyApi.toggleIpVersion(preferIpv6)
      
      // 清除状态缓存
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

  /**
   * 获取内核配置
   */
  async getKernelConfig(): Promise<KernelConfig> {
    // 暂时返回默认配置
    return {
      proxy_mode: 'manual',
      api_port: 9090,
      proxy_port: 7890,
      prefer_ipv6: false,
      auto_start: false,
    }
  }

  /**
   * 更新内核配置
   */
  async updateKernelConfig(config: Partial<KernelConfig>): Promise<{ success: boolean; message: string }> {
    try {
      console.log('🔧 更新内核配置:', config)
      
      // 暂时返回成功
      return { success: true, message: '配置更新成功' }
    } catch (error) {
      console.error('更新内核配置失败:', error)
      return { 
        success: false, 
        message: error instanceof Error ? error.message : '更新内核配置失败' 
      }
    }
  }

  /**
   * 检查内核健康状态 - 使用新命令
   */
  async checkKernelHealth(): Promise<{ healthy: boolean; issues: string[] }> {
    try {
      // 获取当前端口配置
      const { useAppStore } = await import('@/stores/app/AppStore')
      const appStore = useAppStore()
      await appStore.waitForDataRestore(3000) // 等待端口配置加载完成
      
      const apiPort = appStore.apiPort
      console.log('🏥 检查内核健康状态，使用API端口:', apiPort)
      
      return await invoke<{ healthy: boolean; issues: string[] }>('kernel_check_health', { 
        apiPort 
      })
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
  onKernelStatusChange(callback: (status: KernelStatus) => void): () => void {
    return eventService.on('kernel-status-changed', callback)
  }

  onKernelReady(callback: () => void): () => void {
    return eventService.on('kernel-ready', callback)
  }

  onKernelError(callback: (error: string) => void): () => void {
    return eventService.on('kernel-error', callback)
  }
}

// 导出单例
export const kernelService = new KernelService()
export default kernelService