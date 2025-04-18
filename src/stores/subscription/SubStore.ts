import { ref } from 'vue'
import { defineStore } from 'pinia'

export interface Subscription {
  name: string
  url: string
  isLoading: boolean
  lastUpdate?: number
  isManual: boolean
  manualContent?: string
  useSubscriptionRules: boolean
}

export const useSubStore = defineStore(
  'sub',
  () => {
    const list = ref<Subscription[]>([])
    const activeIndex = ref<number | null>(null)

    const add = (
      name: string,
      url: string,
      isManual: boolean = false,
      manualContent?: string,
      useSubscriptionRules: boolean = false,
    ) => {
      list.value.push({
        name,
        url,
        isLoading: false,
        lastUpdate: undefined,
        isManual,
        manualContent,
        useSubscriptionRules,
      })
    }

    return {
      list,
      activeIndex,
      add,
    }
  },
  {
    persist: true,
  },
)
