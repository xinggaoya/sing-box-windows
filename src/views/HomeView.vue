<template>
  <div class="home-main">
    <n-card>
      <n-space vertical align="center">
        <n-button type="primary" @click="downloadTheKernel">
          下载内核
        </n-button>
        <n-input v-model:value="url" placeholder="请输入订阅链接" />
        <n-button type="primary" @click="downloadSubscription">
          下载订阅
        </n-button>
        <n-button type="success" @click="runKernel">
          启 动
        </n-button>
        <n-button type="error" @click="stopKernel">
          停 止
        </n-button>
      </n-space>
    </n-card>
  </div>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { ref } from 'vue'

const message = useMessage()
const url = ref('')

const downloadTheKernel = async () => {
  const loading = message.loading('下载内核中')
  const res = await invoke('download_latest_kernel')
  loading.destroy()
  message.success('下载完成')
}

const downloadSubscription = async () => {
  const loading = message.loading('下载订阅中')
  const res = await invoke('download_subscription', { url: url.value })
  loading.destroy()
  message.success('下载完成')
}

// 执行内核
const runKernel = async () => {
  const loading = message.loading('正在启动')
  const res = await invoke('start_kernel')
  loading.destroy()
  message.success('启动成功')
}

const stopKernel = async () => {
  const loading = message.loading('正在停止')
  const res = await invoke('stop_kernel')
  loading.destroy()
  message.success('停止成功')
}
</script>

<style scoped>
.home-main {
  margin: 10px;
}
</style>
