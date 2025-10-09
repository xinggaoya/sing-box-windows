import { invoke } from '@tauri-apps/api/core'
import type { AppConfig, ThemeConfig, LocaleConfig, WindowConfig, UpdateConfig, Subscription } from '@/types/database'

// 数据库存储服务 - 替代原有的前端持久化
export class DatabaseService {
  // 应用配置
  static async getAppConfig(): Promise<AppConfig> {
    return await invoke('db_get_app_config')
  }

  static async saveAppConfig(config: AppConfig): Promise<void> {
    return await invoke('db_save_app_config', { config })
  }

  // 主题配置
  static async getThemeConfig(): Promise<ThemeConfig> {
    return await invoke('db_get_theme_config')
  }

  static async saveThemeConfig(config: ThemeConfig): Promise<void> {
    return await invoke('db_save_theme_config', { config })
  }

  // 语言配置
  static async getLocaleConfig(): Promise<LocaleConfig> {
    return await invoke('db_get_locale_config')
  }

  static async saveLocaleConfig(config: LocaleConfig): Promise<void> {
    return await invoke('db_save_locale_config', { config })
  }

  // 窗口配置
  static async getWindowConfig(): Promise<WindowConfig> {
    return await invoke('db_get_window_config')
  }

  static async saveWindowConfig(config: WindowConfig): Promise<void> {
    return await invoke('db_save_window_config', { config })
  }

  // 更新配置
  static async getUpdateConfig(): Promise<UpdateConfig> {
    return await invoke('db_get_update_config')
  }

  static async saveUpdateConfig(config: UpdateConfig): Promise<void> {
    return await invoke('db_save_update_config', { config })
  }

  // 订阅数据
  static async getSubscriptions(): Promise<Subscription[]> {
    return await invoke('db_get_subscriptions')
  }

  static async saveSubscriptions(subscriptions: Subscription[]): Promise<void> {
    return await invoke('db_save_subscriptions', { subscriptions })
  }
}