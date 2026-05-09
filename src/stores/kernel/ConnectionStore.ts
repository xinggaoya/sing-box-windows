import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { useLocalStorage } from '@vueuse/core'
import { APP_EVENTS } from '@/constants/events'
import { eventService } from '@/services/event-service'
import { connectionService } from '@/services/connection-service'
import type { ConnectionItem, ConnectionsDataPayload, MemoryStatsPayload } from '@/types/events'

type ConnectionTab = 'active' | 'closed'
type ConnectionSortKey =
  | 'start'
  | 'download'
  | 'upload'
  | 'downloadSpeed'
  | 'uploadSpeed'
  | 'host'
  | 'rule'
  | 'process'

export const useConnectionStore = defineStore('connection', () => {
  const MAX_CLOSED_CONNECTIONS = 500

  const activeConnections = ref<ConnectionItem[]>([])
  const closedConnections = ref<ConnectionItem[]>([])
  const latestConnectionMsg = ref<ConnectionsDataPayload | null>(null)
  const latestMemory = ref<MemoryStatsPayload | null>(null)
  const paused = ref(false)
  const loading = ref(false)
  const closingMap = ref<Record<string, boolean>>({})
  const isClosingAll = ref(false)

  const activeTab = useLocalStorage<ConnectionTab>('connections-active-tab', 'active')
  const searchQuery = useLocalStorage('connections-search-query', '')
  const quickFilterEnabled = useLocalStorage('connections-quick-filter-enabled', false)
  const sourceIPFilter = useLocalStorage('connections-source-ip-filter', '')
  const sortKey = useLocalStorage<ConnectionSortKey>('connections-sort-key', 'start')
  const sortDesc = useLocalStorage('connections-sort-desc', true)
  const groupingKey = useLocalStorage<string | null>('connections-grouping-key', null)

  const connectionState = ref({
    connected: false,
    connecting: false,
    error: null as Error | null,
  })

  const memoryState = ref({
    connected: false,
    connecting: false,
    error: null as Error | null,
  })

  const connectionsTotal = computed(() => ({
    upload: latestConnectionMsg.value?.uploadTotal || 0,
    download: latestConnectionMsg.value?.downloadTotal || 0,
  }))

  const memory = computed(() => ({
    inuse: latestMemory.value?.inuse || 0,
    oslimit: latestMemory.value?.oslimit || 0,
    lastUpdated: Date.now(),
  }))

  const connections = computed(() => activeConnections.value)

  let connectionsUnlisten: (() => void) | null = null
  let memoryUnlisten: (() => void) | null = null

  const normalizeConnection = (
    connection: ConnectionItem,
    previous?: ConnectionItem,
  ): ConnectionItem => {
    const downloadSpeed =
      typeof connection.downloadSpeed === 'number'
        ? connection.downloadSpeed
        : Math.max(0, connection.download - (previous?.download || connection.download))
    const uploadSpeed =
      typeof connection.uploadSpeed === 'number'
        ? connection.uploadSpeed
        : Math.max(0, connection.upload - (previous?.upload || connection.upload))

    return {
      ...connection,
      downloadSpeed,
      uploadSpeed,
    }
  }

  const updateConnections = (payload: ConnectionsDataPayload) => {
    latestConnectionMsg.value = payload
    connectionState.value.connected = true
    connectionState.value.connecting = false
    connectionState.value.error = null

    const previousActiveMap = new Map(activeConnections.value.map((item) => [item.id, item]))
    const normalizedActive = payload.connections.map((connection) =>
      normalizeConnection(connection, previousActiveMap.get(connection.id)),
    )

    if (paused.value) {
      return
    }

    const nextIds = new Set(normalizedActive.map((item) => item.id))
    const newlyClosed = activeConnections.value.filter((item) => !nextIds.has(item.id))
    if (newlyClosed.length) {
      const mergedClosed = [...newlyClosed, ...closedConnections.value]
      const dedup = new Map<string, ConnectionItem>()
      mergedClosed.forEach((connection) => {
        if (!dedup.has(connection.id)) {
          dedup.set(connection.id, connection)
        }
      })
      closedConnections.value = Array.from(dedup.values()).slice(0, MAX_CLOSED_CONNECTIONS)
    }

    activeConnections.value = normalizedActive
  }

  const updateMemory = (payload: MemoryStatsPayload) => {
    latestMemory.value = payload
    memoryState.value.connected = true
    memoryState.value.connecting = false
    memoryState.value.error = null
  }

  const setupEventListeners = async () => {
    if (!connectionsUnlisten) {
      connectionsUnlisten = await eventService.onConnectionsData((payload) => {
        updateConnections(payload)
      })
    }

    if (!memoryUnlisten) {
      memoryUnlisten = await eventService.onMemoryData((payload) => {
        updateMemory(payload)
      })
    }
  }

  const cleanupEventListeners = () => {
    try {
      connectionsUnlisten?.()
      memoryUnlisten?.()
      eventService.removeEventListener(APP_EVENTS.connectionsData)
      eventService.removeEventListener(APP_EVENTS.memoryData)
    } finally {
      connectionsUnlisten = null
      memoryUnlisten = null
    }
  }

  const initializeStore = async () => {
    await setupEventListeners()
  }

  const clearClosedConnections = () => {
    closedConnections.value = []
  }

  const togglePaused = () => {
    paused.value = !paused.value
  }

  const closeConnection = async (id: string) => {
    if (closingMap.value[id]) return
    closingMap.value = {
      ...closingMap.value,
      [id]: true,
    }

    try {
      await connectionService.closeOne(id)
    } finally {
      closingMap.value = {
        ...closingMap.value,
        [id]: false,
      }
    }
  }

  const closeAllConnections = async () => {
    if (isClosingAll.value) return
    isClosingAll.value = true
    try {
      await connectionService.closeAll()
    } finally {
      isClosingAll.value = false
    }
  }

  const searchableConnections = computed(() =>
    activeTab.value === 'closed' ? closedConnections.value : activeConnections.value,
  )

  return {
    activeConnections,
    closedConnections,
    latestConnectionMsg,
    latestMemory,
    paused,
    loading,
    closingMap,
    isClosingAll,
    activeTab,
    searchQuery,
    quickFilterEnabled,
    sourceIPFilter,
    sortKey,
    sortDesc,
    groupingKey,
    connectionState,
    memoryState,
    connectionsTotal,
    memory,
    connections,
    searchableConnections,
    setupEventListeners,
    cleanupEventListeners,
    initializeStore,
    updateConnections,
    updateMemory,
    clearClosedConnections,
    togglePaused,
    closeConnection,
    closeAllConnections,
  }
})
