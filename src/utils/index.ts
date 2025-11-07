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
