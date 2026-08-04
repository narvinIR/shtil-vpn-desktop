import { ref } from 'vue'
import { defineStore } from 'pinia'
import i18n from '@/locales'
import { deviceLinkService, type DeviceState } from '@/services/device-link-service'
import { useSubStore } from './SubStore'

/** Как часто спрашиваем бота, подтвердили ли код. Сервер обычно просит 5 секунд. */
const WAIT_STEP_MS = 5000

/**
 * Как часто заглядываем в собственный будильник. Спрашивать сервер или ещё рано —
 * решает служебная часть по периоду, который назвал сам сервер.
 */
const POLL_TICK_MS = 60_000

export const useDeviceLinkStore = defineStore('deviceLink', () => {
  /** Компьютер привязан к боту. */
  const linked = ref(false)
  /** Код на экране, пока ждём подтверждения. */
  const code = ref('')
  const waiting = ref(false)
  const requesting = ref(false)
  /** Последняя ошибка ожидания — кодом, слова подбирает экран. */
  const failure = ref('')

  const subscription = ref<DeviceState['subscription']>('none')
  const isTrial = ref(false)
  const daysLeft = ref(0)
  const expiresAt = ref<string | null>(null)
  /** paid | trial | over — новость, которую человек ещё не закрыл. */
  const news = ref('')

  let waitTimer: ReturnType<typeof setTimeout> | null = null
  let pollTimer: ReturnType<typeof setInterval> | null = null

  const keyName = () => i18n.global.t('key.defaultName')

  const stopWaiting = () => {
    if (waitTimer) {
      clearTimeout(waitTimer)
      waitTimer = null
    }
    waiting.value = false
  }

  /** Разложить ответ сервера по состоянию экрана. */
  const absorb = async (state: DeviceState) => {
    linked.value = state.linked
    if (!state.linked) {
      code.value = ''
      subscription.value = 'none'
      daysLeft.value = 0
      expiresAt.value = null
      return
    }

    subscription.value = state.subscription
    isTrial.value = state.is_trial
    daysLeft.value = state.days_left
    expiresAt.value = state.expires_at
    if (state.news) {
      news.value = state.news
    }

    // Ссылка приезжает в КАЖДОМ ответе, поэтому применяем только незнакомую:
    // иначе приложение качало бы подписку и дёргало ядро каждые полчаса впустую.
    const subStore = useSubStore()
    if (state.sub_url && !subStore.list.some((item) => item.url === state.sub_url)) {
      await subStore.applySubscriptionUrl(state.sub_url, keyName()).catch(() => undefined)
    }
  }

  /**
   * Спросить сервер о подписке. `force` — не ждать конца периода.
   * Сеть молчит — прошлое состояние остаётся: «не дозвонились» это не
   * «подписка кончилась».
   */
  const refresh = async (force = false) => {
    try {
      const state = await deviceLinkService.poll(force)
      if (state) await absorb(state)
    } catch {
      // Молчим намеренно: это фоновая проверка, человеку показывать нечего.
    }
  }

  /** Попросить у бота новый код и ждать подтверждения. */
  const requestCode = async () => {
    requesting.value = true
    failure.value = ''
    try {
      const started = await deviceLinkService.start()
      code.value = started.code
      waiting.value = true
      scheduleWait()
    } catch (error) {
      failure.value = String(error)
    } finally {
      requesting.value = false
    }
  }

  const scheduleWait = () => {
    if (waitTimer) clearTimeout(waitTimer)
    waitTimer = setTimeout(async () => {
      if (!waiting.value) return
      try {
        const status = await deviceLinkService.status()
        if (status.status === 'linked' && status.sub_url) {
          stopWaiting()
          linked.value = true
          await useSubStore().applySubscriptionUrl(status.sub_url, keyName())
          await refresh(true)
          return
        }
        if (status.status === 'no_subscription') {
          failure.value = 'no_subscription'
        }
        if (status.status === 'expired' || status.status === 'not_found') {
          // Код живёт четверть часа: просроченный меняем на свежий сами.
          await requestCode()
          return
        }
      } catch {
        // Сеть моргнула — просто спросим на следующем круге.
      }
      scheduleWait()
    }, WAIT_STEP_MS)
  }

  /** Что знаем без сети + один опрос при запуске. */
  const initialize = async () => {
    try {
      const snapshot = await deviceLinkService.snapshot()
      linked.value = snapshot.linked
      if (snapshot.linked) {
        await refresh(true)
      }
    } catch {
      // Приложение должно подняться и без ответа сервера.
    }
    if (!pollTimer) {
      pollTimer = setInterval(() => {
        void refresh(false)
      }, POLL_TICK_MS)
    }
  }

  const dismissNews = () => {
    news.value = ''
  }

  const forget = async () => {
    stopWaiting()
    await deviceLinkService.forget().catch(() => undefined)
    linked.value = false
    code.value = ''
    subscription.value = 'none'
    daysLeft.value = 0
    expiresAt.value = null
  }

  const cleanup = () => {
    stopWaiting()
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  }

  return {
    linked,
    code,
    waiting,
    requesting,
    failure,
    subscription,
    isTrial,
    daysLeft,
    expiresAt,
    news,
    initialize,
    requestCode,
    stopWaiting,
    refresh,
    dismissNews,
    forget,
    cleanup,
  }
})
