export function formatBandwidth(kbps: number) {
  const valueInKb = kbps / 1024
  const valueInMb = valueInKb / 1024
  const valueInGb = valueInMb / 1024

  if (valueInGb >= 1) {
    return `${valueInGb.toFixed(2)} GB`
  }

  if (valueInMb >= 1) {
    return `${valueInMb.toFixed(2)} MB`
  }

  return `${valueInKb.toFixed(2)} KB`
}

export function formatBytes(bytes?: number): string {
  if (!bytes) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const value = bytes / Math.pow(1024, index)
  return `${value.toFixed(2)} ${units[index]}`
}

export function formatSpeed(bytes: number): string {
  return `${formatBytes(bytes)}/s`
}

/**
 * 时间格式化
 *
 * 此前项目里有三份等价实现（LogView 的 `formatTime`、subscription-utils 的
 * `formatLocalTime`、SubView 的 `formatTime` 转调），各自 `toLocaleString()`。
 * 统一到此处，并允许传入 locale —— 切换语言后时间格式也应跟随，
 * 而 `toLocaleString()` 默认跟随浏览器环境，与应用内选定语言可能不一致。
 */

/** 本地化日期时间（如 `2026/9/7 15:04`）。`locale` 缺省时跟随浏览器环境。 */
export function formatDateTime(value: number | string | Date, locale?: string): string {
  const date = value instanceof Date ? value : new Date(value)
  return locale ? date.toLocaleString(locale) : date.toLocaleString()
}

/** 本地化日期（不含时间）。 */
export function formatDate(value: number | string | Date, locale?: string): string {
  const date = value instanceof Date ? value : new Date(value)
  return locale ? date.toLocaleDateString(locale) : date.toLocaleDateString()
}

/**
 * 带单位的相对时间（如 `3 分钟前`）。
 *
 * `t` 形参为 vue-i18n 的翻译函数；单位文案走语言包，数值用 `{count}` 占位。
 */
export function formatRelativeTime(
  value: number | string | Date,
  t: (key: string, params?: Record<string, unknown>) => string,
  now = Date.now(),
): string {
  const timestamp = value instanceof Date ? value.getTime() : new Date(value).getTime()
  const diff = now - timestamp

  // 未来时间 / 同一秒：直接给绝对时间，避免出现「-1 秒前」
  if (diff < 60_000)
    return t('connections.secondsAgo', { count: Math.max(1, Math.floor(diff / 1000)) })
  if (diff < 3_600_000) return t('connections.minutesAgo', { count: Math.floor(diff / 60_000) })
  if (diff < 86_400_000) return t('connections.hoursAgo', { count: Math.floor(diff / 3_600_000) })
  return formatDateTime(timestamp)
}
