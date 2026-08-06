<template>
  <div class="page-shell diagnostics-page">
    <PageHeader :title="t('diagnostics.title')" :subtitle="t('diagnostics.subtitle')" />

    <n-tabs v-model:value="tab" type="segment" size="small">
      <n-tab-pane name="log" :tab="t('diagnostics.tabLog')">
        <LogPanel />
      </n-tab-pane>
      <n-tab-pane name="connections" :tab="t('diagnostics.tabConnections')">
        <ConnectionsPanel />
      </n-tab-pane>
    </n-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import PageHeader from '@/components/common/PageHeader.vue'
import LogPanel from '@/components/diagnostics/LogPanel.vue'
import ConnectionsPanel from '@/components/diagnostics/ConnectionsPanel.vue'

defineOptions({
  name: 'DiagnosticsView',
})

const { t } = useI18n()
const tab = ref<'log' | 'connections'>('log')
</script>

<style scoped>
.diagnostics-page {
  max-width: var(--content-max-width, 1440px);
  margin: 0 auto;
}
</style>
