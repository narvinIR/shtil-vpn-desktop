<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <div class="app-layout">
      <!-- 顶栏 -->
      <AppHeader
        :app-name="t('common.appName')"
        :kernel-status-class="kernelStatusClass"
        :app-status-label="appStatusLabel"
        :upload-speed="trafficStore.traffic.up"
        :download-speed="trafficStore.traffic.down"
        :connection-count="connectionStore.activeConnections.length"
        @home="onSelect('home')"
        @minimize="windowStore.minimizeWindow"
        @toggle-maximize="windowStore.toggleMaximize"
        @close="() => windowStore.closeToTray(router)"
      />

      <!-- 侧栏 + 内容 -->
      <div class="app-body">
        <AppSidebar
          :collapsed="collapsed"
          :current-menu="currentMenu"
          :menu-items="menuItems"
          :groups="menuGroups"
          :is-dark="themeStore.isDark"
          @select="onSelect"
          @toggle-theme="themeStore.toggleTheme"
          @toggle-collapse="collapsed = !collapsed"
        />

        <main class="app-content">
          <div class="content-container">
            <router-view v-slot="{ Component }">
              <transition name="page-fade" mode="out-in">
                <component :is="Component" :key="$route.path" />
              </transition>
            </router-view>
          </div>
        </main>
      </div>
    </div>

    <!-- 更新弹窗 -->
    <UpdateModal
      v-model:show="showUpdateModal"
      :latest-version="updateInfo.latestVersion"
      :current-version="updateInfo.currentVersion"
      :download-url="updateInfo.downloadUrl"
      :release-page-url="updateInfo.releasePageUrl"
      :release-notes="updateInfo.releaseNotes"
      :release-date="updateInfo.releaseDate"
      :file-size="updateInfo.fileSize"
      :supports-in-app-update="updateInfo.supportsInAppUpdate"
      @update="handleUpdate"
      @cancel="handleUpdateCancel"
      @skip="handleUpdateSkip"
    />
  </n-config-provider>
</template>

<script lang="ts" setup>
import { computed, ref, onMounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { useWindowStore } from '@/stores/app/WindowStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useI18n } from 'vue-i18n'
import { HomeOutline, KeyOutline, PulseOutline, SettingsOutline } from '@vicons/ionicons5'
import { useMessage } from 'naive-ui'
import UpdateModal from '@/components/UpdateModal.vue'
import AppHeader from './AppHeader.vue'
import AppSidebar, { type NavGroup, type NavItem } from './AppSidebar.vue'
import { useKernelStatus } from '@/composables/useKernelStatus'

defineOptions({
  name: 'MainLayout',
})

const router = useRouter()
const route = useRoute()
const collapsed = ref(false)
const message = useMessage()

// Stores
const themeStore = useThemeStore()
const windowStore = useWindowStore()
const updateStore = useUpdateStore()
const kernelStore = useKernelStore()
const trafficStore = useTrafficStore()
const connectionStore = useConnectionStore()
const { t } = useI18n()
const { statusState: kernelStatusState, statusClass: kernelStatusClass } =
  useKernelStatus(kernelStore)

const appStatusLabel = computed(() => {
  switch (kernelStatusState.value) {
    case 'starting':
      return t('status.starting')
    case 'stopping':
      return t('status.stopping')
    case 'running':
      return t('status.running')
    case 'disconnected':
      return t('status.disconnected')
    case 'failed':
      return t('status.failed')
    case 'crashed':
      return t('status.crashed')
    default:
      return t('status.stopped')
  }
})

// 更新弹窗状态
const showUpdateModal = ref(false)
const updateInfo = ref({
  latestVersion: '',
  currentVersion: '',
  downloadUrl: '',
  releasePageUrl: '',
  releaseNotes: '',
  releaseDate: '',
  fileSize: 0,
  supportsInAppUpdate: false,
})

// 主题配置
const theme = computed(() => themeStore.naiveTheme)
const themeOverrides = computed(() => themeStore.themeOverrides)

// Меню: главный экран, ключ, диагностика, настройки. Диагностика стоит здесь,
// а не только в обслуживании: нужна она ровно тогда, когда что-то не работает,
// и искать её в такую минуту через четыре нажатия никто не станет.
const currentMenu = computed(() => {
  const path = route.path
  if (path === '/' || path === '/home') return 'home'

  const pathToMenuMap: Record<string, string> = {
    '/sub': 'key',
    '/setting': 'settings',
    '/log': 'diagnostics',
  }
  return pathToMenuMap[path] || path.slice(1)
})

const menuItems = computed<NavItem[]>(() => [
  { label: t('nav.home'), key: 'home', icon: HomeOutline },
  { label: t('nav.key'), key: 'key', icon: KeyOutline },
  { label: t('diagnostics.entryTitle'), key: 'diagnostics', icon: PulseOutline },
  { label: t('nav.settings'), key: 'settings', icon: SettingsOutline },
])

const menuGroups = computed<NavGroup[]>(() => [
  { key: 'main', items: menuItems.value.slice(0, 2) },
  { key: 'more', items: menuItems.value.slice(2) },
])

const onSelect = (key: string) => {
  if (key === 'home') {
    router.push('/')
    return
  }
  const routeMap: Record<string, string> = {
    key: '/sub',
    diagnostics: '/log',
    settings: '/setting',
  }
  router.push(routeMap[key] || '/')
}

// Предложение обновиться. Саму проверку ведёт корень приложения (App.vue):
// раскладка живёт только при открытом окне, и уход в трей снимал её вместе с
// таймером — замер на Mac Studio 06.08.2026 показал, что после этого
// приложение о новой версии не спрашивало ни разу.
const openUpdateModal = () => {
  updateInfo.value = {
    latestVersion: updateStore.latestVersion,
    currentVersion: updateStore.appVersion,
    downloadUrl: updateStore.downloadUrl,
    releasePageUrl: updateStore.releasePageUrl,
    releaseNotes: updateStore.releaseNotes,
    releaseDate: updateStore.releaseDate,
    fileSize: updateStore.fileSize,
    supportsInAppUpdate: updateStore.supportsInAppUpdate,
  }
  showUpdateModal.value = true
}

const handleUpdate = async () => {
  try {
    if (updateInfo.value.supportsInAppUpdate) {
      message.info(t('setting.update.preparingDownload'))
      await updateStore.downloadAndInstallUpdate()
    } else {
      await updateStore.openReleasePage()
      showUpdateModal.value = false
    }
    showUpdateModal.value = false
  } catch (error) {
    // Плагин обновления отвечает своим машинным текстом, а это единственный
    // путь обновления в продукте: человеку нужна фраза, а не ответ библиотеки.
    console.warn('[update]', error)
    message.error(t('setting.update.updateFailed'))
  }
}

const handleUpdateCancel = () => {
  showUpdateModal.value = false
}

const handleUpdateSkip = async () => {
  showUpdateModal.value = false
  await updateStore.skipCurrentVersion()
  message.success(t('setting.update.skipSuccess'))
}

// Новая версия могла найтись, пока окно лежало в трее: показываем предложение
// сразу, как окно вернулось, и дальше по ходу проверок.
onMounted(() => {
  if (updateStore.hasUpdate) openUpdateModal()
})

watch(
  () => updateStore.hasUpdate,
  (hasUpdate) => {
    if (hasUpdate) openUpdateModal()
  },
)
</script>

<style scoped>
.app-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-base);
  /* На Маке угол окна скругляем сами: украшений системы у окна нет. На
     остальных системах здесь ноль, и правило ничего не меняет. */
  border-radius: var(--window-radius);
  overflow: hidden;
}

/* Стеклянная кромка по краю окна: тонкий свет, повторяющий скругление */
.platform-macos .app-layout {
  box-shadow: inset 0 0 0 0.5px var(--glass-border);
  /* Маска заставляет WebKit применить скругление и к тем слоям, которые
     он рисует отдельно (движущаяся волна на главном экране). Без неё
     `overflow: hidden` их не касается, и правый нижний угол оставался
     квадратным, пока три остальных были скруглены. Цвета маски роли не
     играют — важна только её непрозрачность. */
  -webkit-mask-image: -webkit-radial-gradient(white, black);
}

.app-body {
  flex: 1;
  display: flex;
  min-height: 0;
}

.app-content {
  flex: 1;
  background: var(--bg-base);
  position: relative;
  min-width: 0;
}

/* На Маке фон уже положен корнем окна. Второй раз тем же цветом — и стекло
   системы под окном перестаёт просвечивать: два слоя по 0.34 дают 0.56. */
.platform-macos .app-content {
  background: transparent;
}

.content-container {
  height: 100%;
  overflow-y: auto;
  overflow-x: hidden;
}

/* 页面过渡 */
.page-fade-enter-active,
.page-fade-leave-active {
  transition:
    opacity var(--transition-fast),
    transform var(--transition-fast);
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.page-fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
