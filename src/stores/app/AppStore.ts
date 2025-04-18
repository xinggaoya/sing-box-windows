import { defineStore } from 'pinia'
import { ref, onMounted } from 'vue'
import { enable, isEnabled } from '@tauri-apps/plugin-autostart'
import mitt from '@/utils/mitt'

// 代理模式类型
export type ProxyMode = 'system' | 'tun'

export const useAppStore = defineStore(
  'app',
  () => {
    // 应用运行状态
    const isRunning = ref(false)

    // 托盘实例ID - 由TrayStore使用
    const trayInstanceId = ref<string | null>(null)

    // 代理模式
    const proxyMode = ref<ProxyMode>('system')

    // 自动启动设置
    const autoStartApp = ref(false)
    const autoStartKernel = ref(false)

    // IP版本设置
    const preferIpv6 = ref(false)

    onMounted(async () => {
      autoStartApp.value = await isEnabled()
    })

    // 应用运行状态变更
    const setRunningState = (state: boolean) => {
      if (isRunning.value !== state) {
        isRunning.value = state
        // 发送进程状态变更事件
        mitt.emit('process-status')
      }
    }

    // 切换自动启动
    const toggleAutoStart = async (enabled: boolean) => {
      try {
        if (enabled) {
          await enable()
        } else {
          // 这里需要导入disable
          const { disable } = await import('@tauri-apps/plugin-autostart')
          await disable()
        }
        autoStartApp.value = enabled
      } catch (error) {
        console.error('切换自动启动失败:', error)
        throw error
      }
    }

    // 代理模式切换
    const switchProxyMode = async (targetMode: ProxyMode) => {
      // 如果当前模式与目标模式相同，则不需要切换
      if (proxyMode.value === targetMode) return

      // 更新状态
      proxyMode.value = targetMode

      // 发出代理模式变更事件，通知其他组件
      mitt.emit('proxy-mode-changed')
    }

    return {
      isRunning,
      trayInstanceId,
      proxyMode,
      autoStartApp,
      autoStartKernel,
      preferIpv6,
      setRunningState,
      toggleAutoStart,
      switchProxyMode
    }
  },
  {
    persist: true,
  }
)
