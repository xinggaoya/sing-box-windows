<template>
  <n-space vertical>
    <n-card>
      <template #header>
        <n-space align="center">
          <n-h3 style="margin: 0">内核管理</n-h3>
          <n-tag type="info" round>{{ infoStore.version.version }}</n-tag>
          <n-tag v-if="hasNewVersion" type="warning" round>
            新版本可用: {{ infoStore.newVersion }}
          </n-tag>
        </n-space>
      </template>
      <n-space vertical>
        <n-alert v-if="hasNewVersion" type="warning" title="发现新版本">
          有新版本的内核可供下载，建议更新以获得更好的体验。
        </n-alert>

        <n-progress
          v-if="downloading"
          type="line"
          :percentage="downloadProgress"
          :processing="downloadProgress < 100"
          :indicator-placement="'inside'"
        >
          {{ downloadMessage }}
        </n-progress>

        <n-space align="center">
          <n-button
            type="primary"
            @click="downloadTheKernel"
            :loading="loading"
            :disabled="downloading"
          >
            <template #icon>
              <n-icon>
                <download-outline />
              </n-icon>
            </template>
            {{ hasNewVersion ? '下载新版本' : '重新下载当前版本' }}
          </n-button>
          <n-text depth="3" style="font-size: 14px">
            当前版本：{{ infoStore.version.version }}
            <n-text v-if="hasNewVersion" type="warning">
              新版本可用: {{ infoStore.newVersion }}
            </n-text>
          </n-text>
        </n-space>
      </n-space>
    </n-card>

    <n-card>
      <template #header>
        <n-h3 style="margin: 0">启动设置</n-h3>
      </template>
      <n-form
        label-placement="left"
        :model="appStore"
        label-width="120"
        :style="{
          maxWidth: '640px',
        }"
      >
        <n-form-item label="开机自启">
          <n-space align="center">
            <n-switch v-model:value="appStore.autoStart" @update-value="onAutoStartChange">
              <template #checked> 开启 </template>
              <template #unchecked> 关闭 </template>
            </n-switch>
            <n-text depth="3">
              {{ appStore.autoStart ? '应用将在系统启动时自动运行' : '应用需要手动启动' }}
            </n-text>
          </n-space>
        </n-form-item>

        <n-form-item label="自动启动内核">
          <n-space align="center">
            <n-switch v-model:value="appStore.autoStartKernel">
              <template #checked> 开启 </template>
              <template #unchecked> 关闭 </template>
            </n-switch>
            <n-text depth="3">
              {{ appStore.autoStartKernel ? '应用启动时将自动启动内核' : '需要手动启动内核' }}
            </n-text>
          </n-space>
        </n-form-item>

        <n-form-item label="IPv6优先">
          <n-space align="center">
            <n-switch v-model:value="appStore.preferIpv6" @update-value="onIpVersionChange">
              <template #checked> 开启 </template>
              <template #unchecked> 关闭 </template>
            </n-switch>
            <n-text depth="3">
              {{ appStore.preferIpv6 ? '优先使用IPv6连接' : '仅使用IPv4连接' }}
            </n-text>
          </n-space>
        </n-form-item>
      </n-form>
    </n-card>

    <n-card>
      <template #header>
        <n-h3 style="margin: 0">关于</n-h3>
      </template>
      <n-descriptions bordered>
        <n-descriptions-item label="应用版本"> 1.0.0 </n-descriptions-item>
        <n-descriptions-item label="内核版本">
          {{ infoStore.version.version }}
        </n-descriptions-item>
        <n-descriptions-item label="系统"> Windows </n-descriptions-item>
        <n-descriptions-item label="开源协议"> MIT License </n-descriptions-item>
      </n-descriptions>
    </n-card>
  </n-space>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMessage } from 'naive-ui'
import { enable, disable } from '@tauri-apps/plugin-autostart'
import { useInfoStore } from '@/stores/infoStore'
import { useAppStore } from '@/stores/AppStore'
import { DownloadOutline } from '@vicons/ionicons5'
import { listen } from '@tauri-apps/api/event'
import { tauriApi } from '@/services/tauri-api'

const message = useMessage()
const appStore = useAppStore()
const infoStore = useInfoStore()
const loading = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadMessage = ref('')

// 监听下载进度事件
listen(
  'download-progress',
  (event: { payload: { status: string; progress: number; message: string } }) => {
    const { status, progress, message: msg } = event.payload
    downloadProgress.value = progress
    downloadMessage.value = msg

    if (status === 'completed') {
      downloading.value = false
      message.success('内核下载完成！')
    }
  },
)

const hasNewVersion = computed(() => {
  if (!infoStore.newVersion || !infoStore.version.version) return false
  return infoStore.newVersion.includes(infoStore.version.version)
})

const downloadTheKernel = async () => {
  try {
    loading.value = true
    downloading.value = true
    downloadProgress.value = 0
    downloadMessage.value = '准备下载...'

    await tauriApi.subscription.downloadLatestKernel()
  } catch (error) {
    message.error(error as string)
    downloading.value = false
  } finally {
    loading.value = false
  }
}

const onAutoStartChange = async (value: boolean) => {
  try {
    if (!value) {
      await disable()
      message.success('已关闭开机自启')
    } else {
      await enable()
      message.success('已开启开机自启')
    }
  } catch (error) {
    message.error('设置失败')
    // 回滚状态
    appStore.autoStart = !value
  }
}

const onIpVersionChange = async (value: boolean) => {
  try {
    await tauriApi.proxy.toggleIpVersion(value)
    // 切换后重启内核
    if (appStore.isRunning) {
      await tauriApi.kernel.restartKernel()
    }
  } catch (error: unknown) {
    message.error(`设置失败: ${error instanceof Error ? error.message : String(error)}`)
    // 回滚状态
    appStore.preferIpv6 = !value
  }
}
</script>

<style scoped>
.n-card {
  margin-bottom: 16px;
}
</style>
