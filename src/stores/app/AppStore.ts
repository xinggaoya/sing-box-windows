import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import type { MessageApi } from 'naive-ui/es/message'
import { kernelService } from '@/services/kernel-service'
import { useAppMessaging } from './composables/messaging'
import { createAppPersistence } from './composables/persistence'
import {
  appConfigRefs,
  createAppConfigFields,
  type TrayCloseBehavior,
} from './composables/appConfigSchema'
import { flushWatchers } from '@/utils/store-init'

// 代理模式类型
export type ProxyMode = 'system' | 'tun' | 'manual'
// TrayCloseBehavior 由 schema 声明（收窄的窄类型字段），此处转出保持既有导入路径可用
export type { TrayCloseBehavior } from './composables/appConfigSchema'

export const useAppStore = defineStore(
  'app',
  () => {
    const messaging = useAppMessaging()

    const setMessageInstance = (instance: MessageApi) => {
      messaging.setMessageInstance(instance)
    }

    const {
      showSuccessMessage,
      showErrorMessage,
      showWarningMessage,
      showInfoMessage,
      clearMessages,
    } = messaging

    // 应用运行状态
    const isRunning = ref(false)
    // 连接中状态（正在启动内核但尚未完成事件流就绪）
    const isConnecting = ref(false)

    // ---------------------------------------------------------------------
    // 可持久化配置字段
    //
    // 字段清单、默认值、后端键名与特殊转换全部由 schema 声明（见 appConfigSchema.ts）：
    // 此前这 53 个字段在 AppStore 声明 / 持久化入参 / store return、
    // persistence 的接口 / load / save / watch 共 7 处手工枚举，
    // 加一个配置项要改 7 个地方且漏改无任何报错。
    // ---------------------------------------------------------------------
    const fields = createAppConfigFields()

    const {
      trayInstanceId,
      systemProxyEnabled,
      tunEnabled,
      autoStartKernel,
      autoStartApp,
      autoHideToTrayOnAutostart,
      trayCloseBehavior,
      preferIpv6,
      allowLanAccess,
      proxyPort,
      apiPort,
      systemProxyBypass,
      tunIpv4,
      tunIpv6,
      tunMtu,
      tunAutoRoute,
      tunStrictRoute,
      tunStack,
      tunEnableIpv6,
      tunRouteExcludeAddress,
      activeConfigPath,
      installedKernelVersion,
      singboxDnsProxy,
      singboxDnsCn,
      singboxDnsResolver,
      singboxUrltestUrl,
      singboxDefaultProxyOutbound,
      singboxBlockAds,
      singboxDownloadDetour,
      singboxDnsHijack,
      singboxFakeDnsEnabled,
      singboxFakeDnsIpv4Range,
      singboxFakeDnsIpv6Range,
      singboxFakeDnsFilterMode,
      singboxEnableAppGroups,
      tunSelfHealEnabled,
      tunSelfHealCooldownSecs,
      kernelUpdateTrack,
      singboxDnsOptimisticCache,
      singboxDnsTimeout,
      singboxDnsUseMdns,
      singboxEnableTlsSpoof,
      tunDnsMode,
      tunIncludeMacs,
      tunExcludeMacs,
      hysteria2DisableChromeParrot,
      hysteria2ObfsType,
      clashMode,
      enableWebDashboard,
      enableTailscaleEndpoint,
      tailscaleRunSshServer,
      tailscaleTaildropDirectory,
    } = appConfigRefs(fields)

    // 向后兼容：从独立开关派生 proxyMode
    const proxyMode = computed<ProxyMode>(() => {
      if (tunEnabled.value) return 'tun'
      if (systemProxyEnabled.value) return 'system'
      return 'manual'
    })

    const {
      isDataRestored,
      startInitialization,
      finishInitialization,
      loadFromBackend,
      saveToBackend,
      waitForDataRestore,
      waitForSaveCompletion,
      markDataRestored,
      stopAutoSave,
    } = createAppPersistence(fields)

    // 同步开机自启设置与系统状态
    const syncAutoStartWithSystem = async () => {
      try {
        // 检查系统实际的自启状态
        const systemEnabled = await isEnabled()

        // 如果数据库中设置为启用，但系统未注册，则重新注册
        if (autoStartApp.value && !systemEnabled) {
          await enable()
        }
        // 如果数据库中设置为禁用，但系统已注册，则取消注册
        else if (!autoStartApp.value && systemEnabled) {
          await disable()
        }
      } catch (error) {
        console.error('同步开机自启状态失败:', error)
        // 不抛出错误，避免影响应用正常启动
      }
    }

    // Store初始化方法
    const initializeStore = async () => {
      startInitialization()

      try {
        await loadFromBackend()

        // 同步开机自启设置与系统状态（修复更新后设置丢失的问题）
        await syncAutoStartWithSystem()

        // 注意：自动启动内核的逻辑现在由 App.vue 统一处理
        // 这里只加载数据，不执行启动逻辑，避免重复

        // 等待 Vue 调度器 flush：由本次加载引发的 watch 回调必须先跑完再解除保存抑制
        // （原先用 setTimeout(100ms) 赌调度时机，慢机器上不够用）
        await flushWatchers()
      } finally {
        finishInitialization()
      }
    }

    // Store清理方法
    const cleanupStore = () => {
      stopAutoSave()
    }

    // 应用运行状态变更
    const setRunningState = (state: boolean) => {
      if (isRunning.value !== state) {
        isRunning.value = state

        if (!state) {
          // 运行停止时同步清理连接中的状态
          isConnecting.value = false
        }

        // 进程状态变更现在通过Pinia响应式系统处理
      }
    }

    // 设置连接中状态
    const setConnectingState = (state: boolean) => {
      isConnecting.value = state
      // 连接状态变更现在通过Pinia响应式系统处理
    }

    // 切换系统开机自启
    const toggleAutoStart = async (enabled: boolean) => {
      try {
        if (enabled) {
          await enable()
        } else {
          await disable()
        }

        // 更新并持久化系统自启动状态
        autoStartApp.value = enabled
        await waitForSaveCompletion()

        // 注意：这里不应该改变 autoStartKernel，因为这是两个独立的设置
        // 系统开机自启 ≠ 启动内核
        // 只保存系统自启动状态，autoStartKernel 的值由用户单独控制
      } catch (error) {
        console.error('切换系统开机自启失败:', error)

        // 检测已知的无害错误，功能实际生效时仍然抛出错误以保持一致性
        const errorMessage = String(error)
        const isHarmlessError =
          errorMessage.includes('os error 2') ||
          errorMessage.includes('system') ||
          errorMessage.includes('No such file or directory')

        if (isHarmlessError) {
          console.warn('Autostart 插件已知的无害错误，功能已生效:', error)
          // 仍然抛出错误，因为调用者需要知道操作完成了
          // 但在上层UI中已经被处理为不显示错误
        }

        throw error
      }
    }

    // 切换自动启动内核设置
    const toggleAutoStartKernel = async (enabled: boolean) => {
      try {
        // 只更新 autoStartKernel 设置
        autoStartKernel.value = enabled
        await waitForSaveCompletion()
      } catch (error) {
        console.error('切换自动启动内核设置失败:', error)
        throw error
      }
    }

    const setTrayCloseBehavior = async (behavior: TrayCloseBehavior) => {
      trayCloseBehavior.value = behavior
      await waitForSaveCompletion()
    }

    const setAutoHideToTrayOnAutostart = async (enabled: boolean) => {
      autoHideToTrayOnAutostart.value = enabled
      await waitForSaveCompletion()
    }

    // 切换系统代理
    const toggleSystemProxy = async (enabled: boolean) => {
      systemProxyEnabled.value = enabled
      await waitForSaveCompletion()
    }

    // 切换TUN模式
    const toggleTun = async (enabled: boolean) => {
      tunEnabled.value = enabled
      await waitForSaveCompletion()
    }

    // 向后兼容：代理模式切换（已弃用，使用toggleSystemProxy和toggleTun）
    const switchProxyMode = async (targetMode: ProxyMode) => {
      switch (targetMode) {
        case 'system':
          systemProxyEnabled.value = true
          tunEnabled.value = false
          break
        case 'tun':
          systemProxyEnabled.value = false
          tunEnabled.value = true
          break
        case 'manual':
          systemProxyEnabled.value = false
          tunEnabled.value = false
          break
      }
      await waitForSaveCompletion()
    }

    // 向后兼容：设置代理模式（已弃用）
    const setProxyMode = async (mode: 'system' | 'tun' | 'manual') => {
      await switchProxyMode(mode)
    }

    // 更新端口配置
    const updatePorts = async (newProxyPort: number, newApiPort: number) => {
      proxyPort.value = newProxyPort
      apiPort.value = newApiPort
      // 保存会在 watch 中自动处理
    }

    const updateProxyAdvancedSettings = async (settings: {
      systemProxyBypass?: string
      tunIpv4?: string
      tunIpv6?: string
      tunMtu?: number
      tunAutoRoute?: boolean
      tunStrictRoute?: boolean
      tunStack?: 'system' | 'gvisor' | 'mixed'
      tunEnableIpv6?: boolean
      tunRouteExcludeAddress?: string[] | null
      tunSelfHealEnabled?: boolean
      tunSelfHealCooldownSecs?: number
    }) => {
      if (typeof settings.systemProxyBypass === 'string') {
        systemProxyBypass.value = settings.systemProxyBypass
      }
      if (typeof settings.tunIpv4 === 'string') {
        tunIpv4.value = settings.tunIpv4
      }
      if (typeof settings.tunIpv6 === 'string') {
        tunIpv6.value = settings.tunIpv6
      }
      if (typeof settings.tunMtu === 'number') {
        tunMtu.value = settings.tunMtu
      }
      if (typeof settings.tunAutoRoute === 'boolean') {
        tunAutoRoute.value = settings.tunAutoRoute
      }
      if (typeof settings.tunStrictRoute === 'boolean') {
        tunStrictRoute.value = settings.tunStrictRoute
      }
      if (settings.tunStack && ['system', 'gvisor', 'mixed'].includes(settings.tunStack)) {
        tunStack.value = settings.tunStack
      }
      if (typeof settings.tunEnableIpv6 === 'boolean') {
        tunEnableIpv6.value = settings.tunEnableIpv6
      }
      if ('tunRouteExcludeAddress' in settings) {
        tunRouteExcludeAddress.value = settings.tunRouteExcludeAddress?.length
          ? [...settings.tunRouteExcludeAddress]
          : null
      }
      if (typeof settings.tunSelfHealEnabled === 'boolean') {
        tunSelfHealEnabled.value = settings.tunSelfHealEnabled
      }
      if (typeof settings.tunSelfHealCooldownSecs === 'number') {
        tunSelfHealCooldownSecs.value = settings.tunSelfHealCooldownSecs
      }

      await waitForSaveCompletion()
    }

    // 同步端口配置到sing-box配置文件
    const syncPortsToSingbox = async () => {
      try {
        await kernelService.updateSingboxPorts(proxyPort.value, apiPort.value)
      } catch (error) {
        console.error('同步端口配置到sing-box失败:', error)
        throw error
      }
    }

    // 设置IPv6偏好
    const setPreferIpv6 = async (prefer: boolean) => {
      preferIpv6.value = prefer
      // 保存会在 watch 中自动处理
    }

    const setAllowLanAccess = async (enabled: boolean) => {
      allowLanAccess.value = enabled
      await waitForSaveCompletion()
    }

    // 设置托盘实例ID
    const setTrayInstanceId = async (instanceId: string | null) => {
      trayInstanceId.value = instanceId
      // 保存会在 watch 中自动处理
    }

    // 设置激活的配置文件路径
    const setActiveConfigPath = async (path: string | null) => {
      activeConfigPath.value = path
      // 变化会自动触发 stopAutoSave 中的 watcher 保存到后端
      await waitForSaveCompletion()
    }

    return {
      isRunning,
      isConnecting,
      isDataRestored,
      trayInstanceId,
      systemProxyEnabled,
      tunEnabled,
      proxyMode,
      autoStartKernel,
      autoStartApp,
      autoHideToTrayOnAutostart,
      trayCloseBehavior,
      preferIpv6,
      allowLanAccess,
      proxyPort,
      apiPort,
      systemProxyBypass,
      tunIpv4,
      tunIpv6,
      tunMtu,
      tunAutoRoute,
      tunStrictRoute,
      tunStack,
      tunEnableIpv6,
      tunRouteExcludeAddress,
      activeConfigPath,
      installedKernelVersion,
      singboxDnsProxy,
      singboxDnsCn,
      singboxDnsResolver,
      singboxUrltestUrl,
      singboxDefaultProxyOutbound,
      singboxBlockAds,
      singboxDownloadDetour,
      singboxDnsHijack,
      singboxFakeDnsEnabled,
      singboxFakeDnsIpv4Range,
      singboxFakeDnsIpv6Range,
      singboxFakeDnsFilterMode,
      singboxEnableAppGroups,
      tunSelfHealEnabled,
      tunSelfHealCooldownSecs,
      // === sing-box 1.14 新增字段 ===
      kernelUpdateTrack,
      singboxDnsOptimisticCache,
      singboxDnsTimeout,
      singboxDnsUseMdns,
      singboxEnableTlsSpoof,
      tunDnsMode,
      tunIncludeMacs,
      tunExcludeMacs,
      hysteria2DisableChromeParrot,
      hysteria2ObfsType,
      clashMode,
      enableWebDashboard,
      enableTailscaleEndpoint,
      tailscaleRunSshServer,
      tailscaleTaildropDirectory,
      setRunningState,
      setConnectingState,
      toggleAutoStart,
      toggleAutoStartKernel,
      setAutoHideToTrayOnAutostart,
      setTrayCloseBehavior,
      toggleSystemProxy,
      toggleTun,
      switchProxyMode,
      setProxyMode,
      setMessageInstance,
      showSuccessMessage,
      showErrorMessage,
      showWarningMessage,
      showInfoMessage,
      clearMessages,
      updatePorts,
      syncPortsToSingbox,
      setPreferIpv6,
      setAllowLanAccess,
      updateProxyAdvancedSettings,
      setTrayInstanceId,
      initializeStore,
      cleanupStore,
      markDataRestored,
      waitForDataRestore,
      syncAutoStartWithSystem,
      loadFromBackend,
      saveToBackend,
      setActiveConfigPath,
    }
  },
  // 移除 persist 配置，现在使用后端存储
)
