<template>
  <n-config-provider :theme="configProviderTheme" :theme-overrides="themeOverrides">
    <n-global-style />
    <n-dialog-provider>
      <n-modal-provider>
        <n-notification-provider>
          <n-message-provider>
            <!-- 消息消费组件 -->
            <MessageConsumer />

            <!-- 主路由视图 -->
            <div class="app-container">
              <router-view />
            </div>

            <!-- 更新通知组件 -->
            <UpdateNotification />
          </n-message-provider>
        </n-notification-provider>
      </n-modal-provider>
    </n-dialog-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, defineComponent, onMounted, onBeforeUnmount } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'
import { useMessage } from 'naive-ui'

import {
  useThemeStore,
  useAppStore,
  useLocaleStore,
  useWindowStore,
  useTrayStore,
  useKernelStore,
  useUpdateStore,
  useSubStore,
  useTrafficStore,
  useConnectionStore,
  useLogStore,
} from '@/stores'

import UpdateNotification from '@/components/UpdateNotification.vue'
import { useAppBootstrap } from '@/boot/useAppBootstrap'

const MessageConsumer = defineComponent({
  name: 'MessageConsumer',
  setup() {
    const message = useMessage()

    onMounted(() => {
      mitt.emit('message-instance-ready', message)
    })

    return () => null
  },
})

const router = useRouter()
const { locale } = useI18n()

const themeStore = useThemeStore()
const appStore = useAppStore()
const localeStore = useLocaleStore()
const windowStore = useWindowStore()
const subStore = useSubStore()
const kernelStore = useKernelStore()
const updateStore = useUpdateStore()
const trafficStore = useTrafficStore()
const connectionStore = useConnectionStore()
const logStore = useLogStore()
const configProviderTheme = computed(() => themeStore.naiveTheme)
const themeOverrides = computed(() => themeStore.themeOverrides)

const cleanupFunctions: (() => void)[] = []

const handleBeforeUnload = () => {
  cleanup()
}

onMounted(async () => {
  window.addEventListener('beforeunload', handleBeforeUnload)
  cleanupFunctions.push(() => {
    window.removeEventListener('beforeunload', handleBeforeUnload)
  })

  try {
    const { initialize, cleanup: cleanupBootstrap } = useAppBootstrap({
      router,
      localeRef: locale,
      stores: {
        themeStore,
        appStore,
        localeStore,
        windowStore,
        subStore,
        kernelStore,
        updateStore,
        trafficStore,
        connectionStore,
        logStore,
        trayStore: useTrayStore(),
      },
    })

    await initialize()
    cleanupFunctions.push(() => cleanupBootstrap())
  } catch (error) {
    console.error('应用初始化失败:', error)
  }
})

function cleanup() {
  cleanupFunctions.forEach((fn) => fn())
  cleanupFunctions.length = 0
}

onBeforeUnmount(() => {
  cleanup()
})

</script>

<style>
/* 应用容器基础样式 */
#app {
  height: 100vh;
}

.app-container {
  height: 100%;
  width: 100%;
}
</style>
