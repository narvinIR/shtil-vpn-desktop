import i18n from '@/locales'

export function formatBandwidth(bytesPerSecond: number) {
  const valueInKb = bytesPerSecond / 1024
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

/**
 * Скорость в битах в секунду — так её меряют спидтесты и соседние клиенты.
 *
 * До 07.08.2026 здесь были мегабайты: цифра выходила в восемь раз меньше
 * ожидаемой, и живой канал на 59 Мбит/с читался с экрана как «килобиты».
 */
export function formatSpeed(bytesPerSecond?: number): string {
  const bits = Math.max(0, (bytesPerSecond || 0) * 8)
  const units = ['speed.bps', 'speed.kbps', 'speed.mbps', 'speed.gbps']
  const index = bits < 1 ? 0 : Math.min(Math.floor(Math.log10(bits) / 3), units.length - 1)
  const value = bits / Math.pow(1000, index)

  return `${value.toFixed(index === 0 ? 0 : 1)} ${i18n.global.t(units[index])}`
}
