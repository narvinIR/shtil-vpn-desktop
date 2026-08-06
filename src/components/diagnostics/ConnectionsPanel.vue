<template>
  <div class="connections-panel">
    <ToolbarBar>
      <template #filters>
        <n-input
          v-model:value="connectionStore.searchQuery"
          :placeholder="t('connections.search')"
          clearable
          size="small"
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
        <n-button size="small" secondary @click="connectionStore.togglePaused()">
          {{ connectionStore.paused ? t('connections.resume') : t('connections.pause') }}
        </n-button>
      </template>
      <template #stats>
        <n-tag size="small" round :bordered="false" type="info">
          {{ t('connections.count') }}: {{ rows.length }}
        </n-tag>
        <n-tag size="small" round :bordered="false" type="success">
          ↓ {{ formatBytes(connectionStore.connectionsTotal.download) }}
        </n-tag>
        <n-tag size="small" round :bordered="false" type="warning">
          ↑ {{ formatBytes(connectionStore.connectionsTotal.upload) }}
        </n-tag>
      </template>
    </ToolbarBar>

    <div v-if="rows.length" class="table-card">
      <div class="table-wrap">
        <table class="connections-table">
          <thead>
            <tr>
              <th>{{ t('connections.destination') }}</th>
              <th>{{ t('connections.process') }}</th>
              <th>{{ t('connections.path') }}</th>
              <th>{{ t('connections.traffic') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="row in rows" :key="row.id">
              <td class="destination-cell" :title="row.destination">{{ row.destination }}</td>
              <td class="process-cell" :title="row.process">{{ row.process }}</td>
              <td>
                <n-tag size="small" round :bordered="false" :type="row.direct ? 'default' : 'primary'">
                  {{ row.direct ? t('connections.pathDirect') : t('connections.pathVpn') }}
                </n-tag>
              </td>
              <td class="traffic-cell">↓ {{ formatBytes(row.download) }} · ↑ {{ formatBytes(row.upload) }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <EmptyState v-else :title="t('connections.empty')" :icon="SwapHorizontalOutline" />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { SearchOutline, SwapHorizontalOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import ToolbarBar from '@/components/common/ToolbarBar.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { formatBytes } from '@/utils'
import type { ConnectionItem } from '@/types/events'

defineOptions({
  name: 'ConnectionsPanel',
})

const { t } = useI18n()
const connectionStore = useConnectionStore()

// Ядро отдаёт путь к программе вместе с владельцем: «/usr/bin/curl (dimas)».
// Человеку нужно имя, а не путь.
const processName = (path: string) => {
  const withoutOwner = path.replace(/\s*\(.*\)\s*$/, '').trim()
  const name = withoutOwner.split(/[\\/]/).pop() || ''
  return name
}

const destinationOf = (connection: ConnectionItem) => {
  const { host, destinationIP, destinationPort } = connection.metadata
  const target = host || destinationIP || ''
  return destinationPort ? `${target}:${destinationPort}` : target
}

const rows = computed(() => {
  const query = connectionStore.searchQuery.trim().toLowerCase()

  return connectionStore.activeConnections
    .map((connection) => ({
      id: connection.id,
      destination: destinationOf(connection),
      // Пусто, пока ядро не нашло программу: так бывает у служебных соединений
      // самого ядра.
      process: processName(connection.metadata.processPath || '') || '—',
      // Первый в цепочке — конечный выход: «direct» значит мимо VPN.
      direct: connection.chains[0] === 'direct',
      download: connection.download,
      upload: connection.upload,
    }))
    .filter((row) =>
      query ? row.destination.toLowerCase().includes(query) || row.process.toLowerCase().includes(query) : true,
    )
})
</script>

<style scoped>
.connections-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.table-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--panel-shadow);
  padding: var(--space-3) var(--space-4);
}

.table-wrap {
  overflow-x: auto;
}

.connections-table {
  width: 100%;
  min-width: 640px;
  border-collapse: collapse;
  table-layout: fixed;
}

.connections-table th {
  padding: 0 var(--space-3) var(--space-3);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: 600;
  text-align: left;
  white-space: nowrap;
}

.connections-table td {
  padding: var(--space-3);
  border-top: 1px solid var(--border-color);
  color: var(--text-secondary);
  vertical-align: middle;
  font-size: var(--text-sm);
}

.connections-table th:nth-child(2),
.connections-table td:nth-child(2) {
  width: 140px;
}

.connections-table th:nth-child(3),
.connections-table td:nth-child(3) {
  width: 120px;
}

.connections-table th:nth-child(4),
.connections-table td:nth-child(4) {
  width: 190px;
}

.destination-cell,
.process-cell {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.destination-cell {
  color: var(--text-primary);
  font-family: var(--font-mono);
}

.traffic-cell {
  font-family: var(--font-mono);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}
</style>
