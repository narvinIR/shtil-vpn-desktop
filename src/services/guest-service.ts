import { invoke } from '@tauri-apps/api/core'

/** Выданный гостевой доступ: ссылка подписки и срок от сервера. */
export interface GuestAccess {
  sub_url: string
  expires_at: string
  ttl_hours: number
  traffic_gb: number
}

/** Что известно о госте без похода в сеть. */
export interface GuestRecord {
  expires_at: string
  traffic_gb: number
}

/**
 * Гостевой доступ без Telegram: связь на пару часов, чтобы человек дошёл до
 * бота и купил подписку. Отказ приходит кодом — слова подбирает экран.
 */
export const guestService = {
  start() {
    return invoke<GuestAccess>('guest_start')
  },

  snapshot() {
    return invoke<GuestRecord>('guest_snapshot')
  },
}
