<template>
  <div class="setting-section">
    <h3 class="setting-section-title">{{ props.t('setting.network.title') }}</h3>

    <div class="setting-row">
      <div class="setting-info">
        <div class="setting-label">{{ props.t('setting.network.ipv6') }}</div>
        <div class="setting-desc">{{ props.t('setting.network.ipv6Desc') }}</div>
      </div>
      <n-switch :value="props.appStore.preferIpv6" @update:value="props.onIpVersionChange" />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <div class="setting-label">{{ props.t('setting.network.ports') }}</div>
        <div class="setting-desc">{{ props.t('setting.network.portsDesc') }}</div>
      </div>
      <n-button size="small" secondary @click="props.showPortSettings">
        <template #icon><n-icon :size="14"><SettingsOutline /></n-icon></template>
        {{ props.t('setting.network.configure') }}
      </n-button>
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <div class="setting-label">{{ props.t('setting.network.allowLanAccess') }}</div>
        <div class="setting-desc">{{ props.t('setting.network.allowLanAccessDesc') }}</div>
      </div>
      <n-switch
        :value="props.appStore.allowLanAccess"
        @update:value="props.onLanAccessChange"
      />
    </div>

    <h3 class="setting-section-title">{{ props.t('setting.proxyAdvanced.title') }}</h3>

    <div class="collapsible-header" @click="toggleSection('proxy')">
      <span class="collapsible-label">{{ props.t('setting.proxyAdvanced.systemBypass') }}</span>
      <n-icon :size="16" class="collapse-arrow" :class="{ expanded: expandedSections.proxy }">
        <ChevronDownOutline />
      </n-icon>
    </div>
    <transition name="collapse">
      <div v-if="expandedSections.proxy" class="collapsible-body">
        <n-form label-placement="top" class="advanced-form">
          <n-form-item :label="props.t('setting.proxyAdvanced.systemBypass')">
            <n-input
              v-model:value="proxyAdvancedForm.systemProxyBypass"
              type="textarea"
              :rows="3"
              :placeholder="props.t('setting.proxyAdvanced.systemBypassPlaceholder')"
            />
          </n-form-item>

          <div class="form-section-title">{{ props.t('setting.proxyAdvanced.tunTitle') }}</div>

          <n-form-item :label="props.t('setting.proxyAdvanced.tunMtu')">
            <n-input-number v-model:value="proxyAdvancedForm.tunMtu" :min="576" :max="9000" />
          </n-form-item>

          <n-form-item :label="props.t('setting.proxyAdvanced.tunRouteExcludeAddress')">
            <n-input
              v-model:value="proxyAdvancedForm.tunRouteExcludeAddressText"
              type="textarea"
              :rows="3"
              :placeholder="props.t('setting.proxyAdvanced.tunRouteExcludeAddressPlaceholder')"
            />
          </n-form-item>

          <div class="setting-toggles-grid">
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.proxyAdvanced.enableIpv6') }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunEnableIpv6" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.proxyAdvanced.autoRoute') }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunAutoRoute" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.proxyAdvanced.strictRoute') }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunStrictRoute" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.proxyAdvanced.tunSelfHeal') }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunSelfHealEnabled" />
            </div>
          </div>

          <n-form-item
            v-if="proxyAdvancedForm.tunSelfHealEnabled"
            :label="props.t('setting.proxyAdvanced.tunSelfHealCooldown')"
          >
            <n-input-number
              v-model:value="proxyAdvancedForm.tunSelfHealCooldownSecs"
              :min="15"
              :max="600"
            />
          </n-form-item>

          <n-button
            type="primary"
            block
            :loading="savingAdvanced"
            @click="saveProxyAdvancedSettings"
          >
            {{ props.t('setting.proxyAdvanced.save') }}
          </n-button>
        </n-form>
      </div>
    </transition>

  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { SettingsOutline, ChevronDownOutline } from '@vicons/ionicons5'
import { useMessage } from 'naive-ui'
import type { useAppStore } from '@/stores'
import { useAdvancedSettingsForm } from '@/views/setting/useAdvancedSettingsForm'

type AppStoreLike = ReturnType<typeof useAppStore>

const props = defineProps<{
  t: (key: string, params?: Record<string, string | number>) => string
  appStore: AppStoreLike
  onIpVersionChange: (value: boolean) => void | Promise<void>
  onLanAccessChange: (value: boolean) => void | Promise<void>
  showPortSettings: () => void
}>()

const message = useMessage()

const expandedSections = reactive({
  proxy: false,
})

const toggleSection = (key: keyof typeof expandedSections) => {
  expandedSections[key] = !expandedSections[key]
}

const { savingAdvanced, proxyAdvancedForm, saveProxyAdvancedSettings } = useAdvancedSettingsForm({
  appStore: props.appStore,
  message,
  t: props.t,
})

</script>

<style scoped>
.collapsible-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) 0;
  cursor: pointer;
  user-select: none;
  border-top: 1px solid var(--border-color);
}

.collapsible-header:hover .collapsible-label {
  color: var(--primary-color);
}

.collapsible-label {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-secondary);
  transition: color var(--transition-fast);
}

.collapse-arrow {
  color: var(--text-tertiary);
  transition: transform var(--transition-fast);
  flex-shrink: 0;
}

.collapse-arrow.expanded {
  transform: rotate(180deg);
}

.collapsible-body {
  padding: 0 0 var(--space-4);
}

.form-section-title {
  font-size: var(--text-sm);
  font-weight: 600;
  color: var(--text-secondary);
  margin: var(--space-4) 0 var(--space-2);
  padding-bottom: var(--space-2);
  border-bottom: 1px solid var(--border-color);
}

.form-section-title:first-child {
  margin-top: var(--space-2);
}

.setting-hint {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: 1.5;
  margin: var(--space-1) 0 0;
}

/* Тот же случай, что в setting-shared.css: у открытого состояния не было
   предела высоты, поэтому раскрытия не происходило вовсе. */
.collapse-enter-active,
.collapse-leave-active {
  transition:
    opacity var(--transition-fast),
    max-height var(--transition-base);
  overflow: hidden;
}

.collapse-enter-to,
.collapse-leave-from {
  max-height: 600px;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
}

@media (prefers-reduced-motion: reduce) {
  .collapse-enter-active,
  .collapse-leave-active {
    transition: opacity var(--transition-fast);
  }
}
</style>
