export interface ProxyHistoryEntry {
  time: string
  delay: number
}

export interface ProxyNode {
  name: string
  type: string
  history?: ProxyHistoryEntry[]
  alive?: boolean
  udp?: boolean
  xudp?: boolean
  tfo?: boolean
  now?: string
  all?: string[]
  hidden?: boolean
  icon?: string
  provider?: string
  testUrl?: string
  extra?: Record<string, { history?: ProxyHistoryEntry[] }>
}

export interface ProxyProvider {
  name: string
  type?: string
  vehicleType?: string
  updatedAt?: string
  updateAt?: string
  behavior?: string
  path?: string
  count?: number
  testUrl?: string
  subscriptionInfo?: {
    upload?: number
    download?: number
    total?: number
    expire?: number
  }
  proxies?: ProxyNode[]
}

export interface ProxiesResponse {
  proxies: Record<string, ProxyNode>
}

export interface ProxyProvidersResponse {
  providers: Record<string, ProxyProvider>
}

export interface RuleItem {
  index?: number
  type: string
  payload: string
  proxy: string
  size?: number
  extra?: {
    disabled?: boolean
  }
}

export interface RulesResponse {
  rules: Record<string, RuleItem> | RuleItem[]
}

export interface RuleProvider {
  name: string
  type?: string
  behavior?: string
  format?: string
  vehicleType?: string
  updatedAt?: string
  updateAt?: string
  count?: number
}

export interface RuleProvidersResponse {
  providers: Record<string, RuleProvider>
}
