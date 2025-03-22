<template>
  <n-layout position="absolute">
    <n-layout-header bordered style="height: 64px; padding: 12px 24px">
      <n-flex justify="space-between" align="center" data-tauri-drag-region>
        <n-space align="center" :size="16">
          <n-image
            :src="logo"
            width="36"
            height="36"
            preview-disabled
            style="transition: all 0.3s ease"
          />
          <n-h2 style="margin: 0; font-weight: 600">
            Sing-Box
            <n-text depth="3" style="font-size: 14px; margin-left: 4px">Windows</n-text>
          </n-h2>
        </n-space>
        <n-space :size="16">
          <n-button quaternary size="medium" @click="appStore.toggleTheme" class="header-button">
            <template #icon>
              <n-icon>
                <moon-outline v-if="appStore.isDark" />
                <sunny-outline v-else />
              </n-icon>
            </template>
          </n-button>
          <n-button quaternary size="medium" @click="onToggleFullScreen" class="header-button">
            <template #icon>
              <n-icon>
                <expand-outline v-if="!appStore.windowState.isFullscreen" />
                <contract-outline v-else />
              </n-icon>
            </template>
          </n-button>
          <n-button quaternary size="medium" @click="appStore.minimizeWindow" class="header-button">
            <template #icon>
              <n-icon>
                <remove-outline />
              </n-icon>
            </template>
          </n-button>
          <n-button quaternary size="medium" @click="appStore.hideWindow" class="header-button">
            <template #icon>
              <n-icon>
                <close-outline />
              </n-icon>
            </template>
          </n-button>
        </n-space>
      </n-flex>
    </n-layout-header>
    <n-layout has-sider position="absolute" style="top: 64px">
      <n-layout-sider
        bordered
        collapse-mode="width"
        :collapsed-width="70"
        :width="220"
        show-trigger
        style="background-color: var(--n-color-base)"
        @collapse="collapsed = true"
        @expand="collapsed = false"
        class="custom-sider"
      >
        <div class="custom-menu" :class="{ 'menu-collapsed': collapsed }">
          <div
            v-for="(item, index) in menuOptions"
            :key="index"
            class="menu-item"
            :class="{
              'menu-item-active': currentMenu === item.key,
              'menu-item-disabled': item.disabled,
            }"
            @click="!item.disabled && onSelect(item.key)"
          >
            <div class="menu-indicator" :class="{ active: currentMenu === item.key }" />
            <div class="menu-item-content">
              <n-icon :size="24" class="menu-icon">
                <component :is="item.icon" />
              </n-icon>
              <span v-show="!collapsed" class="menu-label">{{ item.label }}</span>
            </div>
          </div>
        </div>
      </n-layout-sider>
      <n-layout-content content-style="padding: 0;">
        <n-scrollbar style="max-height: calc(100vh - 64px)" class="main-scrollbar">
          <router-view />
        </n-scrollbar>
      </n-layout-content>
    </n-layout>
  </n-layout>

  <!-- 更新对话框 -->
  <update-modal
    v-model:show="showUpdateModal"
    :latest-version="updateInfo.latest_version"
    :current-version="appStore.appVersion"
    :download-url="updateInfo.download_url"
    @update="handleUpdate"
    @cancel="handleCancelUpdate"
  />
</template>

<script setup lang="ts">
import { darkTheme, useOsTheme, NIcon, useNotification, NButton, NProgress } from 'naive-ui'
import type { NotificationReactive } from 'naive-ui'
import { h, ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import mitt from '@/utils/mitt'
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
import { useAppStore } from '@/stores/AppStore'
import { listen } from '@tauri-apps/api/event'
import logo from '@/assets/icon.png'
import UpdateModal from '@/components/UpdateModal.vue'

const router = useRouter()
const appWindow = Window.getCurrent()
const appStore = useAppStore()
const notification = useNotification()
const osThemeRef = useOsTheme()
const isDark = ref(osThemeRef.value === 'dark')
const theme = ref(isDark.value ? darkTheme : null)
const collapsed = ref(false)
const currentMenu = ref(0)
const isFullscreen = ref(false)

// 更新对话框相关状态
const showUpdateModal = ref(false)
const updateInfo = ref({
  latest_version: '',
  download_url: '',
  has_update: false,
})

// 检查更新
const checkUpdateWithNotification = async () => {
  try {
    const result = await appStore.checkUpdate()
    if (result?.has_update) {
      updateInfo.value = result
      showUpdateModal.value = true
    }
  } catch (error) {
    console.error('检查更新失败:', error)
  }
}

// 处理更新
const handleUpdate = async (downloadUrl: string) => {
  try {
    await appStore.downloadAndInstallUpdate()
  } catch (error) {
    notification.error({
      title: '更新失败',
      content: String(error),
      duration: 5000,
    })
  }
}

// 取消更新
const handleCancelUpdate = () => {
  showUpdateModal.value = false
}

const toggleTheme = () => {
  isDark.value = !isDark.value
  theme.value = isDark.value ? darkTheme : null
}

const onToggleFullScreen = async () => {
  await appStore.toggleFullScreen()
}

const menuOptions = computed(() => [
  {
    label: '主页',
    key: 0,
    icon: HomeOutline,
  },
  {
    label: '代理',
    key: 1,
    disabled: !appStore.isRunning,
    icon: SwapHorizontalOutline,
  },
  {
    label: '规则',
    key: 2,
    disabled: !appStore.isRunning,
    icon: FilterOutline,
  },
  {
    label: '连接',
    key: 3,
    disabled: !appStore.isRunning,
    icon: LinkOutline,
  },
  {
    label: '订阅',
    key: 4,
    icon: AtCircleOutline,
  },
  {
    label: '日志',
    key: 5,
    icon: DocumentTextOutline,
  },
  {
    label: '设置',
    key: 6,
    icon: SettingsOutline,
  },
])

function onSelect(key: number) {
  switch (key) {
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
  currentMenu.value = key
}

// 监听窗口事件
onMounted(async () => {
  // 获取当前版本号并检查更新
  await appStore.fetchAppVersion()
  await checkUpdateWithNotification()

  // 监听窗口显示
  await appWindow.listen('tauri://show', () => {
    mitt.emit('window-show')
  })

  // 监听窗口恢复
  await appWindow.listen('tauri://restore', () => {
    mitt.emit('window-restore')
  })
})
</script>

<style scoped>
.custom-sider {
  transition: all 0.3s ease;
  border-right: 1px solid var(--n-border-color);
}

.custom-menu {
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.menu-collapsed {
  padding: 12px 8px;
  align-items: center;
}

.menu-item {
  position: relative;
  padding: 12px 16px;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s ease;
  user-select: none;
  overflow: hidden;
}

.menu-item-content {
  display: flex;
  align-items: center;
  gap: 12px;
  position: relative;
  z-index: 2;
}

.menu-label {
  font-size: 15px;
  font-weight: 500;
  transition: all 0.25s ease;
  color: var(--n-text-color-1);
}

.menu-icon {
  transition: all 0.25s ease;
  color: var(--n-text-color-2);
}

.menu-item:hover {
  color: var(--primary-color);
  background-color: rgba(64, 128, 255, 0.08);
}

.menu-item-active {
  color: var(--primary-color);
  background-color: rgba(64, 128, 255, 0.15);
  box-shadow: 0 2px 8px rgba(64, 128, 255, 0.1);
}

.menu-item-active .menu-icon,
.menu-item-active .menu-label {
  color: var(--primary-color);
  font-weight: 600;
}

.menu-indicator {
  position: absolute;
  left: 0;
  top: 50%;
  height: 60%;
  width: 4px;
  border-radius: 4px;
  transform: translateY(-50%);
  background-color: transparent;
  transition: all 0.3s ease;
}

.menu-indicator.active {
  background-color: var(--primary-color);
  box-shadow: 0 0 8px rgba(64, 128, 255, 0.5);
}

.menu-item-disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.header-button {
  transition: all 0.3s ease;
}

.header-button:hover {
  transform: translateY(-1px);
}

/* 增加选中特效 */
.menu-item::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: transparent;
  z-index: 1;
  transition: all 0.3s ease;
}

.menu-item-active::before {
  background-color: rgba(64, 128, 255, 0.05);
}

:deep(.dark) .custom-sider {
  background-color: rgba(34, 34, 38, 0.95);
}

:deep(.dark) .menu-item:hover {
  background-color: rgba(64, 128, 255, 0.15);
}

:deep(.dark) .menu-item-active {
  background-color: rgba(64, 128, 255, 0.25);
  box-shadow: 0 2px 10px rgba(64, 128, 255, 0.2);
}

:deep(.dark) .header-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

:deep(.dark) .menu-label {
  color: rgba(255, 255, 255, 0.85);
}

:deep(.dark) .menu-icon {
  color: rgba(255, 255, 255, 0.75);
}

.main-scrollbar {
  border-radius: 0;
  padding-right: 8px;
}

:deep(.n-scrollbar-rail) {
  right: 0 !important;
}
</style>
