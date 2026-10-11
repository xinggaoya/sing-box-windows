import { computed, type ComputedRef, type Ref } from 'vue'

/**
 * 「排序 + 分组」列表的通用计算逻辑
 *
 * 背景：日志页与连接页的可排序/可分组列表结构完全同构，但此前各写了一份
 * `sortedXxx` / `groupedXxx` computed，comparator 主体逐字相同（数值走减法、
 * 其余走 `localeCompare`，统一乘 `desc ? -1 : 1`）。两份实现一旦要改（比如处理
 * `null`、统一空值位置）就必须改两处，且极易只改一处。
 *
 * 这里抽为唯一实现，两个页面各自只提供「取值函数」与「分组键函数」。
 */

/** 排序/分组的持久化状态（由 store 提供） */
export interface SortGroupState<K extends string> {
  /** 排序字段 */
  sortKey: Ref<K> | ComputedRef<K>
  /** 是否降序 */
  sortDesc: Ref<boolean> | ComputedRef<boolean>
  /** 分组字段；空串表示不分组 */
  groupingKey: Ref<K | ''> | ComputedRef<K | ''>
}

export interface SortGroupOptions<T, K extends string> {
  /** 已过滤的列表 */
  source: ComputedRef<readonly T[]>
  state: SortGroupState<K>
  /** 取出参与比较的值；返回 `number` 走数值比较，其余按本地化字符串比较 */
  getSortValue: (item: T, key: K) => number | string | null | undefined
  /** 取出分组键；空值会统一归到 `emptyGroupLabel` */
  getGroupKey: (item: T, key: K) => string | null | undefined
  /** 分组值为空时使用的标签 */
  emptyGroupLabel?: string
}

export interface SortGroupResult<T> {
  sorted: ComputedRef<T[]>
  groups: ComputedRef<GroupedItem<T>[]>
  /** 不分组时的扁平列表（等价于 `sorted`） */
  flat: ComputedRef<T[]>
}

export interface GroupedItem<T> {
  key: string
  count: number
  items: T[]
}

export function useSortableGroupedList<T, K extends string>(
  options: SortGroupOptions<T, K>,
): SortGroupResult<T> {
  const { source, state, getSortValue, getGroupKey, emptyGroupLabel = '-' } = options

  const sorted = computed<T[]>(() => {
    const list = [...source.value]
    const factor = state.sortDesc.value ? -1 : 1
    const key = state.sortKey.value

    return list.sort((left, right) => {
      const leftValue = getSortValue(left, key)
      const rightValue = getSortValue(right, key)

      // 数值比较
      if (typeof leftValue === 'number' && typeof rightValue === 'number') {
        return (leftValue - rightValue) * factor
      }

      // 混合类型（如 start 是数字、host 是字符串）：统一转字符串
      return String(leftValue).localeCompare(String(rightValue)) * factor
    })
  })

  const groups = computed<GroupedItem<T>[]>(() => {
    const grouping = state.groupingKey.value
    if (!grouping) {
      return [{ key: '', count: sorted.value.length, items: sorted.value }]
    }

    const buckets = new Map<string, T[]>()
    for (const item of sorted.value) {
      const raw = getGroupKey(item, grouping)
      const key = raw ? String(raw) : emptyGroupLabel
      const bucket = buckets.get(key)
      if (bucket) {
        bucket.push(item)
      } else {
        buckets.set(key, [item])
      }
    }

    return Array.from(buckets.entries()).map(([key, items]) => ({
      key,
      count: items.length,
      items,
    }))
  })

  return { sorted, groups, flat: sorted }
}
