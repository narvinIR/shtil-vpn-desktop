import { invoke } from '@tauri-apps/api/core'

/** Код на экране, который человек называет боту. */
export interface LinkStart {
  code: string
  expires_at: string
  poll_interval: number
}

/** Подтвердили код в боте или ещё нет. */
export interface LinkStatus {
  status: 'pending' | 'linked' | 'expired' | 'not_found' | 'no_subscription'
  sub_url: string | null
}

/** Что сервер знает о подписке этого компьютера. */
export interface DeviceState {
  linked: boolean
  subscription: 'active' | 'expired' | 'none'
  is_trial: boolean
  days_left: number
  expires_at: string | null
  sub_url: string | null
  /** paid | trial | over — только в момент смены. */
  news: string | null
}

export interface LinkSnapshot {
  linked: boolean
  code: string | null
}

/**
 * Привязка компьютера к боту: код виден на экране, ключ приезжает сам.
 * Порты приложения этим командам не нужны, поэтому идём напрямую.
 */
export const deviceLinkService = {
  start() {
    return invoke<LinkStart>('device_link_start')
  },

  status() {
    return invoke<LinkStatus>('device_link_status')
  },

  /** `null` — спрашивать сервер ещё рано, прошлое состояние остаётся в силе. */
  poll(force = false) {
    return invoke<DeviceState | null>('device_link_poll', { force })
  },

  snapshot() {
    return invoke<LinkSnapshot>('device_link_snapshot')
  },

  forget() {
    return invoke<void>('device_link_forget')
  },
}
