import type { FrontendSubscription } from '@/stores/subscription/types'

type TranslateFn = (key: string, params?: Record<string, unknown>) => string

export const generateConfigFileName = (name: string) => {
  const safe = name
    .toLowerCase()
    .replace(/[^a-z0-9-_]/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
  return `${safe || 'subscription'}-${Date.now()}.json`
}

export const formatIntervalLabel = (
  minutes: number | undefined,
  t: TranslateFn,
  fallbackMinutes: number,
) => {
  const value = minutes ?? fallbackMinutes
  if (!value) return t('sub.autoUpdateOff')
  if (value % 1440 === 0) return t('sub.autoUpdate1d')
  if (value % 720 === 0) return t('sub.autoUpdate12h')
  if (value % 360 === 0) return t('sub.autoUpdate6h')
  return `${value} min`
}

export const isJsonContent = (value: string) => {
  try {
    const parsed = JSON.parse(value)
    return typeof parsed === 'object' && parsed !== null
  } catch {
    return false
  }
}

export const hasSubscriptionTraffic = (item: FrontendSubscription) => {
  return (
    item.subscriptionUpload !== undefined ||
    item.subscriptionDownload !== undefined ||
    item.subscriptionTotal !== undefined
  )
}

export const formatBytes = (bytes?: number) => {
  if (!bytes) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const value = bytes / Math.pow(1024, index)
  return `${value.toFixed(2)} ${units[index]}`
}

export const formatTrafficSummary = (item: FrontendSubscription, t: TranslateFn) => {
  const upload = item.subscriptionUpload ?? 0
  const download = item.subscriptionDownload ?? 0
  const used = upload + download
  const total = item.subscriptionTotal

  if (total !== undefined) {
    const remaining = Math.max(total - used, 0)
    return t('sub.trafficWithTotal', {
      used: formatBytes(used),
      total: formatBytes(total),
      remaining: formatBytes(remaining),
    })
  }

  return t('sub.trafficUsedOnly', { used: formatBytes(used) })
}

export const formatExpireTime = (timestamp: number | undefined, t: TranslateFn) => {
  if (!timestamp) return ''
  const date = new Date(timestamp * 1000)
  if (Number.isNaN(date.getTime())) return ''
  return t('sub.expireAt', { time: date.toLocaleString() })
}

export const formatLocalTime = (timestamp: number) => {
  return new Date(timestamp).toLocaleString()
}
