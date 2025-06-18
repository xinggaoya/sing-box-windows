import { invoke } from '@tauri-apps/api/core'
import type {
  KernelVersion,
  ProxyNode,
  ProxyGroup,
  TrafficStats,
  ApiResponse,
  Rule,
} from '@/types/api'

// API基础类
class ApiBase {
  protected async invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    try {
      return await invoke<T>(command, args)
    } catch (error) {
      console.error(`命令 ${command} 执行失败:`, error)
      throw error
    }
  }
}

// 内核服务API
class KernelApi extends ApiBase {
  async start(proxyMode?: string, apiPort?: number): Promise<void> {
    return this.invokeCommand('start_kernel', { proxyMode, apiPort })
  }

  async stop(): Promise<void> {
    return this.invokeCommand('stop_kernel')
  }

  async restart(): Promise<void> {
    return this.invokeCommand('restart_kernel')
  }

  async checkVersion(): Promise<string> {
    return this.invokeCommand('check_kernel_version')
  }

  async downloadLatest(): Promise<void> {
    return this.invokeCommand('download_latest_kernel')
  }

  async isRunning(): Promise<boolean> {
    return this.invokeCommand('is_kernel_running')
  }
}

// 代理服务API
class ProxyApi extends ApiBase {
  async setSystemProxy(enable: boolean): Promise<void> {
    return this.invokeCommand('set_system_proxy', { enable })
  }

  async setManualProxy(enable: boolean): Promise<void> {
    return this.invokeCommand('set_manual_proxy', { enable })
  }

  async setTunProxy(enable: boolean): Promise<void> {
    return this.invokeCommand('set_tun_proxy', { enable })
  }

  async getProxies(): Promise<{ proxies: Record<string, ProxyNode | ProxyGroup> }> {
    return this.invokeCommand('get_proxies')
  }

  async changeProxy(name: string, select: string): Promise<void> {
    return this.invokeCommand('change_proxy', { name, select })
  }

  async testNodeDelay(name: string, url?: string, timeout?: number): Promise<number> {
    return this.invokeCommand('test_node_delay', { name, url, timeout })
  }

  async testGroupDelay(name: string): Promise<Record<string, number>> {
    return this.invokeCommand('test_group_delay', { name })
  }

  async getRules(): Promise<{ rules: Rule[] }> {
    return this.invokeCommand('get_rules')
  }
}

// 订阅服务API
class SubscriptionApi extends ApiBase {
  async download(id: string, url: string): Promise<void> {
    return this.invokeCommand('download_subscription', { id, url })
  }

  async addManual(id: string, content: string): Promise<void> {
    return this.invokeCommand('add_manual_subscription', { id, content })
  }

  async getCurrentConfig(): Promise<string> {
    return this.invokeCommand('get_current_config')
  }

  async toggleProxyMode(mode: string): Promise<void> {
    return this.invokeCommand('toggle_proxy_mode', { mode })
  }

  async getCurrentProxyMode(): Promise<string> {
    return this.invokeCommand('get_current_proxy_mode')
  }
}

// 系统服务API
class SystemApi extends ApiBase {
  async checkAdmin(): Promise<boolean> {
    return this.invokeCommand('check_admin')
  }

  async restartAsAdmin(): Promise<void> {
    return this.invokeCommand('restart_as_admin')
  }

  async toggleDevtools(): Promise<void> {
    return this.invokeCommand('toggle_devtools')
  }

  async openDevtools(): Promise<void> {
    return this.invokeCommand('open_devtools')
  }

  async closeDevtools(): Promise<void> {
    return this.invokeCommand('close_devtools')
  }

  async isDevtoolsOpen(): Promise<boolean> {
    return this.invokeCommand('is_devtools_open')
  }
}

// 更新服务API
class UpdateApi extends ApiBase {
  async checkUpdate(): Promise<{ hasUpdate: boolean; version?: string }> {
    return this.invokeCommand('check_update')
  }

  async downloadAndInstall(): Promise<void> {
    return this.invokeCommand('download_and_install_update')
  }
}

// 配置服务API
class ConfigApi extends ApiBase {
  async updateSingboxPorts(proxyPort: number, apiPort: number): Promise<void> {
    return this.invokeCommand('update_singbox_ports', { proxyPort, apiPort })
  }

  async getApiToken(): Promise<string> {
    return this.invokeCommand('get_api_token')
  }
}

// 导出API实例
export const api = {
  kernel: new KernelApi(),
  proxy: new ProxyApi(),
  subscription: new SubscriptionApi(),
  system: new SystemApi(),
  update: new UpdateApi(),
  config: new ConfigApi(),
}

// 导出API类型
export type Api = typeof api
