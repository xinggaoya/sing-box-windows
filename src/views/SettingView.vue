<template>
  <n-card style="height: 100%" content-style="height: 100%;padding: 10px">
    <n-scrollbar>
      <n-form label-placement="left">
        <n-form-item label="内核设置">
          <n-button type="primary" @click="downloadTheKernel">
            下载内核
          </n-button>
        </n-form-item>
        <n-form-item label="开机自启">
          <n-switch v-model:value="appStore.autoStart" @update-value="onAutoStartChange" />
        </n-form-item>
      </n-form>
    </n-scrollbar>
  </n-card>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { useAppStore } from '@/stores/AppStore'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'

const message = useMessage()
const appStore = useAppStore()

const downloadTheKernel = async () => {
  const loading = message.loading('下载内核中')
  const res = await invoke('download_latest_kernel')
  loading.destroy()
  message.success('下载完成')
}

const onAutoStartChange = async (value: boolean) => {
  if (await isEnabled()) {
    await disable()
    message.success('已关闭开机自启')
  } else {
    await enable()
    message.success('已开启开机自启')
  }
}

</script>

<style scoped>

</style>