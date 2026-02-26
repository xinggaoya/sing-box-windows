import type { FrontendSubscription } from '@/stores/subscription/types'

interface SubscriptionAutoUpdateOptions {
  getSubscriptions: () => FrontendSubscription[]
  getActiveIndex: () => number | null
  isKernelRunning: () => boolean
  defaultIntervalMinutes: number
  onRefresh: (index: number, applyRuntime: boolean, silent: boolean) => Promise<void>
}

const AUTO_UPDATE_CHECK_INTERVAL = 30 * 60 * 1000

export const useSubscriptionAutoUpdate = (options: SubscriptionAutoUpdateOptions) => {
  let autoUpdateTimer: number | null = null

  const runAutoUpdate = async () => {
    const now = Date.now()
    const subscriptions = options.getSubscriptions()

    for (let i = 0; i < subscriptions.length; i += 1) {
      const item = subscriptions[i]
      if (item.isManual) continue

      const interval = item.autoUpdateIntervalMinutes ?? options.defaultIntervalMinutes
      const last = item.lastUpdate ?? 0
      if (interval > 0 && now - last >= interval * 60 * 1000 && !item.isLoading) {
        const applyRuntime = options.getActiveIndex() === i && options.isKernelRunning()
        await options.onRefresh(i, applyRuntime, true)
      }
    }
  }

  const startAutoUpdateLoop = () => {
    stopAutoUpdateLoop()
    autoUpdateTimer = window.setInterval(runAutoUpdate, AUTO_UPDATE_CHECK_INTERVAL)
  }

  const stopAutoUpdateLoop = () => {
    if (autoUpdateTimer) {
      window.clearInterval(autoUpdateTimer)
      autoUpdateTimer = null
    }
  }

  return {
    runAutoUpdate,
    startAutoUpdateLoop,
    stopAutoUpdateLoop,
  }
}
