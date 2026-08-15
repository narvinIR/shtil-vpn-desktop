import type { FrontendSubscription } from '@/stores/subscription/types'
// Мера объёма в приложении одна — общая, через словарь.
import { formatBytes } from '@/utils'

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

export { formatBytes }

export const formatTrafficSummary = (item: FrontendSubscription, t: TranslateFn) => {
  const upload = item.subscriptionUpload ?? 0
  const download = item.subscriptionDownload ?? 0
  const used = upload + download
  const total = item.subscriptionTotal

  // Ноль в заголовке подписки означает «лимита нет», а не «ноль байт»: у наших
  // клиентов лимита не бывает вовсе, и человек читал «осталось 0 B» как пустой
  // ключ (iMac владельца, 05.08.2026).
  if (!total) {
    return used > 0
      ? t('sub.trafficNoLimitUsed', { used: formatBytes(used) })
      : t('sub.trafficNoLimit')
  }

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

/// Дальше этого срока дата перестаёт быть сроком: «до 31.07.2108» человек читает
/// как ошибку, а не как бессрочный ключ (iMac владельца, 05.08.2026).
const FOREVER_AFTER_YEARS = 10

/** Дата окончания подписки, если она вообще осмысленна: у бессрочного ключа — `null`. */
export const subscriptionExpiryDate = (timestamp: number | undefined): Date | null => {
  if (!timestamp) return null
  const date = new Date(timestamp * 1000)
  if (Number.isNaN(date.getTime())) return null

  const horizon = new Date()
  horizon.setFullYear(horizon.getFullYear() + FOREVER_AFTER_YEARS)
  return date > horizon ? null : date
}

export const formatExpireTime = (timestamp: number | undefined, t: TranslateFn) => {
  if (!timestamp) return ''
  const date = subscriptionExpiryDate(timestamp)
  if (!date) {
    return Number.isNaN(new Date(timestamp * 1000).getTime()) ? '' : t('sub.expireNever')
  }

  return t('sub.expireAt', { time: date.toLocaleDateString() })
}

export const formatLocalTime = (timestamp: number) => {
  return new Date(timestamp).toLocaleString()
}

const formatDurationMinutes = (totalMinutes: number) => {
  if (totalMinutes <= 0) return '0m'
  if (totalMinutes < 60) return `${totalMinutes}m`
  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60
  if (minutes === 0) return `${hours}h`
  return `${hours}h ${minutes}m`
}

export const formatAutoUpdateHealth = (item: FrontendSubscription, t: TranslateFn) => {
  const failCount = item.autoUpdateFailCount ?? 0
  if (failCount <= 0) {
    return ''
  }

  const errType = item.lastAutoUpdateErrorType || 'unknown'
  const errorHint = item.lastAutoUpdateError || errType
  const backoffUntil = item.lastAutoUpdateBackoffUntil
  if (!backoffUntil) {
    return t('sub.autoUpdateHealthFailed', {
      count: failCount,
      reason: errorHint,
    })
  }

  const now = Date.now()
  const remainingMinutes = Math.max(Math.ceil((backoffUntil - now) / (60 * 1000)), 0)
  return t('sub.autoUpdateHealthBackoff', {
    count: failCount,
    reason: errorHint,
    remaining: formatDurationMinutes(remainingMinutes),
  })
}
