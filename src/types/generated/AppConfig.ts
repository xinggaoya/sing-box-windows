export interface AppConfig {
  auto_start_kernel: boolean
  auto_start_app: boolean
  auto_hide_to_tray_on_autostart: boolean
  tray_close_behavior: string
  prefer_ipv6: boolean
  allow_lan_access: boolean
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
  tun_route_exclude_address: string[] | null
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
  singbox_fake_dns_enabled: boolean
  singbox_fake_dns_ipv4_range: string
  singbox_fake_dns_ipv6_range: string
  singbox_fake_dns_filter_mode: string
  singbox_enable_app_groups: boolean
  tun_self_heal_enabled: boolean
  tun_self_heal_cooldown_secs: number
  // === sing-box 1.14 升级新增字段（ts-rs 自动生成） ===
  /** 内核下载/升级通道：stable / oldstable / beta / testing */
  kernel_update_track: string
  /** 1.14 DNS 乐观缓存 */
  singbox_dns_optimistic_cache: boolean
  /** 1.14 per-server DNS 超时（如 "5s"） */
  singbox_dns_timeout: string
  /** 1.14 启用 mDNS server（*.local / link-local） */
  singbox_dns_use_mdns: boolean
  /** 1.14 TLS spoof（SNI 诱骗；仅 Windows x64/x86 + Admin） */
  singbox_enable_tls_spoof: boolean
  /** 1.14 TUN dns_mode：hijack / tun / off */
  tun_dns_mode: string
  /** 1.14 TUN include_mac_address */
  tun_include_macs: string[]
  /** 1.14 TUN exclude_mac_address */
  tun_exclude_macs: string[]
  /** 1.14 Hysteria2 disable_chrome_parrot */
  hysteria2_disable_chrome_parrot: boolean
  /** 1.14 Hysteria2 obfs 类型：salamander / gecko */
  hysteria2_obfs_type: string
  /** 1.14 Clash Mode 持久化：rule / global / direct */
  clash_mode: string
  /** 1.14 启用 Web Dashboard（sing-box-dashboard） */
  enable_web_dashboard: boolean
  /** 1.14 启用 Tailscale endpoint */
  enable_tailscale_endpoint: boolean
  /** Tailscale 节点 SSH server */
  tailscale_run_ssh_server: boolean
  /** Tailscale Taildrop 收件箱目录 */
  tailscale_taildrop_directory: string
}
