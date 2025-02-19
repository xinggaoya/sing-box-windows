import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tauriApi } from '@/services/tauri-api'

export const useAppStore = defineStore('app', () => {
  // 应用运行状态
  const isRunning = ref(false)

  // 代理模式
  const mode = ref<'system' | 'tun'>('system')

  // 自动启动设置
  const autoStart = ref(false)
  const autoStartKernel = ref(false)

  // 切换代理模式
  const switchProxyMode = async (targetMode: 'system' | 'tun') => {
    try {
      if (targetMode === 'system') {
        await tauriApi.proxy.setSystemProxy()
        mode.value = 'system'
      } else {
        const isAdmin = await tauriApi.proxy.checkAdmin()
        if (!isAdmin) {
          await tauriApi.proxy.restartAsAdmin()
          return false
        }
        await tauriApi.proxy.setTunProxy()
        mode.value = 'tun'
      }
      return true
    } catch (error) {
      console.error('切换代理模式失败:', error)
      return false
    }
  }

  return {
    isRunning,
    mode,
    autoStart,
    autoStartKernel,
    switchProxyMode,
  }
})
