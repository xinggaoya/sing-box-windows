import { existsSync, readdirSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  ALL_COUNTRY_CODES,
  COUNTRY_CODE_ALIASES,
  COUNTRY_NAME_PATTERNS,
  INFORMATIONAL_NODE_PATTERNS,
  ISO_COUNTRY_CODES,
  PROXY_PREFIX_COUNTRY_CODES,
  REGION_COUNTRY_CODES,
} from '@/constants/countries'

/**
 * `flag-icons` 国家码同步守护。
 *
 * 原先把 258 行国家码硬编码在 `ProxyView.vue`，且已与包实际提供的文件漂移：
 * 包里多出 ARAB / ASEAN / CEFTA / GB-ENG 等码（视图没有），
 * 视图里又有包已移除的 AC / EA / TA —— 表现为部分节点的国旗图标静默不显示。
 *
 * 国家码现由 `scripts/gen-country-codes.mjs` 从包生成，这里断言二者同步，
 * 使「升级 flag-icons 后忘记重新生成」在 CI 立刻失败而不是悄悄出错。
 */

const FLAGS_DIR = join(process.cwd(), 'node_modules', 'flag-icons', 'flags', '4x3')

function packageCountryCodes(): string[] {
  if (!existsSync(FLAGS_DIR)) return []
  return readdirSync(FLAGS_DIR)
    .filter((name) => name.endsWith('.svg'))
    .map((name) => name.replace('.svg', '').toUpperCase())
    .sort()
}

describe('国家码与 flag-icons 同步', () => {
  const packageCodes = packageCountryCodes()

  it('flag-icons 包存在（否则下面的断言无意义）', () => {
    expect(packageCodes.length).toBeGreaterThan(200)
  })

  it('源码里的国家码与包完全一致', () => {
    expect([...ALL_COUNTRY_CODES].sort()).toEqual(packageCodes)
  })

  it('没有包已移除的废弃码', () => {
    const removed = ['AC', 'EA', 'TA']
    for (const code of removed) {
      expect(ALL_COUNTRY_CODES, `包已移除 ${code}`).not.toContain(code)
    }
  })

  it('包含此前漏掉的地区码与子区域码', () => {
    // 这些码在 flag-icons 里存在，但旧的硬编码列表漏了
    for (const code of ['ARAB', 'ASEAN', 'CEFTA', 'EAC', 'GB-ENG', 'GB-SCT']) {
      expect(ALL_COUNTRY_CODES, `缺少 ${code}`).toContain(code)
    }
  })
})

describe('国家码集合的一致性', () => {
  it('ISO 与地区码互为补集', () => {
    for (const code of ALL_COUNTRY_CODES) {
      const inIso = ISO_COUNTRY_CODES.has(code)
      const inRegion = REGION_COUNTRY_CODES.has(code)
      expect(inIso, code).not.toBe(inRegion)
    }
    expect(ISO_COUNTRY_CODES.size + REGION_COUNTRY_CODES.size).toBe(ALL_COUNTRY_CODES.length)
  })

  it('ISO 集合内全部是两位大写字母', () => {
    for (const code of ISO_COUNTRY_CODES) {
      expect(code, code).toMatch(/^[A-Z]{2}$/)
    }
  })

  it('国家码无重复', () => {
    expect(new Set(ALL_COUNTRY_CODES).size).toBe(ALL_COUNTRY_CODES.length)
  })
})

describe('别名表', () => {
  it('每个别名的目标都是合法 ISO 码', () => {
    for (const [alias, target] of Object.entries(COUNTRY_CODE_ALIASES)) {
      expect(alias.length, alias).toBe(2)
      expect(ISO_COUNTRY_CODES.has(target), `${alias} -> ${target}`).toBe(true)
    }
  })
})

describe('国名反查表', () => {
  it('每条规则的推断码都是合法 ISO 码', () => {
    for (const [pattern, code] of COUNTRY_NAME_PATTERNS) {
      expect(ISO_COUNTRY_CODES.has(code), `${pattern} -> ${code}`).toBe(true)
    }
  })

  it('推断码不重复（同一码不写两条正则，否则后者永远不生效）', () => {
    const codes = COUNTRY_NAME_PATTERNS.map(([, code]) => code)
    expect(new Set(codes).size).toBe(codes.length)
  })

  it('含英文写法的规则必须 case-insensitive', () => {
    // 纯中文规则（东京 / 台湾 / 澳洲）没有英文部分，不需要 i 标志
    const needsIgnoreCase = COUNTRY_NAME_PATTERNS.filter(([pattern]) =>
      /[a-z]/i.test(pattern.source),
    )
    expect(needsIgnoreCase.length).toBeGreaterThan(0)
    for (const [pattern] of needsIgnoreCase) {
      expect(pattern.flags, pattern.source).toContain('i')
    }
  })

  it('实际能匹配中英文写法', () => {
    const find = (name: string) =>
      COUNTRY_NAME_PATTERNS.find(([pattern]) => pattern.test(name))?.[1]

    expect(find('香港 01')).toBe('HK')
    expect(find('Hong Kong 01')).toBe('HK')
    expect(find('日本 东京 01')).toBe('JP')
    expect(find('United States 01')).toBe('US')
  })
})

describe('信息节点识别', () => {
  it('能识别机场订阅的提示条目', () => {
    const isInfo = (name: string) =>
      INFORMATIONAL_NODE_PATTERNS.some((pattern) => pattern.test(name))

    expect(isInfo('剩余流量：1024GB')).toBe(true)
    expect(isInfo('套餐到期：2027-01-01')).toBe(true)
    expect(isInfo('官网：https://example.com')).toBe(true)
    expect(isInfo('距离下次重置剩余：30天')).toBe(true)
  })

  it('正常节点名不会被误判', () => {
    const isInfo = (name: string) =>
      INFORMATIONAL_NODE_PATTERNS.some((pattern) => pattern.test(name))

    expect(isInfo('🇭🇰 香港 01')).toBe(false)
    expect(isInfo('US Los Angeles Premium')).toBe(false)
  })
})

describe('协议前缀码', () => {
  it('包含容易被误认成国家码的协议前缀', () => {
    for (const code of ['CF', 'CL', 'SS']) {
      expect(PROXY_PREFIX_COUNTRY_CODES.has(code), code).toBe(true)
    }
  })
})
