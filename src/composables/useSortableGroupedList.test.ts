import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, h, nextTick } from 'vue'
import { useSortableGroupedList } from '@/composables/useSortableGroupedList'
import { computed, ref } from 'vue'

/**
 * 日志页与连接页此前各写了一份逐字相同的排序/分组 computed，
 * comparator 主体完全一致；任何一次「处理 null / 统一空值位置」的改动都必须改两处。
 * 抽成 composable 后，这里把行为钉死。
 */

interface Row {
  seq: number
  type: string
  timestamp: number
  host: string
  latency?: number | null
}

const rows: Row[] = [
  { seq: 3, type: 'warn', timestamp: 3000, host: 'b.example' },
  { seq: 1, type: 'info', timestamp: 1000, host: 'a.example' },
  { seq: 2, type: 'error', timestamp: 2000, host: 'c.example' },
]

function setup(
  overrides: Partial<{
    sortKey: 'seq' | 'type' | 'host'
    sortDesc: boolean
    groupingKey: 'type' | 'host' | ''
  }> = {},
) {
  const sortKey = ref(overrides.sortKey ?? 'seq')
  const sortDesc = ref(overrides.sortDesc ?? false)
  const groupingKey = ref(overrides.groupingKey ?? '')

  const result = useSortableGroupedList<Row, 'seq' | 'type' | 'host'>({
    source: computed(() => rows),
    state: { sortKey, sortDesc, groupingKey },
    getSortValue: (row, key) => row[key],
    getGroupKey: (row, key) => String(row[key]),
  })

  return { ...result, sortKey, sortDesc, groupingKey }
}

describe('useSortableGroupedList - 排序', () => {
  it('默认按给定字段升序', () => {
    const { sorted } = setup({ sortKey: 'seq', sortDesc: false })
    expect(sorted.value.map((r) => r.seq)).toEqual([1, 2, 3])
  })

  it('sortDesc 翻转顺序', () => {
    const { sorted } = setup({ sortKey: 'seq', sortDesc: true })
    expect(sorted.value.map((r) => r.seq)).toEqual([3, 2, 1])
  })

  it('字符串字段走本地化字符串比较', () => {
    const { sorted } = setup({ sortKey: 'host', sortDesc: false })
    expect(sorted.value.map((r) => r.host)).toEqual(['a.example', 'b.example', 'c.example'])
  })

  it('切换 sortDesc 后顺序立即翻转', async () => {
    const { sorted, sortDesc } = setup({ sortKey: 'seq', sortDesc: false })
    expect(sorted.value.map((r) => r.seq)).toEqual([1, 2, 3])

    sortDesc.value = true
    await nextTick()
    expect(sorted.value.map((r) => r.seq)).toEqual([3, 2, 1])
  })

  it('切换 sortKey 后按新字段排序', async () => {
    const { sorted, sortKey } = setup({ sortKey: 'seq' })
    expect(sorted.value[0].seq).toBe(1)

    sortKey.value = 'host'
    await nextTick()
    expect(sorted.value[0].host).toBe('a.example')
  })

  it('不修改源数组（保持 computed 幂等）', () => {
    const { sorted } = setup({ sortKey: 'seq', sortDesc: true })
    // 反复求值不应污染源数组，否则第二次读取会得到不同结果
    void sorted.value
    void sorted.value
    expect(rows.map((r) => r.seq)).toEqual([3, 1, 2])
  })
})

describe('useSortableGroupedList - 分组', () => {
  it('groupingKey 为空时返回单个未分组项', () => {
    const { groups } = setup({ groupingKey: '' })
    expect(groups.value).toHaveLength(1)
    expect(groups.value[0].key).toBe('')
    expect(groups.value[0].count).toBe(3)
  })

  it('按字段分组并给出条数', () => {
    const { groups } = setup({ groupingKey: 'type' })
    const byKey = Object.fromEntries(groups.value.map((g) => [g.key, g.count]))
    expect(byKey).toEqual({ warn: 1, info: 1, error: 1 })
  })

  it('同组内保持排序后的顺序', () => {
    const many: Row[] = [
      { seq: 1, type: 'a', timestamp: 1, host: 'h1' },
      { seq: 2, type: 'a', timestamp: 2, host: 'h2' },
      { seq: 3, type: 'a', timestamp: 3, host: 'h3' },
    ]
    const sortKey = ref<'seq' | 'type' | 'host'>('seq')
    const sortDesc = ref(false)
    const groupingKey = ref<'type' | 'host' | ''>('type')

    const { groups } = useSortableGroupedList<Row, 'seq' | 'type' | 'host'>({
      source: computed(() => many),
      state: { sortKey, sortDesc, groupingKey },
      getSortValue: (row, key) => row[key],
      getGroupKey: (row, key) => String(row[key]),
    })

    expect(groups.value).toHaveLength(1)
    expect(groups.value[0].items.map((r) => r.seq)).toEqual([1, 2, 3])
  })

  it('空分组值统一归到 emptyGroupLabel', () => {
    const withEmpty: Row[] = [
      { seq: 1, type: '', timestamp: 1, host: 'h1' },
      { seq: 2, type: 'a', timestamp: 2, host: 'h2' },
    ]
    const sortKey = ref<'seq' | 'type' | 'host'>('seq')
    const sortDesc = ref(false)
    const groupingKey = ref<'type' | 'host' | ''>('type')

    const { groups } = useSortableGroupedList<Row, 'seq' | 'type' | 'host'>({
      source: computed(() => withEmpty),
      state: { sortKey, sortDesc, groupingKey },
      getSortValue: (row, key) => row[key],
      getGroupKey: (row, key) => String(row[key]),
      emptyGroupLabel: '未知',
    })

    const keys = groups.value.map((g) => g.key).sort()
    expect(keys).toEqual(['a', '未知'])
  })

  it('自定义 emptyGroupLabel 生效', () => {
    const withEmpty: Row[] = [{ seq: 1, type: '', timestamp: 1, host: 'h1' }]
    const sortKey = ref<'seq' | 'type' | 'host'>('seq')
    const sortDesc = ref(false)
    const groupingKey = ref<'type' | 'host' | ''>('type')

    const { groups } = useSortableGroupedList<Row, 'seq' | 'type' | 'host'>({
      source: computed(() => withEmpty),
      state: { sortKey, sortDesc, groupingKey },
      getSortValue: (row, key) => row[key],
      getGroupKey: (row, key) => String(row[key]),
      emptyGroupLabel: '-',
    })

    expect(groups.value[0].key).toBe('-')
  })

  it('切换 groupingKey 会重新分组', () => {
    const { groups, groupingKey } = setup({ groupingKey: '' })
    expect(groups.value).toHaveLength(1)

    groupingKey.value = 'host'
    expect(groups.value).toHaveLength(3)
  })
})

describe('useSortableGroupedList - 混合类型与空值', () => {
  it('null 排序值按字符串比较而不是抛错', () => {
    const withNull: Row[] = [
      { seq: 1, type: 'a', timestamp: 1, host: 'h1', latency: null },
      { seq: 2, type: 'a', timestamp: 2, host: 'h2', latency: 50 },
    ]
    const sortKey = ref<'seq' | 'type' | 'host'>('seq')
    const sortDesc = ref(false)
    const groupingKey = ref<'type' | 'host' | ''>('')

    const { sorted } = useSortableGroupedList<Row, 'seq' | 'type' | 'host'>({
      source: computed(() => withNull),
      state: { sortKey, sortDesc, groupingKey },
      getSortValue: (row) => row.latency ?? null,
      getGroupKey: (row) => row.type,
    })

    // "50" < "null"（本地化字符串比较），不应抛异常
    expect(sorted.value).toHaveLength(2)
  })

  it('空数组返回空结果', () => {
    const sortKey = ref<'seq' | 'type' | 'host'>('seq')
    const sortDesc = ref(false)
    const groupingKey = ref<'type' | 'host' | ''>('type')

    const { sorted, groups } = useSortableGroupedList<Row, 'seq' | 'type' | 'host'>({
      source: computed(() => []),
      state: { sortKey, sortDesc, groupingKey },
      getSortValue: (row, key) => row[key],
      getGroupKey: (row, key) => String(row[key]),
    })

    expect(sorted.value).toEqual([])
    expect(groups.value).toEqual([])
  })
})

// 冒烟：确保 composable 能在组件 setup 中使用（Pinia-free 场景）
describe('useSortableGroupedList - 组件内使用', () => {
  it('在 setup 中调用不报错', () => {
    const Comp = defineComponent({
      setup() {
        const sortKey = ref<'seq' | 'type' | 'host'>('seq')
        const sortDesc = ref(false)
        const groupingKey = ref<'type' | 'host' | ''>('')
        const { sorted } = useSortableGroupedList<Row, 'seq' | 'type' | 'host'>({
          source: computed(() => rows),
          state: { sortKey, sortDesc, groupingKey },
          getSortValue: (row, key) => row[key],
          getGroupKey: (row, key) => String(row[key]),
        })
        return () => h('div', String(sorted.value.length))
      },
    })

    const wrapper = mount(Comp)
    expect(wrapper.text()).toBe('3')
    vi.restoreAllMocks()
  })
})
