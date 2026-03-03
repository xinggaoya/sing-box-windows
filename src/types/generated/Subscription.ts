export interface Subscription {
  name: string
  url: string
  is_loading: boolean
  last_update: number | null
  is_manual: boolean
  manual_content: string | null
  use_original_config: boolean
  config_path?: string
  backup_path?: string
  auto_update_interval_minutes?: number
  subscription_upload?: number | null
  subscription_download?: number | null
  subscription_total?: number | null
  subscription_expire?: number | null
  auto_update_fail_count?: number | null
  last_auto_update_attempt?: number | null
  last_auto_update_error?: string | null
  last_auto_update_error_type?: string | null
  last_auto_update_backoff_until?: number | null
}
