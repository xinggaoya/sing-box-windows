import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { useLocalStorage } from '@vueuse/core'
import { APP_EVENTS } from '@/constants/events'
import { eventService } from '@/services/event-service'
import type { LogEventPayload } from '@/types/events'

export type MessageType = 'success' | 'info' | 'error' | 'warning'

export interface LogEntry {
  seq: number
  type: string
  payload: string
  timestamp: number
}

export const useLogStore = defineStore('log', () => {
  const logs = ref<LogEntry[]>([])
  const paused = ref(false)
  const searchQuery = useLocalStorage('logs-search-query', '')
  const filterType = useLocalStorage<string | null>('logs-filter-type', null)
  const groupingKey = useLocalStorage<string | null>('logs-grouping-key', null)
  const sortKey = useLocalStorage<'seq' | 'type' | 'timestamp'>('logs-sort-key', 'seq')
  const sortDesc = useLocalStorage('logs-sort-desc', true)
  const maxLogs = useLocalStorage('logs-max-rows', 500)

  let seq = 1
  let unlisten: (() => void) | null = null
  let messageCallback: ((type: MessageType, content: string) => void) | null = null

  const processLogData = (data: LogEventPayload) => {
    if (!data?.payload || paused.value) return
    addLog(data.type, data.payload, data.timestamp)
  }

  const addLog = (type: string, payload: string, timestamp = Date.now()) => {
    const entry: LogEntry = {
      seq,
      type,
      payload,
      timestamp,
    }

    seq += 1
    logs.value = [entry, ...logs.value].slice(0, maxLogs.value)
  }

  const clearLogs = () => {
    logs.value = []
    seq = 1
  }

  const setMessageCallback = (callback: (type: MessageType, content: string) => void) => {
    messageCallback = callback
  }

  const showMessage = (type: MessageType, content: string) => {
    addLog(type, content)
    messageCallback?.(type, content)
  }

  const setupLogListener = async () => {
    if (unlisten) return true
    unlisten = await eventService.onLogData((data) => processLogData(data))
    return true
  }

  const cleanupListeners = () => {
    try {
      unlisten?.()
      eventService.removeEventListener(APP_EVENTS.logData)
    } finally {
      unlisten = null
    }
  }

  const initializeStore = async () => {
    await setupLogListener()
  }

  const togglePaused = () => {
    paused.value = !paused.value
  }

  const filteredLogs = computed(() => {
    return logs.value.filter((log) => {
      const matchesQuery =
        !searchQuery.value ||
        log.payload.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
        log.type.toLowerCase().includes(searchQuery.value.toLowerCase())
      const matchesType = !filterType.value || log.type === filterType.value
      return matchesQuery && matchesType
    })
  })

  return {
    logs,
    paused,
    searchQuery,
    filterType,
    groupingKey,
    sortKey,
    sortDesc,
    maxLogs,
    filteredLogs,
    addLog,
    clearLogs,
    setMessageCallback,
    showMessage,
    setupLogListener,
    cleanupListeners,
    initializeStore,
    togglePaused,
  }
})
