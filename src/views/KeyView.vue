<template>
  <div class="key-page">
    <BrandWave />

    <div class="key-inner">
      <header class="key-head">
        <h1 class="key-title">{{ t('key.title') }}</h1>
        <p class="key-subtitle">{{ t('key.subtitle') }}</p>
      </header>

      <!-- Ваш ключ: что сейчас есть и до какого числа он работает -->
      <section class="card current">
        <div class="card-head">
          <h2 class="card-title">{{ t('key.current.title') }}</h2>
          <n-button
            v-if="activeKey"
            size="small"
            secondary
            :loading="refreshing"
            @click="refreshActive"
          >
            {{ t('key.current.refresh') }}
          </n-button>
        </div>

        <template v-if="activeKey">
          <div class="key-name">{{ activeKey.name }}</div>
          <div class="key-facts">
            <span v-if="activeKey.subscriptionExpire" class="key-fact strong">
              {{ formatExpire(activeKey.subscriptionExpire) }}
            </span>
            <span v-if="trafficLine" class="key-fact">{{ trafficLine }}</span>
            <span v-if="activeKey.lastUpdate" class="key-fact">
              {{ t('key.current.updated', { time: formatLocalTime(activeKey.lastUpdate) }) }}
            </span>
          </div>
          <!-- Тот же ключ на телефон: снять камерой быстрее, чем пересылать себе текст -->
          <div v-if="keyQr" class="qr-row">
            <img class="qr" :src="keyQr" alt="" />
            <p class="card-note">{{ t('key.current.qrHint') }}</p>
          </div>
          <!-- Ключ перестал подходить: третий ответ — что делать дальше -->
          <details class="stale">
            <summary>{{ t('key.current.staleTitle') }}</summary>
            <p class="card-note">{{ t('key.current.staleText') }}</p>
          </details>
        </template>
        <p v-else class="card-text">{{ t('key.current.none') }}</p>
      </section>

      <!-- Главный путь: ключ выдаёт бот по коду, руками ничего не переносят -->
      <section class="card primary">
        <h2 class="card-title">{{ t('key.fromBot.title') }}</h2>

        <!-- Компьютер уже привязан: дальше всё происходит само -->
        <template v-if="deviceLink.linked">
          <p class="card-text">{{ t('key.fromBot.linkedText') }}</p>
          <div class="card-actions">
            <!-- Оплата, срок и поддержка живут в боте, поэтому вход туда
                 нужен и после привязки, а не только в момент получения кода -->
            <n-button type="primary" @click="openBot">
              <template #icon>
                <n-icon><PaperPlaneOutline /></n-icon>
              </template>
              {{ t('key.fromBot.open') }}
            </n-button>
            <n-button secondary @click="unlink">{{ t('key.fromBot.unlink') }}</n-button>
          </div>
        </template>

        <!-- Код получен: человек называет его боту и ждёт -->
        <template v-else-if="deviceLink.code">
          <p class="card-text">{{ t('key.fromBot.codeText') }}</p>
          <div class="bot-row">
            <div class="bot-actions">
              <div class="link-code">{{ groupedCode }}</div>
              <p class="card-note wait" v-if="deviceLink.waiting">
                {{ t('key.fromBot.waiting') }}
              </p>
              <p class="card-note warn" v-if="deviceLink.failure === 'no_subscription'">
                {{ t('key.fromBot.noSubscription') }}
              </p>
              <div class="card-actions">
                <n-button type="primary" @click="openBot">
                  <template #icon>
                    <n-icon><PaperPlaneOutline /></n-icon>
                  </template>
                  {{ t('key.fromBot.open') }}
                </n-button>
                <n-button quaternary @click="deviceLink.stopWaiting">
                  {{ t('key.fromBot.cancel') }}
                </n-button>
              </div>
            </div>
            <!-- Telegram у большинства на телефоне: код уезжает в бота одним наведением -->
            <div v-if="codeQr" class="qr-row">
              <img class="qr" :src="codeQr" alt="" />
              <p class="card-note">{{ t('key.fromBot.codeQrHint') }}</p>
            </div>
          </div>
        </template>

        <!-- Ключа нет: одна кнопка -->
        <template v-else>
          <p class="card-text">{{ t('key.fromBot.text') }}</p>
          <div class="bot-row">
            <div class="bot-actions">
              <n-button type="primary" :loading="deviceLink.requesting" @click="askCode">
                {{ t('key.fromBot.getCode') }}
              </n-button>
              <p class="card-note warn" v-if="codeFailure">{{ codeFailure }}</p>
              <p class="card-note" v-else>{{ t('key.fromBot.getCodeHint') }}</p>
            </div>
            <div v-if="botQr" class="qr-row">
              <img class="qr" :src="botQr" alt="" />
              <p class="card-note">{{ t('key.fromBot.qrHint') }}</p>
            </div>
          </div>
        </template>
      </section>

      <!-- Ключа нет вовсе: дать связь сразу, иначе до бота не дойти — он тоже за границей -->
      <section v-if="!activeKey && !guest.active" class="card">
        <h2 class="card-title">{{ t('key.guest.title') }}</h2>
        <p class="card-text">
          {{ t('key.guest.text', { hours: GUEST_HOURS, traffic: GUEST_TRAFFIC_GB }) }}
        </p>
        <div class="card-actions">
          <n-button :loading="guest.requesting" @click="takeGuest">
            {{ t('key.guest.take') }}
          </n-button>
        </div>
        <p v-if="guestFailure" class="card-note warn">{{ guestFailure }}</p>
      </section>

      <!-- Пробный доступ идёт: человеку важно видеть, сколько его осталось -->
      <section v-else-if="guest.active" class="card">
        <h2 class="card-title">{{ t('key.guest.title') }}</h2>
        <p class="card-text">
          {{ t('key.guest.left', { left: guest.remaining, traffic: guest.trafficGb }) }}
        </p>
      </section>

      <!-- Остальные пути — под одной свёрнутой строкой. Открытыми они спорили с
           главным действием, и человек не понимал, куда нажимать (05.08.2026) -->
      <details class="card fold">
        <summary>{{ t('key.other.title') }}</summary>
        <div class="fold-body">
          <h3 class="card-title">{{ t('key.link.title') }}</h3>
          <p class="card-text">{{ t('key.link.text') }}</p>
          <n-input
            v-model:value="linkUrl"
            type="textarea"
            :rows="3"
            :placeholder="t('key.link.placeholder')"
            :disabled="applying"
          />
          <div class="card-actions">
            <n-button secondary :disabled="applying" @click="pasteLink">
              {{ t('key.link.paste') }}
            </n-button>
            <n-button type="primary" :loading="applying" @click="applyLink">
              {{ t('key.link.apply') }}
            </n-button>
          </div>

          <h3 class="card-title">{{ t('key.file.title') }}</h3>
          <p class="card-text">{{ t('key.file.text') }}</p>
          <input
            ref="fileInput"
            type="file"
            accept=".json,application/json,text/plain"
            class="file-input"
            @change="applyFile"
          />
          <n-button secondary :loading="applying" @click="fileInput?.click()">
            {{ t('key.file.choose') }}
          </n-button>
        </div>
      </details>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { PaperPlaneOutline } from '@vicons/ionicons5'
import { openUrl } from '@tauri-apps/plugin-opener'
import { readText } from '@tauri-apps/plugin-clipboard-manager'
import QRCode from 'qrcode'
import { useSubStore } from '@/stores/subscription/SubStore'
import { useDeviceLinkStore } from '@/stores/subscription/DeviceLinkStore'
import { useGuestStore } from '@/stores/subscription/GuestStore'
import { useAppStore } from '@/stores'
import { subscriptionService } from '@/services/subscription-service'
import type { SubscriptionPersistResult } from '@/services/subscription-service'
import { DEFAULT_AUTO_UPDATE_MINUTES, type FrontendSubscription } from '@/stores/subscription/types'
import {
  formatExpireTime,
  formatLocalTime,
  formatTrafficSummary,
  generateConfigFileName,
  hasSubscriptionTraffic,
} from '@/views/sub/subscription-utils'
import { useSubscriptionAutoUpdate } from '@/views/sub/useSubscriptionAutoUpdate'
import BrandWave from '@/components/common/BrandWave.vue'

defineOptions({
  name: 'KeyView',
})

/** Бот — единственное место, где выдают ключ и принимают оплату. */
const BOT_URL = 'https://t.me/RealityVPNBot_bot'

/** Сколько длится пробный доступ. Точный срок всё равно приходит от сервера. */
const GUEST_HOURS = 2
const GUEST_TRAFFIC_GB = 1

const { t } = useI18n()
const message = useMessage()
const subStore = useSubStore()
const appStore = useAppStore()
const deviceLink = useDeviceLinkStore()
const guest = useGuestStore()

const linkUrl = ref('')
const botQr = ref('')
const keyQr = ref('')
const codeQr = ref('')
const applying = ref(false)
const refreshing = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

const activeKey = computed<FrontendSubscription | null>(() => {
  if (subStore.activeIndex !== null) return subStore.list[subStore.activeIndex] ?? null
  return subStore.list[0] ?? null
})

const trafficLine = computed(() => {
  const item = activeKey.value
  if (!item || !hasSubscriptionTraffic(item)) return ''
  return formatTrafficSummary(item, t)
})

const formatExpire = (timestamp?: number) => formatExpireTime(timestamp, t)

const openBot = async () => {
  // Код уже в ссылке: бот откроется с ним, вводить руками ничего не нужно.
  await openUrl(deviceLink.code ? `${BOT_URL}?start=link_${deviceLink.code}` : BOT_URL)
}

/** Десять цифр подряд читаются плохо — разбиваем на группы. */
const groupedCode = computed(() => {
  const value = deviceLink.code
  if (value.length !== 10) return value
  return `${value.slice(0, 1)} ${value.slice(1, 4)} ${value.slice(4, 7)} ${value.slice(7)}`
})

/**
 * Почему код не пришёл. Сервер называет причину своим словом, а сеть возвращает
 * текст исключения — человеку в обоих случаях нужна фраза, а не тишина. Раньше
 * экран ждал ровно `network`, а такого значения не приходит ниоткуда: провал
 * «Получить ключ» молчал совсем.
 */
const codeFailure = computed(() => {
  if (!deviceLink.failure) return ''
  if (deviceLink.failure === 'no_subscription') return t('key.fromBot.noSubscription')
  return t('key.fromBot.failed')
})

const askCode = async () => {
  await deviceLink.requestCode()
  if (codeFailure.value) {
    message.error(codeFailure.value)
  }
}

const unlink = async () => {
  await deviceLink.forget()
  message.success(t('key.fromBot.unlinked'))
}

/** Сырой код отказа сервера человеку не показываем — подбираем слова. */
const guestFailure = computed(() => {
  switch (guest.failure) {
    case '':
      return ''
    case 'already_issued':
      return t('key.guest.alreadyIssued')
    case 'ip_limit':
    case 'daily_cap':
    case 'rate_limited':
      return t('key.guest.limit')
    case 'disabled':
    case 'bad_platform':
      return t('key.guest.disabled')
    default:
      return t('key.guest.failed')
  }
})

const takeGuest = async () => {
  if (await guest.take()) message.success(t('key.added'))
}

/** Код рисуется прямо в приложении: интернет для этого не нужен. */
const drawQr = (text: string) =>
  QRCode.toDataURL(text, {
    margin: 1,
    width: 320,
    color: { dark: '#0b1220', light: '#ffffff' },
  }).catch(() => '')

const pasteLink = async () => {
  const text = (await readText().catch(() => '')) ?? ''
  if (!text.trim()) {
    message.warning(t('key.link.clipboardEmpty'))
    return
  }
  linkUrl.value = text.trim()
}

/**
 * Наш адрес `/sub/{token}` отдаёт ГОТОВЫЙ конфиг sing-box целиком, поэтому
 * ключ всегда применяется как есть. Иначе из него достали бы только серверы и
 * подставили чужие маршруты на скачиваемых списках — из России они не
 * отвечают, и связь не поднимется вовсе.
 */
const USE_ORIGINAL_CONFIG = true

const persistKey = async (
  item: Omit<FrontendSubscription, 'isLoading'>,
  result: SubscriptionPersistResult,
) => {
  const savedPath = result.configPath ?? null
  const newItem: FrontendSubscription = {
    ...item,
    isLoading: false,
    lastUpdate: Date.now(),
    configPath: savedPath || undefined,
    backupPath: savedPath ? `${savedPath}.bak` : undefined,
    subscriptionUpload: result.subscriptionUpload,
    subscriptionDownload: result.subscriptionDownload,
    subscriptionTotal: result.subscriptionTotal,
    subscriptionExpire: result.subscriptionExpire,
  }

  subStore.list.push(newItem)
  await subStore.saveToBackend()
  await subStore.setActiveIndex(subStore.list.length - 1)

  if (savedPath) {
    await subscriptionService.setActiveConfig(savedPath, {
      useOriginalConfig: item.useOriginalConfig,
    })
    await appStore.setActiveConfigPath(savedPath)
  }
  message.success(t('key.added'))
}

const applyLink = async () => {
  const value = linkUrl.value.trim()
  if (!value) {
    message.warning(t('key.link.needLink'))
    return
  }
  const name = t('key.defaultName')
  applying.value = true
  try {
    // Из бота приходят две разные вещи: ссылка подписки (её надо скачать) и голый
    // ключ vless:// из-под кнопки «Показать ключ». Человек не обязан их различать —
    // различаем мы. Ключ применяется как ручное содержимое, и «оригинальный конфиг»
    // для него выключен: конфига там нет, есть одна строка узла.
    const isRawKey = /^(vless|vmess|trojan|ss|hysteria2):\/\//i.test(value)
    const result = isRawKey
      ? await subscriptionService.addManualSubscription(value, false, {
          fileName: generateConfigFileName(name),
          applyRuntime: false,
        })
      : await subscriptionService.downloadSubscription(value, USE_ORIGINAL_CONFIG, {
          fileName: generateConfigFileName(name),
          applyRuntime: false,
        })
    await persistKey(
      {
        name,
        url: value,
        isManual: isRawKey,
        useOriginalConfig: isRawKey ? false : USE_ORIGINAL_CONFIG,
        autoUpdateIntervalMinutes: DEFAULT_AUTO_UPDATE_MINUTES,
      },
      result,
    )
    linkUrl.value = ''
  } catch {
    message.error(t('key.failed'))
  } finally {
    applying.value = false
  }
}

const applyFile = async (event: Event) => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return

  applying.value = true
  try {
    const content = (await file.text()).trim()
    if (!content) {
      message.error(t('key.file.badFile'))
      return
    }
    const name = file.name.replace(/\.[^.]+$/, '') || t('key.defaultName')
    const result = await subscriptionService.addManualSubscription(content, USE_ORIGINAL_CONFIG, {
      fileName: generateConfigFileName(name),
      applyRuntime: false,
    })
    await persistKey(
      {
        name,
        url: '',
        isManual: true,
        manualContent: content,
        useOriginalConfig: USE_ORIGINAL_CONFIG,
        autoUpdateIntervalMinutes: 0,
      },
      result,
    )
  } catch {
    message.error(t('key.failed'))
  } finally {
    applying.value = false
  }
}

const refreshKey = async (index: number, applyRuntime = false, silent = false) => {
  const item = subStore.list[index]
  if (!item) return

  const persistOptions = {
    ...(item.configPath ? { configPath: item.configPath } : { fileName: generateConfigFileName(item.name) }),
    applyRuntime,
  }

  try {
    subStore.list[index].isLoading = true
    // У ключа vless конфига нет — только строка узла, поэтому «применять целиком»
    // для него выключено, и обновление обязано идти тем же способом, что и добавление
    const asOriginal = item.useOriginalConfig ?? USE_ORIGINAL_CONFIG
    const result = item.isManual
      ? await subscriptionService.addManualSubscription(
          item.manualContent || item.url || '',
          asOriginal,
          persistOptions,
        )
      : await subscriptionService.downloadSubscription(item.url, asOriginal, persistOptions)

    if (result.configPath) {
      subStore.list[index].configPath = result.configPath
      subStore.list[index].backupPath = `${result.configPath}.bak`
    }
    subStore.list[index].subscriptionUpload = result.subscriptionUpload
    subStore.list[index].subscriptionDownload = result.subscriptionDownload
    subStore.list[index].subscriptionTotal = result.subscriptionTotal
    subStore.list[index].subscriptionExpire = result.subscriptionExpire
    subStore.list[index].lastUpdate = Date.now()
    await subStore.saveToBackend()

    if (result.configPath && applyRuntime) {
      await subscriptionService.setActiveConfig(result.configPath, {
        useOriginalConfig: asOriginal,
      })
      await appStore.setActiveConfigPath(result.configPath)
    }
    if (!silent) {
      message.success(t('key.refreshed'))
    }
  } catch {
    if (!silent) {
      message.error(t('key.failed'))
    }
  } finally {
    if (index >= 0 && index < subStore.list.length) {
      subStore.list[index].isLoading = false
    }
  }
}

const refreshActive = async () => {
  const index = subStore.activeIndex ?? 0
  refreshing.value = true
  try {
    await refreshKey(index, appStore.isRunning)
  } finally {
    refreshing.value = false
  }
}

// Подписка обновляется сама: срок продлили в боте — ключ подтянется без рук.
const { startAutoUpdateLoop, stopAutoUpdateLoop } = useSubscriptionAutoUpdate({
  getSubscriptions: () => subStore.list,
  getActiveIndex: () => subStore.activeIndex,
  isKernelRunning: () => appStore.isRunning,
  defaultIntervalMinutes: DEFAULT_AUTO_UPDATE_MINUTES,
  onRefresh: refreshKey,
})

onMounted(async () => {
  subStore.resetLoadingState()
  startAutoUpdateLoop()
  botQr.value = await drawQr(BOT_URL)
})

// Код ключа перерисовывается вслед за самим ключом: показываем то, что применено сейчас
watch(
  () => activeKey.value?.url,
  async (url) => {
    keyQr.value = url ? await drawQr(url) : ''
  },
  { immediate: true },
)

// Код привязки уезжает в бота ссылкой — телефоном достаточно навести камеру
watch(
  () => deviceLink.code,
  async (code) => {
    codeQr.value = code ? await drawQr(`${BOT_URL}?start=link_${code}`) : ''
  },
  { immediate: true },
)

onUnmounted(() => {
  stopAutoUpdateLoop()
})
</script>

<style scoped>
.key-page {
  position: relative;
  min-height: calc(100vh - var(--header-height));
  overflow: hidden;
  background: var(--page-bg);
  --wave-color: var(--primary-color);
}

.key-inner {
  position: relative;
  z-index: 1;
  max-width: 640px;
  margin: 0 auto;
  padding: clamp(20px, 3vw, 36px) clamp(16px, 2.2vw, 28px);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

/* Заголовок экрана прижат влево: на Маке взгляд идёт по левому краю, а
   заголовок посреди страницы читается как рекламная полоса, а не как раздел. */
.key-head {
  margin-bottom: var(--space-2);
}

.key-title {
  margin: 0;
  font-size: 26px;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--text-primary);
}

.key-subtitle {
  margin: 6px 0 0;
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.card {
  border-radius: var(--radius-md);
  border: 1px solid var(--panel-border);
  background: var(--panel-bg);
  padding: var(--space-4);
}

/* Главную карточку выделяем фоном, а не яркой рамкой: обводка в цвет акцента
   кричит громче самой кнопки внутри и спорит с ней за внимание. */
.card.primary {
  background: var(--primary-soft);
  border-color: var(--border-color);
}

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.card-title {
  margin: 0;
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
}

.card-text {
  margin: var(--space-2) 0 0;
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}

.card-note {
  margin: var(--space-3) 0 0;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.card-actions {
  margin-top: var(--space-4);
  display: flex;
  gap: var(--space-3);
  flex-wrap: wrap;
}

.bot-row {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  flex-wrap: wrap;
}

.bot-actions {
  flex: 1 1 260px;
  margin-top: var(--space-4);
}

/* Код — самое крупное на карточке: его читают с экрана и называют боту */
.link-code {
  font-family: 'SF Mono', 'Cascadia Mono', Consolas, monospace;
  font-size: clamp(24px, 3.4vw, 34px);
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--text-primary);
  padding: var(--space-3) var(--space-4);
  border: 1px dashed var(--primary-color);
  border-radius: var(--radius-sm);
  text-align: center;
}

.card-note.wait {
  color: var(--primary-color);
}

.card-note.warn {
  color: var(--warning-color, #e0a300);
}

.qr-row {
  margin-top: var(--space-3);
  text-align: center;
}

.qr {
  width: 132px;
  height: 132px;
  border-radius: var(--radius-sm);
  background: #fff;
  padding: 6px;
  display: block;
  margin: 0 auto;
}

.qr-row .card-note {
  margin-top: var(--space-2);
  max-width: 160px;
}

.stale {
  margin-top: var(--space-3);
}

.stale summary {
  cursor: pointer;
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.stale .card-note {
  margin-top: var(--space-2);
}

.key-name {
  margin-top: var(--space-3);
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--text-primary);
}

.key-facts {
  margin-top: var(--space-2);
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
}

.key-fact {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.key-fact.strong {
  color: var(--primary-color);
  font-weight: 600;
}

.fold summary {
  cursor: pointer;
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-secondary);
  list-style: none;
}

.fold summary::-webkit-details-marker {
  display: none;
}

.fold summary::after {
  content: '▾';
  float: right;
  color: var(--text-tertiary);
}

.fold[open] summary::after {
  content: '▴';
}

.fold-body {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: var(--space-3);
  margin-top: var(--space-4);
}

.fold-body :deep(.n-input) {
  width: 100%;
}

.file-input {
  display: none;
}
</style>
