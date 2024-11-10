<template>
  <div class="home-main">
    <n-card>
      <n-flex justify="space-between">
        <n-space>
          <n-button type="primary" @click="downloadTheKernel">
            下载内核
          </n-button>
          <n-input v-model:value="url" placeholder="请输入订阅链接" />
          <n-button type="primary" @click="downloadSubscription">
            下载订阅
          </n-button>
        </n-space>
        <n-space>
          <n-button type="success" @click="runKernel">
            启 动
          </n-button>
          <n-button type="error" @click="stopKernel">
            停 止
          </n-button>
        </n-space>
      </n-flex>
    </n-card>
    <n-card v-if="showUI" style="margin-top: 7px">
      <div>
        <iframe src="http://127.0.0.1:9090" width="100%" height="440px"></iframe>
      </div>
    </n-card>
  </div>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { onMounted, ref } from 'vue'

const message = useMessage()
const url = ref('')
const showUI = ref(false)

onMounted(() => {
  showUI.value = localStorage.getItem('showUI') === 'true'
})

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
  showUI.value = true
  localStorage.setItem('showUI', 'true')
}

const stopKernel = async () => {
  const loading = message.loading('正在停止')
  const res = await invoke('stop_kernel')
  loading.destroy()
  message.success('停止成功')
  showUI.value = false
  localStorage.setItem('showUI', 'false')
}
</script>

<style scoped>
.home-main {
  margin: 10px;
}
</style>
