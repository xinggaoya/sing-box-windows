<template>
  <n-config-provider :theme="theme" :theme-overrides="themeOverrides">
    <n-layout class="ultra-modern-layout" position="absolute">
      <!-- 超紧凑顶部栏 - 集成窗口控制 -->
      <n-layout-header class="ultra-header" :style="{ height: '36px' }">
        <div class="header-content" data-tauri-drag-region>
          <!-- 左侧：动态状态指示器 -->
          <div class="header-left">
            <div class="app-logo-mini" @click="onSelect('home')" data-tauri-drag-region="false">
              <img :src="logo" alt="Logo" class="logo-img" />
            </div>
            <div class="status-indicator-mini" :class="statusClass">
              <div class="pulse-dot"></div>
            </div>
            <span class="app-title-mini">{{ t('common.appName') }}</span>
          </div>

          <!-- 右侧：窗口控制 -->
          <div class="header-controls">
            <button class="control-btn minimize" @click="windowStore.minimizeWindow">
              <svg width="10" height="1" viewBox="0 0 10 1"><rect width="10" height="1" fill="currentColor"/></svg>
            </button>
            <button class="control-btn maximize" @click="windowStore.toggleMaximize">
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                <rect x="1" y="1" width="8" height="8" stroke="currentColor" stroke-width="1"/>
              </svg>
            </button>
            <button class="control-btn close" @click="() => windowStore.hideWindow(router)">
              <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
                <path d="M1 1L9 9M9 1L1 9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
              </svg>
            </button>
          </div>
        </div>
      </n-layout-header>

      <!-- 主内容区 - 优化空间利用 -->
      <n-layout has-sider position="absolute" style="top: 36px" class="main-container">
        <!-- 超薄侧边栏 -->
        <n-layout-sider
          class="ultra-sider"
          :width="180"
          :collapsed-width="56"
          :collapsed="collapsed"
          collapse-mode="width"
          :native-scrollbar="false"
        >
          <div class="sider-content">
            <!-- 精简Logo区域 -->
            <div class="logo-section-compact">
              <div class="logo-container" @click="onSelect('home')">
                <div class="logo-icon" :class="{ active: appStore.isRunning }">
                  <div class="logo-core"></div>
                  <div class="logo-ring" v-if="appStore.isRunning"></div>
                </div>
              </div>
              <transition name="fade-slide">
                <div v-if="!collapsed" class="status-text">
                  <span class="status-label">{{ appStore.isRunning ? t('status.running') : t('status.stopped') }}</span>
                </div>
              </transition>
            </div>

            <!-- 垂直导航菜单 -->
            <div class="nav-section-compact">
              <div
                v-for="item in menuItems"
                :key="item.key"
                class="nav-item-compact"
                :class="{
                  'nav-active': currentMenu === item.key,
                  'nav-disabled': item.disabled,
                }"
                @click="!item.disabled && onSelect(item.key)"
              >
                <div class="nav-icon-compact">
                  <n-icon :size="18" :component="item.icon" />
                </div>
                <transition name="fade-slide">
                  <span v-if="!collapsed" class="nav-text">{{ item.label }}</span>
                </transition>
                <div class="nav-glow"></div>
              </div>
            </div>

            <!-- 底部工具区 -->
            <div class="tools-section">
              <div class="tool-item" @click="themeStore.toggleTheme">
                <n-icon :size="16">
                  <MoonOutline v-if="themeStore.isDark" />
                  <SunnyOutline v-else />
                </n-icon>
                <transition name="fade-slide">
                  <span v-if="!collapsed" class="tool-text">{{ themeStore.isDark ? t('setting.theme.dark') : t('setting.theme.light') }}</span>
                </transition>
              </div>
              <div class="divider" v-if="!collapsed"></div>
              <div class="tool-item" @click="collapsed = !collapsed">
                <n-icon :size="16">
                  <ChevronBackOutline v-if="!collapsed" />
                  <ChevronForwardOutline v-else />
                </n-icon>
              </div>
            </div>
          </div>
        </n-layout-sider>

        <!-- 优化内容区域 -->
        <n-layout-content class="ultra-content">
          <div class="content-wrapper-compact">
            <router-view v-slot="{ Component }">
              <transition name="page-fade" mode="default">
                <component :is="Component" :key="$route.path" />
              </transition>
            </router-view>
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

// 状态类计算
const statusClass = computed(() => {
  if (appStore.isRunning) return 'status-running'
  return 'status-stopped'
})

const themeOverrides = computed<GlobalThemeOverrides>(() => ({
  common: {
    borderRadius: '8px',
    borderRadiusSmall: '6px',
    borderRadiusMedium: '8px',
    primaryColor: '#6366f1',
    primaryColorHover: '#818cf8',
    primaryColorPressed: '#4f46e5',
    primaryColorSuppl: '#818cf8',
    fontSize: '13px',
  },
  Layout: {
    headerHeight: '36px',
    headerBorderColor: 'transparent',
    siderBorderColor: 'transparent',
    color: 'transparent',
    siderColor: themeStore.isDark ? 'rgba(17, 24, 39, 0.95)' : 'rgba(255, 255, 255, 0.95)',
    contentColor: 'transparent',
  },
  Card: {
    borderRadius: '12px',
    color: themeStore.isDark ? 'rgba(17, 24, 39, 0.6)' : 'rgba(255, 255, 255, 0.8)',
    borderColor: 'transparent',
  },
  Button: {
    borderRadiusSmall: '6px',
    borderRadiusMedium: '8px',
    heightSmall: '28px',
    heightMedium: '32px',
    heightLarge: '36px',
    fontSizeSmall: '12px',
    fontSizeMedium: '13px',
    fontSizeLarge: '14px',
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

// 监听托盘模式变化
const isLowPowerMode = ref(false)

// 优化GPU使用的样式切换
const toggleGPUPerformance = (enable: boolean) => {
  const root = document.documentElement
  if (enable) {
    // 高性能模式：启用GPU加速和动画
    root.style.setProperty('--gpu-acceleration', 'auto')
    root.style.setProperty('--animation-duration', '0.2s')
    console.log('🚀 启用GPU高性能模式')
  } else {
    // 低功耗模式：禁用GPU加速
    root.style.setProperty('--gpu-acceleration', 'none')
    root.style.setProperty('--animation-duration', '0s')
    console.log('🔋 启用低功耗模式')
  }
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

  // 监听内存清理请求
  mitt.on('memory-cleanup-requested', () => {
    console.log('🧹 MainLayout响应内存清理请求')
    isLowPowerMode.value = true
    toggleGPUPerformance(false)
  })

  // 监听窗口显示事件
  mitt.on('window-show', () => {
    console.log('🪟 窗口显示，恢复GPU性能')
    isLowPowerMode.value = false
    toggleGPUPerformance(true)
  })

  // 监听窗口最小化事件
  mitt.on('window-minimize', () => {
    console.log('🪟 窗口最小化，降低GPU性能')
    isLowPowerMode.value = true
    toggleGPUPerformance(false)
  })
})

onBeforeUnmount(() => {
  mitt.off('show-update-modal', handleShowUpdateModal)
  mitt.off('memory-cleanup-requested')
  mitt.off('window-show')
  mitt.off('window-minimize')
})
</script>

<style scoped>
/* 超现代化布局样式 */
.ultra-modern-layout {
  background: v-bind('themeStore.isDark ? "#0f0f10" : "#fafafa"');
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Inter', 'Roboto', sans-serif;
  font-size: 13px;
  /* 优化GPU使用 */
  transform: translateZ(0);
  will-change: auto;
  backface-visibility: hidden;
}

/* 超紧凑顶部栏 */
.ultra-header {
  /* 减少GPU占用的背景效果 */
  backdrop-filter: blur(8px) saturate(120%);
  -webkit-backdrop-filter: blur(8px) saturate(120%);
  background: v-bind('themeStore.isDark ? "rgba(15, 15, 16, 0.9)" : "rgba(255, 255, 255, 0.85)"');
  border-bottom: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  z-index: 1000;
  box-shadow: 0 1px 0 0 v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.04)" : "rgba(0, 0, 0, 0.02)"');
  /* GPU优化 */
  will-change: auto;
  transform: translateZ(0);
  contain: layout style paint;
}

.header-content {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  user-select: none;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-logo-mini {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.logo-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
  border-radius: 4px;
  transition: transform 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.app-logo-mini:hover .logo-img {
  transform: scale(1.1);
}

.app-logo-mini:active {
  transform: scale(0.95);
}

.status-indicator-mini {
  position: relative;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.status-indicator-mini.status-running {
  background: linear-gradient(135deg, #10b981, #059669);
  box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.2);
}

.status-indicator-mini.status-stopped {
  background: v-bind('themeStore.isDark ? "#374151" : "#d1d5db"');
}

.pulse-dot {
  position: absolute;
  width: 6px;
  height: 6px;
  background: white;
  border-radius: 50%;
  animation: pulse 2s ease-in-out infinite;
}

.status-running .pulse-dot {
  animation: pulse-green 2s ease-in-out infinite;
}

@keyframes pulse-green {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.8; transform: scale(1.1); }
}

.app-title-mini {
  font-size: 13px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f3f4f6" : "#111827"');
  letter-spacing: -0.02em;
}

.header-controls {
  display: flex;
  gap: 2px;
}

.control-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  border-radius: 6px;
  color: v-bind('themeStore.isDark ? "#9ca3af" : "#6b7280"');
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 10px;
}

.control-btn:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.05)"');
  color: v-bind('themeStore.isDark ? "#e5e7eb" : "#374151"');
}

.control-btn.close:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* 主容器 */
.main-container {
  background: v-bind('themeStore.isDark ? "#0f0f10" : "#fafafa"');
}

/* 超薄侧边栏 */
.ultra-sider {
  /* 减少GPU占用的背景效果 */
  backdrop-filter: blur(10px) saturate(140%);
  -webkit-backdrop-filter: blur(10px) saturate(140%);
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.97)" : "rgba(255, 255, 255, 0.97)"') !important;
  border-right: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  /* 减少动画复杂度 */
  transition: opacity 0.2s ease, transform 0.2s ease;
  box-shadow: 2px 0 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.03)"');
  /* GPU优化 */
  will-change: auto;
  transform: translateZ(0);
  contain: layout style paint;
}

.sider-content {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px 8px;
  gap: 20px;
}

/* 精简Logo区域 */
.logo-section-compact {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
}

.logo-container {
  cursor: pointer;
  position: relative;
}

.logo-icon {
  width: 32px;
  height: 32px;
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.logo-core {
  width: 20px;
  height: 20px;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  border-radius: 6px;
  position: relative;
  z-index: 2;
  box-shadow: 0 2px 8px rgba(99, 102, 241, 0.3);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.logo-ring {
  position: absolute;
  width: 32px;
  height: 32px;
  border: 2px solid transparent;
  border-radius: 50%;
  background: conic-gradient(from 0deg, #6366f1, #8b5cf6, #ec4899, #6366f1);
  -webkit-mask: radial-gradient(farthest-side, transparent calc(100% - 2px), black calc(100% - 1px));
  mask: radial-gradient(farthest-side, transparent calc(100% - 2px), black calc(100% - 1px));
  /* 减少动画频率以节省GPU资源 */
  animation: rotate 6s linear infinite;
  /* GPU优化 */
  will-change: transform;
  transform: translateZ(0);
}

.logo-icon.active .logo-core {
  background: linear-gradient(135deg, #10b981, #059669);
  box-shadow: 0 0 16px rgba(16, 185, 129, 0.4);
}

.logo-icon:hover .logo-core {
  transform: scale(1.1);
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
}

@keyframes rotate {
  to { transform: rotate(360deg); }
}

.status-text {
  text-align: center;
}

.status-label {
  font-size: 11px;
  font-weight: 500;
  color: v-bind('themeStore.isDark ? "#9ca3af" : "#6b7280"');
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* 垂直导航菜单 */
.nav-section-compact {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.nav-item-compact {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: v-bind('themeStore.isDark ? "#9ca3af" : "#6b7280"');
  font-size: 13px;
  font-weight: 500;
  min-height: 36px;
}

.nav-item-compact:hover:not(.nav-disabled) {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  color: v-bind('themeStore.isDark ? "#e5e7eb" : "#374151"');
  transform: translateX(2px);
}

.nav-item-compact.nav-active {
  background: v-bind('themeStore.isDark ? "rgba(99, 102, 241, 0.15)" : "rgba(99, 102, 241, 0.1)"');
  color: #6366f1;
  font-weight: 600;
}

.nav-item-compact.nav-active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  background: linear-gradient(180deg, #6366f1, #8b5cf6);
  border-radius: 0 2px 2px 0;
}

.nav-item-compact.nav-disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.nav-icon-compact {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 20px;
  height: 20px;
}

.nav-text {
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
}

.nav-glow {
  position: absolute;
  inset: 0;
  border-radius: 8px;
  background: radial-gradient(circle at center, rgba(99, 102, 241, 0.1) 0%, transparent 70%);
  opacity: 0;
  transition: opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  pointer-events: none;
}

.nav-item-compact.nav-active .nav-glow {
  opacity: 1;
}

/* 底部工具区 */
.tools-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-top: 12px;
  border-top: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
}

.tool-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: v-bind('themeStore.isDark ? "#9ca3af" : "#6b7280"');
  font-size: 12px;
  min-height: 32px;
}

.tool-item:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  color: v-bind('themeStore.isDark ? "#e5e7eb" : "#374151"');
}

.tool-text {
  font-size: 12px;
  font-weight: 500;
  white-space: nowrap;
}

.divider {
  height: 1px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  margin: 4px 0;
}

/* 优化内容区域 */
.ultra-content {
  padding: 16px;
  overflow: auto;
}

.content-wrapper-compact {
  max-width: 1200px;
  margin: 0 auto;
  height: 100%;
  min-height: calc(100vh - 36px);
}

/* 页面切换动画 - 优化版 */
.page-fade-enter-active {
  transition: all 0.15s ease-out;
}

.page-fade-leave-active {
  transition: none;
}

.page-fade-enter-from {
  opacity: 0;
  transform: translateY(4px);
}

.page-fade-leave-to {
  opacity: 1;
}

/* 侧边栏动画 */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-8px);
}

/* 折叠状态优化 */
.ultra-sider[collapsed] .sider-content {
  padding: 16px 4px;
}

.ultra-sider[collapsed] .nav-item-compact {
  justify-content: center;
  padding: 8px;
}

.ultra-sider[collapsed] .tool-item {
  justify-content: center;
  padding: 8px;
}

/* 滚动条美化 */
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

/* 暗黑模式下的特殊效果 */
.ultra-modern-layout :deep(.n-card) {
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.6)" : "rgba(255, 255, 255, 0.8)"') !important;
  backdrop-filter: blur(8px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"') !important;
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.05)"') !important;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .ultra-sider {
    position: fixed;
    height: 100%;
    z-index: 999;
  }

  .content-wrapper-compact {
    padding: 12px;
  }
}

/* 微交互效果 */
.nav-item-compact:active {
  transform: scale(0.98) translateX(2px);
}

.tool-item:active {
  transform: scale(0.98);
}

/* 玻璃态效果增强 */
@media (prefers-reduced-motion: no-preference) {
  .ultra-header {
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
  }

  .ultra-sider {
    backdrop-filter: blur(24px) saturate(180%);
    -webkit-backdrop-filter: blur(24px) saturate(180%);
  }
}
</style>
