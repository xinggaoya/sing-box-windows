import { nextTick, ref, watch, type Ref } from 'vue'
import { DatabaseService } from '@/services/database-service'
import type { AppConfig } from '@/types/database'

export interface PersistenceState {
  proxyMode: Ref<string>
  autoStartKernel: Ref<boolean>
  preferIpv6: Ref<boolean>
  proxyPort: Ref<number>
  apiPort: Ref<number>
  trayInstanceId: Ref<string | null>
}

export function createAppPersistence(state: PersistenceState) {
  const isDataRestored = ref(false)
  const isInitializing = ref(false)

  let dataRestorePromise: Promise<void> | null = null
  let dataRestoreResolve: (() => void) | null = null
  let lastSavePromise: Promise<void> | null = null

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
      state.proxyMode.value = appConfig.proxy_mode
      state.autoStartKernel.value = appConfig.auto_start_kernel
      state.preferIpv6.value = appConfig.prefer_ipv6
      state.proxyPort.value = appConfig.proxy_port
      state.apiPort.value = appConfig.api_port
      state.trayInstanceId.value = appConfig.tray_instance_id || null
    } catch (error) {
      console.error('从数据库加载应用配置失败:', error)
    } finally {
      markDataRestored()
    }
  }

  const saveToBackend = async () => {
    try {
      const config: AppConfig = {
        proxy_mode: state.proxyMode.value,
        auto_start_kernel: state.autoStartKernel.value,
        prefer_ipv6: state.preferIpv6.value,
        proxy_port: state.proxyPort.value,
        api_port: state.apiPort.value,
        tray_instance_id: state.trayInstanceId.value
      }
      await DatabaseService.saveAppConfig(config)
      console.log('✅ 应用配置已保存到数据库')
    } catch (error) {
      console.error('保存应用配置到数据库失败:', error)
    }
  }

  const stopAutoSave = watch(
    [state.proxyMode, state.autoStartKernel, state.preferIpv6, state.proxyPort, state.apiPort, state.trayInstanceId],
    async () => {
      if (isInitializing.value) {
        return
      }
      const savePromise = saveToBackend()
      lastSavePromise = savePromise
      await savePromise
    },
    { deep: true }
  )

  const waitForSaveCompletion = async () => {
    await nextTick()
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
