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
  tunAutoRoute: boolean
  tunStrictRoute: boolean
  singboxDefaultProxyOutbound: string
  singboxDownloadDetour: string
  singboxBlockAds: boolean
  singboxDnsHijack: boolean
  singboxEnableAppGroups: boolean
  singboxDnsProxy: string
  singboxDnsCn: string
  singboxDnsResolver: string
  singboxUrltestUrl: string
  saveToBackend: () => Promise<void>
}

interface UseAdvancedSettingsFormOptions {
  appStore: AppStoreLike
  message: MessageApiLike
  t: (key: string) => string
}

export const useAdvancedSettingsForm = (options: UseAdvancedSettingsFormOptions) => {
  const savingAdvanced = ref(false)
  const proxyAdvancedForm = reactive({
    systemProxyBypass: '',
    tunMtu: 9000,
    tunStack: 'mixed' as 'system' | 'gvisor' | 'mixed',
    tunEnableIpv6: false,
    tunAutoRoute: true,
    tunStrictRoute: true,
  })

  const savingSingboxProfile = ref(false)
  const singboxProfileForm = reactive({
    defaultProxyOutbound: 'manual' as 'manual' | 'auto',
    downloadDetour: 'manual' as 'manual' | 'direct',
    blockAds: true,
    dnsHijack: true,
    enableAppGroups: true,
    dnsProxy: '',
    dnsCn: '',
    dnsResolver: '',
    urltestUrl: '',
  })

  const defaultOutboundOptions = computed(() => [
    { label: options.t('setting.singboxProfile.outboundManual'), value: 'manual' },
    { label: options.t('setting.singboxProfile.outboundAuto'), value: 'auto' },
  ])

  const downloadDetourOptions = computed(() => [
    { label: options.t('setting.singboxProfile.detourManual'), value: 'manual' },
    { label: options.t('setting.singboxProfile.detourDirect'), value: 'direct' },
  ])

  watch(
    () => options.appStore.isDataRestored,
    (restored) => {
      if (!restored) return

      proxyAdvancedForm.systemProxyBypass = options.appStore.systemProxyBypass
      proxyAdvancedForm.tunMtu = options.appStore.tunMtu
      proxyAdvancedForm.tunStack = options.appStore.tunStack as 'system' | 'gvisor' | 'mixed'
      proxyAdvancedForm.tunEnableIpv6 = options.appStore.tunEnableIpv6
      proxyAdvancedForm.tunAutoRoute = options.appStore.tunAutoRoute
      proxyAdvancedForm.tunStrictRoute = options.appStore.tunStrictRoute

      singboxProfileForm.defaultProxyOutbound = options.appStore
        .singboxDefaultProxyOutbound as 'manual' | 'auto'
      singboxProfileForm.downloadDetour = options.appStore.singboxDownloadDetour as
        | 'manual'
        | 'direct'
      singboxProfileForm.blockAds = options.appStore.singboxBlockAds
      singboxProfileForm.dnsHijack = options.appStore.singboxDnsHijack
      singboxProfileForm.enableAppGroups = options.appStore.singboxEnableAppGroups
      singboxProfileForm.dnsProxy = options.appStore.singboxDnsProxy
      singboxProfileForm.dnsCn = options.appStore.singboxDnsCn
      singboxProfileForm.dnsResolver = options.appStore.singboxDnsResolver
      singboxProfileForm.urltestUrl = options.appStore.singboxUrltestUrl
    },
    { immediate: true },
  )

  const saveProxyAdvancedSettings = async () => {
    savingAdvanced.value = true
    try {
      options.appStore.systemProxyBypass = proxyAdvancedForm.systemProxyBypass
      options.appStore.tunMtu = proxyAdvancedForm.tunMtu
      options.appStore.tunAutoRoute = proxyAdvancedForm.tunAutoRoute
      options.appStore.tunStrictRoute = proxyAdvancedForm.tunStrictRoute
      options.appStore.tunStack = proxyAdvancedForm.tunStack
      options.appStore.tunEnableIpv6 = proxyAdvancedForm.tunEnableIpv6

      await options.appStore.saveToBackend()
      options.message.success(options.t('common.saveSuccess'))
    } catch {
      options.message.error(options.t('common.saveFailed'))
    } finally {
      savingAdvanced.value = false
    }
  }

  const saveSingboxProfileSettings = async () => {
    try {
      savingSingboxProfile.value = true

      options.appStore.singboxDefaultProxyOutbound = singboxProfileForm.defaultProxyOutbound
      options.appStore.singboxDownloadDetour = singboxProfileForm.downloadDetour
      options.appStore.singboxBlockAds = singboxProfileForm.blockAds
      options.appStore.singboxDnsHijack = singboxProfileForm.dnsHijack
      options.appStore.singboxEnableAppGroups = singboxProfileForm.enableAppGroups
      options.appStore.singboxDnsProxy =
        singboxProfileForm.dnsProxy.trim() || options.appStore.singboxDnsProxy
      options.appStore.singboxDnsCn =
        singboxProfileForm.dnsCn.trim() || options.appStore.singboxDnsCn
      options.appStore.singboxDnsResolver =
        singboxProfileForm.dnsResolver.trim() || options.appStore.singboxDnsResolver
      options.appStore.singboxUrltestUrl =
        singboxProfileForm.urltestUrl.trim() || options.appStore.singboxUrltestUrl

      await options.appStore.saveToBackend()
      options.message.success(options.t('common.saveSuccess'))
    } catch (error) {
      console.error('保存 sing-box 配置生成高级选项失败:', error)
      options.message.error(options.t('common.saveFailed'))
    } finally {
      savingSingboxProfile.value = false
    }
  }

  return {
    savingAdvanced,
    proxyAdvancedForm,
    savingSingboxProfile,
    singboxProfileForm,
    defaultOutboundOptions,
    downloadDetourOptions,
    saveProxyAdvancedSettings,
    saveSingboxProfileSettings,
  }
}
