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
    total: 0
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

  const logs = ref<any>([])

  onMounted(() => {
    if (appState.isRunning) {
      initWS()
    }
  })

  const initWS = async () => {
    // 流量
    createWebSocket(`ws://127.0.0.1:9090/traffic?token=`, (data) => {
      updateTraffic(data)
    })
    createWebSocket(`ws://127.0.0.1:9090/memory?token=`, (data) => {
      updateMemory(data)
    })
    createWebSocket(`ws://127.0.0.1:9090/logs?token=`, (data) => {
      logs.value.push(data)
    })
  }

  // 检查是否成功
  const startKernel = async () => {
    return new Promise((resolve, reject) => {
      const retryFetch = async () => {
        try {
          await invoke('start_kernel')
          const res = await fetch('http://127.0.0.1:9090/version')
          if (!res.ok) {
            throw new Error(`HTTP error! status: ${res.status}`)
          }
          const json = await res.json()
          resolve(json)
          version.value = json
          appState.isRunning = true
          getLatestVersion()
          initWS()
        } catch (error) {
          console.error('请求失败:', error)
          console.log('将在1秒后重试...')
          setTimeout(retryFetch, 1000) // 等待1秒后重试
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
            total: 0
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
      total: traffic.value.total + currentUp + currentDown
    }
  }

  const resetTraffic = () => {
    traffic.value = {
      up: 0,
      down: 0,
      total: 0
    }
  }

  return { traffic, memory, logs, version, newVersion, startKernel, stopKernel, updateMemory, updateTraffic, resetTraffic }
})
