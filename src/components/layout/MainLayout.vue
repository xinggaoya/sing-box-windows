<template>
  <n-config-provider :theme="theme">
    <n-message-provider>
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
              <n-button quaternary circle size="small" @click="toggleTheme">
                <template #icon>
                  <n-icon>
                    <moon-outline v-if="isDark" />
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
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { darkTheme, useOsTheme, NIcon } from 'naive-ui'
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
import logo from '@/assets/icon.png'

const router = useRouter()
const appWindow = Window.getCurrent()
const appState = useAppStore()
const osThemeRef = useOsTheme()
const isDark = ref(osThemeRef.value === 'dark')
const theme = ref(isDark.value ? darkTheme : null)
const collapsed = ref(false)
const currentMenu = ref(0)
const isFullscreen = ref(false)

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
    disabled: !appState.isRunning,
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
