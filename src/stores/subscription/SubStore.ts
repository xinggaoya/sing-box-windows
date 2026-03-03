import { ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { DatabaseService } from '@/services/database-service'
import type { Subscription } from '@/types/database'
import mitt from '@/utils/mitt'
import { DEFAULT_AUTO_UPDATE_MINUTES, type FrontendSubscription } from './types'

export const useSubStore = defineStore(
  'sub',
  () => {
    const list = ref<FrontendSubscription[]>([])
    const activeIndex = ref<number | null>(null)

    // 从数据库格式转换为前端格式
    const convertToFrontendFormat = (backendSubs: Subscription[]): FrontendSubscription[] => {
      return backendSubs.map((sub) => ({
        name: sub.name,
        url: sub.url,
        isLoading: sub.is_loading,
        lastUpdate: sub.last_update || undefined,
        isManual: sub.is_manual,
        manualContent: sub.manual_content || undefined,
        useOriginalConfig: sub.use_original_config,
        configPath: sub.config_path || undefined,
        backupPath: sub.backup_path || undefined,
        autoUpdateIntervalMinutes: sub.auto_update_interval_minutes ?? DEFAULT_AUTO_UPDATE_MINUTES,
        subscriptionUpload: sub.subscription_upload ?? undefined,
        subscriptionDownload: sub.subscription_download ?? undefined,
        subscriptionTotal: sub.subscription_total ?? undefined,
        subscriptionExpire: sub.subscription_expire ?? undefined,
        autoUpdateFailCount: sub.auto_update_fail_count ?? undefined,
        lastAutoUpdateAttempt: sub.last_auto_update_attempt ?? undefined,
        lastAutoUpdateError: sub.last_auto_update_error ?? undefined,
        lastAutoUpdateErrorType: sub.last_auto_update_error_type ?? undefined,
        lastAutoUpdateBackoffUntil: sub.last_auto_update_backoff_until ?? undefined,
      }))
    }

    // 从前端格式转换为数据库格式
    const convertToBackendFormat = (frontendSubs: FrontendSubscription[]): Subscription[] => {
      return frontendSubs.map((sub) => ({
        name: sub.name,
        url: sub.url,
        is_loading: sub.isLoading,
        last_update: sub.lastUpdate || null,
        is_manual: sub.isManual,
        manual_content: sub.manualContent || null,
        use_original_config: sub.useOriginalConfig,
        config_path: sub.configPath || undefined,
        backup_path: sub.backupPath || undefined,
        auto_update_interval_minutes: sub.autoUpdateIntervalMinutes ?? DEFAULT_AUTO_UPDATE_MINUTES,
        subscription_upload: sub.subscriptionUpload ?? null,
        subscription_download: sub.subscriptionDownload ?? null,
        subscription_total: sub.subscriptionTotal ?? null,
        subscription_expire: sub.subscriptionExpire ?? null,
        auto_update_fail_count: sub.autoUpdateFailCount ?? null,
        last_auto_update_attempt: sub.lastAutoUpdateAttempt ?? null,
        last_auto_update_error: sub.lastAutoUpdateError ?? null,
        last_auto_update_error_type: sub.lastAutoUpdateErrorType ?? null,
        last_auto_update_backoff_until: sub.lastAutoUpdateBackoffUntil ?? null,
      }))
    }

    // 从数据库加载数据
    const loadFromBackend = async () => {
      try {
        // 订阅列表与 AppConfig 分开存储：
        // - 订阅列表：包含每条订阅对应的 configPath
        // - AppConfig.active_config_path：内核实际会读取的"生效配置路径"
        // 因此这里以 active_config_path 作为权威来源来恢复高亮，避免索引漂移造成前端/内核不一致。
        const [subscriptions, appConfig, savedActiveIndex] = await Promise.all([
          DatabaseService.getSubscriptions(),
          DatabaseService.getAppConfig(),
          DatabaseService.getActiveIndex().catch(() => null),
        ])

        // 更新响应式状态
        list.value = convertToFrontendFormat(subscriptions)

        const activePath = appConfig.active_config_path || null

        // 1) 优先使用 active_config_path（更稳定）
        if (activePath) {
          const matchIndex = list.value.findIndex((item) => item.configPath === activePath)
          if (matchIndex >= 0) {
            activeIndex.value = matchIndex

            // 同步修正后端保存的 active_subscription_index，避免后续刷新又被旧索引覆盖高亮
            if (savedActiveIndex !== matchIndex) {
              try {
                await DatabaseService.saveActiveIndex(matchIndex)
              } catch {
                // ignore
              }
            }
          } else {
            // active_config_path 指向的配置不在订阅列表中（可能是默认配置/自定义配置）
            activeIndex.value = null
          }
        }
        // 2) 若没有 active_config_path，则回退读取历史索引
        else if (
          savedActiveIndex !== null &&
          savedActiveIndex >= 0 &&
          savedActiveIndex < list.value.length
        ) {
          activeIndex.value = savedActiveIndex
        } else {
          activeIndex.value = null
        }
      } catch {
        // 加载失败时使用默认值
        list.value = []
        activeIndex.value = null
      }
    }

    // 保存配置到数据库
    const saveToBackend = async () => {
      try {
        await DatabaseService.saveSubscriptions(convertToBackendFormat(list.value))
      } catch {
        // 保存失败时静默处理
      }
    }

    const add = async (
      name: string,
      url: string,
      isManual: boolean = false,
      manualContent?: string,
      useOriginalConfig: boolean = false,
      configPath?: string,
      autoUpdateIntervalMinutes: number = DEFAULT_AUTO_UPDATE_MINUTES,
    ) => {
      list.value.push({
        name,
        url,
        isLoading: false,
        isManual,
        manualContent,
        useOriginalConfig,
        configPath,
        autoUpdateIntervalMinutes,
      })

      // 保存会在 watch 中自动处理
    }

    // 更新订阅
    const update = async (index: number, updates: Partial<FrontendSubscription>) => {
      if (index >= 0 && index < list.value.length) {
        list.value[index] = { ...list.value[index], ...updates }
        // 保存会在 watch 中自动处理
      }
    }

    // 删除订阅
    const remove = async (index: number) => {
      if (index >= 0 && index < list.value.length) {
        list.value.splice(index, 1)
        // 保存会在 watch 中自动处理
      }
    }

    // 设置激活订阅
    const setActiveIndex = async (index: number | null) => {
      activeIndex.value = index
      // 持久化激活索引到本地存储
      try {
        await DatabaseService.saveActiveIndex(index)
      } catch {
        // 保存失败时静默处理
      }
    }

    // 重置所有订阅的加载状态
    const resetLoadingState = async () => {
      if (list.value.length > 0) {
        list.value = list.value.map((item) => ({
          ...item,
          isLoading: false,
        }))
        // 保存会在 watch 中自动处理
      }
    }

    // 设置订阅加载状态
    const setLoadingState = async (index: number, loading: boolean) => {
      if (index >= 0 && index < list.value.length) {
        list.value[index].isLoading = loading
        // 保存会在 watch 中自动处理
      }
    }

    // 更新订阅时间
    const updateLastUpdateTime = async (index: number) => {
      if (index >= 0 && index < list.value.length) {
        list.value[index].lastUpdate = Date.now()
        list.value[index].isLoading = false
        // 保存会在 watch 中自动处理
      }
    }

    // 清空所有订阅
    const clear = async () => {
      list.value = []
      activeIndex.value = null
      // 保存会在 watch 中自动处理
    }

    // 获取当前激活的订阅
    const getActiveSubscription = () => {
      if (
        activeIndex.value !== null &&
        activeIndex.value >= 0 &&
        activeIndex.value < list.value.length
      ) {
        return list.value[activeIndex.value]
      }
      return null
    }

    // 标记是否正在初始化
    let isInitializing = false
    let hasInitialized = false
    let initializePromise: Promise<void> | null = null

    // 监听订阅列表变化并自动保存到数据库
    watch(
      list,
      async () => {
        // 初始化期间不保存
        if (isInitializing) return
        await saveToBackend()
      },
      { deep: true },
    )

    // 初始化方法
    const initializeStore = async () => {
      if (hasInitialized) {
        return
      }
      if (initializePromise) {
        return initializePromise
      }

      initializePromise = (async () => {
        isInitializing = true
        await loadFromBackend()
        // 等待一下确保数据加载完成
        await new Promise((resolve) => setTimeout(resolve, 100))
        isInitializing = false
        hasInitialized = true

        // 监听后端自动刷新事件，重新拉取列表/激活状态
        mitt.on('subscription-updated', async () => {
          await loadFromBackend()
        })
      })()

      return initializePromise
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
