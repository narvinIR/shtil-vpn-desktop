<template>
  <div class="setting-section">
    <div class="about-hero">
      <!-- Здесь стоял значок GitHub — приложение представлялось человеку чужим
           знаком. Знак наш, тот же файл, что в шапке окна. -->
      <div class="about-logo">
        <img :src="logo" alt="" class="about-logo-img" />
      </div>
      <div class="about-identity">
        <div class="about-name">{{ props.t('common.appName') }}</div>
        <div class="about-tagline">{{ props.t('setting.subtitle') }}</div>
      </div>
    </div>

    <div class="about-rows">
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.appVersion') }}</div>
        </div>
        <div class="setting-value">v{{ props.updateStore.appVersion }}</div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.kernel.version') }}</div>
        </div>
        <div class="setting-value">
          {{
            props.kernelStore.hasVersionInfo()
              ? 'v' + props.formatVersion(props.kernelStore.getVersionString())
              : props.t('setting.notInstalled')
          }}
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.about.system') }}</div>
        </div>
        <div class="setting-value">{{ props.platformInfo?.display_name || props.t('common.loading') }}</div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.about.license') }}</div>
        </div>
        <div class="setting-value">MIT License · sing-box-windows (MIT), sing-box (GPLv3)</div>
      </div>
    </div>

    <div class="about-footer">
      <n-button
        text
        tag="a"
        href="https://github.com/narvinIR/shtil-vpn-desktop"
        target="_blank"
      >
        <template #icon>
          <n-icon :size="16"><LogoGithub /></n-icon>
        </template>
        GitHub
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  LogoGithub,
} from '@vicons/ionicons5'
import logo from '@/assets/icon.png'
import type { useKernelStore, useUpdateStore } from '@/stores'

type KernelStoreLike = ReturnType<typeof useKernelStore>
type UpdateStoreLike = ReturnType<typeof useUpdateStore>

const props = defineProps<{
  t: (key: string, params?: Record<string, string | number>) => string
  updateStore: UpdateStoreLike
  kernelStore: KernelStoreLike
  platformInfo: { os: string; arch: string; display_name: string } | null
  formatVersion: (value: string) => string
}>()
</script>

<style scoped>
.about-hero {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-5) 0;
  border-bottom: 1px solid var(--border-color);
}

.about-logo {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-md);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  /* Тень нейтральная, как у знака в шапке окна: синее свечение вокруг тёмной
     плитки читалось подсветкой выбранного элемента. */
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.18);
}

/* Знак сам себе плитка — синяя заливка под ним давала цветную рамку по краю. */
.about-logo-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  display: block;
}

.about-name {
  font-size: var(--text-lg);
  font-weight: 700;
  color: var(--text-primary);
}

.about-tagline {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  margin-top: 2px;
}

.about-rows {
  display: flex;
  flex-direction: column;
}

.setting-value {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-secondary);
}

.about-footer {
  display: flex;
  justify-content: center;
  padding: var(--space-4) 0 var(--space-2);
}
</style>
