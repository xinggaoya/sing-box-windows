import type { CustomRule } from './CustomRule'

/**
 * 可视化模板表单（由表单生成官方骨架模板，无需手写 JSON）。
 * 留空的字段在生成时跟随当前应用设置。
 */
export interface TemplateFormOptions {
  /** 非国内流量默认出站：manual（手动切换）/ auto（自动选择） */
  default_outbound: string
  /** 自动选择组测速地址；空 = 跟随应用设置 */
  urltest_url?: string | null
  /** 广告拦截（DNS + 路由双层 reject） */
  block_ads: boolean
  /** DNS 劫持（hijack-dns） */
  dns_hijack: boolean
  /** Fake DNS（fakeip） */
  fake_dns_enabled: boolean
  /** mDNS（*.local 本地多播解析） */
  dns_use_mdns: boolean
  /** 启用的业务分流组 tag：Telegram/YouTube/Netflix/OpenAI/Google */
  app_groups: string[]
  /** 代理 DNS（DoH）；空 = 跟随应用设置 */
  dns_proxy?: string | null
  /** 国内 DNS；空 = 跟随应用设置 */
  dns_cn?: string | null
  /** 默认 DNS 解析；空 = 跟随应用设置 */
  dns_resolver?: string | null
  /** 自定义分流规则（优先级高于内置 CN/私网分流） */
  custom_rules: CustomRule[]
}
