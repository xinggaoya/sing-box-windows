/**
 * 后端存储服务
 * 替代 Tauri Store，将数据存储到后端
 */
import { invoke } from '@tauri-apps/api/core'

export interface AppConfig {
  auto_start_kernel: boolean
  prefer_ipv6: boolean
  proxy_port: number
  api_port: number
  proxy_mode: 'system' | 'tun' | 'manual'
  tray_instance_id: string | null
}

export interface ThemeConfig {
  is_dark: boolean
}

export interface LocaleConfig {
  locale: 'auto' | 'zh-CN' | 'en-US' | 'ru-RU' | 'ja-JP'
}

export interface WindowConfig {
  is_visible: boolean
  is_fullscreen: boolean
  is_maximized: boolean
  last_visible_path: string
}

export interface UpdateConfig {
  app_version: string
  auto_check_update: boolean
  skip_version: string | null
  accept_prerelease: boolean
}

export interface Subscription {
  name: string
  url: string
  is_loading: boolean
  last_update: number | null
  is_manual: boolean
  manual_content: string | null
  use_original_config: boolean
}

export interface VersionInfo {
  version: string
  meta: boolean
  premium: boolean
  environment?: string
  tags?: string[]
  revision?: string
  cgo?: string
}

export interface KernelInfo {
  version: VersionInfo | null
  new_version: string | null
}

export interface AppState {
  app_config: AppConfig
  theme_config: ThemeConfig
  locale_config: LocaleConfig
  window_config: WindowConfig
  update_config: UpdateConfig
  subscriptions: Subscription[]
  kernel_info: KernelInfo
}

class StorageService {
  /**
   * 加载完整的应用状态
   */
  async loadState(): Promise<AppState> {
    return await invoke('load_state')
  }

  /**
   * 保存完整的应用状态
   */
  async saveState(state: AppState): Promise<void> {
    await invoke('save_state', { state })
  }

  /**
   * 获取应用配置
   */
  async getAppConfig(): Promise<AppConfig> {
    return await invoke('get_app_config')
  }

  /**
   * 更新应用配置
   */
  async updateAppConfig(updates: Partial<AppConfig>): Promise<void> {
    await invoke('update_app_config', { updates })
  }

  /**
   * 获取主题配置
   */
  async getThemeConfig(): Promise<ThemeConfig> {
    return await invoke('get_theme_config')
  }

  /**
   * 更新主题配置
   */
  async updateThemeConfig(isDark: boolean): Promise<void> {
    await invoke('update_theme_config', { isDark })
  }

  /**
   * 获取语言配置
   */
  async getLocaleConfig(): Promise<LocaleConfig> {
    return await invoke('get_locale_config')
  }

  /**
   * 更新语言配置
   */
  async updateLocaleConfig(locale: string): Promise<void> {
    await invoke('update_locale_config', { locale })
  }

  /**
   * 获取窗口配置
   */
  async getWindowConfig(): Promise<WindowConfig> {
    return await invoke('get_window_config')
  }

  /**
   * 更新窗口配置
   */
  async updateWindowConfig(updates: Partial<WindowConfig>): Promise<void> {
    await invoke('update_window_config', { updates })
  }

  /**
   * 获取更新配置
   */
  async getUpdateConfig(): Promise<UpdateConfig> {
    return await invoke('get_update_config')
  }

  /**
   * 更新更新配置
   */
  async updateUpdateConfig(updates: Partial<UpdateConfig>): Promise<void> {
    await invoke('update_update_config', { updates })
  }

  /**
   * 获取订阅列表
   */
  async getSubscriptions(): Promise<Subscription[]> {
    return await invoke('get_subscriptions')
  }

  /**
   * 更新订阅列表
   */
  async updateSubscriptions(subscriptions: Subscription[]): Promise<void> {
    await invoke('update_subscriptions', { subscriptions })
  }

  /**
   * 获取内核信息
   */
  async getKernelInfo(): Promise<KernelInfo> {
    return await invoke('get_kernel_info')
  }

  /**
   * 更新内核信息
   */
  async updateKernelInfo(updates: Partial<KernelInfo>): Promise<void> {
    await invoke('update_kernel_info', { updates })
  }

  /**
   * 重置所有状态
   */
  async resetState(): Promise<void> {
    await invoke('reset_state')
  }

  /**
   * 备份状态
   */
  async backupState(backupPath: string): Promise<void> {
    await invoke('backup_state', { backupPath })
  }

  /**
   * 恢复状态
   */
  async restoreState(backupPath: string): Promise<void> {
    await invoke('restore_state', { backupPath })
  }
}

export const storageService = new StorageService()
export default storageService