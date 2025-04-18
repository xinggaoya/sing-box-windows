import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'

// 声明traffic-data事件的类型
interface TrafficData {
  up: number
  down: number
}

// 流量存储接口
interface TrafficStorage {
  up: number
  down: number
  total: number
  totalUp: number
  totalDown: number
}

export const useTrafficStore = defineStore(
  'traffic',
  () => {
    // 流量信息
    const traffic = ref<TrafficStorage>({
      up: 0,
      down: 0,
      total: 0,
      totalUp: 0, // 上传总流量
      totalDown: 0, // 下载总流量
    })

    // 存储事件监听器清理函数
    let unlistenTrafficFn: (() => void) | null = null

    // 初始化流量监听
    const setupTrafficListener = async () => {
      try {
        // 先清理可能存在的旧监听器
        if (unlistenTrafficFn) {
          unlistenTrafficFn()
          unlistenTrafficFn = null
        }
        
        // 监听流量数据
        unlistenTrafficFn = await listen<TrafficData>('traffic-data', (event) => {
          const data = event.payload
          if (data && 'up' in data && 'down' in data) {
            // 更新当前速率
            traffic.value.up = data.up
            traffic.value.down = data.down
            
            // 更新总流量
            traffic.value.totalUp += data.up
            traffic.value.totalDown += data.down
            traffic.value.total = traffic.value.totalUp + traffic.value.totalDown
          }
        })
        
        return true
      } catch (error) {
        console.error('设置流量监听器失败:', error)
        return false
      }
    }

    // 重置流量统计
    const resetStats = () => {
      traffic.value.totalUp = 0
      traffic.value.totalDown = 0
      traffic.value.total = 0
    }

    // 清理监听器
    const cleanupListeners = () => {
      if (unlistenTrafficFn) {
        unlistenTrafficFn()
        unlistenTrafficFn = null
      }
    }

    return {
      traffic,
      setupTrafficListener,
      resetStats,
      cleanupListeners
    }
  },
  {
    persist: true,
  }
)
