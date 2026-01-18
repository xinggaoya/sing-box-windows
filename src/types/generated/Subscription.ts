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
}
