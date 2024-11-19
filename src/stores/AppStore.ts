import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useAppStore = defineStore(
  'app',
  () => {
    const mode = ref('')
    // 使用流量
    const usedData = ref(0)
    const autoStart = ref(false)
    const autoStartKernel = ref(false)
    const isRunning = ref(false)

    return { mode, autoStart, usedData, autoStartKernel, isRunning }
  },
  {
    persist: true,
  },
)
