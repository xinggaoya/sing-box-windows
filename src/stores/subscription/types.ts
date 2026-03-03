export const DEFAULT_AUTO_UPDATE_MINUTES = 720 // 12h

// 前端订阅模型（camelCase），与后端 snake_case 模型分离，避免视图层反复转换字段。
export interface FrontendSubscription {
  name: string
  url: string
  isLoading: boolean
  lastUpdate?: number
  isManual: boolean
  manualContent?: string
  useOriginalConfig: boolean
  configPath?: string
  backupPath?: string
  autoUpdateIntervalMinutes?: number
  subscriptionUpload?: number
  subscriptionDownload?: number
  subscriptionTotal?: number
  subscriptionExpire?: number
  autoUpdateFailCount?: number
  lastAutoUpdateAttempt?: number
  lastAutoUpdateError?: string
  lastAutoUpdateErrorType?: string
  lastAutoUpdateBackoffUntil?: number
}
