<template>
  <n-card title="应用设置" style="height: 100%" content-style="height: 100%;">
    <n-scrollbar>
      <n-flex>
        <n-text> 内核版本：{{ infoStore.version.version}}</n-text>
        <n-text> 最新版本：{{ infoStore.newVersion}}</n-text>
      </n-flex>
      <div>
        <n-form label-placement="left">
          <n-form-item label="内核设置">
            <n-button type="primary" @click="downloadTheKernel"> 下载内核</n-button>
          </n-form-item>
          <n-form-item label="开机自启">
            <n-switch v-model:value="appStore.autoStart" @update-value="onAutoStartChange" />
          </n-form-item>
          <n-form-item label="自动启动内核">
            <n-switch v-model:value="appStore.autoStartKernel" />
          </n-form-item>
        </n-form>
      </div>
    </n-scrollbar>
  </n-card>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { useAppStore } from '@/stores/AppStore'
import { enable, disable } from '@tauri-apps/plugin-autostart'
import { useInfoStore } from '@/stores/infoStore'

const message = useMessage()
const appStore = useAppStore()
const infoStore = useInfoStore()

const downloadTheKernel = async () => {
  const loading = message.loading('下载内核中')
  const res = await invoke('download_latest_kernel')
  loading.destroy()
  message.success('下载完成')
}

const onAutoStartChange = async (value: boolean) => {
  if (!value) {
    await disable()
    message.success('已关闭开机自启')
  } else {
    await enable()
    message.success('已开启开机自启')
  }
}
</script>

<style scoped></style>
