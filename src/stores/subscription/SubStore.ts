import { ref } from 'vue'
import { defineStore } from 'pinia'
import { storageService, type Subscription } from '@/services/backend-storage-service'

// 为了前端兼容性，创建一个适配器接口
interface FrontendSubscription {
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
    const list = ref<FrontendSubscription[]>([])
    const activeIndex = ref<number | null>(null)

    // 从后端格式转换为前端格式
    const convertToFrontendFormat = (backendSubs: Subscription[]): FrontendSubscription[] => {
      return backendSubs.map(sub => ({
        name: sub.name,
        url: sub.url,
        isLoading: sub.is_loading,
        lastUpdate: sub.last_update || undefined,
        isManual: sub.is_manual,
        manualContent: sub.manual_content || undefined,
        useOriginalConfig: sub.use_original_config,
      }))
    }

    // 从前端格式转换为后端格式
    const convertToBackendFormat = (frontendSubs: FrontendSubscription[]): Subscription[] => {
      return frontendSubs.map(sub => ({
        name: sub.name,
        url: sub.url,
        is_loading: sub.isLoading,
        last_update: sub.lastUpdate || null,
        is_manual: sub.isManual,
        manual_content: sub.manualContent || null,
        use_original_config: sub.useOriginalConfig,
      }))
    }

    // 从后端加载数据
    const loadFromBackend = async () => {
      try {
        console.log('📄 从后端加载订阅配置...')
        const subscriptions = await storageService.getSubscriptions()
        
        // 更新响应式状态
        list.value = convertToFrontendFormat(subscriptions)
        
        console.log('📄 订阅配置加载完成：', { count: list.value.length })
      } catch (error) {
        console.error('从后端加载订阅配置失败:', error)
        // 加载失败时使用默认值
        list.value = []
      }
    }

    // 保存配置到后端
    const saveToBackend = async () => {
      try {
        await storageService.updateSubscriptions(convertToBackendFormat(list.value))
        console.log('✅ 订阅配置已保存到后端')
      } catch (error) {
        console.error('保存订阅配置到后端失败:', error)
      }
    }

    const add = async (
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
        isManual,
        manualContent,
        useOriginalConfig,
      })

      // 保存到后端
      await saveToBackend()
    }

    // 更新订阅
    const update = async (index: number, updates: Partial<FrontendSubscription>) => {
      if (index >= 0 && index < list.value.length) {
        list.value[index] = { ...list.value[index], ...updates }
        await saveToBackend()
      }
    }

    // 删除订阅
    const remove = async (index: number) => {
      if (index >= 0 && index < list.value.length) {
        list.value.splice(index, 1)
        
        // 如果删除的是当前激活的订阅，需要调整激活索引
        if (activeIndex.value !== null) {
          if (activeIndex.value === index) {
            activeIndex.value = list.value.length > 0 ? 0 : null
          } else if (activeIndex.value > index) {
            activeIndex.value = activeIndex.value - 1
          }
        }
        
        await saveToBackend()
      }
    }

    // 设置激活订阅
    const setActiveIndex = async (index: number | null) => {
      activeIndex.value = index
      // 注意：激活索引可能不需要持久化，这里先不保存
    }

    // 重置所有订阅的加载状态
    const resetLoadingState = async () => {
      if (list.value.length > 0) {
        list.value = list.value.map(item => ({
          ...item,
          isLoading: false
        }))
        await saveToBackend()
      }
    }

    // 设置订阅加载状态
    const setLoadingState = async (index: number, loading: boolean) => {
      if (index >= 0 && index < list.value.length) {
        list.value[index].isLoading = loading
        await saveToBackend()
      }
    }

    // 更新订阅时间
    const updateLastUpdateTime = async (index: number) => {
      if (index >= 0 && index < list.value.length) {
        list.value[index].lastUpdate = Date.now()
        list.value[index].isLoading = false
        await saveToBackend()
      }
    }

    // 清空所有订阅
    const clear = async () => {
      list.value = []
      activeIndex.value = null
      await saveToBackend()
    }

    // 获取当前激活的订阅
    const getActiveSubscription = () => {
      if (activeIndex.value !== null && activeIndex.value >= 0 && activeIndex.value < list.value.length) {
        return list.value[activeIndex.value]
      }
      return null
    }

    // 初始化方法
    const initializeStore = async () => {
      await loadFromBackend()
    }

    return {
      list,
      activeIndex,
      add,
      update,
      remove,
      setActiveIndex,
      resetLoadingState,
      setLoadingState,
      updateLastUpdateTime,
      clear,
      getActiveSubscription,
      initializeStore,
      loadFromBackend,
      saveToBackend,
    }
  },
  // 移除 persist 配置，现在使用后端存储
)