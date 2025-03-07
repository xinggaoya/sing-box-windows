import { invoke } from '@tauri-apps/api/core'

// 内核管理相关接口
export const kernelApi = {
  // 获取内存使用情况
  getMemoryUsage: () => invoke<string>('get_memory_usage'),

  // 获取流量数据
  getTrafficData: () => invoke<string>('get_traffic_data'),

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
}

// 订阅相关接口
export const subscriptionApi = {
  // 下载订阅
  downloadSubscription: (url: string) => invoke<void>('download_subscription', { url }),

  // 下载最新内核
  downloadLatestKernel: () => invoke<void>('download_latest_kernel'),

  // 获取当前配置
  getCurrentConfig: () => invoke<string>('get_current_config'),
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
