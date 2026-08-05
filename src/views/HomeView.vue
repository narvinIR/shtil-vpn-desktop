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

      <!-- Круг в центре — он же кнопка. Язык, на котором говорят платные VPN:
           один крупный объект, всё остальное подчинено ему. -->
      <div class="ring-wrap">
        <div v-if="connected" class="ring-halo"></div>
        <button class="ring" :disabled="!ringEnabled" @click="onRingClick">
          <template v-if="connected">
            <span class="ring-word">{{ t('home.state.connected') }}</span>
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
            <span class="ring-action">{{ t('home.action.addKey') }}</span>
          </template>
          <template v-else>
            <n-icon :size="42" class="ring-icon"><PowerOutline /></n-icon>
            <span class="ring-action">{{ t('home.action.connect') }}</span>
            <span class="ring-note">{{ t('home.state.disconnected') }}</span>
          </template>
        </button>
      </div>

      <!-- Куда подключены. Пока туннель не поднят, сервера нет — нет и плашки. -->
      <button v-if="connected" class="server-pill" @click="router.push('/sub')">
        <n-icon :size="18"><GlobeOutline /></n-icon>
        <span class="server-label">{{ t('home.server.label') }}:</span>
        <span class="server-name">{{ serverName }}</span>
      </button>

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

      <!-- Сколько осталось: второй из трёх ответов, которые человек ищет на экране -->
      <p v-if="subscriptionLine" class="subscription-line">{{ subscriptionLine }}</p>

      <p class="hint">{{ hint }}</p>

      <!-- Сбой показываем словами: китайский текст ядра уходит под свёрнутую
           строку, чтобы поддержке было что прочитать, а человеку — нет. -->
      <div v-if="failureText" class="failure">
        <div class="failure-head">
          <n-icon :size="18"><AlertCircleOutline /></n-icon>
          <span>{{ failureText }}</span>
        </div>
        <details v-if="failureDetail" class="failure-details">
          <summary>{{ t('home.error.details') }}</summary>
          <pre>{{ failureDetail }}</pre>
        </details>
      </div>

      <details class="advanced">
        <summary>{{ t('home.advanced.title') }}</summary>
        <div class="advanced-body">
          <div class="advanced-row">
            <div class="advanced-info">
              <span class="advanced-name">{{ t('home.proxyMode.system') }}</span>
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
import { useKernelStatus } from '@/composables/useKernelStatus'
import { formatBytes } from '@/utils'

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

const stateClass = computed(() => {
  if (connected.value) return 'is-connected'
  if (busy.value) return 'is-busy'
  if (!hasKey.value) return 'is-empty'
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

const activeNode = computed(() => proxyStore.proxyGroups[0]?.now || '')
const serverName = computed(() => activeNode.value || t('home.server.searching'))

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

/** Оплата живёт только в боте — отсюда туда и ведём. */
const payInBot = () => openUrl('https://t.me/RealityVPNBot_bot?start=buy_vpn')

const newsText = computed(() => {
  const key = `home.deviceNews.${deviceLink.news}`
  return deviceLink.news && te(key) ? t(key) : ''
})

/** «Сколько осталось» словами: дата, а не число секунд. */
const subscriptionLine = computed(() => {
  // Пробный доступ идёт часами, поэтому у него своя строка и свой остаток.
  if (guest.active) {
    return t('key.guest.left', { left: guest.remaining, traffic: guest.trafficGb })
  }
  if (!deviceLink.linked) return ''
  if (deviceLink.subscription === 'expired') return t('home.subscription.over')
  if (deviceLink.subscription !== 'active') return ''
  const raw = deviceLink.expiresAt
  if (!raw) return ''
  const date = new Date(raw)
  if (Number.isNaN(date.getTime())) return ''
  const text = date.toLocaleDateString(undefined, {
    day: 'numeric',
    month: 'long',
    year: 'numeric',
  })
  return deviceLink.isTrial
    ? t('home.subscription.trialUntil', { date: text })
    : t('home.subscription.activeUntil', { date: text })
})

const hint = computed(() => {
  if (!hasKey.value) return t('home.hint.noKey')
  if (connected.value) return t('home.hint.connected')
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
  const node = proxyStore.proxyGroups[0]?.now
  if (node) {
    await proxyStore.testNodeDelay(node).catch(() => undefined)
  }
}

watch(connected, (isOn) => {
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
.home {
  position: relative;
  min-height: calc(100vh - var(--header-height));
  overflow: hidden;
  background: var(--page-bg);
  --state-color: var(--primary-color);
  --wave-color: var(--primary-color);
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

.home-inner {
  position: relative;
  z-index: 1;
  min-height: calc(100vh - var(--header-height));
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-5);
  padding: clamp(16px, 2.2vw, 28px);
  text-align: center;
}

/* ============ Круг ============ */
.ring-wrap {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.ring-halo {
  position: absolute;
  inset: -22px;
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
    transform: scale(1.1);
    opacity: 0;
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
  transition:
    border-color var(--transition-base),
    transform var(--transition-fast),
    box-shadow var(--transition-base);
}

.ring:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 0 32px var(--primary-soft-strong);
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
  transition: background var(--transition-fast);
}

.server-pill:hover {
  background: var(--primary-soft-strong);
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
  gap: clamp(24px, 6vw, 64px);
}

.stat {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 72px;
}

.stat-value {
  font-size: var(--text-lg);
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  color: var(--text-primary);
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
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--primary-color);
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

/* ============ Для опытных ============ */
.advanced {
  width: min(520px, 100%);
  border-radius: var(--radius-md);
  border: 1px solid var(--panel-border);
  background: var(--panel-bg);
  text-align: left;
}

.advanced summary {
  padding: var(--space-3) var(--space-4);
  cursor: pointer;
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-secondary);
  list-style: none;
}

.advanced summary::-webkit-details-marker {
  display: none;
}

.advanced summary::after {
  content: '▾';
  float: right;
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
