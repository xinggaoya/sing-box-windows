import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useSubStore = defineStore(
  'sub',
  () => {
    const list = ref<
      Array<{
        name: string
        url: string
      }>
    >([])

    const add = (name: string, url: string) => {
      list.value.push({ name, url })
    }

    return { list, add }
  },
  {
    persist: true,
  },
)
