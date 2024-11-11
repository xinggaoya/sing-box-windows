import { ref, computed } from 'vue'
import { defineStore } from 'pinia'

export const useAppStore = defineStore('counter', () => {

  const mode = ref('')

  return { mode }
}, {
  persist: true
})
