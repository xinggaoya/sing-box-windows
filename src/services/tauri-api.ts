import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/app/AppStore'

// 定义接口类型
interface ProxyData {
  type: string
  name: string
  now: string
  all: string[]
  history: Array<{ time: string; delay: number }>
  udp: boolean
}

interface ProxiesData {
  proxies: Record<string, ProxyData>
}

// 内核相关接口
export const kernelApi = {
  startKernel: async (proxyMode?: string, apiPort?: number) => {
    const appStore = useAppStore()
    await appStore.waitForDataRestore()
    return invoke<void>('start_kernel', { proxyMode, apiPort: apiPort || appStore.apiPort })
  },

  stopKernel: () => invoke<void>('stop_kernel'),

  restartKernel: async () => {
    const appStore = useAppStore()
    await appStore.waitForDataRestore()
    return invoke<void>('restart_kernel', { apiPort: appStore.apiPort })
  },

  checkKernelVersion: () => invoke<string>('check_kernel_version'),

  checkKernelStatus: (apiPort?: number) => {
    const appStore = useAppStore()
    return invoke<any>('check_kernel_status', { apiPort: apiPort || appStore.apiPort })
  },

  getKernelRunningState: () => invoke<boolean>('is_kernel_running'),

  getApiToken: () => invoke<string>('get_api_token'),

  downloadKernel: (window: any) => invoke<void>('download_kernel', { window }),

  installKernel: () => invoke<void>('install_kernel'),

  isKernelRunning: () => invoke<boolean>('is_kernel_running')
}

// 代理相关接口
export const proxyApi = {
  setSystemProxy: async () => {
    const appStore = useAppStore()
    return invoke<void>('set_system_proxy', { port: appStore.proxyPort })
  },

  setTunProxy: async () => {
    const appStore = useAppStore()
    return invoke<void>('set_tun_proxy', { port: appStore.proxyPort })
  },

  setManualProxy: async () => {
    const appStore = useAppStore()
    return invoke<void>('set_manual_proxy', { port: appStore.proxyPort })
  },

  clearProxy: () => invoke<void>('clear_proxy'),

  toggleIpVersion: async (preferIpv6: boolean) => 
    invoke<void>('toggle_ip_version', { preferIpv6 }),

  toggleProxyMode: async (mode: string) => 
    invoke<string>('toggle_proxy_mode', { mode }),

  getCurrentProxyMode: () => invoke<string>('get_current_proxy_mode'),

  getProxies: async () => {
    const appStore = useAppStore()
    return invoke<ProxiesData>('get_proxies', { port: appStore.apiPort })
  },

  changeProxy: async (group: string, proxy: string, server?: string, port?: number) => {
    if (port !== undefined) {
      return invoke<void>('change_proxy', { group, proxy, server, port })
    }
    const appStore = useAppStore()
    return invoke<void>('change_proxy', { group, proxy, server, port: appStore.apiPort })
  },

  testNodeDelay: async (proxy: string, server?: string, port?: number) => {
    if (port !== undefined) {
      return invoke<void>('test_node_delay', { proxy, server, port })
    }
    const appStore = useAppStore()
    return invoke<void>('test_node_delay', { proxy, server, port: appStore.apiPort })
  },

  testGroupDelay: async (group: string, server?: string, port?: number) => {
    if (port !== undefined) {
      return invoke<void>('test_group_delay', { group, server, port })
    }
    const appStore = useAppStore()
    return invoke<void>('test_group_delay', { group, server, port: appStore.apiPort })
  },

  testAllNodesDelay: async (port?: number) => {
    if (port !== undefined) {
      return invoke<void>('test_all_nodes_delay', { port })
    }
    const appStore = useAppStore()
    return invoke<void>('test_all_nodes_delay', { port: appStore.apiPort })
  },

  getDelayData: () => invoke<Record<string, number>>('get_delay_data'),

  clearDelayData: () => invoke<void>('clear_delay_data'),

  getRules: async (port?: number) => {
    if (port !== undefined) {
      return invoke<any>('get_rules', { port })
    }
    const appStore = useAppStore()
    return invoke<any>('get_rules', { port: appStore.apiPort })
  }
}

// 订阅服务
export const subscriptionApi = {
  downloadSubscription: async (url: string, useOriginalConfig: boolean) => {
    const appStore = useAppStore()
    return invoke<void>('download_subscription', {
      url,
      useOriginalConfig,
      proxyPort: appStore.proxyPort,
      apiPort: appStore.apiPort
    })
  },

  updateSubscription: async (url: string) => {
    const appStore = useAppStore()
    return invoke<void>('update_subscription', {
      url,
      proxyPort: appStore.proxyPort,
      apiPort: appStore.apiPort
    })
  },

  addManualSubscription: async (content: string, useOriginalConfig: boolean) => {
    const appStore = useAppStore()
    return invoke<void>('add_manual_subscription', {
      content,
      useOriginalConfig,
      proxyPort: appStore.proxyPort,
      apiPort: appStore.apiPort
    })
  },

  getCurrentConfig: async () => {
    const appStore = useAppStore()
    return invoke<any>('get_current_config', {
      apiPort: appStore.apiPort
    })
  }
}

// 系统服务
export const systemApi = {
  checkAdmin: () => invoke<boolean>('check_admin'),

  restartAsAdmin: () => invoke<void>('restart_as_admin'),

  getSystemUptime: () => invoke<number>('get_system_uptime'),

  checkUpdate: (currentVersion?: string, includePrerelease?: boolean) => {
    const version = currentVersion || '1.8.2' // 默认版本
    const includePre = includePrerelease || false
    return invoke<any>('check_update', { 
      currentVersion: version, 
      includePrerelease: includePre 
    })
  },

  downloadUpdate: (window: any) => invoke<void>('download_update', { window }),

  installUpdate: (downloadPath: string, window: any) => 
    invoke<void>('install_update', { downloadPath, window }),

  downloadAndInstallUpdate: async (window: any) => {
    await systemApi.downloadUpdate(window)
    console.warn('downloadAndInstallUpdate 只触发了下载，安装需要手动完成')
  },

  downloadLatestKernel: (window: any) => invoke<void>('download_latest_kernel', { window }),

  openDirectory: (path: string) => invoke<void>('open_directory', { path }),

  openUrl: (url: string) => invoke<void>('open_url', { url }),

  openDevtools: () => invoke<void>('open_devtools')
}

// 配置服务
export const configApi = {
  saveConfig: (config: any) => invoke<void>('save_config', { config }),

  loadConfig: () => invoke<any>('load_config'),

  importConfig: (path: string) => invoke<any>('import_config', { path }),

  exportConfig: (path: string, config: any) => 
    invoke<void>('export_config', { path, config }),

  resetConfig: () => invoke<void>('reset_config'),

  updateSingboxPorts: (proxyPort: number, apiPort: number) => 
    invoke<void>('update_singbox_ports', { proxyPort, apiPort })
}

// 向后兼容的导出
export const proxy = proxyApi
export const subscription = subscriptionApi
export const kernel = kernelApi
export const system = systemApi
export const config = configApi

// 旧的tauriApi导出
export const tauriApi = {
  kernel: kernelApi,
  proxy: proxyApi,
  subscription: subscriptionApi,
  system: systemApi,
  config: configApi,
  update: systemApi,
  downloadLatestKernel: systemApi.downloadLatestKernel,
  isKernelRunning: kernelApi.isKernelRunning,
  getRules: proxyApi.getRules,
  addManualSubscription: subscriptionApi.addManualSubscription,
  getCurrentConfig: subscriptionApi.getCurrentConfig,
  openDevtools: systemApi.openDevtools,
  downloadAndInstallUpdate: systemApi.downloadAndInstallUpdate
}