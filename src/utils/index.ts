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
