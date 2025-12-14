export interface AppConfig {
  auto_start_kernel: boolean
  auto_start_app: boolean
  prefer_ipv6: boolean
  proxy_port: number
  api_port: number
  proxy_mode: string
  system_proxy_enabled: boolean
  tun_enabled: boolean
  tray_instance_id: string | null
  system_proxy_bypass: string
  tun_auto_route: boolean
  tun_strict_route: boolean
  tun_mtu: number
  tun_ipv4: string
  tun_ipv6: string
  tun_stack: string
  tun_enable_ipv6: boolean
  active_config_path: string | null
  installed_kernel_version: string | null
  singbox_dns_proxy: string
  singbox_dns_cn: string
  singbox_dns_resolver: string
  singbox_urltest_url: string
  singbox_default_proxy_outbound: string
  singbox_block_ads: boolean
  singbox_download_detour: string
  singbox_dns_hijack: boolean
  singbox_enable_app_groups: boolean
}
