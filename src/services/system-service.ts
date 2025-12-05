import { invokeWithAppContext } from './invoke-client'

export const systemService = {
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

    checkUpdate(currentVersion?: string, includePrerelease?: boolean) {
        const version = currentVersion || '1.8.2'
        const includePre = includePrerelease || false
        return invokeWithAppContext<unknown>('check_update', {
            currentVersion: version,
            includePrerelease: includePre
        })
    },

    downloadUpdate() {
        return invokeWithAppContext<void>('download_update')
    },

    installUpdate(downloadPath: string) {
        return invokeWithAppContext<void>('install_update', { downloadPath })
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

    waitForNetworkReady(options?: { timeoutMs?: number; checkIntervalMs?: number; strict?: boolean }) {
        return invokeWithAppContext<boolean>('wait_for_network_ready', options, { skipDataRestore: true })
    },

    checkNetworkConnectivity(options?: { strict?: boolean }) {
        return invokeWithAppContext<boolean>('check_network_connectivity', options, { skipDataRestore: true })
    }
}
