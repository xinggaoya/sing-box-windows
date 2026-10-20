import { computed, ref } from 'vue'
import { proxyService } from '@/services/proxy-service'
import type { GroupItem } from '@/types/generated/GroupItem'

/**
 * 出站总览的数据逻辑（内核 GetOutbounds）。
 *
 * 独立成 composable 的原因：
 * - 「出站总览」已经独立成页面（`views/OutboundsView.vue`），**不再占用代理页任何空间**；
 * - 排序 / 过滤 / 统计是纯函数，可以脱离组件单测；
 * - 页面只负责渲染，不掺杂取数细节。
 */

/** 延迟展示：未测速（<= 0）显示占位符 */
export const formatOutboundDelay = (delay: number): string =>
  !delay || delay <= 0 ? '—' : `${delay}ms`

/** 按延迟升序排序；未测速（0）的排到最后 */
export const sortOutboundsByDelay = (list: GroupItem[]): GroupItem[] =>
  [...list].sort((a, b) => {
    const av = a.urlTestDelay > 0 ? a.urlTestDelay : Number.MAX_SAFE_INTEGER
    const bv = b.urlTestDelay > 0 ? b.urlTestDelay : Number.MAX_SAFE_INTEGER
    return av - bv
  })

export interface OutboundStats {
  /** 出站总数 */
  total: number
  /** 已测速（延迟 > 0）的出站数 */
  measured: number
  /** 最快延迟（ms），无有效数据时为 0 */
  fastest: number
  /** 最慢延迟（ms），无有效数据时为 0 */
  slowest: number
}

export function useOutboundList() {
  const outbounds = ref<GroupItem[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  const searchQuery = ref('')

  /** 按延迟升序 */
  const sorted = computed(() => sortOutboundsByDelay(outbounds.value))

  /** 搜索过滤（出站名称或类型，大小写不敏感） */
  const filtered = computed(() => {
    const query = searchQuery.value.trim().toLowerCase()
    if (!query) return sorted.value
    return sorted.value.filter(
      (item) => item.tag.toLowerCase().includes(query) || item.type.toLowerCase().includes(query),
    )
  })

  const stats = computed<OutboundStats>(() => {
    const delays = outbounds.value.map((o) => o.urlTestDelay).filter((d) => d > 0)
    return {
      total: outbounds.value.length,
      measured: delays.length,
      fastest: delays.length ? Math.min(...delays) : 0,
      slowest: delays.length ? Math.max(...delays) : 0,
    }
  })

  const refresh = async () => {
    if (loading.value) return
    loading.value = true
    error.value = null
    try {
      const result = await proxyService.getOutbounds()
      outbounds.value = result?.outbounds ?? []
    } catch (e) {
      console.warn('[useOutboundList] get_outbounds failed:', e)
      outbounds.value = []
      error.value = e instanceof Error ? e.message : String(e)
    } finally {
      loading.value = false
    }
  }

  return { outbounds, loading, error, searchQuery, sorted, filtered, stats, refresh }
}
