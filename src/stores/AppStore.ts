import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export const useAppStore = defineStore('counter', () => {

  const mode = ref('')
  const autoStart = ref(false)

  return { mode, autoStart }
}, {
  persist: true
})
