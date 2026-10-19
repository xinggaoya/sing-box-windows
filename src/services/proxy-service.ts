import { invokeWithAppContext } from './invoke-client'

// === gRPC API 返回类型 ===
//
// 全部来自 `src/types/generated/`（由 Rust 侧 ts-rs 导出，是前后端唯一事实来源）。
// ⚠️ 禁止在此手写这些结构的 TypeScript 声明——后端改字段名前端不会报错，
// 只会静默拿到 undefined。守护规则见 `src/types/generated/grpc-types.test.ts`。

import type { Groups } from '@/types/generated/Groups'
import type { Group } from '@/types/generated/Group'
import type { GroupItem } from '@/types/generated/GroupItem'
import type { OutboundList } from '@/types/generated/OutboundList'
import type { RuleList } from '@/types/generated/RuleList'
import type { ServiceList } from '@/types/generated/ServiceList'
import type { ServiceStatusSnapshot } from '@/types/generated/ServiceStatusSnapshot'
import type { NetworkQualityResult } from '@/types/generated/NetworkQualityResult'
import type { ClashModeStatus } from '@/types/generated/ClashModeStatus'
import type { DeprecatedWarnings } from '@/types/generated/DeprecatedWarnings'

export type { Groups, Group, GroupItem, OutboundList, RuleList, ServiceList, ServiceStatusSnapshot }

/** 节点测速结果（前端封装，对应 /SelectOutbound + /URLTest 调用结果） */
export interface UrlTestResult {
  ok: boolean
  message?: string
}

export interface TunOptionsPayload {
  ipv4_address: string
  ipv6_address: string
  mtu: number
  auto_route: boolean
  strict_route: boolean
  stack: 'system' | 'gvisor' | 'mixed'
  enable_ipv6: boolean
  route_exclude_address?: string[]
}

export class ProxyService {
  private static instance: ProxyService

  private constructor() {}

  public static getInstance(): ProxyService {
    if (!ProxyService.instance) {
      ProxyService.instance = new ProxyService()
    }
    return ProxyService.instance
  }

  // === Inbound 写入（不依赖 gRPC） ===

  async setSystemProxy(systemProxyBypass?: string) {
    const args =
      typeof systemProxyBypass === 'string'
        ? { systemProxyBypass, system_proxy_bypass: systemProxyBypass }
        : undefined
    return invokeWithAppContext<void>('set_system_proxy', args, {
      withProxyPort: 'port',
    })
  }

  async setTunProxy(tunOptions?: TunOptionsPayload) {
    const args = tunOptions ? { tunOptions, tun_options: tunOptions } : undefined
    return invokeWithAppContext<void>('set_tun_proxy', args, {
      withProxyPort: 'port',
    })
  }

  async setManualProxy() {
    return invokeWithAppContext<void>('set_manual_proxy', undefined, {
      withProxyPort: 'port',
    })
  }

  async toggleIpVersion(preferIpv6: boolean) {
    return invokeWithAppContext<void>('toggle_ip_version', { preferIpv6 })
  }

  // === gRPC API 交互（sing-box 1.14+ 官方 type: api） ===

  /** 获取内核版本号（"1.14.0" 等） */
  async getKernelVersion(): Promise<string> {
    return invokeWithAppContext<string>('get_kernel_version')
  }

  /** 获取所有节点组（一次快照） */
  async getGroups(): Promise<Groups> {
    return invokeWithAppContext<Groups>('get_groups')
  }

  /** 切换代理节点（gRPC SelectOutbound） */
  async selectOutbound(group: string, proxy: string): Promise<void> {
    return invokeWithAppContext<void>('select_outbound', { group, proxy })
  }

  /** 整组测速（gRPC URLTest） */
  async urlTest(group: string): Promise<UrlTestResult> {
    try {
      await invokeWithAppContext<void>('url_test', { group })
      return { ok: true }
    } catch (e) {
      return { ok: false, message: String(e) }
    }
  }

  /** 关闭所有连接 */
  async closeAllConnections(): Promise<void> {
    return invokeWithAppContext<void>('close_all_connections')
  }

  /** 获取 Clash 模式状态 */
  async getClashModeStatus(): Promise<ClashModeStatus> {
    return invokeWithAppContext<ClashModeStatus>('get_clash_mode_status')
  }

  /** 运行时切换 Clash 模式（gRPC SetClashMode） */
  async setClashMode(mode: string): Promise<void> {
    return invokeWithAppContext<void>('set_clash_mode', { mode })
  }

  /** 持久化组展开状态（gRPC SetGroupExpand） */
  async setGroupExpand(group: string, isExpand: boolean): Promise<void> {
    return invokeWithAppContext<void>('set_group_expand', {
      group,
      isExpand,
      is_expand: isExpand,
    })
  }

  /** 获取内核启动时间戳（秒） */
  async getStartedAt(): Promise<number> {
    return invokeWithAppContext<number>('get_started_at')
  }

  // === sing-box 1.14 新增 gRPC 接口 ===

  /** 获取所有路由规则（GetRules） */
  async getRules(): Promise<RuleList> {
    return invokeWithAppContext<RuleList>('get_rules')
  }

  /** 获取全量出站快照（GetOutbounds）
   *
   * 与 `getGroups()` 的区别：Groups 是「分组视图」，Outbounds 是「内核里所有出站」的平铺列表，
   * 包含分组本身。用于出站总览与全量按延迟排序。
   */
  async getOutbounds(): Promise<OutboundList> {
    return invokeWithAppContext<OutboundList>('get_outbounds')
  }

  /** 获取运行中的服务列表（GetServices） */
  async getServices(): Promise<ServiceList> {
    return invokeWithAppContext<ServiceList>('get_services')
  }

  /** 探测内核服务运行状态（SubscribeServiceStatus 首帧） */
  async probeServiceStatus(): Promise<ServiceStatusSnapshot> {
    return invokeWithAppContext<ServiceStatusSnapshot>('probe_service_status')
  }

  /** 网络质量测试（NetworkQualityTest）：RTT / 上下行带宽与延迟 */
  async networkQualityTest(): Promise<NetworkQualityResult> {
    return invokeWithAppContext<NetworkQualityResult>('network_quality_test')
  }

  /** 获取已弃用字段告警（GetDeprecatedWarnings） */
  async getDeprecatedWarnings(): Promise<DeprecatedWarnings> {
    return invokeWithAppContext<DeprecatedWarnings>('get_deprecated_warnings')
  }
}

export const proxyService = ProxyService.getInstance()
