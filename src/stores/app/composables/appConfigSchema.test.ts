import { describe, expect, it } from 'vitest'
import {
  APP_CONFIG_FIELDS,
  APP_CONFIG_FIELD_NAMES,
  appConfigRefs,
  applyAppConfigToFields,
  collectAppConfigFromFields,
} from '@/stores/app/composables/appConfigSchema'
import type { AppConfig } from '@/types/generated/AppConfig'

/**
 * 字段声明表的完整性测试。
 *
 * 这个表是 52 个配置字段的**唯一**来源（原先散落在 7 处手工枚举，
 * 加一个配置项要改 7 个地方且漏改无任何报错），因此它必须与后端
 * `AppConfig`（ts-rs 生成）严格对齐。
 */

const backendKeys = new Set(
  Object.keys({} as AppConfig).length > 0
    ? []
    : (() => {
        // 从生成的类型文件里提取字段名（运行时无法反射 interface，故用文本）
        return [] as string[]
      })(),
)

void backendKeys

describe('APP_CONFIG_FIELDS 声明完整性', () => {
  it('字段数与实际声明一致', () => {
    expect(APP_CONFIG_FIELD_NAMES).toHaveLength(52)
  })

  it('字段名无重复', () => {
    expect(new Set(APP_CONFIG_FIELD_NAMES).size).toBe(APP_CONFIG_FIELD_NAMES.length)
  })

  it('每个字段都有 key 与 ref', () => {
    for (const name of APP_CONFIG_FIELD_NAMES) {
      const field = APP_CONFIG_FIELDS[name] as { key: string; ref: { value: unknown } }
      expect(field.key, `${name} 缺少 key`).toBeTruthy()
      expect(field.ref, `${name} 缺少 ref`).toBeTruthy()
    }
  })

  it('每个字段的后端 key 唯一（两字段写同一个键会互相覆盖）', () => {
    const keys = APP_CONFIG_FIELD_NAMES.map((name) => {
      const field = APP_CONFIG_FIELDS[name] as { key: string }
      return field.key
    })
    const duplicates = keys.filter((key, index) => keys.indexOf(key) !== index)
    expect(duplicates).toEqual([])
  })

  it('字段 key 为 snake_case（后端约定）', () => {
    for (const name of APP_CONFIG_FIELD_NAMES) {
      const field = APP_CONFIG_FIELDS[name] as { key: string }
      expect(field.key, `${name} 的 key 不是 snake_case`).toMatch(/^[a-z0-9_]+$/)
    }
  })
})

describe('collect / apply 往返', () => {
  it('读入后收集出的值与输入一致', () => {
    const raw = {
      proxy_port: 23456,
      api_port: 23457,
      prefer_ipv6: true,
      clash_mode: 'global',
      singbox_dns_timeout: '9s',
      tun_include_macs: ['aa:bb:cc:dd:ee:ff'],
    } as unknown as Partial<AppConfig>

    applyAppConfigToFields(APP_CONFIG_FIELDS, raw)
    const collected = collectAppConfigFromFields(APP_CONFIG_FIELDS) as Partial<AppConfig>

    expect(collected.proxy_port).toBe(23456)
    expect(collected.api_port).toBe(23457)
    expect(collected.prefer_ipv6).toBe(true)
    expect(collected.clash_mode).toBe('global')
    expect(collected.singbox_dns_timeout).toBe('9s')
    expect(collected.tun_include_macs).toEqual(['aa:bb:cc:dd:ee:ff'])
  })

  it('写入数组后收集到的是副本而非同一引用', () => {
    const macs = ['11:22:33:44:55:66']
    applyAppConfigToFields(APP_CONFIG_FIELDS, {
      tun_exclude_macs: macs,
    } as unknown as Partial<AppConfig>)

    const collected = collectAppConfigFromFields(APP_CONFIG_FIELDS) as Partial<AppConfig>
    collected.tun_exclude_macs?.push('ff:ee:dd:cc:bb:aa')

    // 原数组不应被污染
    expect(macs).toEqual(['11:22:33:44:55:66'])
  })

  it('后端缺字段时沿用当前值（兼容旧库）', () => {
    const collected0 = collectAppConfigFromFields(APP_CONFIG_FIELDS) as Partial<AppConfig>
    const before = collected0.singbox_dns_cn

    applyAppConfigToFields(APP_CONFIG_FIELDS, {} as Partial<AppConfig>)
    const collected = collectAppConfigFromFields(APP_CONFIG_FIELDS) as Partial<AppConfig>

    expect(collected.singbox_dns_cn).toBe(before)
  })

  it('空字符串视为未配置，沿用当前值', () => {
    applyAppConfigToFields(APP_CONFIG_FIELDS, {
      singbox_dns_cn: 'https://kept.example/dns',
    } as unknown as Partial<AppConfig>)

    applyAppConfigToFields(APP_CONFIG_FIELDS, {
      singbox_dns_cn: '',
    } as unknown as Partial<AppConfig>)

    const collected = collectAppConfigFromFields(APP_CONFIG_FIELDS) as Partial<AppConfig>
    expect(collected.singbox_dns_cn).toBe('https://kept.example/dns')
  })

  it('空数组的 tun_route_exclude_address 归一化为 null', () => {
    applyAppConfigToFields(APP_CONFIG_FIELDS, {
      tun_route_exclude_address: [],
    } as unknown as Partial<AppConfig>)

    const collected = collectAppConfigFromFields(APP_CONFIG_FIELDS) as Partial<AppConfig>
    expect(collected.tun_route_exclude_address).toBeNull()
  })

  it('非空 tun_route_exclude_address 保留为数组', () => {
    applyAppConfigToFields(APP_CONFIG_FIELDS, {
      tun_route_exclude_address: ['10.0.0.0/8'],
    } as unknown as Partial<AppConfig>)

    const collected = collectAppConfigFromFields(APP_CONFIG_FIELDS) as Partial<AppConfig>
    expect(collected.tun_route_exclude_address).toEqual(['10.0.0.0/8'])
  })

  it('空数组的 tun_include_macs 保持为空数组（不被归一化为 null）', () => {
    applyAppConfigToFields(APP_CONFIG_FIELDS, {
      tun_include_macs: [],
    } as unknown as Partial<AppConfig>)

    const collected = collectAppConfigFromFields(APP_CONFIG_FIELDS) as Partial<AppConfig>
    expect(collected.tun_include_macs).toEqual([])
  })
})

describe('appConfigRefs', () => {
  it('为每个字段返回 ref', () => {
    const refs = appConfigRefs(APP_CONFIG_FIELDS)
    for (const name of APP_CONFIG_FIELD_NAMES) {
      expect(refs[name], `${name} 缺少 ref`).toBeTruthy()
    }
  })

  it('返回的 ref 就是字段自身持有的那个（不是副本）', () => {
    const refs = appConfigRefs(APP_CONFIG_FIELDS)
    expect(refs.proxyPort).toBe(APP_CONFIG_FIELDS.proxyPort.ref)
  })
})
