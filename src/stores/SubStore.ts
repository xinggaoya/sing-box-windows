import { ref } from 'vue'
import { defineStore } from 'pinia'

interface Subscription {
  name: string
  url: string
  isLoading?: boolean
  lastUpdate?: number
}

export const useSubStore = defineStore(
  'sub',
  () => {
    const list = ref<Array<Subscription>>([])

    const add = (name: string, url: string) => {
      list.value.push({ 
        name, 
        url,
        isLoading: false,
        lastUpdate: undefined
      })
    }

    return { list, add }
  },
  {
    persist: true,
  },
)
