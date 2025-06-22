import { defineStore } from 'pinia'
import { ref } from 'vue'
import { temporaryStoreManager } from '@/utils/memory-leak-fix'

/**
 * 内核运行时临时数据Store
 * 专门管理不需要持久化的实时数据
 */
export const useKernelRuntimeStore = defineStore(
  'kernel-runtime',
  () => {
    // 内存使用信息 (实时数据)
    const memory = ref({
      inuse: 0,
      oslimit: 0,
    })

    // 程序运行时间（秒）(会话数据)
    const uptime = ref(0)
    let uptimeInterval: NodeJS.Timeout | null = null

    // 启动运行时间计数器
    const startUptimeCounter = () => {
      stopUptimeCounter() // 先停止现有的计数器
      uptime.value = 0

      uptimeInterval = setInterval(() => {
        uptime.value += 1
      }, 1000)

      console.log('⏱️ 运行时间计数器已启动')
    }

    // 停止运行时间计数器
    const stopUptimeCounter = () => {
      if (uptimeInterval) {
        clearInterval(uptimeInterval)
        uptimeInterval = null
        console.log('⏹️ 运行时间计数器已停止')
      }
    }

    // 更新内存使用信息
    const updateMemory = (memData: { inuse: number; oslimit: number }) => {
      memory.value = {
        inuse: memData.inuse,
        oslimit: memData.oslimit,
      }
    }

    // 重置所有临时数据
    const resetRuntimeData = () => {
      // 重置内存使用信息
      memory.value = {
        inuse: 0,
        oslimit: 0,
      }

      // 重置运行时间
      uptime.value = 0

      // 停止计数器
      stopUptimeCounter()

      console.log('🔄 运行时临时数据已重置')
    }

    // Store初始化方法
    const initializeStore = () => {
      // 重置所有临时数据
      resetRuntimeData()

      // 注册到临时Store管理器
      const storeInstance = {
        cleanupStore,
        smartCleanup: () => {
          // 定期重置过大的内存数值，防止数值异常
          const MAX_MEMORY = 16 * 1024 * 1024 * 1024 // 16GB
          if (memory.value.inuse > MAX_MEMORY || memory.value.oslimit > MAX_MEMORY) {
            console.log('🧹 运行时Store智能清理 - 重置异常内存数据')
            memory.value = {
              inuse: 0,
              oslimit: 0,
            }
          }
        },
      }
      temporaryStoreManager.registerStore('kernel-runtime', storeInstance)

      console.log('✅ KernelRuntimeStore初始化完成')
    }

    // Store清理方法
    const cleanupStore = () => {
      stopUptimeCounter()
      resetRuntimeData()

      // 从临时Store管理器注销
      temporaryStoreManager.unregisterStore('kernel-runtime')

      console.log('🧹 KernelRuntimeStore已清理')
    }

    return {
      // 临时数据
      memory,
      uptime,

      // 方法
      startUptimeCounter,
      stopUptimeCounter,
      updateMemory,
      resetRuntimeData,
      initializeStore,
      cleanupStore,
    }
  },
  {
    // 完全禁用持久化 - 这是临时运行时数据
    persist: false,
  },
)
