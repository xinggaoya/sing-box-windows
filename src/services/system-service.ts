import { invokeWithAppContext } from './invoke-client'

export interface BackupExportResult {
  file_path: string
  created_at: number
  subscriptions_count: number
}

export interface BackupImportResult {
  file_path: string
  valid: boolean
  restored: boolean
  subscriptions_count: number
  warnings: string[]
}

export interface PlatformDetailedInfo {
  os: string
  arch: string
  display_name: string
}

export interface AppUpdateInfo {
  latest_version: string
  download_url: string
  release_page_url: string
  has_update: boolean
  release_notes?: string
  release_date?: string
  file_size?: number
  is_prerelease?: boolean
  supports_in_app_update: boolean
}

export const systemService = {
  pickKernelImportFile() {
    return invokeWithAppContext<string | null>('pick_kernel_import_file', undefined, {
      skipDataRestore: true,
    })
  },

  importKernelExecutable(filePath: string) {
    return invokeWithAppContext<{
      imported_version: string
      restarted: boolean
      backup_path?: string | null
      message: string
    }>('import_kernel_executable', { filePath }, { skipDataRestore: true })
  },

  checkAdmin() {
    return invokeWithAppContext<boolean>('check_admin', undefined, { skipDataRestore: true })
  },

  restartAsAdmin() {
    return invokeWithAppContext<void>('restart_as_admin', undefined, { skipDataRestore: true })
  },

  getSystemUptime() {
    return invokeWithAppContext<number>('get_system_uptime', undefined, { skipDataRestore: true })
  },

  getPlatformInfo() {
    return invokeWithAppContext<string>('get_platform_info', undefined, { skipDataRestore: true })
  },

  getDetailedPlatformInfo() {
    return invokeWithAppContext<PlatformDetailedInfo>('get_detailed_platform_info', undefined, {
      skipDataRestore: true,
    })
  },

  checkUpdate(currentVersion?: string, includePrerelease?: boolean, updateChannel?: string) {
    const version = currentVersion || '0.0.0'
    const includePre = includePrerelease || false
    return invokeWithAppContext<AppUpdateInfo>('check_update', {
      currentVersion: version,
      includePrerelease: includePre,
      updateChannel,
    })
  },

  downloadUpdate() {
    return invokeWithAppContext<void>('download_update')
  },

  downloadAndInstallUpdate(downloadUrl?: string) {
    return invokeWithAppContext<void>('download_and_install_update', { downloadUrl })
  },

  downloadKernel(version?: string) {
    return invokeWithAppContext<void>('download_kernel', { version }, { skipDataRestore: true })
  },

  openDevtools() {
    return invokeWithAppContext<void>('open_devtools')
  },

  waitForNetworkReady(options?: {
    timeoutMs?: number
    checkIntervalMs?: number
    strict?: boolean
  }) {
    return invokeWithAppContext<boolean>('wait_for_network_ready', options, {
      skipDataRestore: true,
    })
  },

  checkNetworkConnectivity(options?: { strict?: boolean }) {
    return invokeWithAppContext<boolean>('check_network_connectivity', options, {
      skipDataRestore: true,
    })
  },

  backupExportSnapshot(filePath?: string) {
    const payload =
      filePath && filePath.trim().length > 0 ? { filePath: filePath.trim() } : undefined
    return invokeWithAppContext<BackupExportResult>('backup_export_snapshot', payload, {
      skipDataRestore: true,
    })
  },

  backupImportSnapshot(options?: { filePath?: string; dryRun?: boolean }) {
    const payload: { filePath?: string; dryRun?: boolean } = {}
    if (options?.filePath && options.filePath.trim().length > 0) {
      payload.filePath = options.filePath.trim()
    }
    if (typeof options?.dryRun === 'boolean') {
      payload.dryRun = options.dryRun
    }
    return invokeWithAppContext<BackupImportResult>(
      'backup_import_snapshot',
      Object.keys(payload).length > 0 ? payload : undefined,
      { skipDataRestore: true },
    )
  },
}
