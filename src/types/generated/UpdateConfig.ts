export interface UpdateConfig {
  auto_check: boolean
  last_check: number
  last_version: string | null
  skip_version: string | null
  accept_prerelease: boolean
}
