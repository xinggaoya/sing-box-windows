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
          <n-button quaternary size="medium" @click="themeStore.toggleTheme" class="header-button">
            <template #icon>
              <n-icon>
                <moon-outline v-if="themeStore.isDark" />
                <sunny-outline v-else />
              </n-icon>
            </template>
          </n-button>
          <n-button quaternary size="medium" @click="onToggleFullScreen" class="header-button">
            <template #icon>
              <n-icon>
                <expand-outline v-if="!windowStore.windowState.isFullscreen" />
                <contract-outline v-else />
              </n-icon>
            </template>
          </n-button>
          <n-button
            quaternary
            size="medium"
            @click="windowStore.minimizeWindow"
            class="header-button"
          >
            <template #icon>
              <n-icon>
                <remove-outline />
              </n-icon>
            </template>
          </n-button>
          <n-button quaternary size="medium" @click="windowStore.hideWindow" class="header-button">
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
    :current-version="updateStore.appVersion"
    :download-url="updateInfo.download_url"
    @update="handleUpdate"
    @cancel="handleCancelUpdate"
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
  type MenuOption,
} from 'naive-ui'
import type { NotificationReactive } from 'naive-ui'
import { h, ref, onMounted, computed, onBeforeUnmount } from 'vue'
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
import { useAppStore } from '@/stores/app/AppStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { useWindowStore } from '@/stores/app/WindowStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { listen } from '@tauri-apps/api/event'
import logo from '@/assets/icon.png'
import UpdateModal from '@/components/UpdateModal.vue'
import { useI18n } from 'vue-i18n'
import type { Component } from 'vue'

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

// 渲染图标函数
function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

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
    const result = await updateStore.checkUpdate()
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
    await updateStore.downloadAndInstallUpdate()
  } catch (error) {
    notification.error({
      title: t('notification.updateFailed'),
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

// 监听窗口事件
onMounted(async () => {
  // 获取当前版本号并检查更新
  await updateStore.fetchAppVersion()
  await checkUpdateWithNotification()

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
})
</script>

<style scoped>
.custom-sider {
  transition: all 0.3s ease;
}

.header-button {
  transition: all 0.3s ease;
}

.header-button:hover {
  transform: translateY(-1px);
}

.main-scrollbar {
  border-radius: 0;
  padding-right: 8px;
}

:deep(.n-scrollbar-rail) {
  right: 0 !important;
}
</style>
