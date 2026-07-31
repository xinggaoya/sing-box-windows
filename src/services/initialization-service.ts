/**
 * 应用初始化服务
 * 负责在应用启动时执行一次性初始化逻辑
 */

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

    try {
      // 在这里可以放置未来可能需要的一次性初始化逻辑
      // 例如：加载远程配置、检查数据库等

      this.isInitialized = true

    } catch (error) {
      console.error('❌ 应用初始化失败:', error)
      throw error
    }
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
}

// 导出单例实例
export const initializationService = InitializationService.getInstance()