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

interface NodeDelay {
  delay: number
}

interface VersionInfo {
  version: string
  meta: boolean
  premium: boolean
  environment?: string
  tags?: string[]
  revision?: string
  cgo?: string
}

// 内核相关接口
export const kernelApi = {
  // 启动内核
  startKernel: (proxyMode?: string) => {
    const appStore = useAppStore()
    return invoke<void>('start_kernel', { proxyMode, apiPort: appStore.apiPort })
  },

  // 停止内核
  stopKernel: () => invoke<void>('stop_kernel'),

  // 重启内核
  restartKernel: () => invoke<void>('restart_kernel'),

  // 检查内核版本
  checkKernelVersion: () => invoke<string>('check_kernel_version'),

  // 下载最新内核
  downloadLatestKernel: () => invoke<void>('download_latest_kernel'),

  // 启动WebSocket中继
  startWebSocketRelay: () => {
    const appStore = useAppStore()
    return invoke<void>('start_websocket_relay', { apiPort: appStore.apiPort })
  },

  // 检查内核是否运行
  isKernelRunning: () => invoke<boolean>('is_kernel_running'),

  // 检查内核完整状态
  checkKernelStatus: () =>
    invoke<{
      process_running: boolean
      api_ready: boolean
      websocket_ready: boolean
    }>('check_kernel_status'),
}

// 代理相关接口
export const proxyApi = {
  // 设置系统代理
  setSystemProxy: () => {
    const appStore = useAppStore()
    return invoke<void>('set_system_proxy', { port: appStore.proxyPort })
  },

  // 设置TUN代理
  setTunProxy: () => {
    const appStore = useAppStore()
    return invoke<void>('set_tun_proxy', { port: appStore.proxyPort })
  },

  // 切换IP版本偏好
  toggleIpVersion: (preferIpv6: boolean) => invoke<void>('toggle_ip_version', { preferIpv6 }),

  // 切换代理模式（global, rule, tun）
  toggleProxyMode: (mode: string) => invoke<string>('toggle_proxy_mode', { mode }),

  // 获取当前代理模式
  getCurrentProxyMode: () => invoke<string>('get_current_proxy_mode'),

  // 获取代理列表
  getProxies: (port: number) => invoke<ProxiesData>('get_proxies', { port }),

  // 切换代理
  changeProxy: (group: string, proxy: string, port: number) =>
    invoke<void>('change_proxy', { group, proxy, port }),

  // 测试节点组延迟
  testGroupDelay: (group: string, port: number) =>
    invoke<void>('test_group_delay', { group, port }),

  // 测试节点延迟
  testNodeDelay: (proxy: string) => {
    const appStore = useAppStore()
    return invoke<void>('test_node_delay', { proxy, port: appStore.apiPort })
  },

  // 获取版本信息
  getVersionInfo: () => {
    const appStore = useAppStore()
    return invoke<VersionInfo>('get_version_info', { port: appStore.apiPort })
  },

  // 获取规则列表
  getRules: () => {
    const appStore = useAppStore()
    return invoke<{ rules: Array<{ type: string; payload: string; proxy: string }> }>('get_rules', {
      port: appStore.apiPort,
    })
  },

  // 获取API令牌
  getApiToken: () => invoke<string>('get_api_token'),

  // 设置手动代理
  setManualProxy: () => {
    const appStore = useAppStore()
    return invoke<void>('set_manual_proxy', { port: appStore.proxyPort })
  },
}

// 配置服务
export const config = {
  // 更新sing-box配置文件中的端口设置
  updateSingboxPorts: async (proxyPort: number, apiPort: number) => {
    return await invoke<boolean>('update_singbox_ports', { proxyPort, apiPort })
  },
}

// 订阅相关接口
export const subscriptionApi = {
  // 下载订阅
  downloadSubscription: (url: string, useOriginalConfig: boolean = false) => {
    const appStore = useAppStore()
    return invoke<void>('download_subscription', {
      url,
      useOriginalConfig,
      proxyPort: appStore.proxyPort,
      apiPort: appStore.apiPort,
    })
  },

  // 下载最新内核
  downloadLatestKernel: () => invoke<void>('download_latest_kernel'),

  // 获取当前配置
  getCurrentConfig: () => invoke<string>('get_current_config'),

  // 添加手动配置
  addManualSubscription: (content: string, useOriginalConfig: boolean = false) => {
    const appStore = useAppStore()
    return invoke<void>('add_manual_subscription', {
      content,
      useOriginalConfig,
      proxyPort: appStore.proxyPort,
      apiPort: appStore.apiPort,
    })
  },
}

// 统一导出所有 API
export const tauriApi = {
  kernel: kernelApi,
  proxy: proxyApi,
  subscription: subscriptionApi,
  config: config,

  // 系统服务相关 API
  system: {
    // 检查管理员权限
    checkAdmin: () => invoke<boolean>('check_admin'),
    // 以管理员权限重启
    restartAsAdmin: () => invoke<void>('restart_as_admin'),
    // 退出应用程序
    exitApplication: () => invoke<void>('exit_application'),
    // 切换开发者工具
    toggleDevtools: () => invoke<void>('toggle_devtools'),
    // 打开开发者工具
    openDevtools: () => invoke<void>('open_devtools'),
    // 关闭开发者工具
    closeDevtools: () => invoke<void>('close_devtools'),
    // 检查开发者工具是否已打开
    isDevtoolsOpen: () => invoke<boolean>('is_devtools_open'),
  },

  // 更新相关 API
  update: {
    // 检查更新
    checkUpdate: async (currentVersion: string) => {
      return await invoke<{
        latest_version: string
        download_url: string
        has_update: boolean
        release_notes?: string
        release_date?: string
        file_size?: number
      }>('check_update', { currentVersion })
    },

    // 下载并安装更新
    downloadAndInstallUpdate: async (downloadUrl: string) => {
      return await invoke<void>('download_and_install_update', { downloadUrl })
    },
  },
}

// 代理服务
export const proxy = {
  // 获取代理列表
  getProxies: async () => {
    const appStore = useAppStore()
    return await invoke<unknown>('get_proxies', { port: appStore.apiPort })
  },

  // 切换代理
  changeProxy: async (group: string, proxy: string) => {
    const appStore = useAppStore()
    return await invoke<void>('change_proxy', { group, proxy, port: appStore.apiPort })
  },

  // 测试节点延迟
  testNodeDelay: async (proxy: string, server?: string) => {
    const appStore = useAppStore()
    return await invoke<void>('test_node_delay', { proxy, server, port: appStore.apiPort })
  },

  // 测试节点组延迟
  testGroupDelay: async (group: string, server?: string) => {
    const appStore = useAppStore()
    return await invoke<void>('test_group_delay', { group, server, port: appStore.apiPort })
  },

  // 获取API Token
  getApiToken: async () => {
    return await invoke<string>('get_api_token')
  },

  // 切换IP版本
  toggleIpVersion: async (preferIpv6: boolean) => {
    return await invoke<void>('toggle_ip_version', { preferIpv6 })
  },

  // 设置系统代理
  setSystemProxy: async () => {
    const appStore = useAppStore()
    return await invoke<void>('set_system_proxy', { port: appStore.proxyPort })
  },

  // 设置手动代理
  setManualProxy: async () => {
    const appStore = useAppStore()
    return await invoke<void>('set_manual_proxy', { port: appStore.proxyPort })
  },
}

// 订阅服务
export const subscription = {
  // 下载订阅
  downloadSubscription: async (url: string, useOriginalConfig: boolean) => {
    const appStore = useAppStore()
    return await invoke<void>('download_subscription', {
      url,
      useOriginalConfig,
      proxyPort: appStore.proxyPort,
      apiPort: appStore.apiPort,
    })
  },

  // 手动添加订阅
  addManualSubscription: async (content: string, useOriginalConfig: boolean) => {
    const appStore = useAppStore()
    return await invoke<void>('add_manual_subscription', {
      content,
      useOriginalConfig,
      proxyPort: appStore.proxyPort,
      apiPort: appStore.apiPort,
    })
  },

  // 切换代理模式
  toggleProxyMode: async (mode: string) => {
    return await invoke<string>('toggle_proxy_mode', { mode })
  },

  // 获取当前代理模式
  getCurrentProxyMode: async () => {
    return await invoke<string>('get_current_proxy_mode')
  },
}
