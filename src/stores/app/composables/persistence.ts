import { nextTick, ref, watch, type Ref } from 'vue'
import { DatabaseService } from '@/services/database-service'
import type { AppConfig } from '@/types/generated/AppConfig'

export interface PersistenceState {
  systemProxyEnabled: Ref<boolean>
  tunEnabled: Ref<boolean>
  autoStartKernel: Ref<boolean>
  autoStartApp: Ref<boolean>
  preferIpv6: Ref<boolean>
  proxyPort: Ref<number>
  apiPort: Ref<number>
  trayInstanceId: Ref<string | null>
  systemProxyBypass: Ref<string>
  tunIpv4: Ref<string>
  tunIpv6: Ref<string>
  tunMtu: Ref<number>
  tunAutoRoute: Ref<boolean>
  tunStrictRoute: Ref<boolean>
  tunStack: Ref<string>
  tunEnableIpv6: Ref<boolean>
  activeConfigPath: Ref<string | null>
  installedKernelVersion: Ref<string | null>
}

export function createAppPersistence(state: PersistenceState) {
  const isDataRestored = ref(false)
  const isInitializing = ref(false)
  const SAVE_DEBOUNCE_MS = 300

  let dataRestorePromise: Promise<void> | null = null
  let dataRestoreResolve: (() => void) | null = null
  let lastSavePromise: Promise<void> | null = null
  let saveTimer: ReturnType<typeof setTimeout> | null = null

  const initializeDataRestore = () => {
    if (!dataRestorePromise) {
      dataRestorePromise = new Promise<void>(resolve => {
        dataRestoreResolve = resolve
      })
    }
  }

  const markDataRestored = () => {
    if (isDataRestored.value) {
      return
    }
    isDataRestored.value = true
    dataRestoreResolve?.()
    dataRestoreResolve = null
  }

  const waitForDataRestore = async (timeout = 5000): Promise<boolean> => {
    if (isDataRestored.value) {
      return true
    }

    if (!dataRestorePromise) {
      console.warn('⚠️ 数据恢复Promise未初始化，可能存在时序问题')
      return false
    }

    try {
      await Promise.race([
        dataRestorePromise,
        new Promise((_, reject) => {
          setTimeout(() => reject(new Error('数据恢复超时')), timeout)
        })
      ])
      return true
    } catch (error) {
      console.error('等待数据恢复失败:', error)
      markDataRestored()
      return false
    }
  }

  const loadFromBackend = async () => {
    try {
      const appConfig = await DatabaseService.getAppConfig()

      state.activeConfigPath.value = appConfig.active_config_path || null
      state.installedKernelVersion.value = appConfig.installed_kernel_version || null

      // 加载新的独立开关字段（如果数据库有的话）
      // 如果数据库没有，则从旧的proxy_mode派生
      if (appConfig.system_proxy_enabled !== undefined) {
        state.systemProxyEnabled.value = appConfig.system_proxy_enabled
      } else {
        // 向后兼容：从旧的proxy_mode派生
        state.systemProxyEnabled.value = appConfig.proxy_mode === 'system'
      }

      if (appConfig.tun_enabled !== undefined) {
        state.tunEnabled.value = appConfig.tun_enabled
      } else {
        // 向后兼容：从旧的proxy_mode派生
        state.tunEnabled.value = appConfig.proxy_mode === 'tun'
      }

      state.autoStartKernel.value = appConfig.auto_start_kernel
      state.autoStartApp.value = appConfig.auto_start_app
      state.preferIpv6.value = appConfig.prefer_ipv6
      state.proxyPort.value = appConfig.proxy_port
      state.apiPort.value = appConfig.api_port
      state.trayInstanceId.value = appConfig.tray_instance_id || null
      state.systemProxyBypass.value = appConfig.system_proxy_bypass
      state.tunIpv4.value = appConfig.tun_ipv4
      state.tunIpv6.value = appConfig.tun_ipv6
      state.tunMtu.value = appConfig.tun_mtu
      state.tunAutoRoute.value = appConfig.tun_auto_route
      state.tunStrictRoute.value = appConfig.tun_strict_route
      state.tunStack.value = appConfig.tun_stack
      state.tunEnableIpv6.value = appConfig.tun_enable_ipv6
    } catch (error) {
      console.error('从数据库加载应用配置失败:', error)
    } finally {
      markDataRestored()
    }
  }

  const saveToBackend = async () => {
    try {
      // Derive proxyMode from independent toggles for backward compatibility
      let proxyMode = 'manual'
      if (state.tunEnabled.value) {
        proxyMode = 'tun'
      } else if (state.systemProxyEnabled.value) {
        proxyMode = 'system'
      }

      const config: AppConfig = {
        proxy_mode: proxyMode,
        system_proxy_enabled: state.systemProxyEnabled.value,
        tun_enabled: state.tunEnabled.value,
        auto_start_kernel: state.autoStartKernel.value,
        auto_start_app: state.autoStartApp.value,
        prefer_ipv6: state.preferIpv6.value,
        proxy_port: state.proxyPort.value,
        api_port: state.apiPort.value,
        tray_instance_id: state.trayInstanceId.value,
        system_proxy_bypass: state.systemProxyBypass.value,
        tun_ipv4: state.tunIpv4.value,
        tun_ipv6: state.tunIpv6.value,
        tun_mtu: state.tunMtu.value,
        tun_auto_route: state.tunAutoRoute.value,
        tun_strict_route: state.tunStrictRoute.value,
        tun_stack: state.tunStack.value,
        tun_enable_ipv6: state.tunEnableIpv6.value,
        active_config_path: state.activeConfigPath.value,
        installed_kernel_version: state.installedKernelVersion.value,
      }
      await DatabaseService.saveAppConfig(config)
      console.log('✅ 应用配置已保存到数据库')
    } catch (error) {
      console.error('保存应用配置到数据库失败:', error)
    }
  }

  const scheduleSave = () => {
    if (isInitializing.value) {
      return
    }
    if (saveTimer) {
      clearTimeout(saveTimer)
    }
    saveTimer = setTimeout(() => {
      const savePromise = saveToBackend()
      lastSavePromise = savePromise
    }, SAVE_DEBOUNCE_MS)
  }

  const stopAutoSave = watch(
    [
      state.systemProxyEnabled,
      state.tunEnabled,
      state.autoStartKernel,
      state.autoStartApp,
      state.preferIpv6,
      state.proxyPort,
      state.apiPort,
      state.trayInstanceId,
      state.systemProxyBypass,
      state.tunIpv4,
      state.tunIpv6,
      state.tunMtu,
      state.tunAutoRoute,
      state.tunStrictRoute,
      state.tunStack,
      state.tunEnableIpv6,
      state.activeConfigPath,
    ],
    scheduleSave,
    { deep: true }
  )

  const waitForSaveCompletion = async () => {
    await nextTick()
    if (saveTimer) {
      await new Promise(resolve => setTimeout(resolve, SAVE_DEBOUNCE_MS))
    }
    if (lastSavePromise) {
      await lastSavePromise
    }
  }

  const startInitialization = () => {
    isInitializing.value = true
    initializeDataRestore()
  }

  const finishInitialization = () => {
    isInitializing.value = false
  }

  return {
    isDataRestored,
    startInitialization,
    finishInitialization,
    loadFromBackend,
    saveToBackend,
    waitForDataRestore,
    waitForSaveCompletion,
    markDataRestored,
    stopAutoSave
  }
}
