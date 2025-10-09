/**
 * 应用初始化服务
 * 负责在应用启动时初始化所有必要的 Store
 */
import { storeManager } from '@/stores/StoreManager'

export class InitializationService {
  private static instance: InitializationService
  private isInitialized = false

  static getInstance(): InitializationService {
    if (!InitializationService.instance) {
      InitializationService.instance = new InitializationService()
    }
    return InitializationService.instance
  }

  /**
   * 初始化整个应用
   */
  async initializeApp(): Promise<void> {
    if (this.isInitialized) {
      console.warn('⚠️ 应用已经初始化过了')
      return
    }

    console.log('🚀 开始初始化应用...')

    try {
      // 1. 初始化 Store 管理器
      console.log('📦 初始化 Store 管理器...')
      await storeManager.initialize()

      // 2. 等待核心 Store 初始化完成
      console.log('⏳ 等待核心 Store 初始化完成...')
      await this.waitForCoreStores()

      // 3. 预加载其他必要的 Store
      console.log('📦 预加载其他 Store...')
      await this.preloadAdditionalStores()

      this.isInitialized = true
      console.log('✅ 应用初始化完成')

    } catch (error) {
      console.error('❌ 应用初始化失败:', error)
      throw error
    }
  }

  /**
   * 等待核心 Store 初始化完成
   */
  private async waitForCoreStores(): Promise<void> {
    const coreStoreTypes: Array<'app' | 'theme' | 'locale' | 'window' | 'update'> = ['app', 'theme', 'locale', 'window', 'update']
    
    // 确保所有核心 Store 都已加载
    await storeManager.preloadStores(coreStoreTypes)

    // 等待一段时间确保所有 Store 的数据都已从后端加载
    await new Promise(resolve => setTimeout(resolve, 1000))

    // 验证核心 Store 是否已正确初始化
    for (const storeType of coreStoreTypes) {
      const store = storeManager.getLoadedStore(storeType)
      if (!store) {
        throw new Error(`核心 Store "${storeType}" 初始化失败`)
      }
    }

    console.log('✅ 所有核心 Store 已成功初始化')
  }

  /**
   * 预加载其他必要的 Store
   */
  private async preloadAdditionalStores(): Promise<void> {
    // 预加载订阅相关 Store
    await storeManager.preloadStores(['subscription'])
    
    // 预加载内核相关 Store（但不要完全初始化，等待用户操作）
    await storeManager.preloadStores(['kernel'])

    console.log('✅ 额外 Store 预加载完成')
  }

  /**
   * 检查应用是否已初始化
   */
  isAppInitialized(): boolean {
    return this.isInitialized
  }

  /**
   * 重置初始化状态（主要用于测试）
   */
  reset(): void {
    this.isInitialized = false
  }

  /**
   * 获取初始化状态报告
   */
  getInitializationReport(): {
    isInitialized: boolean
    storeStats: ReturnType<typeof storeManager.getStats>
    coreStoreStatus: Record<string, boolean>
  } {
    const coreStoreTypes = ['app', 'theme', 'locale', 'window', 'update']
    const coreStoreStatus: Record<string, boolean> = {}

    for (const storeType of coreStoreTypes) {
      coreStoreStatus[storeType] = storeManager.isStoreLoaded(storeType as 'app' | 'theme' | 'locale' | 'window' | 'update')
    }

    return {
      isInitialized: this.isInitialized,
      storeStats: storeManager.getStats(),
      coreStoreStatus,
    }
  }
}

// 导出单例实例
export const initializationService = InitializationService.getInstance()