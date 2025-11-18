<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-layout class="modern-layout" position="absolute">
      <!-- 现代化顶部栏 -->
      <n-layout-header class="modern-header" :style="{ height: '48px' }">
        <div class="header-content" data-tauri-drag-region>
          <!-- 左侧：品牌区域 -->
          <div class="header-brand">
            <div class="brand-logo" @click="onSelect('home')" data-tauri-drag-region="false">
              <div class="logo-container">
                <img
                  :src="logo"
                  alt="App Logo"
                  class="logo-image"
                  :class="{ 'logo-active': appStore.isRunning }"
                />
              </div>
            </div>
            <div class="brand-info">
              <div class="brand-info-row">
                <h1 class="app-title">{{ t('common.appName') }}</h1>
                <div class="status-badge" :class="statusClass">
                  <div class="status-dot"></div>
                  <span class="status-text">{{
                    appStore.isRunning ? t('status.running') : t('status.stopped')
                  }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 右侧：窗口控制 -->
          <div class="window-controls">
            <button class="window-btn minimize" @click="windowStore.minimizeWindow">
              <svg width="12" height="12" viewBox="0 0 12 12">
                <rect x="2" y="5.5" width="8" height="1" rx="0.5" fill="currentColor" />
              </svg>
            </button>
            <button class="window-btn maximize" @click="windowStore.toggleMaximize">
              <svg width="12" height="12" viewBox="0 0 12 12">
                <rect
                  x="2"
                  y="2"
                  width="8"
                  height="8"
                  rx="1"
                  stroke="currentColor"
                  stroke-width="1.5"
                  fill="none"
                />
              </svg>
            </button>
            <button class="window-btn close" @click="() => windowStore.hideWindow(router)">
              <svg width="12" height="12" viewBox="0 0 12 12">
                <path
                  d="M3 3L9 9M9 3L3 9"
                  stroke="currentColor"
                  stroke-width="1.5"
                  stroke-linecap="round"
                />
              </svg>
            </button>
          </div>
        </div>
      </n-layout-header>

      <!-- 主内容区域 -->
      <n-layout has-sider position="absolute" style="top: 48px" class="main-container">
        <!-- 现代化侧边栏 -->
        <n-layout-sider
          class="modern-sider"
          :width="200"
          :collapsed-width="56"
          :collapsed="collapsed"
          collapse-mode="width"
          :native-scrollbar="false"
          :show-trigger="false"
        >
          <div class="sider-content">
            <!-- 导航区域 -->
            <div class="nav-section">
              <div class="nav-header" v-if="!collapsed">
                <span class="nav-title">{{ t('nav.navigation') }}</span>
              </div>

              <div class="nav-menu">
                <div
                  v-for="item in menuItems"
                  :key="item.key"
                  class="nav-item"
                  :class="{
                    'nav-active': currentMenu === item.key,
                    'nav-disabled': item.disabled,
                  }"
                  @click="!item.disabled && onSelect(item.key)"
                >
                  <div class="nav-icon-wrapper">
                    <n-icon :size="20" :component="item.icon" />
                  </div>
                  <transition name="nav-text">
                    <span v-if="!collapsed" class="nav-label">{{ item.label }}</span>
                  </transition>
                </div>
              </div>
            </div>

            <!-- 底部工具区 -->
            <div class="tools-section">
              <div class="tool-group">
                <div class="tool-item" @click="themeStore.toggleTheme">
                  <div class="tool-icon">
                    <n-icon :size="18">
                      <MoonOutline v-if="themeStore.isDark" />
                      <SunnyOutline v-else />
                    </n-icon>
                  </div>
                  <transition name="nav-text">
                    <span v-if="!collapsed" class="tool-label">
                      {{ themeStore.isDark ? t('setting.theme.dark') : t('setting.theme.light') }}
                    </span>
                  </transition>
                </div>

                <div class="tool-divider" v-if="!collapsed"></div>

                <div class="tool-item collapse-toggle" @click="collapsed = !collapsed">
                  <div class="tool-icon">
                    <n-icon :size="18">
                      <ChevronBackOutline v-if="!collapsed" />
                      <ChevronForwardOutline v-else />
                    </n-icon>
                  </div>
                  <transition name="nav-text">
                    <span v-if="!collapsed" class="tool-label">
                      {{ collapsed ? t('nav.expand') : t('nav.collapse') }}
                    </span>
                  </transition>
                </div>
              </div>
            </div>
          </div>
        </n-layout-sider>

        <!-- 内容区域 -->
        <n-layout-content class="modern-content">
          <div class="content-wrapper">
            <router-view v-slot="{ Component }">
              <transition name="page-transition" mode="out-in">
                <component :is="Component" :key="$route.path" />
              </transition>
            </router-view>
          </div>
        </n-layout-content>
      </n-layout>
    </n-layout>

    <!-- 更新弹窗 -->
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
import { useI18n } from 'vue-i18n'
import { darkTheme, type GlobalThemeOverrides } from 'naive-ui'
import {
  HomeOutline,
  SwapHorizontalOutline,
  LinkOutline,
  AnalyticsOutline,
  DocumentTextOutline,
  SettingsOutline,
  FolderOutline,
  MoonOutline,
  SunnyOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
} from '@vicons/ionicons5'
import { useMessage } from 'naive-ui'
import mitt from 'mitt'
import UpdateModal from '@/components/UpdateModal.vue'
import logo from '@/assets/icon.png'

defineOptions({
  name: 'MainLayout',
})

const router = useRouter()
const route = useRoute()
const collapsed = ref(false)
const message = useMessage()
const mittInstance = mitt()

// 更新弹窗相关
const showUpdateModal = ref(false)
const updateInfo = ref({
  latestVersion: '',
  currentVersion: '',
  downloadUrl: '',
  releaseNotes: '',
  releaseDate: '',
  fileSize: 0,
})

// Store实例
const themeStore = useThemeStore()
const localeStore = useLocaleStore()
const appStore = useAppStore()
const windowStore = useWindowStore()
const updateStore = useUpdateStore()
const { t } = useI18n()

// 主题配置
const theme = computed(() => (themeStore.isDark ? darkTheme : null))

const themeOverrides = computed<GlobalThemeOverrides>(() => ({
  common: {
    primaryColor: '#5b4cfd',
    primaryColorHover: '#7b6dfd',
    primaryColorPressed: '#4b3ced',
    primaryColorSuppl: '#6b5dfd',
    borderRadius: '12px',
    borderRadiusSmall: '8px',
    borderRadiusMedium: '10px',
    fontSize: '14px',
    fontFamily:
      '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  },
  Layout: {
    headerHeight: '48px',
    siderBorderColor: 'transparent',
    color: 'transparent',
    siderColor: themeStore.isDark ? 'rgba(24, 24, 28, 0.95)' : 'rgba(255, 255, 255, 0.95)',
    contentColor: 'transparent',
  },
  Card: {
    borderRadius: '16px',
    color: themeStore.isDark ? 'rgba(24, 24, 28, 0.8)' : 'rgba(255, 255, 255, 0.9)',
    borderColor: 'transparent',
  },
  Button: {
    borderRadiusSmall: '8px',
    borderRadiusMedium: '10px',
    heightSmall: '32px',
    heightMedium: '36px',
    heightLarge: '42px',
    fontSizeSmall: '13px',
    fontSizeMedium: '14px',
    fontSizeLarge: '15px',
    fontWeight: '500',
  },
}))

// 状态类计算
const statusClass = computed(() => {
  if (appStore.isRunning) return 'status-running'
  return 'status-stopped'
})

// 当前菜单
const currentMenu = computed(() => {
  const path = route.path
  if (path === '/' || path === '/home') return 'home'

  // 处理路由path到菜单key的反向映射
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

// 菜单配置
const menuItems = computed(() => [
  {
    label: t('nav.home'),
    key: 'home',
    icon: HomeOutline,
    disabled: false,
  },
  {
    label: t('nav.subscription'),
    key: 'subscription',
    icon: FolderOutline,
    disabled: false,
  },
  {
    label: t('nav.proxy'),
    key: 'proxy',
    icon: SwapHorizontalOutline,
    disabled: false,
  },
  {
    label: t('nav.connections'),
    key: 'connections',
    icon: LinkOutline,
    disabled: false,
  },
  {
    label: t('nav.logs'),
    key: 'logs',
    icon: DocumentTextOutline,
    disabled: false,
  },
  {
    label: t('nav.rules'),
    key: 'rules',
    icon: AnalyticsOutline,
    disabled: false,
  },
  {
    label: t('nav.settings'),
    key: 'settings',
    icon: SettingsOutline,
    disabled: false,
  },
])

// 导航选择处理
const onSelect = (key: string) => {
  if (key === 'home') {
    router.push('/')
  } else {
    // 处理菜单key和路由path的映射关系
    const routeMap: Record<string, string> = {
      logs: '/log',
      subscription: '/sub',
      settings: '/setting',
      connections: '/connections',
      proxy: '/proxy',
      rules: '/rules',
    }
    const routePath = routeMap[key] || `/${key}`
    router.push(routePath)
  }
}

// 事件监听器
const handleShowUpdateModal = (data: any) => {
  // 处理更新模态框显示
  if (data && typeof data === 'object') {
    updateInfo.value = {
      latestVersion: data.latestVersion || '',
      currentVersion: data.currentVersion || updateStore.appVersion,
      downloadUrl: data.downloadUrl || '',
      releaseNotes: data.releaseNotes || '',
      releaseDate: data.releaseDate || '',
      fileSize: data.fileSize || 0,
    }
    showUpdateModal.value = true
  }
}

// 更新处理函数
const handleUpdate = async (downloadUrl: string) => {
  try {
    message.info('开始下载更新...')
    await updateStore.downloadAndInstallUpdate()
    showUpdateModal.value = false
  } catch (error) {
    message.error(`更新失败: ${error}`)
  }
}

const handleUpdateCancel = () => {
  showUpdateModal.value = false
  message.info('已取消更新')
}

const handleUpdateSkip = () => {
  showUpdateModal.value = false
  updateStore.skipCurrentVersion()
  message.info('已跳过此版本')
}

onMounted(() => {
  // 自动启动内核逻辑
  if (appStore.autoStartKernel && !appStore.isRunning) {
    // 可以在这里添加自动启动逻辑
  }

  // 设置事件监听器
  mittInstance.on('show-update-modal', handleShowUpdateModal)
})

onUnmounted(() => {
  mittInstance.off('show-update-modal', handleShowUpdateModal)
})
</script>

<style scoped>
/* 现代化布局样式 */
.modern-layout {
  background: v-bind('themeStore.isDark ? "#18181b" : "#f8fafc"');
  font-family:
    -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

/* 现代化顶部栏 */
.modern-header {
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  border-bottom: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 1px 3px 0 v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
  z-index: 1000;
}

.header-content {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  user-select: none;
}

.header-brand {
  display: flex;
  align-items: center;
  gap: 14px;
}

.brand-logo {
  cursor: pointer;
  transition: transform 0.2s ease;
}

.brand-logo:hover {
  transform: scale(1.05);
}

.logo-container {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.logo-image {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  transition: all 0.3s ease;
  position: relative;
  object-fit: contain;
}

.logo-image.logo-active {
  box-shadow: 0 0 20px rgba(16, 185, 129, 0.3);
}

.logo-image.logo-active::before {
  content: '';
  position: absolute;
  inset: -2px;
  background: linear-gradient(135deg, #10b981, #059669, #10b981);
  border-radius: 10px;
  opacity: 0.3;
  animation: logo-glow 2s ease-in-out infinite;
  z-index: -1;
}

@keyframes logo-glow {
  0%,
  100% {
    transform: scale(1);
    opacity: 0.3;
  }
  50% {
    transform: scale(1.1);
    opacity: 0.1;
  }
}

.brand-info {
  display: flex;
  align-items: center;
}

.brand-info-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-title {
  font-size: 16px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0;
  letter-spacing: -0.02em;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.status-badge.status-running {
  background: v-bind('themeStore.isDark ? "rgba(16, 185, 129, 0.15)" : "rgba(16, 185, 129, 0.1)"');
  color: #10b981;
}

.status-badge.status-stopped {
  background: v-bind(
    'themeStore.isDark ? "rgba(107, 114, 128, 0.15)" : "rgba(107, 114, 128, 0.1)"'
  );
  color: #6b7280;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
}

.status-running .status-dot {
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.2);
  }
}

.window-controls {
  display: flex;
  gap: 6px;
}

.window-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 8px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.window-btn:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  color: v-bind('themeStore.isDark ? "#cbd5e1" : "#475569"');
}

.window-btn.close:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* 主容器 */
.main-container {
  background: v-bind('themeStore.isDark ? "#18181b" : "#f8fafc"');
}

/* 现代化侧边栏 */
.modern-sider {
  backdrop-filter: blur(16px) saturate(180%);
  -webkit-backdrop-filter: blur(16px) saturate(180%);
  background: v-bind(
    'themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"'
  ) !important;
  border-right: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 4px 0 24px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.03)"');
}

.sider-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 20px 12px;
  gap: 24px;
}

/* 导航区域 */
.nav-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.nav-header {
  padding: 0 12px;
  margin-bottom: 8px;
}

.nav-title {
  font-size: 11px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#64748b" : "#94a3b8"');
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.nav-menu {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-size: 14px;
  font-weight: 500;
  min-height: 44px;
}

.nav-item:hover:not(.nav-disabled) {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  color: v-bind('themeStore.isDark ? "#cbd5e1" : "#475569"');
  transform: translateX(2px);
}

.nav-item.nav-active {
  background: v-bind('themeStore.isDark ? "rgba(91, 76, 253, 0.15)" : "rgba(91, 76, 253, 0.1)"');
  color: #5b4cfd;
  font-weight: 600;
}

.nav-item.nav-active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 24px;
  background: linear-gradient(180deg, #5b4cfd, #7c3aed);
  border-radius: 0 2px 2px 0;
}

.nav-item.nav-disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.nav-icon-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 24px;
  height: 24px;
}

.nav-label {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

/* 底部工具区 */
.tools-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-top: 16px;
  border-top: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

.tool-group {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.tool-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-size: 13px;
  font-weight: 500;
  min-height: 40px;
}

.tool-item:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  color: v-bind('themeStore.isDark ? "#cbd5e1" : "#475569"');
}

.tool-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 20px;
  height: 20px;
}

.tool-label {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.tool-divider {
  height: 1px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  margin: 8px 12px;
}

/* 内容区域 */
.modern-content {
  overflow: auto;
}

.content-wrapper {
  max-width: 1400px;
  margin: 0 auto;
  height: 100%;
  min-height: calc(100vh - 48px);
}

/* 页面切换动画 */
.page-transition-enter-active,
.page-transition-leave-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}

.page-transition-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.page-transition-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

.nav-text-enter-active,
.nav-text-leave-active {
  transition: all 0.2s ease;
}

.nav-text-enter-from,
.nav-text-leave-to {
  opacity: 0;
  transform: translateX(-8px);
}

/* 折叠状态优化 */
.modern-sider[collapsed] .sider-content {
  padding: 20px 8px;
}

.modern-sider[collapsed] .nav-item {
  justify-content: center;
  padding: 10px;
}

.modern-sider[collapsed] .tool-item {
  justify-content: center;
  padding: 10px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .modern-sider {
    position: fixed;
    height: 100%;
    z-index: 999;
  }

  .content-wrapper {
    padding: 16px;
  }

  .header-content {
    padding: 0 16px;
  }

  .brand-info {
    display: none;
  }
}

@media (max-width: 480px) {

  .content-wrapper {
    padding: 0;
  }
}

/* 移除 Naive UI 滚动条样式覆盖，使用官方主题系统 */
</style>
