import { onMounted, ref } from 'vue'
import { defineStore } from 'pinia'
import { createWebSocket } from '@/utils'
import { useAppStore } from '@/stores/AppStore'
import { invoke } from '@tauri-apps/api/core'

export const useInfoStore = defineStore('info', () => {
  const appState = useAppStore()

  const traffic = ref({
    up: 0,
    down: 0,
    total: 0,
  })

  const memory = ref({
    inuse: 0,
    oslimit: 0,
  })

  const version = ref({
    meta: true,
    premium: true,
    version: '',
  })

  const newVersion = ref('')

  const MAX_LOGS = 300 // 最大日志条数
  const logs = ref<any>([])

  onMounted(() => {
    if (appState.isRunning) {
      initWS()
    }
  })

  // 添加日志
  const addLog = (data: any) => {
    logs.value.push(data)
    // 如果超过最大日志数，删除最早的日志
    if (logs.value.length > MAX_LOGS) {
      logs.value = logs.value.slice(-MAX_LOGS)
    }
  }

  const initWS = async () => {
    // 流量
    createWebSocket(`ws://127.0.0.1:9090/traffic?token=`, (data) => {
      updateTraffic(data)
    })
    createWebSocket(`ws://127.0.0.1:9090/memory?token=`, (data) => {
      updateMemory(data)
    })
    createWebSocket(`ws://127.0.0.1:9090/logs?token=`, (data) => {
      addLog(data)
    })
  }

  // 检查是否成功
  const startKernel = async () => {
    // 启动内核
    await invoke('start_kernel')
    return new Promise((resolve, reject) => {
      let retryCount = 0
      const maxRetries = 5

      const retryFetch = async () => {
        try {
          // 等待内核启动并检查状态
          const res = await fetch('http://127.0.0.1:9090/version')
          if (!res.ok) {
            throw new Error(`HTTP error! status: ${res.status}`)
          }

          const json = await res.json()
          version.value = json
          appState.isRunning = true

          // 获取最新版本信息
          await getLatestVersion()

          // 初始化WebSocket连接
          await initWS()

          resolve(json)
        } catch (error) {
          console.error('启动失败:', error)

          if (retryCount < maxRetries) {
            retryCount++
            console.log(`重试第 ${retryCount} 次，共 ${maxRetries} 次`)
            setTimeout(retryFetch, 1000) // 等待1秒后重试
          } else {
            reject(new Error(`启动失败，已重试 ${maxRetries} 次: ${error}`))
          }
        }
      }

      retryFetch()
    })
  }

  // 停止内核
  const stopKernel = () => {
    return new Promise((resolve, reject) => {
      invoke('stop_kernel')
        .then(() => {
          resolve(true)
          appState.isRunning = false
          memory.value = { inuse: 0, oslimit: 0 }
          logs.value = []
          traffic.value = {
            up: 0,
            down: 0,
            total: 0,
          }
        })
        .catch(() => {
          reject()
        })
    })
  }

  // 获取最新版本
  const getLatestVersion = async () => {
    const res = await fetch('https://api.github.com/repos/SagerNet/sing-box/releases/latest')
    const json = await res.json()
    newVersion.value = json.tag_name
  }

  const updateMemory = (data: any) => {
    memory.value = data
  }

  const updateTraffic = (data: any) => {
    const currentUp = Number(data.up) || 0
    const currentDown = Number(data.down) || 0

    traffic.value = {
      up: currentUp,
      down: currentDown,
      total: traffic.value.total + currentUp + currentDown,
    }
  }

  const resetTraffic = () => {
    traffic.value = {
      up: 0,
      down: 0,
      total: 0,
    }
  }

  return {
    traffic,
    memory,
    logs,
    version,
    newVersion,
    startKernel,
    stopKernel,
    updateMemory,
    updateTraffic,
    resetTraffic,
  }
})
