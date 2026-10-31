import { ref, type Ref } from 'vue'
import type { AppConfig } from '@/types/generated/AppConfig'

/**
 * 应用配置的字段声明表（单一事实来源）
 *
 * ## 为什么需要它
 *
 * `AppStore` 的 53 个配置字段此前在 **7 处**手工枚举：
 * 1. `AppStore.ts` 的 `ref()` 声明
 * 2. `AppStore.ts` 传给 `createAppPersistence({...})` 的对象
 * 3. `AppStore.ts` 的 `return {}`
 * 4. `persistence.ts` 的 `PersistenceState` 接口（53 个 `Ref<T>`）
 * 5. `persistence.ts` 的 `loadFromBackend()` 逐字段读
 * 6. `persistence.ts` 的 `saveToBackend()` 逐字段写
 * 7. `persistence.ts` 的 `watch([...])` 数组
 *
 * 也就是说**加一个 sing-box 设置项要改 7 个地方**，漏改任何一处都不会报错，只会在下次
 * 自动保存时把默认值悄悄写回数据库（原 `persistence.ts` 注释里就写明了这个风险）。
 *
 * 本表把「字段名 / 默认值 / 后端键名 / 特殊转换」集中声明一次，
 * `createAppPersistence` 的 load / save / watch 全部由本表推导生成。
 */

/**
 * 单个字段的声明。
 *
 * `V` 是该字段在前端的取值类型，约束为 `AppConfig[K]` 的子类型，
 * 因此可以比后端更精确（例如 `tun_stack` 收窄成 `'system' | 'gvisor' | 'mixed'`），
 * 但不能与之矛盾。
 */
export interface ConfigField<K extends keyof AppConfig = keyof AppConfig, V = AppConfig[K]> {
  /** 后端 `AppConfig` 中的字段名 */
  key: K
  /** 响应式引用（store 暴露的就是它） */
  ref: Ref<V>
  /** 写入后端时的转换；缺省为原值 */
  write?: (value: V) => AppConfig[K]
  /** 从后端值读取时的转换；缺省为「空值则沿用当前值」 */
  read?: (raw: AppConfig[K] | undefined, current: V) => V
}

/** 字段的读写转换选项 */
interface FieldOptions<K extends keyof AppConfig, V> {
  write?: (value: V) => AppConfig[K]
  read?: (raw: AppConfig[K] | undefined, current: V) => V
}

/**
 * 声明一个字段（取值类型 = 后端 `AppConfig[K]`）。
 *
 * `defaultValue` 的形参类型是 `AppConfig[K]` 而非泛型 `V`，这样 TS 不会把
 * `ref(false)` 推断成 `Ref<false>`（那会让该字段只能被赋回默认值）。
 */
function defineField<K extends keyof AppConfig>(
  key: K,
  defaultValue: AppConfig[K],
  options: FieldOptions<K, AppConfig[K]> = {},
): ConfigField<K> {
  return {
    key,
    ref: ref(defaultValue) as Ref<AppConfig[K]>,
    ...(options.write ? { write: options.write } : {}),
    ...(options.read ? { read: options.read } : {}),
  }
}

/**
 * 声明一个**窄类型**字段：前端取值类型比后端更精确。
 *
 * 用于后端用 `TEXT` 存储但前端取值受限的枚举（如 `tray_close_behavior`）。
 * 之所以要单独一个函数：`ref('hide')` 会被推断成 `Ref<'hide'>`，
 * 而 `defineField` 的形参会把它拓宽成 `string`，后者对 `TrayCloseBehavior` 这类
 * 联合类型来说会削弱调用处的检查。
 */
function defineNarrowField<K extends keyof AppConfig, V extends AppConfig[K]>(
  key: K,
  defaultValue: V,
  options: FieldOptions<K, V> = {},
): ConfigField<K, V> {
  return {
    key,
    ref: ref(defaultValue) as Ref<V>,
    ...(options.write ? { write: options.write } : {}),
    ...(options.read ? { read: options.read } : {}),
  }
}

/** 托盘关闭行为（后端存为 TEXT） */
export type TrayCloseBehavior = 'hide' | 'lightweight'

/** TUN 网络栈（后端存为 TEXT） */
export type TunStack = 'system' | 'gvisor' | 'mixed'

// --- 通用读取策略（参数取 `unknown` 以便与任意字段类型组合，避免泛型推断歧义） ---

/** 空值（undefined / null）则沿用当前值：兼容旧库缺列 */
const keepCurrent = <V>(raw: unknown, current: V): V =>
  raw === undefined || raw === null ? current : (raw as V)

/** 空字符串也视作未配置：旧库里常见的"字段存在但为空" */
const keepCurrentIfBlank = <V extends string>(raw: unknown, current: V): V =>
  typeof raw === 'string' && raw !== '' ? (raw as V) : current

/** 空字符串视作无值，返回 null */
const readOptionalString = (raw: unknown, current: string | null): string | null =>
  typeof raw === 'string' && raw !== '' ? raw : current

/**
 * 数组字段：后端给了数组就原样拷贝（**包括空数组** —— 空数组表示“用户清空了这几项”，
 * 必须生效），只有非数组（旧库缺列）才沿用当前值。
 */
const readStringList = (raw: unknown, current: string[]): string[] =>
  Array.isArray(raw) ? [...(raw as string[])] : current

/** 数组字段的读取变体：空数组归一化为 null（用于 tun_route_exclude_address） */
const readListOrCurrent = (raw: unknown, current: string[] | null): string[] | null =>
  Array.isArray(raw) && raw.length > 0 ? [...(raw as string[])] : current

/** 写入策略：空数组归一化为 `null`（与旧实现一致） */
const writeListOrNull = (value: string[] | null): string[] | null =>
  value && value.length > 0 ? [...value] : null

/**
 * 全部可持久化字段。
 *
 * 顺序即 UI 分组顺序；新增字段只需在此追加一行。
 */
export const APP_CONFIG_FIELDS = {
  // --- 托盘 / 启动 ---
  trayInstanceId: defineField('tray_instance_id', null as string | null, {
    read: readOptionalString,
  }),
  autoStartKernel: defineField('auto_start_kernel', true),
  autoStartApp: defineField('auto_start_app', false),
  autoHideToTrayOnAutostart: defineField('auto_hide_to_tray_on_autostart', true),
  trayCloseBehavior: defineNarrowField('tray_close_behavior', 'hide' as TrayCloseBehavior, {
    read: keepCurrentIfBlank,
  }),

  // --- 代理开关 / 模式 ---
  systemProxyEnabled: defineField('system_proxy_enabled', false),
  tunEnabled: defineField('tun_enabled', false),
  preferIpv6: defineField('prefer_ipv6', false),
  allowLanAccess: defineField('allow_lan_access', false),

  // --- 端口 ---
  proxyPort: defineField('proxy_port', 12080),
  apiPort: defineField('api_port', 12081),
  systemProxyBypass: defineField(
    'system_proxy_bypass',
    'localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*',
    { read: keepCurrentIfBlank },
  ),

  // --- TUN ---
  tunIpv4: defineField('tun_ipv4', '172.19.0.1/30', { read: keepCurrentIfBlank }),
  tunIpv6: defineField('tun_ipv6', 'fdfe:dcba:9876::1/126', { read: keepCurrentIfBlank }),
  tunMtu: defineField('tun_mtu', 1500),
  tunAutoRoute: defineField('tun_auto_route', true),
  tunStrictRoute: defineField('tun_strict_route', true),
  tunStack: defineNarrowField('tun_stack', 'mixed' as TunStack, {
    read: keepCurrentIfBlank,
  }),
  tunEnableIpv6: defineField('tun_enable_ipv6', false),
  tunRouteExcludeAddress: defineField('tun_route_exclude_address', null as string[] | null, {
    read: readListOrCurrent,
    write: writeListOrNull,
  }),

  // --- sing-box 配置生成（订阅模板） ---
  singboxDnsProxy: defineField('singbox_dns_proxy', 'https://1.1.1.1/dns-query', {
    read: keepCurrentIfBlank,
  }),
  singboxDnsCn: defineField('singbox_dns_cn', 'https://dns.alidns.com/dns-query', {
    read: keepCurrentIfBlank,
  }),
  singboxDnsResolver: defineField('singbox_dns_resolver', '114.114.114.114', {
    read: keepCurrentIfBlank,
  }),
  singboxUrltestUrl: defineField('singbox_urltest_url', 'http://cp.cloudflare.com/generate_204', {
    read: keepCurrentIfBlank,
  }),
  singboxDefaultProxyOutbound: defineField('singbox_default_proxy_outbound', 'manual', {
    read: keepCurrentIfBlank,
  }),
  singboxBlockAds: defineField('singbox_block_ads', true),
  singboxDownloadDetour: defineField('singbox_download_detour', 'direct', {
    read: keepCurrentIfBlank,
  }),
  singboxDnsHijack: defineField('singbox_dns_hijack', true),
  singboxFakeDnsEnabled: defineField('singbox_fake_dns_enabled', false),
  singboxFakeDnsIpv4Range: defineField('singbox_fake_dns_ipv4_range', '198.18.0.0/15', {
    read: keepCurrentIfBlank,
  }),
  // 上游 sing-box 1.14.4 起推荐 2001:2::/48（RFC 5180 基准测试段）；
  // fc00::/18 属于 ULA，会与企业内网真实 ULA 地址冲突。
  singboxFakeDnsIpv6Range: defineField('singbox_fake_dns_ipv6_range', '2001:2::/48', {
    read: keepCurrentIfBlank,
  }),
  singboxFakeDnsFilterMode: defineField('singbox_fake_dns_filter_mode', 'proxy_only', {
    read: keepCurrentIfBlank,
  }),
  singboxEnableAppGroups: defineField('singbox_enable_app_groups', true),
  tunSelfHealEnabled: defineField('tun_self_heal_enabled', true),
  tunSelfHealCooldownSecs: defineField('tun_self_heal_cooldown_secs', 90),

  // --- sing-box 1.14 新增 ---
  kernelUpdateTrack: defineField('kernel_update_track', 'stable', {
    read: keepCurrentIfBlank,
  }),
  // 乐观 DNS 缓存默认关闭：sing-box 的 `optimistic: true` 隐含 **3 天**陈旧窗口，
  // 被污染的解析结果会被钉住 3 天，表现为「后台跑久了网络就坏、只能重启内核」。
  singboxDnsOptimisticCache: defineField('singbox_dns_optimistic_cache', false),
  // 开启乐观缓存时的陈旧窗口上限（显式写出，避免落到内核隐含的 3 天）
  singboxDnsOptimisticTimeout: defineField('singbox_dns_optimistic_timeout', '30m', {
    read: keepCurrentIfBlank,
  }),
  singboxDnsTimeout: defineField('singbox_dns_timeout', '10s', { read: keepCurrentIfBlank }),
  // 未命中任何分流规则的域名用哪个解析器兜底：
  // 'cn' = 国内解析器（直连，不依赖代理）/ 'proxy' = 境外解析器（经代理）
  singboxDnsFinalStrategy: defineField('singbox_dns_final_strategy', 'cn', {
    read: keepCurrentIfBlank,
  }),
  singboxDnsUseMdns: defineField('singbox_dns_use_mdns', true),
  singboxEnableTlsSpoof: defineField('singbox_enable_tls_spoof', false),
  tunDnsMode: defineField('tun_dns_mode', 'hijack', { read: keepCurrentIfBlank }),
  tunIncludeMacs: defineField('tun_include_macs', [] as string[], {
    read: readStringList,
    write: (value) => [...value],
  }),
  tunExcludeMacs: defineField('tun_exclude_macs', [] as string[], {
    read: readStringList,
    write: (value) => [...value],
  }),
  hysteria2DisableChromeParrot: defineField('hysteria2_disable_chrome_parrot', false),
  hysteria2ObfsType: defineField('hysteria2_obfs_type', 'salamander', {
    read: keepCurrentIfBlank,
  }),
  clashMode: defineField('clash_mode', 'rule', { read: keepCurrentIfBlank }),
  enableWebDashboard: defineField('enable_web_dashboard', false),
  enableTailscaleEndpoint: defineField('enable_tailscale_endpoint', false),
  tailscaleRunSshServer: defineField('tailscale_run_ssh_server', false),
  tailscaleTaildropDirectory: defineField('tailscale_taildrop_directory', 'Taildrop', {
    read: keepCurrentIfBlank,
  }),

  // --- 运行时信息（只读回填，不参与 watch 之外的处理） ---
  activeConfigPath: defineField('active_config_path', null as string | null, {
    read: readOptionalString,
  }),
  installedKernelVersion: defineField('installed_kernel_version', null as string | null, {
    read: readOptionalString,
  }),
} as const

/** 字段集合（与 `APP_CONFIG_FIELDS` 同类型，便于泛型消费） */
export type AppConfigFields = typeof APP_CONFIG_FIELDS

/** 字段名联合 */
export type AppConfigFieldName = keyof AppConfigFields

/** 字段名列表（运行时） */
export const APP_CONFIG_FIELD_NAMES = Object.keys(APP_CONFIG_FIELDS) as AppConfigFieldName[]

/**
 * 创建一份独立的字段集合。
 *
 * 每个 Pinia store 实例只能有一份（否则 watch 会串扰），因此用工厂而不是直接导出常量。
 */
export function createAppConfigFields(): AppConfigFields {
  // 直接复用声明表即可：ref 在模块加载时创建一次，而 AppStore 是单例 store，
  // 整个应用生命周期内只会初始化一次。
  return APP_CONFIG_FIELDS
}

/** 把后端 `AppConfig` 读入字段集合 */
export function applyAppConfigToFields(
  fields: AppConfigFields,
  appConfig: Partial<AppConfig>,
): void {
  for (const name of APP_CONFIG_FIELD_NAMES) {
    const field = fields[name] as ConfigField
    const raw = appConfig[field.key] as never
    const current = field.ref.value as never
    if (field.read) {
      field.ref.value = field.read(raw, current)
    } else {
      field.ref.value = keepCurrent(raw, current)
    }
  }
}

/** 从字段集合组装出后端 `AppConfig`（不含派生的 `proxy_mode`） */
export function collectAppConfigFromFields(fields: AppConfigFields): Partial<AppConfig> {
  const result: Record<string, unknown> = {}
  for (const name of APP_CONFIG_FIELD_NAMES) {
    const field = fields[name] as ConfigField
    const value = field.ref.value as never
    result[field.key] = field.write ? field.write(value) : value
  }
  return result as Partial<AppConfig>
}

/** 字段对应的 ref 列表（供 `watch` 使用） */
export function appConfigFieldRefs(fields: AppConfigFields): Ref<unknown>[] {
  return APP_CONFIG_FIELD_NAMES.map((name) => (fields[name] as ConfigField).ref as Ref<unknown>)
}

/** 字段名映射（`proxy_mode` 由两个独立开关派生，不在声明表中） */
export type DerivedAppConfigField = 'proxy_mode'
/**
 * 取出「字段名 -> ref」的映射，供 store 解构使用。
 *
 * `fields.x` 是声明对象，而 store 需要的是响应式引用本身；
 * 这里一次性做映射，store 里就能保持 `const { tunEnabled, ... } = appConfigRefs()`
 * 的可读写法（与重构前一致），无需到处写 `.ref`。
 */
export function appConfigRefs(fields: AppConfigFields): {
  [K in AppConfigFieldName]: AppConfigFields[K]['ref']
} {
  const result = {} as Record<AppConfigFieldName, Ref<unknown>>
  for (const name of APP_CONFIG_FIELD_NAMES) {
    result[name] = (fields[name] as ConfigField).ref as Ref<unknown>
  }
  return result as ReturnType<typeof appConfigRefs>
}
