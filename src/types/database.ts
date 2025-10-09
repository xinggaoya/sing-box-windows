// 数据库配置类型定义
export interface AppConfig {
  auto_start_kernel: boolean
  prefer_ipv6: boolean
  proxy_port: number
  api_port: number
  proxy_mode: string
  tray_instance_id?: string | null
}

export interface ThemeConfig {
  is_dark: boolean
}

export interface LocaleConfig {
  locale: string
}

export interface WindowConfig {
  is_maximized: boolean
  width: number
  height: number
}

export interface UpdateConfig {
  auto_check: boolean
  last_check: number
  last_version?: string | null
  skip_version?: string | null
}

export interface Subscription {
  name: string
  url: string
  is_loading: boolean
  last_update?: number | null
  is_manual: boolean
  manual_content?: string | null
  use_original_config: boolean
}