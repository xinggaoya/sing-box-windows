<template>
  <n-layout position="absolute">
    <n-layout-header bordered style="height: 56px; padding: 8px 16px">
      <n-flex justify="space-between" align="center" data-tauri-drag-region>
        <n-space align="center">
          <n-image :src="logo" width="32" height="32" />
          <n-h2 style="margin: 0">
            Sing-Box
            <n-text depth="3" style="font-size: 12px">Windows</n-text>
          </n-h2>
        </n-space>
        <n-space :size="12">
          <n-button quaternary circle size="small" @click="appStore.toggleTheme">
            <template #icon>
              <n-icon>
                <moon-outline v-if="appStore.isDark" />
                <sunny-outline v-else />
              </n-icon>
            </template>
          </n-button>
          <n-button quaternary circle size="small" @click="handleFullScreen">
            <template #icon>
              <n-icon>
                <expand-outline v-if="!isFullscreen" />
                <contract-outline v-else />
              </n-icon>
            </template>
          </n-button>
          <n-button quaternary circle size="small" @click="handleMinimize">
            <template #icon>
              <n-icon>
                <remove-outline />
              </n-icon>
            </template>
          </n-button>
          <n-button quaternary circle size="small" @click="handleClose">
            <template #icon>
              <n-icon>
                <close-outline />
              </n-icon>
            </template>
          </n-button>
        </n-space>
      </n-flex>
    </n-layout-header>
    <n-layout has-sider position="absolute" style="top: 56px">
      <n-layout-sider
        bordered
        collapse-mode="width"
        :collapsed-width="64"
        :width="200"
        show-trigger
        @collapse="collapsed = true"
        @expand="collapsed = false"
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
            <div class="menu-item-content">
              <n-icon :size="22" class="menu-icon">
                <component :is="item.icon" />
              </n-icon>
              <span v-show="!collapsed" class="menu-label">{{ item.label }}</span>
            </div>
            <div
              v-show="!collapsed"
              class="menu-indicator"
              :class="{ active: currentMenu === item.key }"
            />
          </div>
        </div>
      </n-layout-sider>
      <n-layout-content content-style="padding: 8px;">
        <n-scrollbar style="max-height: calc(100vh - 56px)">
          <router-view />
        </n-scrollbar>
      </n-layout-content>
    </n-layout>
  </n-layout>
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
} from '@vicons/ionicons5'
import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/AppStore'
import { listen } from '@tauri-apps/api/event'
import logo from '@/assets/icon.png'

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

// 检查更新并显示通知
const checkUpdateWithNotification = async () => {
  try {
    const result = await appStore.checkUpdate()
    if (result?.has_update) {
      const notificationReactive = ref<NotificationReactive | null>(null)
      const updateProgress = ref(0)
      const isUpdating = ref(false)

      // 监听更新进度
      const unlistenProgress = await listen(
        'update-progress',
        (event: { payload: { status: string; progress: number; message: string } }) => {
          const { status, progress } = event.payload
          if (status === 'downloading') {
            updateProgress.value = progress
          } else if (status === 'completed') {
            notification.success({
              title: '更新下载完成',
              content: '即将安装更新...',
              duration: 3000,
            })
            unlistenProgress()
          }
        },
      )

      notificationReactive.value = notification.create({
        title: '发现新版本',
        content: () =>
          h('div', [
            h('p', `新版本 ${result.latest_version} 已发布，是否立即更新？`),
            h(
              'p',
              { style: 'margin: 4px 0; color: var(--n-text-color-3);' },
              `当前版本：${appStore.appVersion}`,
            ),
            isUpdating.value
              ? h(NProgress, {
                  type: 'line',
                  percentage: updateProgress.value,
                  indicatorPlacement: 'inside',
                  processing: updateProgress.value < 100,
                  style: 'margin-top: 8px',
                })
              : null,
            h(
              'div',
              {
                style: 'margin-top: 8px; display: flex; gap: 12px;',
              },
              [
                h(
                  NButton,
                  {
                    type: 'primary',
                    size: 'small',
                    loading: isUpdating.value,
                    disabled: isUpdating.value,
                    onClick: async () => {
                      try {
                        isUpdating.value = true
                        await appStore.downloadAndInstallUpdate()
                      } catch (error) {
                        notification.error({
                          title: '更新失败',
                          content: String(error),
                          duration: 5000,
                        })
                        unlistenProgress()
                      }
                    },
                  },
                  {
                    default: () =>
                      isUpdating.value ? `正在更新 ${updateProgress.value}%` : '立即更新',
                  },
                ),
                h(
                  NButton,
                  {
                    size: 'small',
                    disabled: isUpdating.value,
                    onClick: () => {
                      unlistenProgress()
                      notification.destroyAll()
                    },
                  },
                  { default: () => '下次再说' },
                ),
              ],
            ),
          ]),
        duration: 0,
      })
    }
  } catch (error) {
    console.error('检查更新失败:', error)
  }
}

const toggleTheme = () => {
  isDark.value = !isDark.value
  theme.value = isDark.value ? darkTheme : null
}

const handleFullScreen = async () => {
  isFullscreen.value = await appWindow.isFullscreen()
  await appWindow.setFullscreen(!isFullscreen.value)
  isFullscreen.value = !isFullscreen.value
}

const handleMinimize = async () => {
  mitt.emit('window-minimize')
  await appWindow.minimize()
}

const handleClose = async () => {
  mitt.emit('window-hide')
  await appWindow.hide()
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
    label: '订阅',
    key: 2,
    icon: AtCircleOutline,
  },
  {
    label: '日志',
    key: 3,
    icon: DocumentTextOutline,
  },
  {
    label: '设置',
    key: 4,
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
      router.push('/sub')
      break
    case 3:
      router.push('/log')
      break
    case 4:
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

<style>
body {
  margin: 0;
  font-family:
    v-sans,
    system-ui,
    -apple-system,
    BlinkMacSystemFont,
    'Segoe UI',
    sans-serif;
}

a {
  text-decoration: none;
  color: inherit;
}

[data-tauri-drag-region] {
  cursor: move;
}

.custom-menu {
  padding: 12px;
  transition: all 0.3s ease;
}

.menu-collapsed {
  padding: 12px 4px;
}

.menu-item {
  position: relative;
  margin-bottom: 8px;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
  user-select: none;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
}

.menu-item:hover:not(.menu-item-disabled) {
  background-color: rgba(51, 102, 255, 0.1);
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(51, 102, 255, 0.15);
}

.menu-item-active {
  background-color: #3366ff;
  color: white;
  box-shadow: 0 4px 12px rgba(51, 102, 255, 0.25);
}

.menu-item-active:hover {
  background-color: #5a7eff !important;
  box-shadow: 0 4px 12px rgba(51, 102, 255, 0.3);
}

.menu-item-disabled {
  opacity: 0.5;
  cursor: not-allowed;
  background-color: rgba(0, 0, 0, 0.02);
}

.menu-item-content {
  display: flex;
  align-items: center;
  gap: 12px;
  z-index: 1;
  position: relative;
}

.menu-icon {
  flex-shrink: 0;
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.menu-item:hover:not(.menu-item-disabled) .menu-icon {
  transform: scale(1.1);
}

.menu-label {
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
  opacity: 1;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.menu-indicator {
  position: absolute;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 0;
  background-color: #3366ff;
  border-radius: 1.5px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  opacity: 0;
}

.menu-indicator.active {
  height: 24px;
  opacity: 1;
}

.menu-item-active .menu-indicator {
  background-color: white;
  box-shadow: 0 0 8px rgba(255, 255, 255, 0.5);
}

:root {
  --primary-color: #3366ff;
  --primary-color-hover: #5a7eff;
  --primary-color-pressed: #1a4bff;
  --primary-color-rgb: 51, 102, 255;
  --card-color: var(--n-color);
  --card-color-hover: var(--n-color-hover);
  --text-color-1: var(--n-text-color-base);
  --text-color-2: var(--n-text-color-2);
  --text-color-3: var(--n-text-color-3);
  --divider-color: var(--n-divider-color);
}
</style>
