<template>
  <n-card style="height: 100%" content-style="height: 100%;padding: 10px">
    <n-scrollbar>
      <n-flex vertical>
        <n-button type="primary" @click="downloadTheKernel">
          下载内核
        </n-button>
      </n-flex>
    </n-scrollbar>
  </n-card>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'

const message = useMessage()

const downloadTheKernel = async () => {
  const loading = message.loading('下载内核中')
  const res = await invoke('download_latest_kernel')
  loading.destroy()
  message.success('下载完成')
}

</script>

<style scoped>

</style>