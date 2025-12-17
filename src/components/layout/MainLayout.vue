<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-layout class="app-layout" position="absolute">
      <!-- Top Header -->
      <n-layout-header class="app-header" bordered data-tauri-drag-region>
        <div class="header-brand">
          <div class="brand-logo-wrapper" @click="onSelect('home')">
            <img
              :src="logo"
              alt="Logo"
              class="brand-logo"
              :class="{ 'is-running': kernelStatusClass === 'running' }"
            />
          </div>
          <div class="brand-text">
            <h1 class="app-name">{{ t('common.appName') }}</h1>
            <div class="app-status" :class="kernelStatusClass">
              <span class="status-dot"></span>
              {{ appStatusLabel }}
            </div>
          </div>
        </div>

        <!-- Window Controls -->
        <div class="window-controls">
          <button class="control-btn minimize" @click="windowStore.minimizeWindow">
            <n-icon size="16"><RemoveOutline /></n-icon>
          </button>
          <button class="control-btn maximize" @click="windowStore.toggleMaximize">
            <n-icon size="14"><SquareOutline /></n-icon>
          </button>
          <button class="control-btn close" @click="() => windowStore.hideWindow(router)">
            <n-icon size="16"><CloseOutline /></n-icon>
          </button>
        </div>
      </n-layout-header>

      <!-- Main Layout with Sidebar and Content -->
      <n-layout has-sider position="absolute" style="top: 50px; bottom: 0">
        <!-- Modern Sidebar -->
        <n-layout-sider
          class="app-sider"
          :width="240"
          :collapsed-width="72"
          :collapsed="collapsed"
          collapse-mode="width"
          :native-scrollbar="false"
          :show-trigger="false"
          bordered
        >
          <div class="sider-inner">
            <!-- Navigation Menu -->
            <div class="sider-menu">
              <div class="menu-group">
                <div class="menu-label" v-if="!collapsed">{{ t('nav.navigation') }}</div>
                <div
                  v-for="item in menuItems"
                  :key="item.key"
                  class="menu-item"
                  :class="{ active: currentMenu === item.key, disabled: item.disabled }"
                  @click="!item.disabled && onSelect(item.key)"
                >
                  <n-icon :size="22" class="menu-icon">
                    <component :is="item.icon" />
                  </n-icon>
                  <transition name="fade-slide">
                    <span v-if="!collapsed" class="menu-text">{{ item.label }}</span>
                  </transition>
                  <div class="active-indicator" v-if="currentMenu === item.key"></div>
                </div>
              </div>
            </div>

            <!-- Bottom Actions -->
            <div class="sider-footer">
              <div class="footer-actions">
                <n-tooltip placement="right" trigger="hover" :disabled="!collapsed">
                  <template #trigger>
                    <button class="action-btn" @click="themeStore.toggleTheme">
                      <n-icon :size="20">
                        <MoonOutline v-if="themeStore.isDark" />
                        <SunnyOutline v-else />
                      </n-icon>
                    </button>
                  </template>
                  {{ themeStore.isDark ? t('setting.theme.light') : t('setting.theme.dark') }}
                </n-tooltip>

                <button class="action-btn" @click="collapsed = !collapsed">
                  <n-icon :size="20">
                    <ChevronForwardOutline v-if="collapsed" />
                    <ChevronBackOutline v-else />
                  </n-icon>
                </button>
              </div>
            </div>
          </div>
        </n-layout-sider>

        <!-- Main Content Area -->
        <n-layout-content class="app-content" :native-scrollbar="false">
          <!-- Page Content -->
          <div class="content-container">
            <router-view v-slot="{ Component }">
              <transition name="page-fade" mode="out-in">
                <component :is="Component" :key="$route.path" />
              </transition>
            </router-view>
          </div>
        </n-layout-content>
      </n-layout>
    </n-layout>

    <!-- Update Modal -->
    <UpdateModal
      v-model:show="showUpdateModal"
      :latest-version="updateInfo.latestVersion"
      :current-version="updateInfo.currentVersion"
      :download-url="updateInfo.downloadUrl"
      :release-notes="updateInfo.releaseNotes"
      :release-date="updateInfo.releaseDate"
      :file-size="updateInfo.fileSize"
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
import { useLocaleStore } from '@/stores/app/LocaleStore'
import { useAppStore } from '@/stores'
import { useWindowStore } from '@/stores/app/WindowStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useI18n } from 'vue-i18n'
import {
  HomeOutline,
  SwapHorizontalOutline,
  LinkOutline,
  DocumentTextOutline,
  SettingsOutline,
  FolderOutline,
  MoonOutline,
  SunnyOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
  RemoveOutline,
  SquareOutline,
  CloseOutline,
  AnalyticsOutline
} from '@vicons/ionicons5'
import { useMessage } from 'naive-ui'
import mitt from 'mitt'
import UpdateModal from '@/components/UpdateModal.vue'
import logo from '@/assets/icon.png'
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
const localeStore = useLocaleStore()
const appStore = useAppStore()
const windowStore = useWindowStore()
const updateStore = useUpdateStore()
const kernelStore = useKernelStore()
const { t } = useI18n()
const { statusState: kernelStatusState, statusClass: kernelStatusClass } = useKernelStatus(kernelStore)

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
    default:
      return t('status.stopped')
  }
})

// Update Modal State
const showUpdateModal = ref(false)
const updateInfo = ref({
  latestVersion: '',
  currentVersion: '',
  downloadUrl: '',
  releaseNotes: '',
  releaseDate: '',
  fileSize: 0,
})

// Theme Configuration
const theme = computed(() => themeStore.naiveTheme)
const themeOverrides = computed(() => themeStore.themeOverrides)

// Menu Configuration
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

const menuItems = computed(() => [
  { label: t('nav.home'), key: 'home', icon: HomeOutline, disabled: false },
  { label: t('nav.subscription'), key: 'subscription', icon: FolderOutline, disabled: false },
  { label: t('nav.proxy'), key: 'proxy', icon: SwapHorizontalOutline, disabled: false },
  { label: t('nav.connections'), key: 'connections', icon: LinkOutline, disabled: false },
  { label: t('nav.logs'), key: 'logs', icon: DocumentTextOutline, disabled: false },
  { label: t('nav.rules'), key: 'rules', icon: AnalyticsOutline, disabled: false },
  { label: t('nav.settings'), key: 'settings', icon: SettingsOutline, disabled: false },
])

// Navigation
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

// Update Handling
const handleShowUpdateModal = (data: unknown) => {
  if (!data || typeof data !== 'object') return

  const payload = data as Record<string, unknown>
  updateInfo.value = {
    latestVersion: typeof payload.latestVersion === 'string' ? payload.latestVersion : '',
    currentVersion:
      typeof payload.currentVersion === 'string' ? payload.currentVersion : updateStore.appVersion,
    downloadUrl: typeof payload.downloadUrl === 'string' ? payload.downloadUrl : '',
    releaseNotes: typeof payload.releaseNotes === 'string' ? payload.releaseNotes : '',
    releaseDate: typeof payload.releaseDate === 'string' ? payload.releaseDate : '',
    fileSize: typeof payload.fileSize === 'number' ? payload.fileSize : 0,
  }
  showUpdateModal.value = true
}

const handleUpdate = async () => {
  try {
    message.info('Starting download...')
    await updateStore.downloadAndInstallUpdate()
    showUpdateModal.value = false
  } catch (error) {
    message.error(`Update failed: ${error}`)
  }
}

const handleUpdateCancel = () => {
  showUpdateModal.value = false
  message.info('Update cancelled')
}

const handleUpdateSkip = () => {
  showUpdateModal.value = false
  updateStore.skipCurrentVersion()
  message.info('Version skipped')
}

// Lifecycle
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
  background: transparent;
}

/* Header Styles */
.app-header {
  height: 50px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  background: var(--glass-bg) !important;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  z-index: 200;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: default;
}

.brand-logo-wrapper {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform 0.2s ease;
}

.brand-logo-wrapper:hover {
  transform: scale(1.05);
}

.brand-logo {
  width: 28px;
  height: 28px;
  object-fit: contain;
  filter: drop-shadow(0 4px 8px rgba(0, 0, 0, 0.1));
  transition: all 0.3s ease;
}

.brand-logo.is-running {
  filter: drop-shadow(0 0 12px rgba(99, 102, 241, 0.5));
}

.brand-text {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: 12px;
  white-space: nowrap;
}

.app-name {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  line-height: 1;
}

.app-status {
  font-size: 12px;
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background-color: var(--text-tertiary);
  transition: background-color 0.3s ease;
}

.app-status.running .status-dot {
  background-color: var(--success-color, #10b981);
  box-shadow: 0 0 8px var(--success-color, #10b981);
}

.app-status.running {
  color: var(--success-color, #10b981);
}

.app-status.pending,
.app-status.disconnected {
  color: var(--warning-color, #f59e0b);
}

.app-status.pending .status-dot,
.app-status.disconnected .status-dot {
  background-color: var(--warning-color, #f59e0b);
  box-shadow: 0 0 6px var(--warning-color, #f59e0b);
}

.app-status.stopped {
  color: var(--error-color, #ef4444);
}

.app-status.stopped .status-dot {
  background-color: var(--error-color, #ef4444);
  box-shadow: 0 0 6px var(--error-color, #ef4444);
}

/* Window Controls */
.window-controls {
  display: flex;
  gap: 8px;
  -webkit-app-region: no-drag;
}

.control-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.control-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.control-btn.close:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* Sidebar Styles */
.app-sider {
  background: var(--glass-bg) !important;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  z-index: 100;
}

.sider-inner {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px 16px;
}

/* Menu Styles */
.sider-menu {
  flex: 1;
  overflow-y: auto;
}

.menu-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.menu-label {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-tertiary);
  letter-spacing: 0.05em;
  padding: 0 12px;
  margin-bottom: 8px;
}

.menu-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 12px;
  border-radius: 12px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.2s ease;
  height: 48px;
}

.menu-item:hover:not(.disabled) {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.menu-item.active {
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.1), rgba(99, 102, 241, 0.05));
  color: var(--primary-color);
}

.menu-item.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.menu-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

.menu-text {
  font-weight: 500;
  font-size: 14px;
  white-space: nowrap;
}

.active-indicator {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 4px;
  height: 20px;
  background: var(--primary-color);
  border-radius: 4px 0 0 4px;
}

/* Footer Styles */
.sider-footer {
  padding-top: 20px;
  border-top: 1px solid var(--border-color);
}

.footer-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.action-btn {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

/* Content Styles */
.app-content {
  background: var(--bg-primary);
  position: relative;
}

.content-container {
  height: 100%;
  /* padding-top: 40px; Removed as window controls are now in header */
  overflow-y: auto;
  overflow-x: hidden;
}

/* Transitions */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.3s ease;
}

.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-10px);
}

.page-fade-enter-active,
.page-fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.page-fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
