import { invokeWithAppContext } from './invoke-client'

export interface SubscriptionPersistOptions {
  fileName?: string
  configPath?: string
  applyRuntime?: boolean
}

export const subscriptionService = {
  downloadSubscription(url: string, useOriginalConfig: boolean, options: SubscriptionPersistOptions = {}) {
    return invokeWithAppContext<string>(
      'download_subscription',
      {
        url,
        useOriginalConfig,
        fileName: options.fileName,
        configPath: options.configPath,
        applyRuntime: options.applyRuntime,
      },
      { withProxyPort: true, withApiPort: true },
    )
  },

  addManualSubscription(content: string, useOriginalConfig: boolean, options: SubscriptionPersistOptions = {}) {
    return invokeWithAppContext<string>(
      'add_manual_subscription',
      {
        content,
        useOriginalConfig,
        fileName: options.fileName,
        configPath: options.configPath,
        applyRuntime: options.applyRuntime,
      },
      { withProxyPort: true, withApiPort: true },
    )
  },

  setActiveConfig(configPath: string | null) {
    return invokeWithAppContext<void>(
      'set_active_config_path',
      { configPath },
      undefined,
    )
  },

  deleteConfig(configPath: string) {
    return invokeWithAppContext<void>(
      'delete_subscription_config',
      { configPath },
      undefined,
    )
  },

  rollbackConfig(configPath: string) {
    return invokeWithAppContext<string>(
      'rollback_subscription_config',
      { configPath },
      undefined,
    )
  },

  getCurrentConfig() {
    return invokeWithAppContext<unknown>('get_current_config', undefined, { withApiPort: true })
  },
}
