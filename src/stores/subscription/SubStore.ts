import { ref } from 'vue'
import { defineStore } from 'pinia'

export interface Subscription {
  name: string
  url: string
  isLoading: boolean
  lastUpdate?: number
  isManual: boolean
  manualContent?: string
  useOriginalConfig: boolean
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
      useOriginalConfig: boolean = false,
    ) => {
      list.value.push({
        name,
        url,
        isLoading: false,
        lastUpdate: undefined,
        isManual,
        manualContent,
        useOriginalConfig,
      })
    }

    // 重置所有订阅的加载状态
    const resetLoadingState = () => {
      if (list.value.length > 0) {
        list.value = list.value.map(item => ({
          ...item,
          isLoading: false
        }))
      }
    }

    return {
      list,
      activeIndex,
      add,
      resetLoadingState,
    }
  },
  {
    persist: true,
  },
)
