import { computed, reactive, ref, watch } from 'vue'

interface MessageApiLike {
  success: (content: string) => void
  error: (content: string) => void
}

interface AppStoreLike {
  isDataRestored: boolean
  systemProxyBypass: string
  tunMtu: number
  tunStack: string
  tunEnableIpv6: boolean
  tunRouteExcludeAddress: string[] | null
  tunAutoRoute: boolean
  tunStrictRoute: boolean
  tunSelfHealEnabled: boolean
  tunSelfHealCooldownSecs: number
  singboxDefaultProxyOutbound: string
  singboxDownloadDetour: string
  singboxBlockAds: boolean
  singboxDnsHijack: boolean
  singboxFakeDnsEnabled: boolean
  singboxFakeDnsIpv4Range: string
  singboxFakeDnsIpv6Range: string
  singboxFakeDnsFilterMode: string
  singboxEnableAppGroups: boolean
  singboxDnsProxy: string
  singboxDnsCn: string
  singboxDnsResolver: string
  singboxUrltestUrl: string
  // === sing-box 1.14 新增字段（实验性） ===
  singboxDnsOptimisticCache: boolean
  singboxDnsTimeout: string
  singboxDnsUseMdns: boolean
  singboxEnableTlsSpoof: boolean
  hysteria2DisableChromeParrot: boolean
  hysteria2ObfsType: string
  clashMode: string
  enableWebDashboard: boolean
  enableTailscaleEndpoint: boolean
  tailscaleRunSshServer: boolean
  tailscaleTaildropDirectory: string
  saveToBackend: (options?: { applyRuntime?: boolean }) => Promise<void>
}

interface UseAdvancedSettingsFormOptions {
  appStore: AppStoreLike
  message: MessageApiLike
  t: (key: string, params?: Record<string, string | number>) => string
}

const IPV4_CIDR_RE =
  /^(25[0-5]|2[0-4]\d|1?\d?\d)(\.(25[0-5]|2[0-4]\d|1?\d?\d)){3}\/([0-9]|[12]\d|3[0-2])$/
const IPV6_CIDR_RE = /^[0-9A-Fa-f:]+\/([0-9]|[1-9]\d|1[01]\d|12[0-8])$/

const isLikelyCidr = (value: string, family: 'ipv4' | 'ipv6') => {
  const trimmed = value.trim()
  if (!trimmed) return false
  return family === 'ipv4' ? IPV4_CIDR_RE.test(trimmed) : IPV6_CIDR_RE.test(trimmed)
}

const isLikelyAnyCidr = (value: string) =>
  isLikelyCidr(value, 'ipv4') || isLikelyCidr(value, 'ipv6')

const parseTunRouteExcludeAddressLines = (value: string) =>
  value
    .split(/\r?\n/)
    .map((line, index) => ({
      line: index + 1,
      value: line.trim(),
    }))
    .filter((entry) => entry.value.length > 0)

const normalizeTunRouteExcludeAddress = (value: string) => {
  const lines = parseTunRouteExcludeAddressLines(value).map((entry) => entry.value)
  return lines.length > 0 ? lines : null
}

export const useAdvancedSettingsForm = (options: UseAdvancedSettingsFormOptions) => {
  const savingAdvanced = ref(false)
  const proxyAdvancedForm = reactive({
    systemProxyBypass: '',
    tunMtu: 9000,
    tunStack: 'mixed' as 'system' | 'gvisor' | 'mixed',
    tunEnableIpv6: false,
    tunRouteExcludeAddressText: '',
    tunAutoRoute: true,
    tunStrictRoute: true,
    tunSelfHealEnabled: true,
    tunSelfHealCooldownSecs: 90,
  })

  const savingSingboxProfile = ref(false)
  const singboxProfileForm = reactive({
    defaultProxyOutbound: 'manual' as 'manual' | 'auto',
    downloadDetour: 'manual' as 'manual' | 'direct',
    blockAds: true,
    dnsHijack: true,
    fakeDnsEnabled: false,
    fakeDnsIpv4Range: '',
    fakeDnsIpv6Range: '',
    fakeDnsFilterMode: 'proxy_only' as 'proxy_only' | 'global_non_cn',
    enableAppGroups: true,
    dnsProxy: '',
    dnsCn: '',
    dnsResolver: '',
    urltestUrl: '',
  })

  // === sing-box 1.14 实验性 / 高级选项 ===
  const savingSingboxExperimental = ref(false)
  const singboxExperimentalForm = reactive({
    dnsOptimisticCache: true,
    dnsTimeout: '5s',
    dnsUseMdns: true,
    enableTlsSpoof: false,
    hysteria2DisableChromeParrot: false,
    hysteria2ObfsType: 'salamander' as 'salamander' | 'gecko',
    clashMode: 'rule' as 'rule' | 'global' | 'direct',
    enableWebDashboard: false,
    enableTailscaleEndpoint: false,
    tailscaleRunSshServer: false,
    tailscaleTaildropDirectory: 'Taildrop',
  })

  const hysteria2ObfsTypeOptions = computed(() => {
    const zh = (options.t('common.locale') as string | undefined) === 'zh-CN' ||
      (typeof navigator !== 'undefined' && navigator.language?.startsWith('zh'))
    return [
      { label: zh ? 'Salamander（默认）' : 'Salamander (default)', value: 'salamander' },
      { label: zh ? 'Gecko（1.14 新增）' : 'Gecko (1.14 new)', value: 'gecko' },
    ]
  })

  const clashModeOptions = computed(() => {
    const zh = (options.t('common.locale') as string | undefined) === 'zh-CN' ||
      (typeof navigator !== 'undefined' && navigator.language?.startsWith('zh'))
    return [
      { label: zh ? '规则（默认）' : 'Rule (default)', value: 'rule' },
      { label: zh ? '全局代理' : 'Global', value: 'global' },
      { label: zh ? '全局直连' : 'Direct', value: 'direct' },
    ]
  })

  const defaultOutboundOptions = computed(() => [
    { label: options.t('setting.singboxProfile.outboundManual'), value: 'manual' },
    { label: options.t('setting.singboxProfile.outboundAuto'), value: 'auto' },
  ])

  const downloadDetourOptions = computed(() => [
    { label: options.t('setting.singboxProfile.detourManual'), value: 'manual' },
    { label: options.t('setting.singboxProfile.detourDirect'), value: 'direct' },
  ])

  const fakeDnsFilterOptions = computed(() => [
    { label: options.t('setting.singboxProfile.fakeDnsFilterProxyOnly'), value: 'proxy_only' },
    { label: options.t('setting.singboxProfile.fakeDnsFilterGlobalNonCn'), value: 'global_non_cn' },
  ])

  watch(
    () => options.appStore.isDataRestored,
    (restored) => {
      if (!restored) return

      proxyAdvancedForm.systemProxyBypass = options.appStore.systemProxyBypass
      proxyAdvancedForm.tunMtu = options.appStore.tunMtu
      proxyAdvancedForm.tunStack = options.appStore.tunStack as 'system' | 'gvisor' | 'mixed'
      proxyAdvancedForm.tunEnableIpv6 = options.appStore.tunEnableIpv6
      proxyAdvancedForm.tunRouteExcludeAddressText =
        options.appStore.tunRouteExcludeAddress?.join('\n') ?? ''
      proxyAdvancedForm.tunAutoRoute = options.appStore.tunAutoRoute
      proxyAdvancedForm.tunStrictRoute = options.appStore.tunStrictRoute
      proxyAdvancedForm.tunSelfHealEnabled = options.appStore.tunSelfHealEnabled
      proxyAdvancedForm.tunSelfHealCooldownSecs = options.appStore.tunSelfHealCooldownSecs

      singboxProfileForm.defaultProxyOutbound = options.appStore
        .singboxDefaultProxyOutbound as 'manual' | 'auto'
      singboxProfileForm.downloadDetour = options.appStore.singboxDownloadDetour as
        | 'manual'
        | 'direct'
      singboxProfileForm.blockAds = options.appStore.singboxBlockAds
      singboxProfileForm.dnsHijack = options.appStore.singboxDnsHijack
      singboxProfileForm.fakeDnsEnabled = options.appStore.singboxFakeDnsEnabled
      singboxProfileForm.fakeDnsIpv4Range = options.appStore.singboxFakeDnsIpv4Range
      singboxProfileForm.fakeDnsIpv6Range = options.appStore.singboxFakeDnsIpv6Range
      singboxProfileForm.fakeDnsFilterMode = options.appStore.singboxFakeDnsFilterMode as
        | 'proxy_only'
        | 'global_non_cn'
      singboxProfileForm.enableAppGroups = options.appStore.singboxEnableAppGroups
      singboxProfileForm.dnsProxy = options.appStore.singboxDnsProxy
      singboxProfileForm.dnsCn = options.appStore.singboxDnsCn
      singboxProfileForm.dnsResolver = options.appStore.singboxDnsResolver
      singboxProfileForm.urltestUrl = options.appStore.singboxUrltestUrl

      // 1.14 实验性字段
      singboxExperimentalForm.dnsOptimisticCache =
        options.appStore.singboxDnsOptimisticCache
      singboxExperimentalForm.dnsTimeout = options.appStore.singboxDnsTimeout
      singboxExperimentalForm.dnsUseMdns = options.appStore.singboxDnsUseMdns
      singboxExperimentalForm.enableTlsSpoof = options.appStore.singboxEnableTlsSpoof
      singboxExperimentalForm.hysteria2DisableChromeParrot =
        options.appStore.hysteria2DisableChromeParrot
      singboxExperimentalForm.hysteria2ObfsType =
        options.appStore.hysteria2ObfsType as 'salamander' | 'gecko'
      singboxExperimentalForm.clashMode = options.appStore.clashMode as
        | 'rule'
        | 'global'
        | 'direct'
      singboxExperimentalForm.enableWebDashboard = options.appStore.enableWebDashboard
      singboxExperimentalForm.enableTailscaleEndpoint =
        options.appStore.enableTailscaleEndpoint
      singboxExperimentalForm.tailscaleRunSshServer = options.appStore.tailscaleRunSshServer
      singboxExperimentalForm.tailscaleTaildropDirectory =
        options.appStore.tailscaleTaildropDirectory
    },
    { immediate: true },
  )

  const saveProxyAdvancedSettings = async () => {
    if (
      proxyAdvancedForm.tunSelfHealEnabled &&
      (proxyAdvancedForm.tunSelfHealCooldownSecs < 15 || proxyAdvancedForm.tunSelfHealCooldownSecs > 600)
    ) {
      options.message.error(options.t('setting.proxyAdvanced.errors.selfHealCooldownInvalid'))
      return
    }

    const invalidRouteExcludeAddressLine = parseTunRouteExcludeAddressLines(
      proxyAdvancedForm.tunRouteExcludeAddressText,
    ).find((entry) => !isLikelyAnyCidr(entry.value))

    if (invalidRouteExcludeAddressLine) {
      options.message.error(
        options.t('setting.proxyAdvanced.errors.tunRouteExcludeAddressInvalidLine', {
          line: invalidRouteExcludeAddressLine.line,
          value: invalidRouteExcludeAddressLine.value,
        }),
      )
      return
    }

    savingAdvanced.value = true
    try {
      options.appStore.systemProxyBypass = proxyAdvancedForm.systemProxyBypass
      options.appStore.tunMtu = proxyAdvancedForm.tunMtu
      options.appStore.tunAutoRoute = proxyAdvancedForm.tunAutoRoute
      options.appStore.tunStrictRoute = proxyAdvancedForm.tunStrictRoute
      options.appStore.tunStack = proxyAdvancedForm.tunStack
      options.appStore.tunEnableIpv6 = proxyAdvancedForm.tunEnableIpv6
      options.appStore.tunRouteExcludeAddress = normalizeTunRouteExcludeAddress(
        proxyAdvancedForm.tunRouteExcludeAddressText,
      )
      options.appStore.tunSelfHealEnabled = proxyAdvancedForm.tunSelfHealEnabled
      options.appStore.tunSelfHealCooldownSecs = proxyAdvancedForm.tunSelfHealCooldownSecs

      await options.appStore.saveToBackend({ applyRuntime: true })
      options.message.success(options.t('common.saveSuccess'))
    } catch {
      options.message.error(options.t('common.saveFailed'))
    } finally {
      savingAdvanced.value = false
    }
  }

  const saveSingboxProfileSettings = async () => {
    if (singboxProfileForm.fakeDnsEnabled) {
      if (!isLikelyCidr(singboxProfileForm.fakeDnsIpv4Range, 'ipv4')) {
        options.message.error(options.t('setting.singboxProfile.fakeDnsIpv4Invalid'))
        return
      }
      if (!isLikelyCidr(singboxProfileForm.fakeDnsIpv6Range, 'ipv6')) {
        options.message.error(options.t('setting.singboxProfile.fakeDnsIpv6Invalid'))
        return
      }
    }

    try {
      savingSingboxProfile.value = true

      options.appStore.singboxDefaultProxyOutbound = singboxProfileForm.defaultProxyOutbound
      options.appStore.singboxDownloadDetour = singboxProfileForm.downloadDetour
      options.appStore.singboxBlockAds = singboxProfileForm.blockAds
      options.appStore.singboxDnsHijack = singboxProfileForm.dnsHijack
      options.appStore.singboxFakeDnsEnabled = singboxProfileForm.fakeDnsEnabled
      options.appStore.singboxFakeDnsIpv4Range =
        singboxProfileForm.fakeDnsIpv4Range.trim() || options.appStore.singboxFakeDnsIpv4Range
      options.appStore.singboxFakeDnsIpv6Range =
        singboxProfileForm.fakeDnsIpv6Range.trim() || options.appStore.singboxFakeDnsIpv6Range
      options.appStore.singboxFakeDnsFilterMode = singboxProfileForm.fakeDnsFilterMode
      options.appStore.singboxEnableAppGroups = singboxProfileForm.enableAppGroups
      options.appStore.singboxDnsProxy =
        singboxProfileForm.dnsProxy.trim() || options.appStore.singboxDnsProxy
      options.appStore.singboxDnsCn =
        singboxProfileForm.dnsCn.trim() || options.appStore.singboxDnsCn
      options.appStore.singboxDnsResolver =
        singboxProfileForm.dnsResolver.trim() || options.appStore.singboxDnsResolver
      options.appStore.singboxUrltestUrl =
        singboxProfileForm.urltestUrl.trim() || options.appStore.singboxUrltestUrl

      await options.appStore.saveToBackend({ applyRuntime: true })
      options.message.success(options.t('common.saveSuccess'))
    } catch (error) {
      console.error('保存 sing-box 配置生成高级选项失败:', error)
      options.message.error(options.t('common.saveFailed'))
    } finally {
      savingSingboxProfile.value = false
    }
  }

  // === sing-box 1.14 实验性 / 高级选项保存 ===
  const saveSingboxExperimentalSettings = async () => {
    savingSingboxExperimental.value = true
    try {
      options.appStore.singboxDnsOptimisticCache = singboxExperimentalForm.dnsOptimisticCache
      options.appStore.singboxDnsTimeout = singboxExperimentalForm.dnsTimeout.trim() || '5s'
      options.appStore.singboxDnsUseMdns = singboxExperimentalForm.dnsUseMdns
      options.appStore.singboxEnableTlsSpoof = singboxExperimentalForm.enableTlsSpoof
      options.appStore.hysteria2DisableChromeParrot =
        singboxExperimentalForm.hysteria2DisableChromeParrot
      options.appStore.hysteria2ObfsType = singboxExperimentalForm.hysteria2ObfsType
      options.appStore.clashMode = singboxExperimentalForm.clashMode
      options.appStore.enableWebDashboard = singboxExperimentalForm.enableWebDashboard
      options.appStore.enableTailscaleEndpoint = singboxExperimentalForm.enableTailscaleEndpoint
      options.appStore.tailscaleRunSshServer = singboxExperimentalForm.tailscaleRunSshServer
      options.appStore.tailscaleTaildropDirectory =
        singboxExperimentalForm.tailscaleTaildropDirectory.trim() || 'Taildrop'

      await options.appStore.saveToBackend({ applyRuntime: true })
      options.message.success(options.t('common.saveSuccess'))
    } catch (error) {
      console.error('保存 sing-box 1.14 实验性选项失败:', error)
      options.message.error(options.t('common.saveFailed'))
    } finally {
      savingSingboxExperimental.value = false
    }
  }

  return {
    savingAdvanced,
    proxyAdvancedForm,
    savingSingboxProfile,
    singboxProfileForm,
    savingSingboxExperimental,
    singboxExperimentalForm,
    defaultOutboundOptions,
    downloadDetourOptions,
    fakeDnsFilterOptions,
    hysteria2ObfsTypeOptions,
    clashModeOptions,
    saveProxyAdvancedSettings,
    saveSingboxProfileSettings,
    saveSingboxExperimentalSettings,
  }
}
