import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { useLocalStorage } from '@vueuse/core'
import { connectionService } from '@/services/connection-service'
import { proxyService, type DelayTestOptions, type ProxyDelayTestResult } from '@/services/proxy-service'
import type { ProxyNode, ProxyProvider } from '@/types/controller'

export type ProxyGroup = ProxyNode & Required<Pick<ProxyNode, 'name' | 'type' | 'all'>>
type ProxyViewTab = 'groups' | 'providers'
type ProxyDisplayMode = 'card' | 'list'
type ProxyOrdering = 'natural' | 'latency' | 'name'

const DEFAULT_TIMEOUT_MS = 8000

export const useProxyStore = defineStore('proxy', () => {
  const loading = ref(false)
  const batchTesting = ref(false)
  const rawProxies = ref<Record<string, ProxyNode>>({})
  const rawProviders = ref<Record<string, ProxyProvider>>({})
  const latencyResults = ref<Record<string, ProxyDelayTestResult>>({})
  const nodeTestingMap = ref<Record<string, boolean>>({})
  const groupTestingMap = ref<Record<string, boolean>>({})
  const providerUpdatingMap = ref<Record<string, boolean>>({})

  const favorites = useLocalStorage<string[]>('proxy-favorites', [])
  const excludedRecommendations = useLocalStorage<string[]>('proxy-recommendation-excluded', [])
  const expandedGroups = useLocalStorage<string[]>('proxy-expanded-groups', [])
  const viewTab = useLocalStorage<ProxyViewTab>('proxy-view-tab', 'groups')
  const displayMode = useLocalStorage<ProxyDisplayMode>('proxy-display-mode', 'card')
  const ordering = useLocalStorage<ProxyOrdering>('proxy-ordering', 'natural')
  const hideUnavailable = useLocalStorage('proxy-hide-unavailable', false)
  const latencyTestUrl = useLocalStorage('proxy-latency-url', '')
  const latencyTimeoutMs = useLocalStorage('proxy-latency-timeout-ms', DEFAULT_TIMEOUT_MS)
  const autoCloseConnections = useLocalStorage('proxy-auto-close-connections', true)

  const proxyGroups = computed<ProxyGroup[]>(() =>
    Object.entries(rawProxies.value)
      .filter(([name, proxy]) => {
        if (!proxy?.all?.length) return false
        return !['GLOBAL', 'DIRECT', 'REJECT', 'direct', 'reject'].includes(name)
      })
      .map(([name, proxy]) => ({
        ...proxy,
        name,
        type: proxy.type || 'Selector',
        all: [...(proxy.all || [])],
      })),
  )

  const proxyProviders = computed(() =>
    Object.values(rawProviders.value).sort((left, right) => left.name.localeCompare(right.name)),
  )

  const proxyNodeMap = computed(() => {
    const nodes: Record<string, ProxyNode> = {}

    Object.values(rawProxies.value).forEach((proxy) => {
      if (proxy.name) {
        nodes[proxy.name] = proxy
      }
    })

    proxyProviders.value.forEach((provider) => {
      provider.proxies?.forEach((proxy) => {
        nodes[proxy.name] = {
          ...proxy,
          provider: provider.name,
          testUrl: proxy.testUrl || provider.testUrl,
        }
      })
    })

    return nodes
  })

  const groupCount = computed(() => proxyGroups.value.length)
  const nodeCount = computed(() => {
    const names = new Set<string>()
    proxyGroups.value.forEach((group) => group.all.forEach((name) => names.add(name)))
    return names.size
  })

  const resolveLatencyFromProxy = (proxyName: string) => {
    const proxy = proxyNodeMap.value[proxyName] || rawProxies.value[proxyName]
    const history = proxy?.history || []
    const lastDelay = history.length ? history[history.length - 1]?.delay : 0
    return lastDelay || 0
  }

  const getLatency = (proxyName: string) => {
    const latestResult = latencyResults.value[proxyName]
    if (latestResult?.ok) {
      return latestResult.delay
    }
    return resolveLatencyFromProxy(proxyName)
  }

  const isProxyGroup = (proxyName: string) =>
    proxyGroups.value.some((proxy) => proxy.name === proxyName)

  const isFavorite = (proxyName: string) => favorites.value.includes(proxyName)

  const toggleFavorite = (proxyName: string) => {
    if (isFavorite(proxyName)) {
      favorites.value = favorites.value.filter((item) => item !== proxyName)
      return
    }
    favorites.value = [...favorites.value, proxyName]
  }

  const isRecommendationExcluded = (proxyName: string) =>
    excludedRecommendations.value.includes(proxyName)

  const toggleRecommendationExclusion = (proxyName: string) => {
    if (isRecommendationExcluded(proxyName)) {
      excludedRecommendations.value = excludedRecommendations.value.filter((item) => item !== proxyName)
      return
    }
    excludedRecommendations.value = [...excludedRecommendations.value, proxyName]
  }

  const toggleGroupExpanded = (groupName: string) => {
    if (expandedGroups.value.includes(groupName)) {
      expandedGroups.value = expandedGroups.value.filter((item) => item !== groupName)
      return
    }
    expandedGroups.value = [...expandedGroups.value, groupName]
  }

  const resolveDelayOptions = (options?: DelayTestOptions): DelayTestOptions => ({
    timeoutMs: options?.timeoutMs || latencyTimeoutMs.value || DEFAULT_TIMEOUT_MS,
    url: options?.url || latencyTestUrl.value || undefined,
    concurrency: options?.concurrency,
    samples: options?.samples,
  })

  const updateLatencyResults = (results: ProxyDelayTestResult[]) => {
    const next = { ...latencyResults.value }
    results.forEach((result) => {
      next[result.proxy] = result
    })
    latencyResults.value = next
  }

  const fetchProxies = async () => {
    loading.value = true
    try {
      const [proxiesResponse, providersResponse] = await Promise.allSettled([
        proxyService.getProxies(),
        proxyService.getProxyProviders(),
      ])

      if (proxiesResponse.status === 'fulfilled') {
        rawProxies.value = proxiesResponse.value.proxies || {}
      } else {
        throw proxiesResponse.reason
      }

      rawProviders.value =
        providersResponse.status === 'fulfilled' ? providersResponse.value.providers || {} : {}
    } finally {
      loading.value = false
    }
  }

  const changeProxy = async (group: string, proxy: string) => {
    await proxyService.changeProxy(group, proxy)
    const current = rawProxies.value[group]
    if (current) {
      rawProxies.value = {
        ...rawProxies.value,
        [group]: {
          ...current,
          now: proxy,
        },
      }
    }
    if (autoCloseConnections.value) {
      await connectionService.closeAll().catch(() => undefined)
    }
  }

  const testNodeDelay = async (proxyName: string, options?: DelayTestOptions) => {
    if (nodeTestingMap.value[proxyName]) {
      return latencyResults.value[proxyName] || null
    }

    nodeTestingMap.value = {
      ...nodeTestingMap.value,
      [proxyName]: true,
    }

    try {
      const result = await proxyService.testNodeDelay(
        proxyName,
        resolveDelayOptions(options).url,
      )
      updateLatencyResults([result])
      return result
    } finally {
      nodeTestingMap.value = {
        ...nodeTestingMap.value,
        [proxyName]: false,
      }
    }
  }

  const testGroupDelay = async (groupName: string, options?: DelayTestOptions) => {
    groupTestingMap.value = {
      ...groupTestingMap.value,
      [groupName]: true,
    }

    try {
      const results = await proxyService.testGroupDelay(
        groupName,
        resolveDelayOptions(options).url,
        undefined,
        resolveDelayOptions(options),
      )
      updateLatencyResults(results)
      return results
    } finally {
      groupTestingMap.value = {
        ...groupTestingMap.value,
        [groupName]: false,
      }
    }
  }

  const testMultipleNodes = async (proxyNames: string[], options?: DelayTestOptions) => {
    const results = await proxyService.testNodesDelay(proxyNames, resolveDelayOptions(options))
    updateLatencyResults(results)
    return results
  }

  const testAllGroups = async (options?: DelayTestOptions) => {
    const groupNames = proxyGroups.value.map((group) => group.name)
    batchTesting.value = true
    try {
      const results = await Promise.all(groupNames.map((group) => testGroupDelay(group, options)))
      return results.flat()
    } finally {
      batchTesting.value = false
    }
  }

  const updateProvider = async (providerName: string) => {
    providerUpdatingMap.value = {
      ...providerUpdatingMap.value,
      [providerName]: true,
    }

    try {
      await proxyService.updateProxyProvider(providerName)
      await fetchProxies()
    } finally {
      providerUpdatingMap.value = {
        ...providerUpdatingMap.value,
        [providerName]: false,
      }
    }
  }

  const getSortedNodesForGroup = (group: ProxyGroup) => {
    let list = [...group.all]

    if (hideUnavailable.value) {
      list = list.filter((node) => {
        const proxyNode = proxyNodeMap.value[node]
        if (proxyNode?.alive === false) return false
        return true
      })
    }

    if (ordering.value === 'name') {
      list.sort((left, right) => left.localeCompare(right))
    }

    if (ordering.value === 'latency') {
      list.sort((left, right) => {
        const leftLatency = getLatency(left) || Number.MAX_SAFE_INTEGER
        const rightLatency = getLatency(right) || Number.MAX_SAFE_INTEGER
        return leftLatency - rightLatency
      })
    }

    return list
  }

  const getRecommendedNode = (group: ProxyGroup) => {
    const candidates = getSortedNodesForGroup(group).filter((proxyName) => {
      if (proxyName === 'DIRECT' || proxyName === 'REJECT' || proxyName === 'direct') return false
      return !isRecommendationExcluded(proxyName)
    })

    return candidates.find((name) => {
      const latency = getLatency(name)
      return latency > 0
    }) || candidates[0] || null
  }

  const switchToRecommended = async (group: ProxyGroup) => {
    const recommended = getRecommendedNode(group)
    if (!recommended) return null
    await changeProxy(group.name, recommended)
    return recommended
  }

  return {
    loading,
    batchTesting,
    rawProxies,
    rawProviders,
    latencyResults,
    nodeTestingMap,
    groupTestingMap,
    providerUpdatingMap,
    favorites,
    excludedRecommendations,
    expandedGroups,
    viewTab,
    displayMode,
    ordering,
    hideUnavailable,
    latencyTestUrl,
    latencyTimeoutMs,
    autoCloseConnections,
    proxyGroups,
    proxyProviders,
    proxyNodeMap,
    groupCount,
    nodeCount,
    isProxyGroup,
    isFavorite,
    toggleFavorite,
    isRecommendationExcluded,
    toggleRecommendationExclusion,
    toggleGroupExpanded,
    getLatency,
    fetchProxies,
    changeProxy,
    testNodeDelay,
    testGroupDelay,
    testMultipleNodes,
    testAllGroups,
    updateProvider,
    getSortedNodesForGroup,
    getRecommendedNode,
    switchToRecommended,
  }
})
