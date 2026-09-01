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
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { useWindowStore } from '@/stores/app/WindowStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useTrafficStore } from '@/stores/kernel/TrafficStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useI18n } from 'vue-i18n'
import {
  HomeOutline,
  SwapHorizontalOutline,
  LinkOutline,
  DocumentTextOutline,
  SettingsOutline,
  FolderOutline,
} from '@vicons/ionicons5'
import { useMessage } from 'naive-ui'
import mitt from 'mitt'
import UpdateModal from '@/components/UpdateModal.vue'
import AppHeader from './AppHeader.vue'
import AppSidebar, { type NavItem } from './AppSidebar.vue'
import { useKernelStatus } from '@/composables/useKernelStatus'

defineOptions({
  name: 'MainLayout',
})

const router = useRouter()
const route = useRoute()
const collapsed = ref(false)
const message = useMessage()
const mittInstance = mitt()

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

// 菜单
const currentMenu = computed(() => {
  const path = route.path
  if (path === '/' || path === '/home') return 'home'

  const pathToMenuMap: Record<string, string> = {
    '/log': 'logs',
    '/sub': 'subscription',
    '/setting': 'settings',
    '/connections': 'connections',
    '/proxy': 'proxy',
    '/rules': 'rules',
  }
  return pathToMenuMap[path] || path.slice(1)
})

const menuItems = computed<NavItem[]>(() => [
  { label: t('nav.home'), key: 'home', icon: HomeOutline },
  { label: t('nav.subscription'), key: 'subscription', icon: FolderOutline },
  { label: t('nav.proxy'), key: 'proxy', icon: SwapHorizontalOutline },
  { label: t('nav.connections'), key: 'connections', icon: LinkOutline },
  { label: t('nav.logs'), key: 'logs', icon: DocumentTextOutline },
  // 规则页隐藏：sing-box 官方 gRPC API 未暴露规则管理接口（see docs/sing-box-api-migration.md）
  // 等待上游 API 扩展或自建后端命令后恢复：{ label: t('nav.rules'), key: 'rules', icon: AnalyticsOutline },
  { label: t('nav.settings'), key: 'settings', icon: SettingsOutline },
])

// 导航
const onSelect = (key: string) => {
  if (key === 'home') {
    router.push('/')
  } else {
    const routeMap: Record<string, string> = {
      logs: '/log',
      subscription: '/sub',
      settings: '/setting',
      connections: '/connections',
      proxy: '/proxy',
      rules: '/rules',
    }
    router.push(routeMap[key] || `/${key}`)
  }
}

// 更新处理
const handleShowUpdateModal = (data: unknown) => {
  if (!data || typeof data !== 'object') return

  const payload = data as Record<string, unknown>
  updateInfo.value = {
    latestVersion:
      typeof payload.latestVersion === 'string'
        ? payload.latestVersion
        : typeof payload.latest_version === 'string'
          ? payload.latest_version
          : '',
    currentVersion:
      typeof payload.currentVersion === 'string' ? payload.currentVersion : updateStore.appVersion,
    downloadUrl:
      typeof payload.downloadUrl === 'string'
        ? payload.downloadUrl
        : typeof payload.download_url === 'string'
          ? payload.download_url
          : '',
    releasePageUrl:
      typeof payload.releasePageUrl === 'string'
        ? payload.releasePageUrl
        : typeof payload.release_page_url === 'string'
          ? payload.release_page_url
          : updateStore.releasePageUrl,
    releaseNotes:
      typeof payload.releaseNotes === 'string'
        ? payload.releaseNotes
        : typeof payload.release_notes === 'string'
          ? payload.release_notes
          : '',
    releaseDate:
      typeof payload.releaseDate === 'string'
        ? payload.releaseDate
        : typeof payload.release_date === 'string'
          ? payload.release_date
          : '',
    fileSize:
      typeof payload.fileSize === 'number'
        ? payload.fileSize
        : typeof payload.file_size === 'number'
          ? payload.file_size
          : 0,
    supportsInAppUpdate:
      typeof payload.supportsInAppUpdate === 'boolean'
        ? payload.supportsInAppUpdate
        : typeof payload.supports_in_app_update === 'boolean'
          ? payload.supports_in_app_update
          : updateStore.supportsInAppUpdate,
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
    const errMsg = error instanceof Error ? error.message : String(error)
    message.error(errMsg)
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

// 生命周期
onMounted(() => {
  mittInstance.on('show-update-modal', handleShowUpdateModal)
})

onUnmounted(() => {
  mittInstance.off('show-update-modal', handleShowUpdateModal)
})
</script>

<style scoped>
.app-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-base);
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
