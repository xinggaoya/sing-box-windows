import { nextTick, ref, watch, type Ref } from 'vue'
import { DatabaseService } from '@/services/database-service'
import type { AppConfig } from '@/types/generated/AppConfig'

export interface PersistenceState {
  systemProxyEnabled: Ref<boolean>
  tunEnabled: Ref<boolean>
  autoStartKernel: Ref<boolean>
  autoStartApp: Ref<boolean>
  autoHideToTrayOnAutostart: Ref<boolean>
  trayCloseBehavior: Ref<string>
  preferIpv6: Ref<boolean>
  allowLanAccess: Ref<boolean>
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
  tunRouteExcludeAddress: Ref<string[] | null>
  activeConfigPath: Ref<string | null>
  installedKernelVersion: Ref<string | null>
  singboxDnsProxy: Ref<string>
  singboxDnsCn: Ref<string>
  singboxDnsResolver: Ref<string>
  singboxUrltestUrl: Ref<string>
  singboxDefaultProxyOutbound: Ref<string>
  singboxBlockAds: Ref<boolean>
  singboxDownloadDetour: Ref<string>
  singboxDnsHijack: Ref<boolean>
  singboxFakeDnsEnabled: Ref<boolean>
  singboxFakeDnsIpv4Range: Ref<string>
  singboxFakeDnsIpv6Range: Ref<string>
  singboxFakeDnsFilterMode: Ref<string>
  singboxEnableAppGroups: Ref<boolean>
  tunSelfHealEnabled: Ref<boolean>
  tunSelfHealCooldownSecs: Ref<number>
  // === sing-box 1.14 升级新增字段 ===
  kernelUpdateTrack: Ref<string>
  singboxDnsOptimisticCache: Ref<boolean>
  singboxDnsTimeout: Ref<string>
  singboxDnsUseMdns: Ref<boolean>
  singboxEnableTlsSpoof: Ref<boolean>
  tunDnsMode: Ref<string>
  tunIncludeMacs: Ref<string[]>
  tunExcludeMacs: Ref<string[]>
  hysteria2DisableChromeParrot: Ref<boolean>
  hysteria2ObfsType: Ref<string>
  clashMode: Ref<string>
  enableWebDashboard: Ref<boolean>
  enableTailscaleEndpoint: Ref<boolean>
  tailscaleRunSshServer: Ref<boolean>
  tailscaleTaildropDirectory: Ref<string>
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
      state.autoHideToTrayOnAutostart.value =
        appConfig.auto_hide_to_tray_on_autostart ?? state.autoHideToTrayOnAutostart.value
      state.trayCloseBehavior.value = appConfig.tray_close_behavior || state.trayCloseBehavior.value
      state.preferIpv6.value = appConfig.prefer_ipv6
      state.allowLanAccess.value = appConfig.allow_lan_access ?? state.allowLanAccess.value
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
      state.tunRouteExcludeAddress.value =
        Array.isArray(appConfig.tun_route_exclude_address) &&
        appConfig.tun_route_exclude_address.length > 0
          ? [...appConfig.tun_route_exclude_address]
          : null

      // sing-box 配置生成高级选项（旧版本数据库可能没有这些字段）
      state.singboxDnsProxy.value = appConfig.singbox_dns_proxy || state.singboxDnsProxy.value
      state.singboxDnsCn.value = appConfig.singbox_dns_cn || state.singboxDnsCn.value
      state.singboxDnsResolver.value =
        appConfig.singbox_dns_resolver || state.singboxDnsResolver.value
      state.singboxUrltestUrl.value = appConfig.singbox_urltest_url || state.singboxUrltestUrl.value
      state.singboxDefaultProxyOutbound.value =
        appConfig.singbox_default_proxy_outbound || state.singboxDefaultProxyOutbound.value
      state.singboxBlockAds.value = appConfig.singbox_block_ads ?? state.singboxBlockAds.value
      state.singboxDownloadDetour.value =
        appConfig.singbox_download_detour || state.singboxDownloadDetour.value
      state.singboxDnsHijack.value = appConfig.singbox_dns_hijack ?? state.singboxDnsHijack.value
      state.singboxFakeDnsEnabled.value =
        appConfig.singbox_fake_dns_enabled ?? state.singboxFakeDnsEnabled.value
      state.singboxFakeDnsIpv4Range.value =
        appConfig.singbox_fake_dns_ipv4_range || state.singboxFakeDnsIpv4Range.value
      state.singboxFakeDnsIpv6Range.value =
        appConfig.singbox_fake_dns_ipv6_range || state.singboxFakeDnsIpv6Range.value
      state.singboxFakeDnsFilterMode.value =
        appConfig.singbox_fake_dns_filter_mode || state.singboxFakeDnsFilterMode.value
      state.singboxEnableAppGroups.value =
        appConfig.singbox_enable_app_groups ?? state.singboxEnableAppGroups.value
      state.tunSelfHealEnabled.value =
        appConfig.tun_self_heal_enabled ?? state.tunSelfHealEnabled.value
      state.tunSelfHealCooldownSecs.value =
        appConfig.tun_self_heal_cooldown_secs ?? state.tunSelfHealCooldownSecs.value

      // sing-box 1.14 升级新增字段（与 saveToBackend 的写入列表一一对应；
      // 漏读任意字段都会导致重启后 UI 回退默认值，且下一次自动保存会把
      // 默认值覆盖回数据库）
      state.kernelUpdateTrack.value =
        appConfig.kernel_update_track || state.kernelUpdateTrack.value
      state.singboxDnsOptimisticCache.value =
        appConfig.singbox_dns_optimistic_cache ?? state.singboxDnsOptimisticCache.value
      state.singboxDnsTimeout.value = appConfig.singbox_dns_timeout || state.singboxDnsTimeout.value
      state.singboxDnsUseMdns.value =
        appConfig.singbox_dns_use_mdns ?? state.singboxDnsUseMdns.value
      state.singboxEnableTlsSpoof.value =
        appConfig.singbox_enable_tls_spoof ?? state.singboxEnableTlsSpoof.value
      state.tunDnsMode.value = appConfig.tun_dns_mode || state.tunDnsMode.value
      state.tunIncludeMacs.value = Array.isArray(appConfig.tun_include_macs)
        ? [...appConfig.tun_include_macs]
        : state.tunIncludeMacs.value
      state.tunExcludeMacs.value = Array.isArray(appConfig.tun_exclude_macs)
        ? [...appConfig.tun_exclude_macs]
        : state.tunExcludeMacs.value
      state.hysteria2DisableChromeParrot.value =
        appConfig.hysteria2_disable_chrome_parrot ?? state.hysteria2DisableChromeParrot.value
      state.hysteria2ObfsType.value =
        appConfig.hysteria2_obfs_type || state.hysteria2ObfsType.value
      state.clashMode.value = appConfig.clash_mode || state.clashMode.value
      state.enableWebDashboard.value =
        appConfig.enable_web_dashboard ?? state.enableWebDashboard.value
      state.enableTailscaleEndpoint.value =
        appConfig.enable_tailscale_endpoint ?? state.enableTailscaleEndpoint.value
      state.tailscaleRunSshServer.value =
        appConfig.tailscale_run_ssh_server ?? state.tailscaleRunSshServer.value
      state.tailscaleTaildropDirectory.value =
        appConfig.tailscale_taildrop_directory || state.tailscaleTaildropDirectory.value
    } catch {
      // 加载失败时静默处理
    } finally {
      markDataRestored()
    }
  }

  const saveToBackend = async (options?: { applyRuntime?: boolean }) => {
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
      auto_hide_to_tray_on_autostart: state.autoHideToTrayOnAutostart.value,
      tray_close_behavior: state.trayCloseBehavior.value,
      prefer_ipv6: state.preferIpv6.value,
      allow_lan_access: state.allowLanAccess.value,
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
      tun_route_exclude_address: state.tunRouteExcludeAddress.value?.length
        ? [...state.tunRouteExcludeAddress.value]
        : null,
      active_config_path: state.activeConfigPath.value,
      installed_kernel_version: state.installedKernelVersion.value,
      singbox_dns_proxy: state.singboxDnsProxy.value,
      singbox_dns_cn: state.singboxDnsCn.value,
      singbox_dns_resolver: state.singboxDnsResolver.value,
      singbox_urltest_url: state.singboxUrltestUrl.value,
      singbox_default_proxy_outbound: state.singboxDefaultProxyOutbound.value,
      singbox_block_ads: state.singboxBlockAds.value,
      singbox_download_detour: state.singboxDownloadDetour.value,
      singbox_dns_hijack: state.singboxDnsHijack.value,
      singbox_fake_dns_enabled: state.singboxFakeDnsEnabled.value,
      singbox_fake_dns_ipv4_range: state.singboxFakeDnsIpv4Range.value,
      singbox_fake_dns_ipv6_range: state.singboxFakeDnsIpv6Range.value,
      singbox_fake_dns_filter_mode: state.singboxFakeDnsFilterMode.value,
      singbox_enable_app_groups: state.singboxEnableAppGroups.value,
      tun_self_heal_enabled: state.tunSelfHealEnabled.value,
      tun_self_heal_cooldown_secs: state.tunSelfHealCooldownSecs.value,
      // === sing-box 1.14 升级新增字段 ===
      kernel_update_track: state.kernelUpdateTrack?.value ?? 'stable',
      singbox_dns_optimistic_cache:
        state.singboxDnsOptimisticCache?.value ?? true,
      singbox_dns_timeout: state.singboxDnsTimeout?.value ?? '5s',
      singbox_dns_use_mdns: state.singboxDnsUseMdns?.value ?? true,
      singbox_enable_tls_spoof: state.singboxEnableTlsSpoof?.value ?? false,
      tun_dns_mode: state.tunDnsMode?.value ?? 'hijack',
      tun_include_macs: [...(state.tunIncludeMacs?.value ?? [])],
      tun_exclude_macs: [...(state.tunExcludeMacs?.value ?? [])],
      hysteria2_disable_chrome_parrot:
        state.hysteria2DisableChromeParrot?.value ?? false,
      hysteria2_obfs_type: state.hysteria2ObfsType?.value ?? 'salamander',
      clash_mode: state.clashMode?.value ?? 'rule',
      enable_web_dashboard: state.enableWebDashboard?.value ?? false,
      enable_tailscale_endpoint: state.enableTailscaleEndpoint?.value ?? false,
      tailscale_run_ssh_server: state.tailscaleRunSshServer?.value ?? false,
      tailscale_taildrop_directory:
        state.tailscaleTaildropDirectory?.value ?? 'Taildrop',
    }
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

  const stopAutoSave = watch(
    [
      state.systemProxyEnabled,
      state.tunEnabled,
      state.autoStartKernel,
      state.autoStartApp,
      state.autoHideToTrayOnAutostart,
      state.trayCloseBehavior,
      state.preferIpv6,
      state.allowLanAccess,
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
      state.tunRouteExcludeAddress,
      state.activeConfigPath,
      state.singboxDnsProxy,
      state.singboxDnsCn,
      state.singboxDnsResolver,
      state.singboxUrltestUrl,
      state.singboxDefaultProxyOutbound,
      state.singboxBlockAds,
      state.singboxDownloadDetour,
      state.singboxDnsHijack,
      state.singboxFakeDnsEnabled,
      state.singboxFakeDnsIpv4Range,
      state.singboxFakeDnsIpv6Range,
      state.singboxFakeDnsFilterMode,
      state.singboxEnableAppGroups,
      state.tunSelfHealEnabled,
      state.tunSelfHealCooldownSecs,
      // === sing-box 1.14 升级新增字段（与 loadFromBackend 的读取列表一一对应） ===
      state.kernelUpdateTrack,
      state.singboxDnsOptimisticCache,
      state.singboxDnsTimeout,
      state.singboxDnsUseMdns,
      state.singboxEnableTlsSpoof,
      state.tunDnsMode,
      state.tunIncludeMacs,
      state.tunExcludeMacs,
      state.hysteria2DisableChromeParrot,
      state.hysteria2ObfsType,
      state.clashMode,
      state.enableWebDashboard,
      state.enableTailscaleEndpoint,
      state.tailscaleRunSshServer,
      state.tailscaleTaildropDirectory,
    ],
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
