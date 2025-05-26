<template>
  <n-layout position="absolute">
    <n-layout-header class="modern-header" style="height: 68px; padding: 0">
      <div class="header-backdrop"></div>
      <div class="header-content" data-tauri-drag-region>
        <!-- 左侧Logo和标题区域 -->
        <div class="header-left">
          <div class="logo-container">
            <n-image :src="logo" width="32" height="32" preview-disabled class="app-logo" />
            <div class="logo-glow"></div>
          </div>
          <div class="app-title-section">
            <h1 class="app-title">
              Sing-Box
              <span class="app-subtitle">Window</span>
            </h1>
            <n-tag
              :type="appStore.isRunning ? 'success' : 'default'"
              :bordered="false"
              size="small"
              round
              class="status-tag"
            >
              <template #icon>
                <div class="status-dot" :class="{ active: appStore.isRunning }"></div>
              </template>
              {{ appStore.isRunning ? t('common.running') : t('common.stopped') }}
            </n-tag>
          </div>
        </div>

        <!-- 中间搜索区域（可选，暂时隐藏） -->
        <div class="header-center">
          <!-- 预留中间区域用于搜索或其他功能 -->
        </div>

        <!-- 右侧控制区域 -->
        <div class="header-right">
          <!-- 功能按钮组 -->
          <div class="control-group function-group">
            <n-tooltip trigger="hover" :delay="800">
              <template #trigger>
                <button class="header-btn theme-btn" @click="themeStore.toggleTheme">
                  <n-icon size="16">
                    <moon-outline v-if="themeStore.isDark" />
                    <sunny-outline v-else />
                  </n-icon>
                </button>
              </template>
              {{ themeStore.isDark ? t('common.lightMode') : t('common.darkMode') }}
            </n-tooltip>

            <n-tooltip trigger="hover" :delay="800">
              <template #trigger>
                <button class="header-btn fullscreen-btn" @click="onToggleFullScreen">
                  <n-icon size="16">
                    <expand-outline v-if="!windowStore.windowState.isFullscreen" />
                    <contract-outline v-else />
                  </n-icon>
                </button>
              </template>
              {{
                windowStore.windowState.isFullscreen
                  ? t('common.exitFullscreen')
                  : t('common.fullscreen')
              }}
            </n-tooltip>
          </div>

          <!-- 窗口控制按钮组 -->
          <div class="control-group window-group">
            <button class="header-btn minimize-btn" @click="windowStore.minimizeWindow">
              <n-icon size="16">
                <remove-outline />
              </n-icon>
            </button>

            <button class="header-btn close-btn" @click="windowStore.hideWindow">
              <n-icon size="16">
                <close-outline />
              </n-icon>
            </button>
          </div>
        </div>
      </div>
    </n-layout-header>
    <n-layout has-sider position="absolute" style="top: 68px">
      <n-layout-sider
        bordered
        collapse-mode="width"
        :collapsed-width="64"
        :width="180"
        :collapsed="collapsed"
        show-trigger
        @collapse="collapsed = true"
        @expand="collapsed = false"
      >
        <n-menu
          v-model:value="currentMenu"
          :collapsed="collapsed"
          :collapsed-width="64"
          :collapsed-icon-size="22"
          :options="menuOptions"
          @update:value="onSelect"
        />
      </n-layout-sider>
      <n-layout-content content-style="padding: 0;">
        <n-scrollbar style="max-height: calc(100vh - 68px)" class="main-scrollbar">
          <router-view />
        </n-scrollbar>
      </n-layout-content>
    </n-layout>
  </n-layout>

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
import {
  darkTheme,
  useOsTheme,
  NIcon,
  useNotification,
  NButton,
  NProgress,
  NTag,
  type MenuOption,
} from 'naive-ui'
import type { NotificationReactive } from 'naive-ui'
import { h, ref, onMounted, computed, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import mitt from '@/utils/mitt'
import type { UpdateModalData } from '@/utils/mitt'
import {
  HomeOutline,
  SettingsOutline,
  SwapHorizontalOutline,
  DocumentTextOutline,
  MoonOutline,
  SunnyOutline,
  AtCircleOutline,
  ExpandOutline,
  ContractOutline,
  RemoveOutline,
  CloseOutline,
  FilterOutline,
  LinkOutline,
} from '@vicons/ionicons5'
import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/app/AppStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { useWindowStore } from '@/stores/app/WindowStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { listen } from '@tauri-apps/api/event'
import logo from '@/assets/icon.png'
import { useI18n } from 'vue-i18n'
import type { Component } from 'vue'
import UpdateModal from '@/components/UpdateModal.vue'

const router = useRouter()
const appWindow = Window.getCurrent()
const appStore = useAppStore()
const themeStore = useThemeStore()
const windowStore = useWindowStore()
const updateStore = useUpdateStore()
const notification = useNotification()
const collapsed = ref(false)
const currentMenu = ref<string | null>('0')
const isFullscreen = ref(false)
const { t } = useI18n()

// 全局更新弹窗状态
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

// 渲染图标函数
function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

const toggleTheme = () => {
  themeStore.toggleTheme()
}

const onToggleFullScreen = async () => {
  await windowStore.toggleFullScreen()
}

const menuOptions = computed<MenuOption[]>(() => [
  {
    label: t('nav.home'),
    key: '0',
    icon: renderIcon(HomeOutline),
    disabled: false,
  },
  {
    label: t('nav.proxy'),
    key: '1',
    disabled: !appStore.isRunning,
    icon: renderIcon(SwapHorizontalOutline),
  },
  {
    label: t('nav.sub'),
    key: '4',
    icon: renderIcon(AtCircleOutline),
    disabled: false,
  },
  {
    label: t('nav.rules'),
    key: '2',
    disabled: !appStore.isRunning,
    icon: renderIcon(FilterOutline),
  },
  {
    label: t('nav.connections'),
    key: '3',
    disabled: !appStore.isRunning,
    icon: renderIcon(LinkOutline),
  },
  {
    label: t('nav.log'),
    key: '5',
    icon: renderIcon(DocumentTextOutline),
    disabled: false,
  },
  {
    label: t('nav.setting'),
    key: '6',
    icon: renderIcon(SettingsOutline),
    disabled: false,
  },
])

function onSelect(key: string) {
  const numKey = parseInt(key)
  switch (numKey) {
    case 0:
      router.push('/')
      break
    case 1:
      router.push('/proxy')
      break
    case 2:
      router.push('/rules')
      break
    case 3:
      router.push('/connections')
      break
    case 4:
      router.push('/sub')
      break
    case 5:
      router.push('/log')
      break
    case 6:
      router.push('/setting')
      break
    default:
      break
  }
}

// 处理全局更新弹窗事件
const handleShowUpdateModal = (data: UpdateModalData) => {
  updateModalData.value = { ...data }
  showUpdateModal.value = true
}

// 处理更新操作
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

// 处理取消更新
const handleGlobalCancel = () => {
  showUpdateModal.value = false
}

// 处理跳过版本
const handleGlobalSkip = () => {
  updateStore.skipCurrentVersion()
  showUpdateModal.value = false
}

// 监听窗口事件
onMounted(async () => {
  // 获取当前版本号
  await updateStore.fetchAppVersion()

  // 监听全局更新弹窗事件
  mitt.on('show-update-modal', handleShowUpdateModal)

  // 确保设置窗口事件处理器
  windowStore.setupWindowEventHandlers(router)

  // 监听窗口显示
  await appWindow.listen('tauri://show', () => {
    mitt.emit('window-show')
  })

  // 监听窗口恢复
  await appWindow.listen('tauri://restore', () => {
    mitt.emit('window-restore')
  })

  // 监听窗口隐藏
  await appWindow.listen('tauri://close-requested', async () => {
    // 改为使用 hide 代替默认关闭行为
    windowStore.hideWindow()
  })
})

// 清理事件监听
onBeforeUnmount(() => {
  windowStore.cleanupWindowEvents()
  mitt.off('show-update-modal', handleShowUpdateModal)
})
</script>

<style scoped>
/* 现代化顶部栏样式 */
.modern-header {
  position: relative;
  border: none !important;
  overflow: hidden;
}

/* 毛玻璃背景层 */
.header-backdrop {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--n-card-color);
  opacity: 0.95;
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border-bottom: 1px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 主要内容容器 */
.header-content {
  position: relative;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  z-index: 1;
}

/* 左侧Logo和标题区域 */
.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  min-width: 0;
}

.logo-container {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
}

.app-logo {
  border-radius: 8px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  filter: drop-shadow(0 2px 8px rgba(0, 0, 0, 0.1));
}

.logo-glow {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 40px;
  height: 40px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 10px;
  opacity: 0;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: -1;
}

.logo-container:hover .logo-glow {
  opacity: 0.2;
  transform: translate(-50%, -50%) scale(1.1);
}

.app-title-section {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.app-title {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  letter-spacing: -0.5px;
  color: var(--n-text-color);
  display: flex;
  align-items: baseline;
  gap: 6px;
}

.app-subtitle {
  font-size: 12px;
  font-weight: 600;
  opacity: 0.8;
  color: var(--n-text-color);
  -webkit-text-fill-color: var(--n-text-color);
  background: unset;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

/* 状态标签 */
.status-tag {
  font-size: 11px !important;
  font-weight: 500 !important;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--n-text-color-disabled);
  flex-shrink: 0;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.status-dot.active {
  background: var(--n-success-color);
  box-shadow: 0 0 8px var(--n-success-color-pressed);
  animation: pulse 2s infinite;
}

/* 中间区域 */
.header-center {
  flex: 1;
  display: flex;
  justify-content: center;
  max-width: 400px;
  margin: 0 20px;
}

/* 右侧控制区域 */
.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 4px;
}

.function-group {
  padding: 4px;
  background: var(--n-color-target);
  border: 1px solid var(--n-border-color);
  border-radius: 10px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.window-group {
  margin-left: 8px;
}

/* 头部按钮样式 */
.header-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--n-text-color);
  position: relative;
  overflow: hidden;
}

.header-btn::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: currentColor;
  opacity: 0;
  transition: opacity 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: inherit;
}

.header-btn:hover::before {
  opacity: 0.1;
}

.header-btn:active {
  transform: scale(0.95);
}

/* 主题切换按钮 */
.theme-btn:hover {
  color: var(--n-warning-color);
  transform: translateY(-1px);
}

/* 全屏按钮 */
.fullscreen-btn:hover {
  color: var(--n-info-color);
  transform: translateY(-1px);
}

/* 最小化按钮 */
.minimize-btn:hover {
  background: var(--n-info-color-hover);
  color: var(--n-info-color);
  transform: translateY(-1px);
}

/* 关闭按钮 */
.close-btn:hover {
  background: var(--n-error-color-hover);
  color: var(--n-error-color);
  transform: translateY(-1px);
}

/* 动画定义 */
@keyframes pulse {
  0%,
  100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.7;
    transform: scale(1.1);
  }
}

/* 侧边栏样式优化 */
.custom-sider {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

/* 主滚动条样式 */
.main-scrollbar {
  border-radius: 0;
  padding-right: 8px;
}

:deep(.n-scrollbar-rail) {
  right: 0 !important;
}

/* 响应式优化 */
@media (max-width: 768px) {
  .header-content {
    padding: 0 16px;
  }

  .header-center {
    display: none;
  }

  .app-title {
    font-size: 16px;
  }

  .status-indicator {
    display: none;
  }
}

/* 高对比度模式支持 */
@media (prefers-contrast: high) {
  .header-backdrop {
    backdrop-filter: none;
    opacity: 1;
  }

  .header-btn::before {
    opacity: 0.2;
  }
}

/* 减少动画模式支持 */
@media (prefers-reduced-motion: reduce) {
  * {
    transition: none !important;
    animation: none !important;
  }
}
</style>
