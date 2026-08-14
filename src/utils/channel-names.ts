/**
 * Человеческие имена каналов — те же, что на телефоне
 * (`mobile/app/.../ui/DisplayNames.kt`): один продукт говорит одинаково.
 *
 * Служебное имя узла («VPN-HY2») человеку не показываем: он не поймёт, что это
 * и стоит ли туда переключаться.
 */
const CHANNEL_KEYS: Record<string, string> = {
  'VPN-Auto': 'auto',
  auto: 'auto',
  Auto: 'auto',
  'VPN-CDN': 'main',
  VlessReality: 'main',
  'VPN-Yahoo': 'backup',
  'VPN-HY2': 'fast',
  direct: 'direct',
  DIRECT: 'direct',
}

/** Ключ словаря для узла или null, если узел чужой (вставленный руками ключ). */
export function channelKey(tag: string): string | null {
  return CHANNEL_KEYS[tag] ?? null
}
