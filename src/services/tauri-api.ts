import { invoke } from '@tauri-apps/api/core'

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

// 内核管理相关接口
export const kernelApi = {
  // 启动内核
  startKernel: () => invoke<void>('start_kernel'),

  // 停止内核
  stopKernel: () => invoke<void>('stop_kernel'),

  // 重启内核
  restartKernel: () => invoke<void>('restart_kernel'),

  // 获取进程状态
  getProcessStatus: () => invoke<string>('get_process_status'),

  // 检查内核版本
  checkKernelVersion: () => invoke<string>('check_kernel_version'),

  // 启动WebSocket数据中继
  startWebsocketRelay: () => invoke<void>('start_websocket_relay'),
}

// 代理模式相关接口
export const proxyApi = {
  // 设置系统代理模式
  setSystemProxy: () => invoke<void>('set_system_proxy'),

  // 设置 TUN 代理模式
  setTunProxy: () => invoke<void>('set_tun_proxy'),

  // 检查管理员权限
  checkAdmin: () => invoke<boolean>('check_admin'),

  // 以管理员权限重启
  restartAsAdmin: () => invoke<void>('restart_as_admin'),

  // 切换 IP 版本
  toggleIpVersion: (preferIpv6: boolean) => invoke<void>('toggle_ip_version', { preferIpv6 }),

  // 切换代理模式（global, rule, tun）
  toggleProxyMode: (mode: string) => invoke<string>('toggle_proxy_mode', { mode }),

  // 获取当前代理模式
  getCurrentProxyMode: () => invoke<string>('get_current_proxy_mode'),

  // 获取代理列表
  getProxies: () => invoke<ProxiesData>('get_proxies'),

  // 切换代理
  changeProxy: (group: string, proxy: string) => invoke<void>('change_proxy', { group, proxy }),

  // 测试节点组延迟
  testGroupDelay: (group: string) => invoke<void>('test_group_delay', { group }),

  // 获取版本信息
  getVersionInfo: () => invoke<VersionInfo>('get_version_info'),

  // 获取规则列表
  getRules: () =>
    invoke<{ rules: Array<{ type: string; payload: string; proxy: string }> }>('get_rules'),
}

// 订阅相关接口
export const subscriptionApi = {
  // 下载订阅
  downloadSubscription: (url: string, useSubscriptionRules: boolean = false) =>
    invoke<void>('download_subscription', { url, useSubscriptionRules }),

  // 下载最新内核
  downloadLatestKernel: () => invoke<void>('download_latest_kernel'),

  // 获取当前配置
  getCurrentConfig: () => invoke<string>('get_current_config'),

  // 添加手动配置
  addManualSubscription: (content: string, useSubscriptionRules: boolean = false) =>
    invoke<void>('add_manual_subscription', { content, useSubscriptionRules }),
}

// 统一导出所有 API
export const tauriApi = {
  kernel: kernelApi,
  proxy: proxyApi,
  subscription: subscriptionApi,

  // 更新相关 API
  update: {
    // 检查更新
    checkUpdate: async (currentVersion: string) => {
      return await invoke<{
        latest_version: string
        download_url: string
        has_update: boolean
      }>('check_update', { currentVersion })
    },

    // 下载并安装更新
    downloadAndInstallUpdate: async (downloadUrl: string) => {
      return await invoke<void>('download_and_install_update', { downloadUrl })
    },
  },
}
