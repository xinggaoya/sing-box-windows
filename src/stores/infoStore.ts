import { ref } from 'vue'
import { defineStore } from 'pinia'
import { createWebSocket } from '@/utils'
import { useAppStore } from '@/stores/AppStore'

export const useInfoStore = defineStore('info', () => {
  const appState = useAppStore()

  const traffic = ref({
    up: 0,
    down: 0
  })

  const memory = ref({
    inuse: 0,
    oslimit: 0
  })

  const logs = ref<any>([])

  const isRunning = ref(false)

  const initWS = async () => {
    // 流量
    createWebSocket(`ws://127.0.0.1:9090/traffic?token=`, (data) => {
      traffic.value = data
      // 转int
      appState.usedData += Number(data.up + data.down)
      if (!isRunning.value) {
        isRunning.value = true
      }
    }, () => {
      isRunning.value = false
      logs.value = []
    })
    createWebSocket(`ws://127.0.0.1:9090/memory?token=`, (data) => {
      memory.value = data
    })
    createWebSocket(`ws://127.0.0.1:9090/logs?token=`, (data) => {
      logs.value.push(data)
    })
  }

  initWS()

  return { traffic, memory, logs, isRunning }
})
