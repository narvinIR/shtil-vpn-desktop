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
        </template>
        <p v-else class="card-text">{{ t('key.current.none') }}</p>
      </section>

      <!-- Главный путь: ключ выдаёт бот -->
      <section class="card primary">
        <h2 class="card-title">{{ t('key.fromBot.title') }}</h2>
        <p class="card-text">{{ t('key.fromBot.text') }}</p>
        <div class="card-actions">
          <n-button type="primary" @click="openBot">
            <template #icon>
              <n-icon><PaperPlaneOutline /></n-icon>
            </template>
            {{ t('key.fromBot.open') }}
          </n-button>
        </div>
        <p class="card-note">{{ t('key.fromBot.soon') }}</p>
      </section>

      <!-- Запасные пути: свёрнуты, чтобы не путать на первом запуске -->
      <details class="card fold">
        <summary>{{ t('key.link.title') }}</summary>
        <div class="fold-body">
          <n-input
            v-model:value="linkName"
            :placeholder="t('key.link.namePlaceholder')"
            :disabled="applying"
          />
          <n-input
            v-model:value="linkUrl"
            type="textarea"
            :rows="2"
            :placeholder="t('key.link.placeholder')"
            :disabled="applying"
          />
          <n-button type="primary" :loading="applying" @click="applyLink">
            {{ t('key.link.apply') }}
          </n-button>
        </div>
      </details>

      <details class="card fold">
        <summary>{{ t('key.file.title') }}</summary>
        <div class="fold-body">
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
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { PaperPlaneOutline } from '@vicons/ionicons5'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useSubStore } from '@/stores/subscription/SubStore'
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

const { t } = useI18n()
const message = useMessage()
const subStore = useSubStore()
const appStore = useAppStore()

const linkName = ref('')
const linkUrl = ref('')
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
  await openUrl(BOT_URL)
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
    useOriginalConfig: USE_ORIGINAL_CONFIG,
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
      useOriginalConfig: USE_ORIGINAL_CONFIG,
    })
    await appStore.setActiveConfigPath(savedPath)
  }
  message.success(t('key.added'))
}

const applyLink = async () => {
  const url = linkUrl.value.trim()
  if (!url) {
    message.warning(t('key.link.needLink'))
    return
  }
  const name = linkName.value.trim() || t('key.defaultName')
  applying.value = true
  try {
    const result = await subscriptionService.downloadSubscription(url, USE_ORIGINAL_CONFIG, {
      fileName: generateConfigFileName(name),
      applyRuntime: false,
    })
    await persistKey(
      {
        name,
        url,
        isManual: false,
        useOriginalConfig: USE_ORIGINAL_CONFIG,
        autoUpdateIntervalMinutes: DEFAULT_AUTO_UPDATE_MINUTES,
      },
      result,
    )
    linkUrl.value = ''
    linkName.value = ''
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
    const result = item.isManual
      ? await subscriptionService.addManualSubscription(
          item.manualContent || '',
          USE_ORIGINAL_CONFIG,
          persistOptions,
        )
      : await subscriptionService.downloadSubscription(item.url, USE_ORIGINAL_CONFIG, persistOptions)

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
        useOriginalConfig: USE_ORIGINAL_CONFIG,
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

onMounted(() => {
  subStore.resetLoadingState()
  startAutoUpdateLoop()
})

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

.key-head {
  text-align: center;
  margin-bottom: var(--space-2);
}

.key-title {
  margin: 0;
  font-size: var(--text-2xl);
  font-weight: 700;
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

.card.primary {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 1px var(--primary-soft);
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
