import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { useLocalStorage } from '@vueuse/core'
import { connectionService } from '@/services/connection-service'
import { proxyService, type ProxyGroup as ApiGroup } from '@/services/proxy-service'
import type { GroupsData } from '@/services/proxy-service'
import { useAppStore } from '@/stores/app/AppStore'

export interface ProxyNode {
  name: string
  type: string
  all: string[]
  now: string
  history: { time: string; delay: number }[]
  alive?: boolean
  testUrl?: string
  provider?: string
  delay?: number
}

export interface ProxyGroup extends ProxyNode {
  tag: string
  selected: string
  isExpand: boolean
  items: ApiGroup['items']
}

type ProxyDisplayMode = 'card' | 'list'
type ProxyOrdering = 'natural' | 'latency' | 'name'

const DEFAULT_TIMEOUT_MS = 8000

export const useProxyStore = defineStore('proxy', () => {
  const loading = ref(false)
  const batchTesting = ref(false)
  /** 原始 gRPC Groups 数据 */
  const rawGroups = ref<GroupsData>({ group: [] })
  const nodeTestingMap = ref<Record<string, boolean>>({})
  const groupTestingMap = ref<Record<string, boolean>>({})

  const favorites = useLocalStorage<string[]>('proxy-favorites', [])
  const excludedRecommendations = useLocalStorage<string[]>('proxy-recommendation-excluded', [])
  const expandedGroups = useLocalStorage<string[]>('proxy-expanded-groups', [])
  const displayMode = useLocalStorage<ProxyDisplayMode>('proxy-display-mode', 'card')
  const ordering = useLocalStorage<ProxyOrdering>('proxy-display-mode-natural', 'natural')
  const hideUnavailable = useLocalStorage('proxy-hide-unavailable', false)
  const latencyTimeoutMs = useLocalStorage('proxy-latency-timeout-ms', DEFAULT_TIMEOUT_MS)
  const autoCloseConnections = useLocalStorage('proxy-auto-close-connections', true)
  const selectedGroup = ref<string>('')
  /**
   * 上次 fetchProxies 拉取数据时对应的 activeConfigPath。
   * 用来在进入代理页 / 注册监听器时自检:
   * 如果当前活跃配置路径已经变了(例如在订阅页切换了订阅),
   * 即使 kernel-ready 事件错过,也会主动重新拉取,避免显示旧数据。
   */
  const lastFetchedConfigPath = ref<string | null>(null)

  /**
   * 把 gRPC ProxyGroup 转为前端展示用 ProxyGroup（兼容 view 的 `name` / `all` 字段）
   */
  const toViewGroup = (g: ApiGroup): ProxyGroup => {
    const all = g.items.map((it) => it.tag)
    return {
      name: g.tag,
      tag: g.tag,
      type: g.type,
      selected: g.selected,
      isExpand: g.isExpand,
      items: g.items,
      all,
      now: g.selected,
      history: [],
      delay: 0,
    }
  }

  const proxyGroups = computed<ProxyGroup[]>(() =>
    rawGroups.value.group
      .filter((g) => g.items.length > 0)
      .map(toViewGroup),
  )

  const groupCount = computed(() => proxyGroups.value.length)
  const nodeCount = computed(() => {
    const names = new Set<string>()
    proxyGroups.value.forEach((g) => g.items.forEach((it) => names.add(it.tag)))
    return names.size
  })

  const getLatency = (proxyName: string): number => {
    for (const g of proxyGroups.value) {
      const item = g.items.find((it) => it.tag === proxyName)
      if (item && item.urlTestDelay > 0) return item.urlTestDelay
    }
    return 0
  }

  const isProxyGroup = (name: string) => proxyGroups.value.some((g) => g.name === name)

  const isFavorite = (name: string) => favorites.value.includes(name)

  const toggleFavorite = (name: string) => {
    if (isFavorite(name)) {
      favorites.value = favorites.value.filter((n) => n !== name)
      return
    }
    favorites.value = [...favorites.value, name]
  }

  const isRecommendationExcluded = (name: string) => excludedRecommendations.value.includes(name)

  const toggleRecommendationExclusion = (name: string) => {
    if (isRecommendationExcluded(name)) {
      excludedRecommendations.value = excludedRecommendations.value.filter((n) => n !== name)
      return
    }
    excludedRecommendations.value = [...excludedRecommendations.value, name]
  }

  const toggleGroupExpanded = (groupName: string) => {
    if (expandedGroups.value.includes(groupName)) {
      expandedGroups.value = expandedGroups.value.filter((n) => n !== groupName)
      return
    }
    expandedGroups.value = [...expandedGroups.value, groupName]
  }

  const fetchProxies = async () => {
    loading.value = true
    try {
      const groups = await proxyService.getGroups()
      rawGroups.value = groups
      if (!selectedGroup.value && proxyGroups.value.length) {
        selectedGroup.value = proxyGroups.value[0].name
      }
      // 记录本次拉取对应的 activeConfigPath,用于进入代理页时自检
      lastFetchedConfigPath.value = useAppStore().activeConfigPath
    } finally {
      loading.value = false
    }
  }

  const changeProxy = async (group: string, proxy: string) => {
    await proxyService.selectOutbound(group, proxy)
    // 乐观更新本地选择状态
    rawGroups.value = {
      group: rawGroups.value.group.map((g) =>
        g.tag === group ? { ...g, selected: proxy } : g,
      ),
    }
    if (autoCloseConnections.value) {
      await connectionService.closeAll().catch(() => undefined)
    }
  }

  /** 整组测速（gRPC URLTest） */
  const testGroupDelay = async (groupName: string) => {
    groupTestingMap.value = { ...groupTestingMap.value, [groupName]: true }
    try {
      const result = await proxyService.urlTest(groupName)
      if (!result.ok) return false
      // sing-box 1.14 URLTest 完成后**不会**自动推 SubscribeGroups frame
      // (Node 实测 8 秒内 0 个新 frame),所以测速完成后前端必须主动拉一次最新 groups
      // 才能拿到更新后的 url_test_delay。
      // 经验等待:小规模组 3-5 秒足够,大规模组可能更久,这里固定 3 秒。
      await new Promise((r) => setTimeout(r, 3000))
      await fetchProxies()
      return true
    } finally {
      groupTestingMap.value = { ...groupTestingMap.value, [groupName]: false }
    }
  }

  /**
   * 单节点测速。
   * ⚠️ 节点 tag 在 sing-box 1.14 里通常属于多个 selector 组(如 "自动选择" / "Telegram" 等),
   * 不能用 `find` 反查(总是命中第一个组 "自动选择")。
   * 必须由 view 层传当前激活的 group 进来。
   */
  const testNodeDelay = async (groupTag: string, proxyName: string) => {
    if (!groupTag) {
      throw new Error('testNodeDelay 缺少 groupTag(节点可能在多个组里)')
    }
    nodeTestingMap.value = { ...nodeTestingMap.value, [proxyName]: true }
    try {
      const result = await proxyService.urlTest(groupTag)
      // 等测速完成(同 testGroupDelay 同样的 3 秒),主动拉 groups 拿最新延迟
      if (result.ok) {
        await new Promise((r) => setTimeout(r, 3000))
        await fetchProxies()
      }
      return { ok: result.ok, delay: 0, proxy: proxyName } as {
        ok: boolean
        delay: number
        proxy: string
      }
    } finally {
      nodeTestingMap.value = { ...nodeTestingMap.value, [proxyName]: false }
    }
  }

  const testAllGroups = async () => {
    const groupNames = proxyGroups.value.map((g) => g.name)
    batchTesting.value = true
    // 标记所有组在测速中(因为我们这里直接调 urlTest,绕过 testGroupDelay 的 loading 标记)
    groupTestingMap.value = groupNames.reduce(
      (acc, g) => ({ ...acc, [g]: true }),
      { ...groupTestingMap.value },
    )
    try {
      // 直接调 urlTest,不走 testGroupDelay —— 否则 N 个组每个等 3 秒 = 30s+
      const results = await Promise.all(groupNames.map((g) => proxyService.urlTest(g)))
      // 等待所有组测速完成
      await new Promise((r) => setTimeout(r, 5000))
      await fetchProxies()
      return results.map((r) => ({ ok: r.ok }))
    } finally {
      batchTesting.value = false
      groupTestingMap.value = groupNames.reduce(
        (acc, g) => ({ ...acc, [g]: false }),
        { ...groupTestingMap.value },
      )
    }
  }

  const getSortedNodesForGroup = (group: ProxyGroup) => {
    let list = [...group.all]

    if (hideUnavailable.value) {
      list = list.filter((n) => {
        const item = group.items.find((it) => it.tag === n)
        return item ? item.urlTestDelay > 0 : true
      })
    }

    if (ordering.value === 'name') {
      list.sort((a, b) => a.localeCompare(b))
    }
    if (ordering.value === 'latency') {
      list.sort((a, b) => getLatency(a) - getLatency(b))
    }
    return list
  }

  const getRecommendedNode = (group: ProxyGroup) => {
    const candidates = getSortedNodesForGroup(group).filter((n) => {
      if (n === 'direct' || n === 'reject' || n === 'DIRECT' || n === 'REJECT') return false
      return !isRecommendationExcluded(n)
    })
    return candidates.find((n) => getLatency(n) > 0) || candidates[0] || null
  }

  const switchToRecommended = async (group: ProxyGroup) => {
    const recommended = getRecommendedNode(group)
    if (!recommended) return null
    await changeProxy(group.name, recommended)
    return recommended
  }

  // 订阅后端 SubscribeGroups 推送,URLTest 测速完成后会自动回写延迟。
  // 之前完全靠前端 getGroups() 一次性快照,测速后不重取 → 延迟不刷新。
  // 同时监听 kernel-ready 事件:订阅切换 / 代理模式切换 / 端口变更等会触发内核重启,
  // 重启后需要重新拉取代理组,否则代理页会一直显示旧配置对应的代理列表(需手动点刷新)。
  const groupsDataUnlisten = ref<(() => void) | null>(null)
  const kernelReadyUnlisten = ref<(() => void) | null>(null)
  const setupGroupsDataListener = async () => {
    const { eventService } = await import('@/services/event-service')
    const { APP_EVENTS } = await import('@/constants/events')

    // 兜底自检:如果进入代理页时,后端活跃配置路径已经和上次拉取时的不一致,
    // 说明中途切换过订阅/配置,立即重新拉取。
    // 解决"切订阅后立即进入代理页,kernel-ready 事件已错过"的时序竞态。
    const currentActivePath = useAppStore().activeConfigPath
    if (lastFetchedConfigPath.value !== currentActivePath) {
      fetchProxies().catch(() => {
        // 静默失败:内核可能正在重启,API 还没就绪,后续 kernel-ready / groups-data 会兜底
      })
    }

    if (!groupsDataUnlisten.value) {
      groupsDataUnlisten.value = await eventService.on(
        APP_EVENTS.groupsData,
        (payload: unknown) => {
          // 后端 emit 的是 gRPC Groups 结构(可能含 group / outboundList 等字段)
          // 直接覆盖 rawGroups 即可
          const data = payload as GroupsData
          if (data && Array.isArray((data as { group?: unknown[] }).group)) {
            rawGroups.value = data
            // SubscribeGroups 推送的快照也是对应当前 activeConfigPath,顺手记录避免重复 fetch
            lastFetchedConfigPath.value = useAppStore().activeConfigPath
          } else if (data && Array.isArray((data as { outbounds?: unknown[] }).outbounds)) {
            // 兼容:某些 sing-box 版本发 OutboundList 而不是 Groups
            rawGroups.value = { group: [] }
          }
        },
      )
    }

    if (!kernelReadyUnlisten.value) {
      // 内核就绪后重新拉取一次代理组,确保代理页与内核当前配置一致。
      // 注意:首次进入应用时 HomeView 的 onMounted 也会调用 fetchProxies,
      // 这里再次调用是幂等的,只会产生一次额外的 gRPC 请求,可接受。
      kernelReadyUnlisten.value = await eventService.on(APP_EVENTS.kernelReady, () => {
        fetchProxies().catch(() => {
          // 静默失败:可能是内核刚启动,API 还没完全就绪,下次 groups-data 推送会兜底
        })
      })
    }
  }
  const cleanupGroupsDataListener = () => {
    groupsDataUnlisten.value?.()
    groupsDataUnlisten.value = null
    kernelReadyUnlisten.value?.()
    kernelReadyUnlisten.value = null
  }

  return {
    loading,
    batchTesting,
    rawGroups,
    nodeTestingMap,
    groupTestingMap,
    favorites,
    excludedRecommendations,
    expandedGroups,
    displayMode,
    ordering,
    hideUnavailable,
    latencyTimeoutMs,
    autoCloseConnections,
    selectedGroup,
    proxyGroups,
    groupCount,
    nodeCount,
    getLatency,
    isProxyGroup,
    isFavorite,
    toggleFavorite,
    isRecommendationExcluded,
    toggleRecommendationExclusion,
    toggleGroupExpanded,
    fetchProxies,
    changeProxy,
    testNodeDelay,
    testGroupDelay,
    testAllGroups,
    setupGroupsDataListener,
    cleanupGroupsDataListener,
    getSortedNodesForGroup,
    getRecommendedNode,
    switchToRecommended,
  }
})