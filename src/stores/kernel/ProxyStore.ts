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
  // 节点级测速状态:已删除。sing-box 1.14 gRPC 没有"测单节点" RPC(只有 URLTest 测整组),
  // 官方 sing-box-dashboard 也没有"测单节点"按钮。延迟刷新完全靠 SubscribeGroups 推送。
  const groupTestingMap = ref<Record<string, boolean>>({})
  /**
   * "已测过但失败的节点"集合。set 里的节点会被 getLatencyStatus 判定为 'failed'。
   *
   * 为什么需要这个:sing-box 1.14 URLTestGroup 测速失败时调 `DeleteURLTestHistory`,
   * history 不存在时 proto 序列化默认 `urlTestTime=0` 和 `urlTestDelay=0`——
   * **失败节点跟"还没测"长得一样**,无法仅靠 urlTestTime 区分。
   *
   * 因此前端必须主动追踪"哪些节点刚被测过且失败":
   * - URLTest 触发时:不立即 fill,启动 15s timer
   * - 15s 后(sing-box URLTestGroup 通常 < 10s 完成):检查所有该组节点,
   *   urlTestDelay 仍为 0 的 → 加入 testedNodes
   *   urlTestDelay > 0 的 → 已经在 SubscribeGroups 推过来时显示"Xms",不需要标 failed
   * - 为什么不立刻 fill:立刻 fill + SubscribeGroups 中间推 frame(测速未完成时)
   *   会让"最终会成功"的节点先被标 failed(因为 urlTestDelay 暂时还是 0),导致
   *   "先失败后成功"的闪烁。
   * - SubscribeGroups callback 仍保留清理逻辑(切订阅/重启时,testedNodes 里
   *   已经不存在的 tag 会被移除,避免错位)
   */
  const testedNodes = ref<Set<string>>(new Set())

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

  /**
   * 节点延迟状态(配合 `formatLatency` 区分显示):
   * - `'ok'`:测速成功(urlTestDelay > 0)
   * - `'failed'`:URLTest 已发出但该节点 urlTestDelay 仍为 0
   *   sing-box 1.14 URLTestGroup 测速失败时调 DeleteURLTestHistory,导致
   *   urlTestTime 和 urlTestDelay 都回到 0,跟"未测"无法区分。
   *   所以前端用 `testedNodes` set 追踪"URLTest 已发但结果未到"——这些节点
   *   在 SubscribeGroups 新 frame 到达时,如果 urlTestDelay > 0 会从 set 移除
   *   (测速成功),剩下的就是 failed。
   * - `'untested'`:还没测过(URLTest 没触发过该节点)
   */
  const getLatencyStatus = (proxyName: string): 'untested' | 'failed' | 'ok' => {
    for (const g of proxyGroups.value) {
      const item = g.items.find((it) => it.tag === proxyName)
      if (!item) continue
      if (item.urlTestDelay > 0) return 'ok'
      if (testedNodes.value.has(proxyName)) return 'failed'
      return 'untested'
    }
    return 'untested'
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

  /**
   * 整组测速（gRPC URLTest）。
   *
   * 对齐官方 sing-box-dashboard (src/api/daemon.ts) 行为:
   *   async urlTest(outboundTag: string): Promise<void> {
   *     await this.client.uRLTest({ outboundTag });
   *   }
   * — 官方不调 fetchProxies,完全等 SubscribeGroups 流推送新 frame,前端 store 在
   * update callback 里直接覆盖 groups。
   *
   * 我们之前"测速后等 3s + fetchProxies"是兜底补丁(误以为 SubscribeGroups 不推送),
   * 但 fetchProxies 走的是 subscribe_groups().next() 取首帧,如果测速还没完成,
   * 首帧是 URLTest 前的快照,urlTestDelay 全是 0,反而覆盖掉 SubscribeGroups 推过来的
   * 新数据。移除兜底,让延迟刷新完全靠 relay 推送。
   *
   * 测速成功 / 失败的判定时序:
   * - URLTest 触发后 ~5-15s 内,sing-box URLTestGroup 陆续完成各节点测速
   * - 测速完成的节点通过 SubscribeGroups 推过来的新 frame,urlTestDelay > 0
   *   → getLatencyStatus 走 'ok' 分支,UI 显示 "Xms"
   * - 测速失败节点 urlTestDelay === 0 且 urlTestTime 也会被 DeleteURLTestHistory 清成 0
   *   → 单看 proto 字段无法跟"未测"区分
   * - 因此 15s 后用 setTimeout 把"组内 urlTestDelay 仍为 0 的节点"加入 testedNodes
   *   → getLatencyStatus 走 'failed' 分支,UI 显示 "测速失败"
   *
   * 为什么不立刻 fill testedNodes:立刻 fill 会让 SubscribeGroups 中间推 frame
   * (测速未完成时 urlTestDelay 暂时还是 0) 把"最终会成功"的节点误标 failed,
   * 出现"先失败后成功"的闪烁。
   */
  const testGroupDelay = async (groupName: string) => {
    groupTestingMap.value = { ...groupTestingMap.value, [groupName]: true }
    try {
      const result = await proxyService.urlTest(groupName)
      // 8s 后:loading 结束 + 标 failed。
      // - 测速成功的节点:期间 SubscribeGroups 推新 frame,urlTestDelay > 0,
      //   getLatencyStatus 自然走 'ok' 路径,UI 显示 "Xms"。
      // - 测速失败的节点:urlTestDelay 仍 0,加入 testedNodes,UI 显示红色"测速失败"。
      // - 未测的节点:urlTestDelay=0 + testedNodes 没它,UI 仍是灰色"点击测试"。
      // 选 8s 是因为 sing-box 1.14 URLTestGroup 默认 timeout 5s,大多数订阅 8s 内
      // 完成测速;15s 太长让用户以为"测速早就完成"。
      setTimeout(() => {
        const group = proxyGroups.value.find((g) => g.name === groupName)
        if (group) {
          const next = new Set(testedNodes.value)
          for (const it of group.items) {
            if (it.urlTestDelay === 0) next.add(it.tag)
          }
          testedNodes.value = next
        }
        groupTestingMap.value = { ...groupTestingMap.value, [groupName]: false }
      }, 8000)
      return result.ok
    } catch {
      // 异常(URLTest RPC 失败):立即结束 loading,不要再 setTimeout
      groupTestingMap.value = { ...groupTestingMap.value, [groupName]: false }
      return false
    }
  }

  /**
   * 批量测速所有组（gRPC URLTest）。同 testGroupDelay 的延迟 8s 标 failed 逻辑。
   */
  const testAllGroups = async () => {
    const groupNames = proxyGroups.value.map((g) => g.name)
    batchTesting.value = true
    // 标记所有组在测速中
    groupTestingMap.value = groupNames.reduce(
      (acc, g) => ({ ...acc, [g]: true }),
      { ...groupTestingMap.value },
    )
    try {
      // 并发触发所有组 URLTest,延迟刷新靠 SubscribeGroups relay 推送
      const results = await Promise.all(groupNames.map((g) => proxyService.urlTest(g)))
      // 8s 后:loading 结束 + 标 failed
      setTimeout(() => {
        const next = new Set(testedNodes.value)
        for (const g of proxyGroups.value) {
          for (const it of g.items) {
            if (it.urlTestDelay === 0) next.add(it.tag)
          }
        }
        testedNodes.value = next
        batchTesting.value = false
        groupTestingMap.value = groupNames.reduce(
          (acc, g) => ({ ...acc, [g]: false }),
          { ...groupTestingMap.value },
        )
      }, 8000)
      return results.map((r) => ({ ok: r.ok }))
    } catch {
      // 异常:立即结束 loading
      batchTesting.value = false
      groupTestingMap.value = groupNames.reduce(
        (acc, g) => ({ ...acc, [g]: false }),
        { ...groupTestingMap.value },
      )
      throw new Error('urlTest failed')
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

  // 订阅后端 SubscribeGroups relay 推送,URLTest 测速完成后会实时回写延迟。
  // 这是官方 sing-box-dashboard 的做法——测速后不做任何主动 fetch,完全靠流式推送。
  // 同时监听 kernel-ready 事件:订阅切换 / 代理模式切换 / 端口变更等会触发内核重启,
  // 重启后需要重新拉取代理组,否则代理页会一直显示旧配置对应的代理列表(需手动点刷新)。
  const groupsDataUnlisten = ref<(() => void) | null>(null)
  const kernelReadyUnlisten = ref<(() => void) | null>(null)
  const setupGroupsDataListener = async () => {
    const { eventService } = await import('@/services/event-service')
    const { APP_EVENTS } = await import('@/constants/events')

    // 自检:如果进入代理页时,后端活跃配置路径已经和上次拉取时的不一致,
    // 说明中途切换过订阅/配置,立即重新拉取(否则 relay 推的还是旧 configPath 的 groups)。
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
          // 直接覆盖 rawGroups 即可,URLTest 完成后的 urlTestDelay 也会在这里更新
          const data = payload as GroupsData
          // 诊断:在 dev tools console 打一条,确认事件是否真的到前端 + payload 形状。
          // 字段名应该都是 camelCase:group/tag/type/selectable/selected/isExpand/items,
          // GroupItem:tag/type/urlTestTime/urlTestDelay。
          const firstGroup = (data as { group?: unknown[] }).group?.[0] as
            | { tag?: string; items?: { tag?: string; urlTestDelay?: number }[] }
            | undefined
          const firstItem = firstGroup?.items?.[0]
          const firstDelay = firstItem?.urlTestDelay
          // eslint-disable-next-line no-console
          console.debug(
            `[groups-data] groups=${(data as { group?: unknown[] }).group?.length ?? 0}` +
              ` firstGroup=${firstGroup?.tag} firstItem=${firstItem?.tag}` +
              ` firstDelay=${firstDelay} (raw payload keys: ${Object.keys(data ?? {}).join(',')})`,
          )
          if (data && Array.isArray((data as { group?: unknown[] }).group)) {
            // 清理 testedNodes:对 set 中每个 tag,看新 frame 的对应 item:
            // - 新 frame 没该 tag(切订阅/重启内核了)→ 移除
            // - 新 frame 有该 tag 且 urlTestDelay > 0(测速成功)→ 移除
            // - 新 frame 有该 tag 且 urlTestDelay === 0(测速失败)→ 保留
            //   保留的节点会被 getLatencyStatus 判定为 'failed'。
            if (testedNodes.value.size > 0) {
              const newItemsByTag = new Set<string>()
              for (const g of (data as { group: { items: { tag: string; urlTestDelay: number }[] }[] }).group) {
                for (const it of g.items) newItemsByTag.add(it.tag)
              }
              const next = new Set<string>()
              for (const tag of testedNodes.value) {
                if (!newItemsByTag.has(tag)) continue  // 新 frame 里没了(切订阅/重启)
                // 新 frame 里有该 tag
                let newDelay = 0
                for (const g of (data as { group: { items: { tag: string; urlTestDelay: number }[] }[] }).group) {
                  const it = g.items.find((i) => i.tag === tag)
                  if (it) {
                    newDelay = it.urlTestDelay
                    break
                  }
                }
                if (newDelay > 0) continue  // 测速成功,移除
                next.add(tag)  // 仍为 0,保留为 'failed'
              }
              if (next.size !== testedNodes.value.size) {
                testedNodes.value = next
              }
            }
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
    getLatencyStatus,
    isProxyGroup,
    isFavorite,
    toggleFavorite,
    isRecommendationExcluded,
    toggleRecommendationExclusion,
    toggleGroupExpanded,
    fetchProxies,
    changeProxy,
    testGroupDelay,
    testAllGroups,
    setupGroupsDataListener,
    cleanupGroupsDataListener,
    getSortedNodesForGroup,
    getRecommendedNode,
    switchToRecommended,
  }
})