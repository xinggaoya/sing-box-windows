import { invokeWithAppContext } from './invoke-client'

export const configApi = {
  saveConfig(config: unknown) {
    return invokeWithAppContext<void>('save_config', { config })
  },

  loadConfig() {
    return invokeWithAppContext<unknown>('load_config')
  },

  importConfig(path: string) {
    return invokeWithAppContext<unknown>('import_config', { path })
  },

  exportConfig(path: string, config: unknown) {
    return invokeWithAppContext<void>('export_config', { path, config })
  },

  resetConfig() {
    return invokeWithAppContext<void>('reset_config')
  },

  updateSingboxPorts(proxyPort: number, apiPort: number) {
    return invokeWithAppContext<void>(
      'update_singbox_ports',
      { proxyPort, apiPort }
    )
  }
}
