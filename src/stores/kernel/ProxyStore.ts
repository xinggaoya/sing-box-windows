import { defineStore } from 'pinia'
import { ref } from 'vue'
import { proxyService, type ProxiesData } from '@/services/proxy-service'
import { systemService } from '@/services/system-service'
import { useAppStore, type ProxyMode } from '@/stores/app/AppStore'

export const useProxyStore = defineStore('proxy', () => {
  const appStore = useAppStore()
  const loading = ref(false)
  const proxies = ref<ProxiesData['proxies']>({})

  const fetchProxies = async () => {
    loading.value = true
    try {
      const data = await proxyService.getProxies()
      proxies.value = data.proxies
    } catch (error) {
      console.error('Failed to fetch proxies:', error)
    } finally {
      loading.value = false
    }
  }

  const changeProxy = async (group: string, proxy: string) => {
    try {
      await proxyService.changeProxy(group, proxy)
      await fetchProxies()
    } catch (error) {
      console.error('Failed to change proxy:', error)
      throw error
    }
  }

  const testNodeDelay = async (proxy: string) => {
    try {
      await proxyService.testNodeDelay(proxy)
      await fetchProxies()
      return 0
    } catch (error) {
      console.error('Failed to test node delay:', error)
      throw error
    }
  }

  const testGroupDelay = async (group: string) => {
    try {
      await proxyService.testGroupDelay(group)
      await fetchProxies()
    } catch (error) {
      console.error('Failed to test group delay:', error)
      throw error
    }
  }

  const switchProxyMode = async (targetMode: ProxyMode) => {
    return await proxyService.switchMode(targetMode)
  }

  const setSystemProxy = async () => {
    await proxyService.setSystemProxy(appStore.systemProxyBypass)
  }

  const setTunProxy = async () => {
    await proxyService.setTunProxy({
      ipv4_address: appStore.tunIpv4,
      ipv6_address: appStore.tunIpv6,
      mtu: appStore.tunMtu,
      auto_route: appStore.tunAutoRoute,
      strict_route: appStore.tunStrictRoute,
      stack: appStore.tunStack,
      enable_ipv6: appStore.tunEnableIpv6,
    })
  }

  const checkAdmin = async () => {
    return await systemService.checkAdmin()
  }

  const restartAsAdmin = async () => {
    await systemService.restartAsAdmin()
  }

  const testAllNodesDelay = async () => {
    await proxyService.testAllNodesDelay()
    await fetchProxies()
    return {}
  }

  return {
    loading,
    proxies,
    fetchProxies,
    changeProxy,
    testNodeDelay,
    testGroupDelay,
    switchProxyMode,
    setSystemProxy,
    setTunProxy,
    checkAdmin,
    restartAsAdmin,
    testAllNodesDelay,
  }
})
