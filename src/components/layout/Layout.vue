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
            :collapsed-width="48"
            :width="180"
            show-trigger
            @collapse="collapsed = true"
            @expand="collapsed = false"
          >
            <n-menu
              :collapsed="collapsed"
              :collapsed-width="48"
              :collapsed-icon-size="20"
              :options="menuOptions"
              :value="currentMenu"
              :icon-size="18"
              :indent="24"
              @update:value="onSelect"
            />
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
import { h, ref, onMounted } from 'vue'
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
  CloseOutline
} from '@vicons/ionicons5'
import { Window } from '@tauri-apps/api/window'
import logo from '@/assets/icon.png'

const router = useRouter()
const appWindow = Window.getCurrent()
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

function renderIcon(icon: any) {
  return () => h(NIcon, { size: 22 }, { default: () => h(icon) })
}

const menuOptions = [
  {
    label: () => h('span', { style: 'font-size: 16px' }, '主页'),
    key: 0,
    icon: renderIcon(HomeOutline)
  },
  {
    label: () => h('span', { style: 'font-size: 16px' }, '代理'),
    key: 1,
    icon: renderIcon(SwapHorizontalOutline)
  },
  {
    label: () => h('span', { style: 'font-size: 16px' }, '订阅'),
    key: 2,
    icon: renderIcon(AtCircleOutline)
  },
  {
    label: () => h('span', { style: 'font-size: 16px' }, '日志'),
    key: 3,
    icon: renderIcon(DocumentTextOutline)
  },
  {
    label: () => h('span', { style: 'font-size: 16px' }, '设置'),
    key: 4,
    icon: renderIcon(SettingsOutline)
  }
]

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
  font-family: v-sans, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

a {
  text-decoration: none;
  color: inherit;
}

[data-tauri-drag-region] {
  cursor: move;
}
</style>
