import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export const useAppStore = defineStore('counter', () => {

  const mode = ref('')
  // 使用流量
  const usedData = ref(0)
  const autoStart = ref(false)

  return { mode, autoStart, usedData }
}, {
  persist: true
})
