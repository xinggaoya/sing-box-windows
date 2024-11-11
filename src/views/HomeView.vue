<template>
  <n-flex vertical class="home-main">
    <n-card content-style="padding: 10px">
      <n-flex justify="space-between">
        <n-radio-group v-model:value="appState.mode" @update-value="onModeChange">
          <n-radio-button label="1" value="system">
            系统代理
          </n-radio-button>
          <n-radio-button label="2" value="tun">
            TUN
          </n-radio-button>
        </n-radio-group>
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
    <n-flex justify="space-between">
      <n-card
        class="home-card-item"
        hoverable
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
        content-style="padding: 7px"
        hoverable
        class="home-card-item"
      >
        <div id="kernel-log">
          下载
        </div>
        <div id="subscription-log">
          {{ traffic.down }} kb/s
        </div>
      </n-card>
      <n-card
        content-style="padding: 7px"
        hoverable
        class="home-card-item"
      >
        <div id="kernel-log">
          使用内存
        </div>
        <div id="subscription-log">
          {{ (memory.inuse / 1024 / 1024).toFixed(2) }} MB
        </div>
      </n-card>
      <n-card
        content-style="padding: 7px"
        hoverable
        class="home-card-item"
      >
        <div id="kernel-log">
          使用流量
        </div>
        <div id="subscription-log">
          {{ useTotalTraffic.toFixed(2) }} MB
        </div>
      </n-card>
    </n-flex>
    <n-card content-style="padding: 10px" style="height: calc(100vh - 220px)">
      123
    </n-card>
  </n-flex>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { onMounted, ref } from 'vue'
import { createWebSocket } from '@/utils'
import { useAppStore } from '@/stores/counter'

const message = useMessage()
const appState = useAppStore()
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

const onModeChange = async (value: string) => {
  if (value === 'system') {
    await invoke('set_system_proxy')
  } else {
    await invoke('set_tun_proxy')
  }
}
</script>

<style scoped>

.home-card-item {
  width: 130px;
}
</style>
