<template>
  <div class="home-main">
    <div>
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
          <router-link to="/about" class="text-decoration-none">
            代理
          </router-link>
          <n-button type="success" @click="runKernel">
            启 动
          </n-button>
          <n-button type="error" @click="stopKernel">
            停 止
          </n-button>
        </n-space>
      </n-flex>
    </div>
    <n-flex justify="space-between">
      <n-card
        style="margin-top: 7px;width: 160px"
        content-style="padding: 7px"
      >
        <div id="kernel-log">
          上传
        </div>
        <div id="subscription-log">
          {{ traffic.up }} kb/s
        </div>
      </n-card>
      <n-card
        style="margin-top: 7px;width: 160px"
        content-style="padding: 7px"
      >
        <div id="kernel-log">
          下载
        </div>
        <div id="subscription-log">
          {{ traffic.down }} kb/s
        </div>
      </n-card>
      <n-card
        style="margin-top: 7px;width: 160px"
        content-style="padding: 7px"
      >
        <div id="kernel-log">
          使用内存
        </div>
        <div id="subscription-log">
          {{ (memory.inuse / 1024 / 1024).toFixed(2) }} MB
        </div>
      </n-card>
      <n-card
        style="margin-top: 7px;width: 160px"
        content-style="padding: 7px"
      >
        <div id="kernel-log">
          使用流量
        </div>
        <div id="subscription-log">
          {{ useTotalTraffic.toFixed(2) }} MB
        </div>
      </n-card>
    </n-flex>
    <n-card v-if="showUI" style="margin-top: 7px" content-style="padding: 10px">
      <div>
        <iframe src="http://127.0.0.1:9090" width="100%" height="400px"></iframe>
      </div>
    </n-card>
  </div>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { onMounted, ref } from 'vue'
import { createWebSocket } from '@/utils'

const message = useMessage()
const url = ref('')
const showUI = ref(false)
const useTotalTraffic = ref(0)
const traffic = ref({
  up: 0,
  down: 0
})
const memory = ref({
  inuse: 0,
  oslimit: 0
})

onMounted(() => {
  showUI.value = localStorage.getItem('showUI') === 'true'
  initWS()
})

const initWS = async () => {
  // 流量
  createWebSocket(`ws://127.0.0.1:9090/traffic?token=`, (data) => {
    traffic.value = data
    // 转int
    useTotalTraffic.value += Number(((data.up + data.down) / 1024 / 1024))
  })
  createWebSocket(`ws://127.0.0.1:9090/memory?token=`, (data) => {
    memory.value = data
  })
}

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
