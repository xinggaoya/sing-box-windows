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
          <n-button type="success" :disabled="appState.isRunning" @click="runKernel">
            <template #icon>
              <n-icon>
                <AirplaneOutline />
              </n-icon>
            </template>
            启动
          </n-button>
          <n-button type="error" :disabled="!appState.isRunning" @click="stopKernel">
            <template #icon>
              <n-icon>
                <StopCircle />
              </n-icon>
            </template>
            停止
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
          {{ memoryStr }}
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
      <Echarts :download-speed="infoStore.traffic.up" :upload-speed="infoStore.traffic.down" />
    </n-card>
  </n-flex>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { computed } from 'vue'
import { formatBandwidth } from '@/utils'
import { AirplaneOutline, StopCircle } from '@vicons/ionicons5'
import { useAppStore } from '@/stores/AppStore'
import Echarts from '@/components/layout/Echarts.vue'
import { useInfoStore } from '@/stores/infoStore'

const message = useMessage()
const appState = useAppStore()
const infoStore = useInfoStore()
const useTotalTraffic = computed(() => {
  // 根据大小显示kb mb gb
  return formatBandwidth(appState.usedData)
})
// 内存
const memoryStr = computed(() => {
  // 根据大小显示kb mb gb
  return formatBandwidth(infoStore.memory.inuse)
})

const trafficStr = computed(() => {
  // 根据大小显示kb mb gb
  const up = formatBandwidth(infoStore.traffic.up)
  const down = formatBandwidth(infoStore.traffic.down)
  return {
    up,
    down
  }
})

// 执行内核
const runKernel = async () => {
  const loading = message.loading('正在启动')
  const res = await infoStore.startKernel()
  loading.destroy()
  message.success('启动成功')
}

const stopKernel = async () => {
  const loading = message.loading('正在停止')
  const res = await infoStore.stopKernel()
  loading.destroy()
  appState.isRunning = false
  message.success('停止成功')
}

const onModeChange = async (value: string) => {
  if (value === 'system') {
    await invoke('set_system_proxy')
    message.success('系统代理模式')
  } else {
    await invoke('set_tun_proxy')
    message.success('TUN模式')
  }
}
</script>

<style scoped>

.home-card-item {
  width: 130px;
}
</style>
