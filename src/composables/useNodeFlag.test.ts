import { describe, expect, it } from 'vitest'
import {
  getDisplayNodeName,
  getFlagIconCountryCode,
  getNodeCountryCode,
  isInformationalNodeName,
  normalizeCountryLabel,
  stripLeadingFlagEmoji,
} from '@/composables/useNodeFlag'

/**
 * 节点名 → 国家码的解析规则。
 *
 * 机场订阅的节点名极不统一，这段解析决定了「国旗显示对不对」：
 * 识别错会让用户看到另一个国家的国旗（比不显示更糟）。
 * 因此这里覆盖各种真实命名风格，以及必须返回空串的负例。
 */

describe('stripLeadingFlagEmoji', () => {
  it('剥离前缀国旗 emoji 与其后空格', () => {
    expect(stripLeadingFlagEmoji('🇭🇰 香港 01')).toBe('香港 01')
    expect(stripLeadingFlagEmoji('🇺🇸US Premium')).toBe('US Premium')
  })

  it('无 emoji 时原样返回（仅 trim）', () => {
    expect(stripLeadingFlagEmoji('  HK-01  ')).toBe('HK-01')
  })

  it('只剥离开头的一组 emoji', () => {
    expect(stripLeadingFlagEmoji('🇯🇵🇭🇰 双区')).toBe('🇭🇰 双区')
  })
})

describe('getFlagIconCountryCode', () => {
  it('大写码转小写', () => {
    expect(getFlagIconCountryCode('HK')).toBe('hk')
  })

  it('EU 特判', () => {
    expect(getFlagIconCountryCode('EU')).toBe('eu')
  })

  it('应用别名表', () => {
    expect(getFlagIconCountryCode('UK')).toBe('gb')
  })

  it('非法码返回空串', () => {
    expect(getFlagIconCountryCode('X')).toBe('')
    expect(getFlagIconCountryCode('HKG')).toBe('')
    expect(getFlagIconCountryCode('')).toBe('')
  })
})

describe('isInformationalNodeName', () => {
  it('识别机场订阅的信息条目', () => {
    expect(isInformationalNodeName('剩余流量：1024GB')).toBe(true)
    expect(isInformationalNodeName('距离下次重置剩余：30天')).toBe(true)
    expect(isInformationalNodeName('套餐到期：2027-01-01')).toBe(true)
    expect(isInformationalNodeName('官网：https://e.com')).toBe(true)
  })

  it('带前缀 emoji 的信息条目也能识别', () => {
    expect(isInformationalNodeName('🇭🇰 剩余流量：1TB')).toBe(true)
  })

  it('正常节点名不会误判', () => {
    expect(isInformationalNodeName('🇭🇰 香港 01')).toBe(false)
    expect(isInformationalNodeName('US Los Angeles')).toBe(false)
  })
})

describe('normalizeCountryLabel', () => {
  it('去掉空白与常见标点', () => {
    expect(normalizeCountryLabel('Hong Kong')).toBe('hongkong')
    expect(normalizeCountryLabel('美国（洛杉矶）')).toBe('美国洛杉矶')
  })

  it('转小写', () => {
    expect(normalizeCountryLabel('HK')).toBe('hk')
  })
})

describe('getNodeCountryCode', () => {
  it('本地化国名（中文）', () => {
    expect(getNodeCountryCode('香港 01')).toBe('hk')
    expect(getNodeCountryCode('日本 东京 02')).toBe('jp')
    expect(getNodeCountryCode('美国 洛杉矶')).toBe('us')
  })

  it('本地化国名（英文）', () => {
    expect(getNodeCountryCode('Hong Kong 01')).toBe('hk')
    expect(getNodeCountryCode('United States Premium')).toBe('us')
  })

  it('裸两字母 token', () => {
    expect(getNodeCountryCode('HK-01')).toBe('hk')
    expect(getNodeCountryCode('US | Los Angeles')).toBe('us')
    expect(getNodeCountryCode('SG 03')).toBe('sg')
  })

  it('跳过协议前缀（CF / CL / SS 会被误认成国家码）', () => {
    // CF 在 flag-icons 里是「中非共和国」，CL 是「智利」，SS 是「南苏丹」
    expect(getNodeCountryCode('CF-日本 IEPL')).toBe('jp')
    expect(getNodeCountryCode('SS 香港 01')).toBe('hk')
  })

  it('EU 特判（无地名时的裸 token）', () => {
    expect(getNodeCountryCode('EU 01')).toBe('eu')
  })

  it('地名层优先于裸 token 层：「EU 荷兰」是荷兰节点走欧盟中转，不是欧盟节点', () => {
    expect(getNodeCountryCode('EU 荷兰 01')).toBe('nl')
    expect(getNodeCountryCode('DE 荷兰 01')).toBe('nl')
  })

  it('信息条目返回空串（不显示国旗）', () => {
    expect(getNodeCountryCode('剩余流量：1024GB')).toBe('')
    expect(getNodeCountryCode('官网：https://e.com')).toBe('')
  })

  it('无法识别时返回空串', () => {
    expect(getNodeCountryCode('神秘节点')).toBe('')
    expect(getNodeCountryCode('')).toBe('')
    expect(getNodeCountryCode('Premium Node')).toBe('')
  })

  it('带前缀 emoji 的节点名同样能解析', () => {
    expect(getNodeCountryCode('🇭🇰 香港 01')).toBe('hk')
    expect(getNodeCountryCode('🇯🇵 东京 01')).toBe('jp')
  })

  it('优先用更具体的地名规则，而非国名子串', () => {
    // 「香港」应判 HK 而不是被「中国/中国香港」之类的更短标签抢先
    expect(getNodeCountryCode('香港')).toBe('hk')
  })
})

describe('getDisplayNodeName', () => {
  it('剥离前缀 emoji', () => {
    expect(getDisplayNodeName('🇭🇰 香港 01')).toBe('香港 01')
  })

  it('空值返回占位符', () => {
    expect(getDisplayNodeName(null)).toBe('-')
    expect(getDisplayNodeName(undefined)).toBe('-')
    expect(getDisplayNodeName('')).toBe('-')
  })
})
