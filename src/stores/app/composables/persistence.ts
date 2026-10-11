import { nextTick, ref, watch } from 'vue'
import { DatabaseService } from '@/services/database-service'
import type { AppConfig } from '@/types/generated/AppConfig'
import {
  appConfigFieldRefs,
  applyAppConfigToFields,
  collectAppConfigFromFields,
  type AppConfigFields,
} from './appConfigSchema'

/**
 * 应用配置的加载 / 保存 / 自动保存。
 *
 * 全部逻辑由 `appConfigSchema` 的字段声明表驱动 —— 这里不再手工枚举任何字段。
 * 新增配置项只需在 `APP_CONFIG_FIELDS` 追加一行。
 */

export function createAppPersistence(fields: AppConfigFields) {
  const isDataRestored = ref(false)
  const isInitializing = ref(false)
  const SAVE_DEBOUNCE_MS = 300

  let dataRestorePromise: Promise<void> | null = null
  let dataRestoreResolve: (() => void) | null = null
  let lastSavePromise: Promise<void> | null = null
  let saveTimer: ReturnType<typeof setTimeout> | null = null

  const initializeDataRestore = () => {
    if (!dataRestorePromise) {
      dataRestorePromise = new Promise<void>((resolve) => {
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
      return false
    }

    try {
      await Promise.race([
        dataRestorePromise,
        new Promise((_, reject) => {
          setTimeout(() => reject(new Error('数据恢复超时')), timeout)
        }),
      ])
      return true
    } catch {
      markDataRestored()
      return false
    }
  }

  const loadFromBackend = async () => {
    try {
      const appConfig = await DatabaseService.getAppConfig()

      // 向后兼容：旧库只有 proxy_mode，需派生回两个独立开关。
      // 后端 AppConfig 类型上这些字段恒存在，这里读 undefined 只是防御旧数据。
      if (appConfig.system_proxy_enabled === undefined) {
        appConfig.system_proxy_enabled = appConfig.proxy_mode === 'system'
      }
      if (appConfig.tun_enabled === undefined) {
        appConfig.tun_enabled = appConfig.proxy_mode === 'tun'
      }

      applyAppConfigToFields(fields, appConfig)
    } catch {
      // 加载失败时静默处理：保持默认值，交由 finally 标记数据已就绪，
      // 避免前端卡在等待状态。
    } finally {
      markDataRestored()
    }
  }

  const saveToBackend = async (options?: { applyRuntime?: boolean }) => {
    const collected = collectAppConfigFromFields(fields)

    // proxy_mode 由两个独立开关派生，仅为兼容旧版本后端/旧配置读取而保留
    const config = {
      ...collected,
      proxy_mode: deriveLegacyProxyMode(
        fields.tunEnabled.ref.value,
        fields.systemProxyEnabled.ref.value,
      ),
    } as AppConfig

    // 前端自动保存仅负责持久化，不直接触发后端运行态重配。
    // 运行态变更统一由显式业务动作触发（如切换代理模式、重启内核、切换订阅）。
    await DatabaseService.saveAppConfig(config, {
      applyRuntime: options?.applyRuntime ?? false,
    })
  }

  const scheduleSave = () => {
    if (isInitializing.value) {
      return
    }
    if (saveTimer) {
      clearTimeout(saveTimer)
    }
    saveTimer = setTimeout(() => {
      const savePromise = saveToBackend().catch(() => undefined)
      lastSavePromise = savePromise
    }, SAVE_DEBOUNCE_MS)
  }

  // 监听列表由字段声明表推导，不再手工枚举
  const stopAutoSave = watch(
    appConfigFieldRefs(fields),
    scheduleSave,
    { deep: true },
  )

  const waitForSaveCompletion = async () => {
    await nextTick()
    if (saveTimer) {
      await new Promise((resolve) => setTimeout(resolve, SAVE_DEBOUNCE_MS))
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
    stopAutoSave,
  }
}

/**
 * 由两个独立开关派生旧版 `proxy_mode`。
 *
 * 保留该字段仅为向后兼容（1.13 及更早的配置只存proxy_mode）。
 */
function deriveLegacyProxyMode(tunEnabled: boolean, systemProxyEnabled: boolean): AppConfig['proxy_mode'] {
  if (tunEnabled) return 'tun'
  if (systemProxyEnabled) return 'system'
  return 'manual'
}
