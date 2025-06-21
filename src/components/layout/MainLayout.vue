<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-layout class="modern-layout" position="absolute">
      <!-- 极简顶部栏 -->
      <n-layout-header class="modern-header">
        <div class="header-content" data-tauri-drag-region>
          <!-- 左侧：仅保留必要的标题 -->
          <div class="header-left">
            <n-gradient-text :size="16" class="app-title-gradient">
              Sing-Box Windows</n-gradient-text
            >
          </div>

          <!-- 右侧：窗口控制 -->
          <div class="header-right">
            <button class="window-btn" @click="windowStore.minimizeWindow">
              <n-icon size="14"><RemoveOutline /></n-icon>
            </button>
            <button class="window-btn" @click="windowStore.toggleMaximize">
              <n-icon size="14">
                <CropOutline v-if="windowStore.windowState.isMaximized" />
                <SquareOutline v-else />
              </n-icon>
            </button>
            <button
              class="window-btn window-btn-close"
              @click="() => windowStore.hideWindow(router)"
            >
              <n-icon size="14"><CloseOutline /></n-icon>
            </button>
          </div>
        </div>
      </n-layout-header>

      <!-- 主内容区 -->
      <n-layout has-sider position="absolute" style="top: 48px" class="main-container">
        <!-- 现代化侧边栏 -->
        <n-layout-sider
          class="modern-sider"
          :width="240"
          :collapsed-width="72"
          :collapsed="collapsed"
          collapse-mode="width"
          :native-scrollbar="false"
        >
          <div class="sider-content">
            <!-- Logo 和状态区域 -->
            <div class="logo-section">
              <div class="logo-wrapper">
                <n-image
                  :src="logo"
                  :width="collapsed ? 40 : 56"
                  :height="collapsed ? 40 : 56"
                  preview-disabled
                  class="app-logo"
                />
                <div class="logo-glow" :class="{ active: appStore.isRunning }"></div>
              </div>

              <transition name="fade">
                <div v-if="!collapsed" class="app-info">
                  <div class="app-name">Sing-Box</div>
                  <div class="status-container">
                    <n-badge
                      :value="appStore.isRunning ? t('common.running') : t('common.stopped')"
                      :type="appStore.isRunning ? 'success' : 'error'"
                      :offset="[0, 0]"
                      class="status-badge"
                    />
                  </div>
                </div>
              </transition>
            </div>

            <!-- 导航菜单 -->
            <div class="nav-section">
              <div
                v-for="item in menuItems"
                :key="item.key"
                class="nav-item"
                :class="{
                  active: currentMenu === item.key,
                  disabled: item.disabled,
                }"
                @click="!item.disabled && onSelect(item.key)"
              >
                <div class="nav-icon">
                  <n-icon :size="20" :component="item.icon" />
                  <div class="nav-indicator"></div>
                </div>
                <transition name="fade">
                  <span v-if="!collapsed" class="nav-label">{{ item.label }}</span>
                </transition>
              </div>
            </div>

            <!-- 底部功能区 -->
            <div class="sider-footer">
              <!-- 主题切换 -->
              <div class="footer-item" @click="themeStore.toggleTheme">
                <n-icon :size="20">
                  <MoonOutline v-if="themeStore.isDark" />
                  <SunnyOutline v-else />
                </n-icon>
                <transition name="fade">
                  <span v-if="!collapsed" class="footer-label">
                    {{ themeStore.isDark ? t('common.darkMode') : t('common.lightMode') }}
                  </span>
                </transition>
              </div>

              <!-- 折叠按钮 -->
              <div class="footer-item" @click="collapsed = !collapsed">
                <n-icon :size="20">
                  <ChevronBackOutline v-if="!collapsed" />
                  <ChevronForwardOutline v-else />
                </n-icon>
                <transition name="fade">
                  <span v-if="!collapsed" class="footer-label">{{ t('common.collapse') }}</span>
                </transition>
              </div>
            </div>
          </div>
        </n-layout-sider>

        <!-- 内容区域 -->
        <n-layout-content class="modern-content">
          <div class="content-wrapper">
            <router-view />
          </div>
        </n-layout-content>
      </n-layout>
    </n-layout>
  </n-config-provider>

  <!-- 全局更新弹窗 -->
  <update-modal
    v-model:show="showUpdateModal"
    :latest-version="updateModalData.latestVersion"
    :current-version="updateModalData.currentVersion"
    :download-url="updateModalData.downloadUrl"
    :release-notes="updateModalData.releaseNotes"
    :release-date="updateModalData.releaseDate"
    :file-size="updateModalData.fileSize"
    @update="handleGlobalUpdate"
    @cancel="handleGlobalCancel"
    @skip="handleGlobalSkip"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { darkTheme, useNotification, type GlobalThemeOverrides } from 'naive-ui'
import {
  HomeOutline,
  SettingsOutline,
  SwapHorizontalOutline,
  DocumentTextOutline,
  MoonOutline,
  SunnyOutline,
  AtCircleOutline,
  FilterOutline,
  LinkOutline,
  RemoveOutline,
  CloseOutline,
  CropOutline,
  SquareOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
} from '@vicons/ionicons5'
import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/app/AppStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { useWindowStore } from '@/stores/app/WindowStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'
import type { UpdateModalData } from '@/utils/mitt'
import UpdateModal from '@/components/UpdateModal.vue'
import logo from '@/assets/icon.png'

const router = useRouter()
const appWindow = Window.getCurrent()
const appStore = useAppStore()
const themeStore = useThemeStore()
const windowStore = useWindowStore()
const updateStore = useUpdateStore()
const notification = useNotification()
const { t } = useI18n()

// 响应式状态
const collapsed = ref(false)
const currentMenu = ref('home')

// 主题配置
const theme = computed(() => (themeStore.isDark ? darkTheme : null))

const themeOverrides = computed<GlobalThemeOverrides>(() => ({
  common: {
    borderRadius: '12px',
    borderRadiusSmall: '8px',
    primaryColor: '#646cff',
    primaryColorHover: '#747bff',
    primaryColorPressed: '#535bf2',
    primaryColorSuppl: '#747bff',
  },
  Layout: {
    headerHeight: '48px',
    headerBorderColor: 'transparent',
    siderBorderColor: 'transparent',
    color: 'transparent',
    siderColor: themeStore.isDark ? 'rgba(24, 24, 28, 0.6)' : 'rgba(255, 255, 255, 0.6)',
    contentColor: 'transparent',
  },
  Card: {
    borderRadius: '16px',
    color: themeStore.isDark ? 'rgba(24, 24, 28, 0.8)' : 'rgba(255, 255, 255, 0.8)',
  },
}))

// 菜单配置
const menuItems = computed(() => [
  {
    label: t('nav.home'),
    key: 'home',
    icon: HomeOutline,
    disabled: false,
  },
  {
    label: t('nav.proxy'),
    key: 'proxy',
    icon: SwapHorizontalOutline,
    disabled: !appStore.isRunning,
  },
  {
    label: t('nav.sub'),
    key: 'sub',
    icon: AtCircleOutline,
    disabled: false,
  },
  {
    label: t('nav.rules'),
    key: 'rules',
    icon: FilterOutline,
    disabled: !appStore.isRunning,
  },
  {
    label: t('nav.connections'),
    key: 'connections',
    icon: LinkOutline,
    disabled: !appStore.isRunning,
  },
  {
    label: t('nav.log'),
    key: 'log',
    icon: DocumentTextOutline,
    disabled: false,
  },
  {
    label: t('nav.setting'),
    key: 'setting',
    icon: SettingsOutline,
    disabled: false,
  },
])

// 路由映射
const routeMap: Record<string, string> = {
  home: '/',
  proxy: '/proxy',
  sub: '/sub',
  rules: '/rules',
  connections: '/connections',
  log: '/log',
  setting: '/setting',
}

// 导航选择
function onSelect(key: string) {
  currentMenu.value = key
  router.push(routeMap[key])
}

// 更新弹窗相关
const showUpdateModal = ref(false)
const updateModalData = ref<UpdateModalData>({
  show: false,
  latestVersion: '',
  currentVersion: '',
  downloadUrl: '',
  releaseNotes: '',
  releaseDate: '',
  fileSize: 0,
})

const handleShowUpdateModal = (data: UpdateModalData) => {
  updateModalData.value = { ...data }
  showUpdateModal.value = true
}

const handleGlobalUpdate = async () => {
  try {
    notification.info({
      title: t('setting.update.downloading'),
      content: t('setting.update.downloadingDescription'),
      duration: 3000,
    })
    await updateStore.downloadAndInstallUpdate()
  } catch (error) {
    notification.error({
      title: t('setting.update.updateError'),
      content: String(error),
      duration: 5000,
    })
  }
}

const handleGlobalCancel = () => {
  showUpdateModal.value = false
}

const handleGlobalSkip = () => {
  updateStore.skipCurrentVersion()
  showUpdateModal.value = false
}

// 生命周期
onMounted(async () => {
  await updateStore.fetchAppVersion()
  mitt.on('show-update-modal', handleShowUpdateModal)

  await appWindow.listen('tauri://close-requested', async () => {
    await windowStore.hideWindow(router)
  })

  // 根据当前路由设置选中项
  const path = router.currentRoute.value.path
  const menuKey = Object.entries(routeMap).find(([_, route]) => route === path)?.[0]
  if (menuKey) {
    currentMenu.value = menuKey
  }
})

onBeforeUnmount(() => {
  mitt.off('show-update-modal', handleShowUpdateModal)
})
</script>

<style scoped>
/* 现代化布局样式 */
.modern-layout {
  background: var(--n-body-color);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
}

/* 极简顶部栏 */
.modern-header {
  height: 48px !important;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  border-bottom: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.08)"');
  z-index: 100;
}

.header-content {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
}

.app-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color);
  letter-spacing: -0.2px;
}

.header-right {
  display: flex;
  gap: 8px;
  align-items: center;
}

.window-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 8px;
  color: var(--n-text-color);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.window-btn:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.04)"');
}

.window-btn-close:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* 主容器 */
.main-container {
  background: v-bind('themeStore.isDark ? "#0a0a0a" : "#f8f9fa"');
}

/* 现代化侧边栏 */
.modern-sider {
  backdrop-filter: blur(24px);
  -webkit-backdrop-filter: blur(24px);
  background: v-bind(
    'themeStore.isDark ? "rgba(24, 24, 28, 0.6)" : "rgba(255, 255, 255, 0.6)"'
  ) !important;
  border-right: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.08)"');
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.sider-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 24px 16px;
}

/* Logo 区域 */
.logo-section {
  margin-bottom: 28px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 8px 0;
}

.logo-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.app-logo {
  border-radius: 18px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.app-logo:hover {
  transform: scale(1.05);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
}

.logo-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 72px;
  height: 72px;
  background: radial-gradient(circle, rgba(100, 108, 255, 0.3) 0%, transparent 70%);
  border-radius: 50%;
  opacity: 0;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  pointer-events: none;
}

.logo-glow.active {
  opacity: 1;
  animation: pulse 3s ease-in-out infinite;
}

@keyframes pulse {
  0%,
  100% {
    transform: translate(-50%, -50%) scale(1);
    opacity: 0.6;
  }
  50% {
    transform: translate(-50%, -50%) scale(1.15);
    opacity: 0.3;
  }
}

.app-info {
  text-align: center;
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.app-name {
  font-size: 16px;
  font-weight: 700;
  color: var(--n-text-color);
  letter-spacing: -0.2px;
}

.status-container {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  text-align: center;
}

.status-badge {
  font-size: 10px;
  font-weight: 500;
  margin: 0 auto;
}

/* 强制n-badge组件及其子元素居中 */
:deep(.n-badge) {
  width: 100%;
  display: flex !important;
  justify-content: center !important;
  text-align: center;
}

:deep(.n-badge .n-badge-sup) {
  position: static !important;
  transform: none !important;
  margin: 0 auto !important;
}

/* 导航区域 */
.nav-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--n-text-color-2);
}

.nav-item:hover:not(.disabled) {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
  color: var(--n-text-color);
}

.nav-item.active {
  background: v-bind(
    'themeStore.isDark ? "rgba(100, 108, 255, 0.15)" : "rgba(100, 108, 255, 0.1)"'
  );
  color: #646cff;
}

.nav-item.disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.nav-icon {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-indicator {
  position: absolute;
  left: -28px;
  width: 3px;
  height: 20px;
  background: #646cff;
  border-radius: 0 3px 3px 0;
  opacity: 0;
  transform: translateX(-4px);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.nav-item.active .nav-indicator {
  opacity: 1;
  transform: translateX(0);
}

.nav-label {
  font-size: 14px;
  font-weight: 500;
  white-space: nowrap;
}

/* 底部功能区 */
.sider-footer {
  margin-top: auto;
  padding-top: 16px;
  border-top: 1px solid
    v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.08)"');
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.footer-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--n-text-color-2);
}

.footer-item:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
  color: var(--n-text-color);
}

.footer-label {
  font-size: 14px;
  white-space: nowrap;
}

/* 内容区域 */
.modern-content {
  padding: 24px;
  overflow: auto;
}

.content-wrapper {
  max-width: 1400px;
  margin: 0 auto;
  height: 100%;
}

/* 过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* 过渡动画已移除，提高稳定性 */

/* 应用标题渐变样式 */
.app-title-gradient {
  font-weight: 600;
  letter-spacing: -0.3px;
  color: black;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.app-title-gradient:hover {
  text-shadow: 0 0 30px rgba(29, 29, 34, 0.5);
  transform: scale(1.02);
}

/* 折叠状态优化 */
.modern-sider.collapsed .logo-section {
  gap: 12px;
  margin-bottom: 24px;
}

.modern-sider.collapsed .app-logo {
  border-radius: 12px;
}

.modern-sider.collapsed .logo-glow {
  width: 56px;
  height: 56px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .modern-sider {
    position: fixed;
    height: 100%;
    z-index: 99;
  }

  .logo-section {
    padding: 4px 0;
    margin-bottom: 20px;
  }

  .app-name {
    font-size: 14px;
  }

  .status-badge {
    font-size: 9px;
  }
}

/* 滚动条样式 */
:deep(.n-scrollbar) {
  height: 100%;
}

:deep(.n-layout-sider-scroll-container) {
  scrollbar-width: none;
  -ms-overflow-style: none;
}

:deep(.n-layout-sider-scroll-container::-webkit-scrollbar) {
  display: none;
}
</style>
