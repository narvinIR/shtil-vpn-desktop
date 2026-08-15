/**
 * Человеческие имена каналов — те же, что на телефоне
 * (`mobile/app/.../ui/DisplayNames.kt`): один продукт говорит одинаково.
 *
 * Клиенту показываем ГДЕ сервер, а не КАК он устроен. Разделение на «быстрый»,
 * «запасной» и «основной» — это наши протоколы внутри одного и того же сервера;
 * человеку выбирать между ними нечего, и знать о них незачем.
 */
const CHANNEL_KEYS: Record<string, string> = {
  'VPN-Auto': 'auto',
  auto: 'auto',
  Auto: 'auto',
  Автовыбор: 'auto',
  'VPN-CDN': 'main',
  'VPN-Yahoo': 'main',
  'VPN-HY2': 'main',
  VlessReality: 'main',
  direct: 'direct',
  DIRECT: 'direct',
}

/** Ключ словаря для узла или null, если узел чужой (вставленный руками ключ). */
export function channelKey(tag: string): string | null {
  return CHANNEL_KEYS[tag] ?? null
}

/**
 * Куски служебного имени, которые человеку не говорят ничего: имя продукта и
 * названия протоколов. В имени узла остаётся то, что имеет смысл — обычно место.
 */
const NOISE = new Set([
  'shtil',
  'vpn',
  'reality',
  'vless',
  'trojan',
  'hy2',
  'hysteria',
  'hysteria2',
  'quic',
  'cdn',
  'tcp',
  'ws',
  'grpc',
  'xtls',
  'node',
  'server',
])

/**
 * Разбор чужого имени узла: ключ подписки могут выдать с любым тегом, и сырое
 * «Shtil-HY2-Tallinn» на главном экране читается как сбой. Служебные куски
 * убираем, остаётся место — «Tallinn».
 */
export function describeChannel(tag: string): string[] {
  const words: string[] = []

  for (const piece of tag.split(/[-_·|,\s]+/)) {
    const clean = piece.trim()
    if (!clean) continue
    if (NOISE.has(clean.toLowerCase())) continue
    words.push(clean)
  }

  return words
}
