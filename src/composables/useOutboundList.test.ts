import { describe, it, expect, beforeEach, vi } from 'vitest'
import {
  useOutboundList,
  formatOutboundDelay,
  sortOutboundsByDelay,
} from '@/composables/useOutboundList'
import type { GroupItem } from '@/types/generated/GroupItem'

const getOutbounds = vi.fn()

vi.mock('@/services/proxy-service', () => ({
  proxyService: {
    getOutbounds: () => getOutbounds(),
  },
}))

const item = (tag: string, type: string, delay: number): GroupItem => ({
  tag,
  type,
  urlTestTime: 0,
  urlTestDelay: delay,
})

describe('useOutboundList', () => {
  beforeEach(() => {
    getOutbounds.mockReset()
  })

  it('按延迟升序排序，未测速（0）的排到最后', () => {
    const sorted = sortOutboundsByDelay([
      item('慢', 'vless', 900),
      item('未测速', 'vmess', 0),
      item('快', 'trojan', 120),
    ])

    expect(sorted.map((o) => o.tag)).toEqual(['快', '慢', '未测速'])
  })

  it('延迟展示：未测速显示占位符，已测速显示毫秒', () => {
    expect(formatOutboundDelay(0)).toBe('—')
    expect(formatOutboundDelay(-1)).toBe('—')
    expect(formatOutboundDelay(230)).toBe('230ms')
  })

  it('搜索按名称或类型过滤（大小写不敏感）', async () => {
    getOutbounds.mockResolvedValue({
      outbounds: [
        item('纽约', 'hysteria2', 230),
        item('香港 01', 'vless', 180),
        item('Telegram', 'selector', 0),
      ],
    })

    const { filtered, searchQuery, refresh, loading } = useOutboundList()
    await refresh()

    expect(loading.value).toBe(false)
    expect(filtered.value).toHaveLength(3)

    searchQuery.value = 'vless'
    expect(filtered.value.map((o) => o.tag)).toEqual(['香港 01'])

    searchQuery.value = 'telegram'
    expect(filtered.value.map((o) => o.tag)).toEqual(['Telegram'])
  })

  it('统计：总数 / 已测速数 / 最快 / 最慢', async () => {
    getOutbounds.mockResolvedValue({
      outbounds: [item('a', 'vless', 300), item('b', 'trojan', 120), item('c', 'vmess', 0)],
    })

    const { stats, refresh } = useOutboundList()
    await refresh()

    expect(stats.value.total).toBe(3)
    expect(stats.value.measured).toBe(2)
    expect(stats.value.fastest).toBe(120)
    expect(stats.value.slowest).toBe(300)
  })

  it('取数失败时清空列表而不是抛错', async () => {
    getOutbounds.mockRejectedValue(new Error('boom'))

    const { outbounds, error, loading, refresh } = useOutboundList()
    await refresh()

    expect(loading.value).toBe(false)
    expect(outbounds.value).toEqual([])
    expect(error.value).toContain('boom')
  })
})
