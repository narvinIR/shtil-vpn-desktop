import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import i18n from '@/locales'
import { guestService } from '@/services/guest-service'
import { useSubStore } from './SubStore'

/** Как часто пересчитываем остаток гостевого времени. */
const TICK_MS = 60_000

export const useGuestStore = defineStore('guest', () => {
  /** Когда гостевой доступ кончится — строкой от сервера, свои часы не в счёт. */
  const expiresAt = ref('')
  const trafficGb = ref(0)
  const requesting = ref(false)
  /** Код отказа сервера — слова подбирает экран. */
  const failure = ref('')
  /** Пересчитывается по будильнику, иначе остаток застывает на экране. */
  const now = ref(Date.now())

  let tickTimer: ReturnType<typeof setInterval> | null = null

  /** Гостевой доступ брали и он ещё не кончился. */
  const active = computed(() => {
    if (!expiresAt.value) return false
    return new Date(expiresAt.value).getTime() > now.value
  })

  /** Сколько осталось, словами из словаря: «1 ч 20 мин» или «45 мин». */
  const remaining = computed(() => {
    if (!active.value) return ''
    const minutes = Math.max(
      0,
      Math.round((new Date(expiresAt.value).getTime() - now.value) / 60_000),
    )
    const hours = Math.floor(minutes / 60)
    const { t } = i18n.global
    return hours > 0
      ? t('key.guest.leftHours', { hours, minutes: minutes % 60 })
      : t('key.guest.leftMinutes', { minutes })
  })

  const initialize = async () => {
    try {
      const snapshot = await guestService.snapshot()
      expiresAt.value = snapshot.expires_at
      trafficGb.value = snapshot.traffic_gb
    } catch {
      // Приложение должно подняться и без записи о госте.
    }
    if (!tickTimer) {
      tickTimer = setInterval(() => {
        now.value = Date.now()
      }, TICK_MS)
    }
  }

  /** Попросить гостевой доступ и сразу применить ключ. */
  const take = async () => {
    requesting.value = true
    failure.value = ''
    try {
      const issued = await guestService.start()
      expiresAt.value = issued.expires_at
      trafficGb.value = issued.traffic_gb
      now.value = Date.now()
      await useSubStore().applySubscriptionUrl(issued.sub_url, i18n.global.t('key.guest.name'))
      return true
    } catch (error) {
      failure.value = String(error)
      return false
    } finally {
      requesting.value = false
    }
  }

  const cleanup = () => {
    if (tickTimer) {
      clearInterval(tickTimer)
      tickTimer = null
    }
  }

  return {
    expiresAt,
    trafficGb,
    requesting,
    failure,
    active,
    remaining,
    initialize,
    take,
    cleanup,
  }
})
