import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import { useKernelStore } from '@/stores/kernel/KernelStore'

type KernelStore = ReturnType<typeof useKernelStore>

export type KernelStatusState = 'starting' | 'stopping' | 'running' | 'disconnected' | 'stopped'

/**
 * 将内核运行状态抽象成可复用的展示数据，保证不同组件显示一致
 */
export const useKernelStatus = (store?: KernelStore) => {
  const kernelStore = store ?? useKernelStore()
  const { isRunning, isReady, isStarting, isStopping, isLoading } = storeToRefs(kernelStore)

  const statusState = computed<KernelStatusState>(() => {
    if (isStarting.value) return 'starting'
    if (isStopping.value) return 'stopping'
    if (isRunning.value) {
      return isReady.value ? 'running' : 'disconnected'
    }
    return 'stopped'
  })

  const statusClass = computed(() => {
    switch (statusState.value) {
      case 'starting':
      case 'stopping':
        return 'pending'
      case 'disconnected':
        return 'disconnected'
      case 'running':
        return 'running'
      default:
        return 'stopped'
    }
  })

  return {
    kernelStore,
    statusState,
    statusClass,
    isRunning,
    isReady,
    isStarting,
    isStopping,
    isLoading,
  }
}
