import { invokeWithAppContext } from './invoke-client'

export interface ProxyLatencyHistoryEntry {
  time: string
  delay: number
}

export interface ProxyData {
  type: string
  name: string
  now: string
  all: string[]
  history: ProxyLatencyHistoryEntry[]
  udp: boolean
}

export interface ProxiesData {
  proxies: Record<string, ProxyData>
}

export const proxyApi = {
  setSystemProxy() {
    return invokeWithAppContext<void>('set_system_proxy', undefined, {
      withProxyPort: 'port'
    })
  },

  setTunProxy() {
    return invokeWithAppContext<void>('set_tun_proxy', undefined, {
      withProxyPort: 'port'
    })
  },

  setManualProxy() {
    return invokeWithAppContext<void>('set_manual_proxy', undefined, {
      withProxyPort: 'port'
    })
  },

  clearProxy() {
    return invokeWithAppContext<void>('clear_proxy')
  },

  toggleIpVersion(preferIpv6: boolean) {
    return invokeWithAppContext<void>('toggle_ip_version', { preferIpv6 })
  },

  toggleProxyMode(mode: string) {
    return invokeWithAppContext<string>('toggle_proxy_mode', { mode })
  },

  getCurrentProxyMode() {
    return invokeWithAppContext<string>('get_current_proxy_mode')
  },

  getProxies() {
    return invokeWithAppContext<ProxiesData>('get_proxies', undefined, {
      withApiPort: 'port'
    })
  },

  changeProxy(group: string, proxy: string, server?: string, port?: number) {
    const args = { group, proxy, server, port }
    return invokeWithAppContext<void>(
      'change_proxy',
      port ? args : { ...args, port: undefined },
      { withApiPort: port ? undefined : 'port' }
    )
  },

  testNodeDelay(proxy: string, server?: string, port?: number) {
    const args = { proxy, server, port }
    return invokeWithAppContext<void>(
      'test_node_delay',
      port ? args : { ...args, port: undefined },
      { withApiPort: port ? undefined : 'port' }
    )
  },

  testGroupDelay(group: string, server?: string, port?: number) {
    const args = { group, server, port }
    return invokeWithAppContext<void>(
      'test_group_delay',
      port ? args : { ...args, port: undefined },
      { withApiPort: port ? undefined : 'port' }
    )
  },

  testAllNodesDelay(port?: number) {
    const args = typeof port === 'number' ? { port } : undefined
    return invokeWithAppContext<void>('test_all_nodes_delay', args, {
      withApiPort: typeof port === 'number' ? undefined : 'port'
    })
  },

  getDelayData() {
    return invokeWithAppContext<Record<string, number>>('get_delay_data')
  },

  clearDelayData() {
    return invokeWithAppContext<void>('clear_delay_data')
  },

  getRules(port?: number) {
    const args = typeof port === 'number' ? { port } : undefined
    return invokeWithAppContext<unknown>('get_rules', args, {
      withApiPort: typeof port === 'number' ? undefined : 'port'
    })
  }
}
