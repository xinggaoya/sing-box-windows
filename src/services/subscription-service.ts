import { invokeWithAppContext } from './invoke-client'

export interface SubscriptionPersistOptions {
  fileName?: string
  configPath?: string
  applyRuntime?: boolean
}

export interface SetActiveConfigOptions {
  useOriginalConfig?: boolean
}

interface BackendSubscriptionPersistResult {
  config_path: string
  subscription_upload?: number | null
  subscription_download?: number | null
  subscription_total?: number | null
  subscription_expire?: number | null
  /** 是否经过 sing-box 内核的 `check` 校验；缺省视为已校验（兼容老后端） */
  validated?: boolean
  /** 校验被跳过时的原因（仅在 `validated = false` 时有意义） */
  validation_skip_reason?: string | null
}

export interface SubscriptionPersistResult {
  configPath: string
  subscriptionUpload?: number
  subscriptionDownload?: number
  subscriptionTotal?: number
  subscriptionExpire?: number
  /** 是否经过 sing-box 内核的 `check` 校验 */
  validated: boolean
  /** 校验被跳过时的原因（仅在 `validated = false` 时有意义） */
  validationSkipReason?: string
}

const mapPersistResult = (result: BackendSubscriptionPersistResult): SubscriptionPersistResult => ({
  configPath: result.config_path,
  subscriptionUpload: result.subscription_upload ?? undefined,
  subscriptionDownload: result.subscription_download ?? undefined,
  subscriptionTotal: result.subscription_total ?? undefined,
  subscriptionExpire: result.subscription_expire ?? undefined,
  validated: result.validated ?? true,
  validationSkipReason: result.validation_skip_reason ?? undefined,
})

export const subscriptionService = {
  downloadSubscription(url: string, useOriginalConfig: boolean, options: SubscriptionPersistOptions = {}) {
    return invokeWithAppContext<BackendSubscriptionPersistResult>(
      'download_subscription',
      {
        url,
        useOriginalConfig,
        fileName: options.fileName,
        configPath: options.configPath,
        applyRuntime: options.applyRuntime,
      },
      { withProxyPort: true, withApiPort: true },
    ).then(mapPersistResult)
  },

  addManualSubscription(content: string, useOriginalConfig: boolean, options: SubscriptionPersistOptions = {}) {
    return invokeWithAppContext<BackendSubscriptionPersistResult>(
      'add_manual_subscription',
      {
        content,
        useOriginalConfig,
        fileName: options.fileName,
        configPath: options.configPath,
        applyRuntime: options.applyRuntime,
      },
      { withProxyPort: true, withApiPort: true },
    ).then(mapPersistResult)
  },

  setActiveConfig(configPath: string | null, options: SetActiveConfigOptions = {}) {
    return invokeWithAppContext<void>(
      'set_active_config_path',
      {
        configPath,
        useOriginalConfig: options.useOriginalConfig,
      },
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
