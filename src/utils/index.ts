import i18n from '@/locales'

/**
 * Объём — байтами и через словарь.
 *
 * Единственная мера объёма в приложении: до 15.08.2026 таких было три —
 * здесь, на экране ключа и в окне обновления, — и все три писали «KB / MB»
 * по-английски мимо словаря, в каком бы языке ни сидел человек.
 */
export function formatBytes(bytes?: number): string {
  const units = ['size.b', 'size.kb', 'size.mb', 'size.gb', 'size.tb']
  if (!bytes || bytes <= 0) return `0 ${i18n.global.t(units[0])}`
  const index = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1)
  const value = bytes / Math.pow(1024, index)
  return `${value.toFixed(2)} ${i18n.global.t(units[index])}`
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
