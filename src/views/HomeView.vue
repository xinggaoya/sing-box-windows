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
          {{ trafficStr.up }}
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
          {{ trafficStr.down }}
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
          {{ memory.inuse }}
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
          {{ useTotalTraffic }}
        </div>
      </n-card>
    </n-flex>
    <n-card content-style="padding: 5px" style="height: calc(100vh - 220px)">
      <Echarts :download-speed="traffic.up" :upload-speed="traffic.down" />
    </n-card>
  </n-flex>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { computed, onMounted, ref } from 'vue'
import { createWebSocket } from '@/utils'
import { useAppStore } from '@/stores/AppStore'
import Echarts from '@/components/layout/Echarts.vue'

const message = useMessage()
const appState = useAppStore()
const useTotalTraffic = computed(() => {
  // 根据大小显示kb mb gb
  return formatBandwidth(appState.usedData)
})
const traffic = ref({
  up: 0,
  down: 0
})
const trafficStr = computed(() => {
  // 根据大小显示kb mb gb
  const up = formatBandwidth(traffic.value.up)
  const down = formatBandwidth(traffic.value.down)
  return {
    up,
    down
  }
})

function formatBandwidth(kbps: number) {
  kbps = kbps / 1024
  // 计算 MB/s 和 GB/s
  const mbps = kbps / 1024 // 将 KB/s 转为 MB/s
  const gbps = mbps / 1024 // 将 MB/s 转为 GB/s

  // 选择最佳单位
  let formattedBandwidth
  if (gbps >= 1) {
    formattedBandwidth = `${gbps.toFixed(2)} GB`
  } else if (mbps >= 1) {
    formattedBandwidth = `${mbps.toFixed(2)} MB`
  } else {
    formattedBandwidth = `${kbps.toFixed(2)} KB`
  }

  // 格式化输出，保持小数点后两位
  return formattedBandwidth
}

const memory = ref({
  inuse: '',
  oslimit: ''
})

onMounted(() => {
  initWS()
})

const initWS = async () => {
  // 流量
  createWebSocket(`ws://127.0.0.1:9090/traffic?token=`, (data) => {
    traffic.value = data
    // 转int
    appState.usedData += Number(data.up + data.down)
  })
  createWebSocket(`ws://127.0.0.1:9090/memory?token=`, (data) => {
    memory.value = {
      inuse: formatBandwidth(data.inuse),
      oslimit: formatBandwidth(data.oslimit)
    }
  })
}


// 执行内核
const runKernel = async () => {
  const loading = message.loading('正在启动')
  const res = await invoke('start_kernel')
  await initWS()
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
