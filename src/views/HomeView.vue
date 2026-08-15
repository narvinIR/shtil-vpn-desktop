<template>
  <div class="home" :class="stateClass">
    <BrandWave :active="connected" />

    <div class="home-inner">
      <!-- Новость от бота: подписку продлили, начался бесплатный период, срок вышел.
           Появляется только на СМЕНЕ состояния и закрывается кнопкой. -->
      <div v-if="newsText" class="news" :class="{ over: deviceLink.news === 'over' }">
        <span class="news-text">{{ newsText }}</span>
        <div class="news-actions">
          <n-button v-if="deviceLink.news === 'over'" size="small" type="primary" @click="payInBot">
            {{ t('home.deviceNews.pay') }}
          </n-button>
          <n-button size="small" quaternary @click="deviceLink.dismissNews">
            {{ t('home.deviceNews.ok') }}
          </n-button>
        </div>
      </div>

      <!-- Одна плита по центру: состояние, круг, сервер и показатели живут вместе,
           как карточка в приложениях Мака, а не россыпью по пустому полю. -->
      <section class="stage">
        <h1 class="stage-title">{{ stateTitle }}</h1>

      <!-- Круг в центре — он же кнопка. Язык, на котором говорят платные VPN:
           один крупный объект, всё остальное подчинено ему. -->
      <div class="ring-wrap">
        <div v-if="connected" class="ring-halo"></div>
        <button class="ring" :disabled="!ringEnabled" @click="onRingClick">
          <!-- Слова «Подключено» в круге нет: оно уже стоит заголовком над ним,
               и вдвоём они читались как заикание. -->
          <template v-if="connected">
            <span class="ring-timer">{{ uptime }}</span>
            <span class="ring-note">{{ t('home.action.disconnect') }}</span>
          </template>
          <template v-else-if="busy">
            <span class="ring-word">{{ busyWord }}</span>
            <ol class="ring-steps">
              <li v-for="step in steps" :key="step.key" :class="{ done: step.done }">
                {{ t(step.key) }}
              </li>
            </ol>
          </template>
          <template v-else-if="!hasKey">
            <n-icon :size="42" class="ring-icon"><KeyOutline /></n-icon>
          </template>
          <template v-else>
            <n-icon :size="42" class="ring-icon"><PowerOutline /></n-icon>
            <span class="ring-action">{{ t('home.action.connect') }}</span>
          </template>
        </button>
      </div>

      <!-- Куда подключены и куда можно переключиться. Пока туннель не поднят,
           сервера нет — нет и плашки. -->
      <n-dropdown
        v-if="connected"
        trigger="click"
        placement="bottom"
        :options="channelOptions"
        :disabled="channelOptions.length < 2"
        @select="onPickChannel"
      >
        <button class="server-pill" :title="t('home.server.pick')">
          <n-icon :size="18"><GlobeOutline /></n-icon>
          <span class="server-label">{{ t('home.server.label') }}:</span>
          <span class="server-name">{{ serverName }}</span>
          <n-icon v-if="channelOptions.length > 1" :size="14" class="server-caret">
            <ChevronDownOutline />
          </n-icon>
        </button>
      </n-dropdown>

      <!-- Отклик, принято, отдано — в том же порядке, что на телефоне. -->
      <div class="stats">
        <div class="stat">
          <span class="stat-value" :class="{ accent: pingIsGood }">{{ pingText }}</span>
          <span class="stat-label">{{ t('home.stats.ping') }}</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ receivedText }}</span>
          <span class="stat-label">{{ t('home.stats.received') }}</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ sentText }}</span>
          <span class="stat-label">{{ t('home.stats.sent') }}</span>
        </div>
      </div>
      </section>

      <!-- Сколько осталось: второй из трёх ответов, которые человек ищет на экране.
           Рядом — куда идти платить, чтобы третий ответ не пришлось искать. -->
      <p v-if="subscriptionLine" class="subscription-line">
        <span>{{ subscriptionLine }}</span>
        <n-button v-if="canRenew" text size="tiny" type="primary" @click="payInBot">
          {{ t('home.subscription.renew') }}
        </n-button>
      </p>

      <p class="hint">{{ hint }}</p>

      <!-- Сбой показываем словами: китайский текст ядра уходит под свёрнутую
           строку, чтобы поддержке было что прочитать, а человеку — нет. -->
      <div v-if="failureText" class="failure">
        <div class="failure-head">
          <n-icon :size="18"><AlertCircleOutline /></n-icon>
          <span>{{ failureText }}</span>
        </div>
        <!-- Что делать: сбой без следующего шага оставляет человека в тупике -->
        <p class="failure-what">{{ t('home.error.whatToDo') }}</p>
        <ol class="failure-steps">
          <li>{{ t('home.error.steps.checkKey') }}</li>
          <li>{{ t('home.error.steps.restart') }}</li>
          <li>{{ t('home.error.steps.support') }}</li>
        </ol>
        <div class="failure-actions">
          <n-button size="small" secondary @click="router.push('/sub')">
            {{ t('home.error.goToKey') }}
          </n-button>
          <n-button size="small" secondary :loading="kernelStore.isLoading" @click="restart">
            {{ t('home.restart') }}
          </n-button>
          <n-button size="small" quaternary @click="openSupportInBot">
            {{ t('home.error.writeSupport') }}
          </n-button>
        </div>
        <details v-if="failureDetail" class="failure-details">
          <summary>{{ t('home.error.details') }}</summary>
          <pre>{{ failureDetail }}</pre>
        </details>
      </div>

      <!-- Полоса внизу молчала о том, что за ней: человек не знал, стоит ли
           открывать. Теперь она сразу отвечает, каким режимом он живёт. -->
      <details class="advanced">
        <summary>
          <span>{{ t('home.advanced.title') }}</span>
          <span class="advanced-current">{{ currentModeLabel }}</span>
        </summary>
        <div class="advanced-body">
          <div class="advanced-row">
            <div class="advanced-info">
              <span class="advanced-name">{{ t('home.proxyMode.system') }}</span>
              <span class="advanced-note">{{ t('home.proxyMode.systemTip') }}</span>
              <code class="advanced-note">{{ proxyAddress }}</code>
            </div>
            <n-switch
              :value="appStore.systemProxyEnabled"
              size="small"
              :disabled="modeSwitchPending"
              @update:value="(v: boolean) => toggleSystemProxy(v)"
            />
          </div>
          <div class="advanced-row">
            <div class="advanced-info">
              <span class="advanced-name">{{ t('home.proxyMode.tun') }}</span>
              <span class="advanced-note">{{ t('home.proxyMode.tunTip') }}</span>
              <span class="advanced-note">{{ t('home.advanced.tunHint') }}</span>
            </div>
            <n-switch
              :value="appStore.tunEnabled"
              size="small"
              :disabled="modeSwitchPending"
              @update:value="(v: boolean) => toggleTunProxy(v)"
            />
          </div>
          <div class="advanced-row">
            <div class="advanced-info">
              <span class="advanced-name">{{ t('home.restart') }}</span>
              <span class="advanced-note">{{ t('home.advanced.restartNote') }}</span>
            </div>
            <n-button size="small" secondary :loading="kernelStore.isLoading" @click="restart">
              {{ t('home.restart') }}
            </n-button>
          </div>
        </div>
      </details>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import {
  PowerOutline,
  KeyOutline,
  GlobeOutline,
  AlertCircleOutline,
  ChevronDownOutline,
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
import { useSubStore } from '@/stores/subscription/SubStore'
import { useDeviceLinkStore } from '@/stores/subscription/DeviceLinkStore'
import { useGuestStore } from '@/stores/subscription/GuestStore'
import { openUrl } from '@tauri-apps/plugin-opener'
import BrandWave from '@/components/common/BrandWave.vue'
import { subscriptionExpiryDate } from '@/views/sub/subscription-utils'
import { useKernelStatus } from '@/composables/useKernelStatus'
import { formatBytes } from '@/utils'
import { channelKey, describeChannel } from '@/utils/channel-names'

defineOptions({
  name: 'HomeView',
})

const { t, te } = useI18n()
const router = useRouter()
const message = useMessage()

const appStore = useAppStore()
const kernelStore = useKernelStore()
const connectionStore = useConnectionStore()
const proxyStore = useProxyStore()
const subStore = useSubStore()
const deviceLink = useDeviceLinkStore()
const guest = useGuestStore()

const { statusState, isReady } = useKernelStatus(kernelStore)

/** Своё ожидание: стор помечает загрузку только на запуске, не на остановке. */
const pending = ref<'connect' | 'disconnect' | null>(null)
const modeSwitchPending = ref(false)
/** Сколько проб канала подряд не дошло: замер отклика — единственная живая
    проверка, что связь не только поднялась, но и везёт. */
const probeFailures = ref(0)
const now = ref(Date.now())
let tick: number | null = null
let serverTick: number | null = null

const hasKey = computed(() => Boolean(appStore.activeConfigPath) || subStore.list.length > 0)
const connected = computed(() => isReady.value)
const busy = computed(
  () =>
    pending.value !== null ||
    statusState.value === 'starting' ||
    statusState.value === 'stopping' ||
    (kernelStore.isRunning && !isReady.value),
)
const ringEnabled = computed(() => !busy.value)

/**
 * Пущен ли трафик через нас. Ядро живо и отвечает — это ещё не защита: при
 * выключенных обоих режимах программы выходят напрямую, а экран до этой
 * правки продолжал писать «Подключено».
 */
const trafficRouted = computed(() => appStore.systemProxyEnabled || appStore.tunEnabled)

/** Канал не ответил на две пробы подряд: связь поднята, а интернета через неё нет. */
const channelSilent = computed(() => connected.value && probeFailures.value >= 2)

const stateClass = computed(() => {
  if (connected.value) {
    if (!trafficRouted.value) return 'is-bypassed'
    return channelSilent.value ? 'is-failed' : 'is-connected'
  }
  if (busy.value) return 'is-busy'
  if (!hasKey.value) return 'is-empty'
  // Сбой обязан быть виден по самому кругу: раньше не подключившееся
  // приложение выглядело ровно как выключенное, и человек жал ту же кнопку.
  if (failureText.value) return 'is-failed'
  return 'is-off'
})

const busyWord = computed(() =>
  pending.value === 'disconnect' || statusState.value === 'stopping'
    ? t('home.state.disconnecting')
    : t('home.state.connecting'),
)

/** Три шага подключения — ровно то, что сообщает ядро о своей готовности. */
const steps = computed(() => {
  const readiness = kernelStore.readiness
  return [
    { key: 'home.steps.config', done: readiness.config_validated === true },
    { key: 'home.steps.tunnel', done: readiness.process_alive },
    { key: 'home.steps.internet', done: readiness.api_ready && readiness.relay_ready },
  ]
})

/**
 * Ядро сообщает свой возраст редко — между ответами счётчик шёл бы по старому
 * числу: стоял на месте, а потом прыгал сразу на несколько секунд. Поэтому
 * запоминаем последнее сказанное ядром вместе с моментом, когда оно пришло, и
 * между ответами досчитываем по местным часам.
 */
const uptimeAnchor = ref({ ms: 0, at: Date.now() })
watch(
  () => kernelStore.status.uptime_ms,
  (reported) => {
    uptimeAnchor.value = { ms: reported || 0, at: Date.now() }
  },
  { immediate: true },
)

const uptime = computed(() => {
  const anchor = uptimeAnchor.value
  const ms = anchor.ms > 0 ? anchor.ms + (now.value - anchor.at) : 0
  const total = Math.max(0, Math.floor(ms / 1000))
  const hours = Math.floor(total / 3600)
  const minutes = Math.floor((total % 3600) / 60)
  const seconds = total % 60
  const pad = (value: number) => String(value).padStart(2, '0')
  return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`
})

/** Группа выбора из подписки: в ней и лежат каналы, между которыми переключаемся. */
const channelGroup = computed(() => proxyStore.proxyGroups[0])

/** Что выбрано человеком: канал или «Автоматически». */
const activeChoice = computed(() => channelGroup.value?.now || '')

/**
 * За «Автоматически» стоит ещё одна группа, а трафик везёт конкретный канал.
 * Разворачиваем до него: отклик меряем у того, кто реально работает, а не у
 * пункта выбора — иначе к честному кругу добавляется лишний, а на экране
 * стоит «Автоматически» без ответа, какой это канал.
 */
const resolveLeaf = (tag: string) => {
  let current = tag
  for (let step = 0; step < 4; step += 1) {
    const group = proxyStore.proxyGroups.find((item) => item.name === current)
    if (!group?.now || group.now === current) break
    current = group.now
  }
  return current
}

const activeNode = computed(() => (activeChoice.value ? resolveLeaf(activeChoice.value) : ''))

/**
 * Служебное имя узла («VPN-HY2») человеку не показываем — только людское. Ключ
 * могут выдать с любым тегом, поэтому у чужого имени убираем служебные части:
 * «Shtil-HY2-Tallinn» превращается в «Tallinn».
 */
const channelLabel = (tag: string) => {
  const key = channelKey(tag)
  if (key) return t(`home.server.channels.${key}`)
  const words = describeChannel(tag)
  return words.length ? words.join(' · ') : tag
}

/** Показываем и выбор человека, и канал за ним: «Автоматически» само по себе
    не отвечает на вопрос, куда именно идёт связь. */
const serverName = computed(() => {
  if (!activeChoice.value) return t('home.server.searching')
  const choice = channelLabel(activeChoice.value)
  if (activeNode.value && activeNode.value !== activeChoice.value) {
    return `${choice} · ${channelLabel(activeNode.value)}`
  }
  return choice
})

/**
 * Каналы на выбор. «Напрямую» из списка убрано: это не сервер, а выход МИМО
 * VPN — выбрав его, человек остаётся без защиты, а экран продолжает писать
 * «Подключено». Российские сайты и без того идут напрямую, это решает сам
 * ключ, а не человек.
 */
const channelOptions = computed(() => {
  // Один сервер часто заведён несколькими способами связи. Человеку это один
  // и тот же выбор, поэтому одинаковые имена в списке не повторяем.
  const seen = new Set<string>()
  return (channelGroup.value?.all || [])
    .filter((tag) => channelKey(tag) !== 'direct')
    .map((tag) => ({ key: tag, label: channelLabel(tag) }))
    .filter((option) => {
      if (seen.has(option.label)) return false
      seen.add(option.label)
      return true
    })
})

const onPickChannel = async (tag: string) => {
  const group = channelGroup.value
  if (!group || tag === activeChoice.value) return
  try {
    await proxyStore.changeProxy(group.name, tag)
    // Новый канал — новая проба: прежние неудачи к нему отношения не имеют.
    probeFailures.value = 0
    await refreshServer()
  } catch {
    message.error(t('home.server.pickFailed'))
  }
}

const ping = computed(() => (activeNode.value ? proxyStore.getLatency(activeNode.value) : 0))
const pingText = computed(() =>
  connected.value && ping.value > 0 ? t('home.stats.pingValue', { ms: ping.value }) : '—',
)
const pingIsGood = computed(() => connected.value && ping.value > 0 && ping.value < 300)
const receivedText = computed(() =>
  connected.value ? formatBytes(connectionStore.connectionsTotal.download) : '—',
)
const sentText = computed(() =>
  connected.value ? formatBytes(connectionStore.connectionsTotal.upload) : '—',
)

const proxyAddress = computed(() => `127.0.0.1:${appStore.proxyPort}`)

/** Каким режимом человек живёт прямо сейчас — видно, не открывая свёртку. */
const currentModeLabel = computed(() => {
  if (appStore.tunEnabled) return t('home.proxyMode.tun')
  return appStore.systemProxyEnabled ? t('home.proxyMode.system') : t('home.proxyMode.manual')
})

/** Оплата живёт только в боте — отсюда туда и ведём. */
const payInBot = () => openUrl('https://t.me/RealityVPNBot_bot?start=buy_vpn')

const newsText = computed(() => {
  const key = `home.deviceNews.${deviceLink.news}`
  return deviceLink.news && te(key) ? t(key) : ''
})

const formatUntil = (date: Date) =>
  date.toLocaleDateString(undefined, { day: 'numeric', month: 'long', year: 'numeric' })

/**
 * Срок, известный самому ключу. Нужен, когда ключ вставлен ссылкой, а не получен
 * по коду: привязки к боту нет, и без этой строки ответа «сколько осталось» на
 * главном экране не было вовсе.
 */
const keyExpiryLine = () => {
  const key = subStore.activeIndex !== null ? subStore.list[subStore.activeIndex] : subStore.list[0]
  const date = subscriptionExpiryDate(key?.subscriptionExpire)
  return date ? t('home.subscription.activeUntil', { date: formatUntil(date) }) : ''
}

/**
 * Срок, который назвал бот. Бессрочный ключ приходит датой далеко в будущем, и
 * «до 31 июля 2108 г. · осталось 29936 дн.» человек читает как сбой, а не как
 * подписку без конца. Горизонт тот же, что и у ключа.
 */
const deviceExpiryDate = computed(() => {
  const raw = deviceLink.expiresAt
  if (!raw) return null
  const parsed = new Date(raw)
  if (Number.isNaN(parsed.getTime())) return null
  return subscriptionExpiryDate(Math.floor(parsed.getTime() / 1000))
})

/** Дату вообще не разобрать — это не «бессрочно», а «срок неизвестен». */
const deviceExpiryBroken = computed(() => {
  const raw = deviceLink.expiresAt
  if (!raw) return false
  return Number.isNaN(new Date(raw).getTime())
})

/** Сколько дней осталось: человек считает днями, а датой сверяется. */
const subscriptionDaysLeft = computed(() => {
  const date = deviceExpiryDate.value
  if (!date) return ''
  const msLeft = date.getTime() - Date.now()
  if (msLeft <= 0) return ''
  const days = Math.ceil(msLeft / 86_400_000)
  return days <= 1 ? t('home.subscription.lastDay') : t('home.subscription.daysLeft', { days })
})

/** «Сколько осталось» словами: дата и остаток днями, а не число секунд. */
const subscriptionLine = computed(() => {
  // Пробный доступ идёт часами, поэтому у него своя строка и свой остаток.
  if (guest.active) {
    return t('key.guest.left', { left: guest.remaining, traffic: guest.trafficGb })
  }
  if (!deviceLink.linked) return keyExpiryLine()
  if (deviceLink.subscription === 'expired') return t('home.subscription.over')
  if (deviceLink.subscription !== 'active') return keyExpiryLine()
  if (!deviceLink.expiresAt || deviceExpiryBroken.value) return keyExpiryLine()
  const date = deviceExpiryDate.value
  if (!date) return t('home.subscription.unlimited')
  const text = formatUntil(date)
  const until = deviceLink.isTrial
    ? t('home.subscription.trialUntil', { date: text })
    : t('home.subscription.activeUntil', { date: text })
  const left = subscriptionDaysLeft.value
  return left ? `${until} · ${left}` : until
})

/** Продлить можно тогда, когда есть что продлевать: подписка идёт или уже кончилась. */
const canRenew = computed(
  () =>
    deviceLink.linked &&
    !guest.active &&
    (deviceLink.subscription === 'active' || deviceLink.subscription === 'expired'),
)

/** Поддержка живёт в боте — там же, где оплата и ключ. */
const openSupportInBot = () => openUrl('https://t.me/RealityVPNBot_bot')

/** Крупная строка над кругом: человек читает состояние раньше, чем ищет кнопку. */
const stateTitle = computed(() => {
  if (!hasKey.value) return t('home.action.addKey')
  if (busy.value) return busyWord.value
  if (connected.value) {
    if (!trafficRouted.value) return t('home.state.bypassed')
    return channelSilent.value ? t('home.state.silent') : t('home.state.connected')
  }
  return failureText.value ? t('home.state.failed') : t('home.state.disconnected')
})

const hint = computed(() => {
  if (!hasKey.value) return t('home.hint.noKey')
  if (connected.value) {
    if (!trafficRouted.value) return t('home.hint.bypassed')
    return channelSilent.value ? t('home.hint.silent') : t('home.hint.connected')
  }
  if (busy.value) return t('home.hint.busy')
  return t('home.hint.ready')
})

/**
 * Ошибку ядра называем по машинному коду, а не его текстом: текст приходит от
 * форка на китайском, а код — один из тринадцати известных.
 */
const failureText = computed(() => {
  const diagnosis = kernelStore.startupDiagnosis
  if (!diagnosis) return ''
  const key = `home.error.kinds.${diagnosis.kind}`
  return te(key) ? t(key) : t('home.error.kinds.unknown')
})

const failureDetail = computed(() => {
  const diagnosis = kernelStore.startupDiagnosis
  if (!diagnosis) return ''
  return [diagnosis.code, diagnosis.message, diagnosis.detail].filter(Boolean).join('\n')
})

const connect = async () => {
  pending.value = 'connect'
  try {
    // Ни системный прокси, ни TUN не включены — трафик пошёл бы мимо туннеля,
    // и «подключено» оказалось бы неправдой. Берём системный прокси: он не
    // требует прав администратора.
    if (!appStore.systemProxyEnabled && !appStore.tunEnabled) {
      await appStore.toggleSystemProxy(true)
      await kernelStore.applyProxySettings()
    }
    const started = await kernelStore.startKernel()
    if (!started) {
      message.error(failureText.value || t('home.error.kinds.unknown'))
    }
  } finally {
    pending.value = null
  }
}

const disconnect = async () => {
  pending.value = 'disconnect'
  try {
    // Системный прокси снимает сама остановка ядра, иначе система осталась бы
    // с адресом, по которому уже никто не отвечает, и интернет пропал бы весь.
    const stopped = await kernelStore.stopKernel()
    if (!stopped) {
      message.error(t('home.error.stopFailed'))
    }
  } finally {
    pending.value = null
  }
}

const onRingClick = () => {
  if (!hasKey.value) {
    router.push('/sub')
    return
  }
  if (connected.value) {
    disconnect()
    return
  }
  connect()
}

const toggleSystemProxy = async (value: boolean) => {
  if (modeSwitchPending.value) return
  modeSwitchPending.value = true
  try {
    await appStore.toggleSystemProxy(value)
    const applied = await kernelStore.applyProxySettings()
    if (!applied) {
      message.error(t('notification.proxyModeChangeFailed'))
    }
  } finally {
    modeSwitchPending.value = false
  }
}

const toggleTunProxy = async (value: boolean) => {
  if (modeSwitchPending.value) return
  modeSwitchPending.value = true
  try {
    await appStore.toggleTun(value)
    const applied = await kernelStore.applyProxySettings()
    if (!applied) {
      await appStore.toggleTun(!value)
      message.error(t('notification.proxyModeChangeFailed'))
      return
    }
    if (kernelStore.isRunning) {
      await kernelStore.restartKernel()
    }
  } finally {
    modeSwitchPending.value = false
  }
}

const restart = async () => {
  const done = await kernelStore.restartKernel()
  if (done) {
    message.success(t('home.restartSuccess'))
  } else {
    message.error(failureText.value || t('home.restartFailed'))
  }
}

/**
 * Сервер и отклик приходят не событием, а по запросу. Спрашиваем в момент, когда
 * туннель поднялся, и дальше раз в полминуты: иначе плашка навсегда осталась бы
 * на «ищем сервер», а отклик — замером часовой давности.
 */
const refreshServer = async () => {
  await proxyStore.fetchProxies().catch(() => undefined)
  const node = activeNode.value
  if (!node) return
  const result = await proxyStore.testNodeDelay(node).catch(() => null)
  probeFailures.value = result?.ok ? 0 : probeFailures.value + 1
}

watch(connected, (isOn) => {
  probeFailures.value = 0
  if (isOn) refreshServer()
})

onMounted(() => {
  tick = window.setInterval(() => {
    now.value = Date.now()
  }, 1000)
  serverTick = window.setInterval(() => {
    if (connected.value) refreshServer()
  }, 30000)
  if (connected.value) refreshServer()
})

onUnmounted(() => {
  if (tick) {
    clearInterval(tick)
    tick = null
  }
  if (serverTick) {
    clearInterval(serverTick)
    serverTick = null
  }
})
</script>

<style scoped>
/* Цвет состояния не берётся из акцента: выбрав зелёный акцент, человек получал
   зелёный круг и зелёную волну на выключенном VPN — ровно тот же вид, что у
   работающего. Зелёный тут значит «защищено», и ничего больше. */
.home {
  position: relative;
  min-height: calc(100vh - var(--header-height));
  overflow: hidden;
  background: var(--page-bg);
  --state-color: var(--text-tertiary);
  --wave-color: var(--text-tertiary);
}

/* Выключено: серый, как и «ключа нет» — оба состояния означают «защиты нет». */
.home.is-off {
  --state-color: var(--text-tertiary);
  --wave-color: var(--text-tertiary);
}

.home.is-connected {
  --state-color: var(--success-color);
  --wave-color: var(--success-color);
}

.home.is-busy {
  --state-color: var(--warning-color);
  --wave-color: var(--warning-color);
}

.home.is-empty {
  --state-color: var(--text-tertiary);
  --wave-color: var(--text-tertiary);
}

/* Не подключились: круг красный, а не такой же, как у выключенного. */
.home.is-failed {
  --state-color: var(--error-color);
  --wave-color: var(--error-color);
}

/* Связь есть, но трафик идёт мимо: не зелёный — защиты сейчас нет. */
.home.is-bypassed {
  --state-color: var(--warning-color);
  --wave-color: var(--warning-color);
}

/* Одна колонка по центру. Раньше ширину задавала себе каждая часть сама:
   плита выходила узкой, подписка и режим — широкими, а по бокам оставались
   два мёртвых поля во весь экран. */
.home-inner {
  position: relative;
  z-index: 1;
  min-height: calc(100vh - var(--header-height));
  width: min(560px, 100%);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  justify-content: center;
  gap: var(--space-5);
  padding: clamp(16px, 2.2vw, 28px);
  text-align: center;
}

/* ============ Плита ============ */
.stage {
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-5);
  padding: var(--space-8) var(--space-10) var(--space-6);
  border-radius: 20px;
  background: var(--glass-bg);
  border: 1px solid var(--glass-border);
  backdrop-filter: blur(var(--glass-blur));
  -webkit-backdrop-filter: blur(var(--glass-blur));
  box-shadow: var(--shadow-lg);
}

.stage-title {
  margin: 0;
  font-size: 26px;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--text-primary);
}

/* ============ Круг ============ */
/* Место под кольцо отдаём самой обёртке: раньше кольцо расходилось за её
   границы и накрывало плашку сервера под кругом. */
.ring-wrap {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 26px;
}

.ring-halo {
  position: absolute;
  inset: 0;
  border-radius: var(--radius-pill);
  border: 3px solid var(--state-color);
  opacity: 0.35;
  animation: halo 2.6s ease-out infinite;
  pointer-events: none;
}

@keyframes halo {
  0% {
    transform: scale(0.86);
    opacity: 0.36;
  }
  100% {
    transform: scale(1);
    opacity: 0;
  }
}

/* Система просила не двигать — не двигаем. Кольцо остаётся, но замирает. */
@media (prefers-reduced-motion: reduce) {
  .ring-halo {
    animation: none;
    opacity: 0.28;
  }

  .ring:active:not(:disabled) {
    transform: none;
  }
}

.ring {
  width: clamp(200px, 32vh, 264px);
  height: clamp(200px, 32vh, 264px);
  border-radius: var(--radius-pill);
  border: 3px solid var(--state-color);
  background: radial-gradient(circle, var(--primary-soft), transparent 72%);
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 0 24px;
  cursor: pointer;
  /* Подъём и нажатие живут в РАЗНЫХ величинах одного transform: раньше
     :active со своим transform стирал подъём от :hover целиком, и кнопка
     дёргалась вниз вместо того, чтобы вжаться. */
  --ring-lift: 0px;
  --ring-press: 1;
  transform: translateY(var(--ring-lift)) scale(var(--ring-press));
  transition:
    border-color var(--transition-base),
    transform var(--transition-fast),
    box-shadow var(--transition-base);
}

/* Наведение — только там, где указатель настоящий: на касании палец рождает
   ложное наведение, и кнопка остаётся приподнятой после отпускания. */
@media (hover: hover) and (pointer: fine) {
  .ring:hover:not(:disabled) {
    --ring-lift: -2px;
    box-shadow: 0 0 32px var(--primary-soft-strong);
  }
}

/* Главная кнопка приложения обязана отзываться на нажатие — иначе непонятно,
   услышали ли тебя, и человек жмёт второй раз. Отклик короче остального
   движения: рука ждёт ответа сразу. */
.ring:active:not(:disabled) {
  --ring-press: 0.97;
  transition-duration: 120ms;
}

/* Клавиатура: круг — единственное действие экрана, до него доходят табом. */
.ring:focus-visible {
  outline: none;
  box-shadow: var(--shadow-focus);
}

.ring:disabled {
  cursor: default;
}

.is-connected .ring {
  box-shadow: 0 0 42px var(--success-soft);
}

.ring-icon {
  color: var(--state-color);
}

.ring-word {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--state-color);
}

.ring-timer {
  font-size: clamp(28px, 4vh, 34px);
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  letter-spacing: -0.02em;
}

.ring-action {
  font-size: var(--text-xl);
  font-weight: 700;
}

.ring-note {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: 1.35;
}

.ring-steps {
  list-style: none;
  margin: 4px 0 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.ring-steps li.done {
  color: var(--success-color);
}

/* ============ Плашка сервера ============ */
.server-pill {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: 9px 16px;
  border-radius: var(--radius-pill);
  border: 1px solid var(--state-color);
  background: var(--primary-soft);
  color: var(--state-color);
  cursor: pointer;
  transition:
    background var(--transition-fast),
    transform 160ms cubic-bezier(0.23, 1, 0.32, 1);
}

.server-pill:hover {
  background: var(--primary-soft-strong);
}

/* Нажатие обязано отзываться: иначе непонятно, услышали ли тебя. */
.server-pill:active {
  transform: scale(0.97);
}

.server-caret {
  opacity: 0.7;
}

.server-label {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.server-name {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
  max-width: 260px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* ============ Показатели ============ */
.stats {
  display: flex;
  align-items: stretch;
  align-self: stretch;
  border-top: 1px solid var(--border-color);
  padding-top: var(--space-4);
}

.stat {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 72px;
}

.stat + .stat {
  border-left: 1px solid var(--border-color);
}

/* Число не переносим: «225.47 КБ» уезжало на вторую строку, и три показателя
   разъезжались по высоте — подпись одного оказывалась ниже соседних. */
.stat-value {
  font-size: var(--text-lg);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--text-primary);
  white-space: nowrap;
}

.stat-value.accent {
  color: var(--state-color);
}

.stat-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

/* ============ Подсказка и сбой ============ */
.hint {
  margin: 0;
  max-width: 520px;
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}

/* Новость от бота: заметная, но не тревожная — тревожный вид только у «срок вышел» */
.news {
  width: min(520px, 100%);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  border: 1px solid var(--primary-color);
  background: var(--primary-soft);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  flex-wrap: wrap;
  text-align: left;
}

.news.over {
  border-color: var(--error-color);
  background: var(--error-soft);
}

.news-text {
  font-size: var(--text-sm);
  color: var(--text-primary);
  flex: 1 1 240px;
}

.news-actions {
  display: flex;
  gap: var(--space-2);
}

.subscription-line {
  margin: 0;
  display: flex;
  align-items: baseline;
  justify-content: center;
  flex-wrap: wrap;
  gap: var(--space-2);
  font-size: var(--text-md);
  font-weight: 600;
  /* Срок подписки красился акцентным синим — 3.00:1 на стекле плиты при норме
     4.5:1, то есть главное число экрана было самым нечитаемым. Акцент здесь
     работал украшением, а не смыслом: цветом на экране выделяется одно
     действие — круг подключения. */
  color: var(--text-primary);
}

.failure {
  width: min(520px, 100%);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  border: 1px solid var(--error-color);
  background: var(--error-soft);
  text-align: left;
}

.failure-head {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  color: var(--error-color);
  font-size: var(--text-sm);
  font-weight: 600;
}

.failure-what {
  margin: var(--space-3) 0 0;
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--text-secondary);
}

.failure-steps {
  margin: var(--space-1) 0 0;
  padding-left: var(--space-4);
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.failure-steps li + li {
  margin-top: 2px;
}

.failure-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: var(--space-3);
}

.failure-details {
  margin-top: var(--space-2);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.failure-details summary {
  cursor: pointer;
}

.failure-details pre {
  margin: var(--space-2) 0 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: var(--font-mono);
}

/* ============ Режим связи ============ */
.advanced {
  width: 100%;
  border-radius: 12px;
  border: 1px solid var(--panel-border);
  background: var(--panel-bg);
  text-align: left;
}

.advanced summary {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  min-height: 44px;
  padding: var(--space-3) var(--space-4);
  cursor: pointer;
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-secondary);
  list-style: none;
}

/* Текущий режим — сразу в строке, до открытия */
.advanced-current {
  margin-left: auto;
  margin-right: var(--space-4);
  font-weight: 500;
  color: var(--text-primary);
}

.advanced summary::-webkit-details-marker {
  display: none;
}

.advanced summary::after {
  content: '▾';
  color: var(--text-tertiary);
}

.advanced[open] summary::after {
  content: '▴';
}

.advanced-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: 0 var(--space-4) var(--space-4);
}

.advanced-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  background: var(--bg-surface-2);
}

.advanced-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.advanced-name {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-primary);
}

.advanced-note {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

code.advanced-note {
  font-family: var(--font-mono);
}
</style>
