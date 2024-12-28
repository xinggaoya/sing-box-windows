<template>
  <n-space vertical size="small">
    <n-card size="small">
      <n-tabs type="segment" animated v-model:value="appState.mode" @update:value="onModeChange">
        <n-tab-pane name="system">
          <template #tab>
            <n-space align="center" size="small" inline>
              <n-icon><DesktopOutline /></n-icon>
              系统代理
            </n-space>
          </template>
          <n-space vertical size="small">
            <n-alert v-if="appState.isRunning" type="success">
              系统代理模式运行中，所有应用程序都将通过代理访问网络
            </n-alert>
            <n-alert v-else type="info">
              选择此模式将自动配置系统代理，所有应用程序都将通过代理访问网络
            </n-alert>
          </n-space>
        </n-tab-pane>
        <n-tab-pane name="tun">
          <template #tab>
            <n-space align="center" size="small" inline>
              <n-icon><GitNetworkOutline /></n-icon>
              TUN 模式
            </n-space>
          </template>
          <n-space vertical size="small">
            <n-alert v-if="appState.isRunning" type="success">
              TUN 模式运行中，流量将通过虚拟网卡进行路由
            </n-alert>
            <n-alert v-else type="info">
              选择此模式将创建虚拟网卡，所有流量将通过 TUN 接口进行路由
            </n-alert>
          </n-space>
        </n-tab-pane>
      </n-tabs>

      <n-divider />

      <n-flex justify="center" align="center">
        <n-space>
          <n-button
            strong
            secondary
            size="large"
            type="primary"
            :disabled="appState.isRunning"
            @click="runKernel"
            :loading="isStarting"
          >
            <template #icon>
              <n-icon><PlayCircleOutline /></n-icon>
            </template>
            启动
          </n-button>
          <n-button
            strong
            secondary
            size="large"
            type="error"
            :disabled="!appState.isRunning"
            @click="stopKernel"
            :loading="isStopping"
          >
            <template #icon>
              <n-icon><StopCircleOutline /></n-icon>
            </template>
            停止
          </n-button>
        </n-space>
      </n-flex>
    </n-card>

    <n-grid :x-gap="8" :y-gap="8" :cols="4">
      <n-grid-item>
        <n-card size="small" hoverable>
          <n-statistic label="上传速度">
            <template #prefix>
              <n-icon color="#18a058"><ArrowUpOutline /></n-icon>
            </template>
            {{ trafficStr.up }}
          </n-statistic>
        </n-card>
      </n-grid-item>
      <n-grid-item>
        <n-card size="small" hoverable>
          <n-statistic label="下载速度">
            <template #prefix>
              <n-icon color="#2080f0"><ArrowDownOutline /></n-icon>
            </template>
            {{ trafficStr.down }}
          </n-statistic>
        </n-card>
      </n-grid-item>
      <n-grid-item>
        <n-card size="small" hoverable>
          <n-statistic label="内存占用">
            <template #prefix>
              <n-icon color="#d03050"><HardwareChipOutline /></n-icon>
            </template>
            {{ memoryStr }}
          </n-statistic>
        </n-card>
      </n-grid-item>
      <n-grid-item>
        <n-card size="small" hoverable>
          <n-statistic label="总流量">
            <template #prefix>
              <n-icon color="#f0a020"><AnalyticsOutline /></n-icon>
            </template>
            {{ useTotalTraffic }}
          </n-statistic>
        </n-card>
      </n-grid-item>
    </n-grid>

    <n-card title="实时流量监控" size="small" style="height: calc(100vh - 420px)">
      <Echarts :download-speed="infoStore.traffic.up" :upload-speed="infoStore.traffic.down" />
    </n-card>
  </n-space>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { computed, ref, onUnmounted, onMounted } from 'vue'
import { formatBandwidth } from '@/utils'
import { Window } from '@tauri-apps/api/window'
import mitt from '@/utils/mitt'
import {
  PlayCircleOutline,
  StopCircleOutline,
  DesktopOutline,
  GitNetworkOutline,
  ArrowUpOutline,
  ArrowDownOutline,
  HardwareChipOutline,
  AnalyticsOutline
} from '@vicons/ionicons5'
import { useAppStore } from '@/stores/AppStore'
import Echarts from '@/components/layout/Echarts.vue'
import { useInfoStore } from '@/stores/infoStore'

const message = useMessage()
const appState = useAppStore()
const infoStore = useInfoStore()
const isStarting = ref(false)
const isStopping = ref(false)
let monitorTimer: number | null = null
const isWindowVisible = ref(true)

// 开始监控
const startMonitoring = () => {
  if (monitorTimer || !isWindowVisible.value) return
  
  // 立即执行一次
  updateMetrics()
  
  // 每秒更新一次
  monitorTimer = window.setInterval(() => {
    updateMetrics()
  }, 1000)
}

// 停止监控
const stopMonitoring = () => {
  if (monitorTimer) {
    clearInterval(monitorTimer)
    monitorTimer = null
  }
}

// 更新指标数据
const updateMetrics = async () => {
  if (!isWindowVisible.value) return
  
  try {
    const [memory, traffic] = await Promise.all([
      invoke('get_memory_usage'),
      invoke('get_traffic_data')
    ])
    infoStore.updateMemory(memory as any)
    infoStore.updateTraffic(traffic as any)
  } catch (error) {
    console.error('Failed to update metrics:', error)
  }
}

// 监听窗口事件
const setupWindowListeners = () => {
  // 监听最小化事件
  mitt.on('window-minimize', () => {
    isWindowVisible.value = false
    stopMonitoring()
  })

  // 监听关闭/隐藏事件
  mitt.on('window-hide', () => {
    isWindowVisible.value = false
    stopMonitoring()
  })

  // 监听窗口显示事件
  mitt.on('window-show', () => {
    isWindowVisible.value = true
    if (appState.isRunning) {
      startMonitoring()
    }
  })

  // 监听窗口恢复事件
  mitt.on('window-restore', () => {
    isWindowVisible.value = true
    if (appState.isRunning) {
      startMonitoring()
    }
  })
}

// 组件挂载时设置监听器
onMounted(() => {
  setupWindowListeners()
})

// 组件卸载时清理
onUnmounted(() => {
  stopMonitoring()
  // 移除所有事件监听
  mitt.off('window-minimize')
  mitt.off('window-hide')
  mitt.off('window-show')
  mitt.off('window-restore')
})

const useTotalTraffic = computed(() => {
  return formatBandwidth(infoStore.traffic.total)
})
const memoryStr = computed(() => formatBandwidth(infoStore.memory.inuse))
const trafficStr = computed(() => ({
  up: formatBandwidth(Number(infoStore.traffic.up) || 0),
  down: formatBandwidth(Number(infoStore.traffic.down) || 0)
}))

const runKernel = async () => {
  try {
    isStarting.value = true
    await invoke('start_kernel')
    appState.isRunning = true
    // 重置流量统计
    infoStore.resetTraffic()
    message.success('内核已启动')
    // 只在窗口可见时启动监控
    if (isWindowVisible.value) {
      startMonitoring()
    }
  } catch (error) {
    appState.isRunning = false
    message.error(error as string)
  } finally {
    isStarting.value = false
  }
}

const stopKernel = async () => {
  try {
    isStopping.value = true
    await invoke('stop_kernel')
    appState.isRunning = false
    message.success('内核已停止')
    stopMonitoring()
    // 停止时保留流量统计，不重置
  } catch (error) {
    message.error(error as string)
  } finally {
    isStopping.value = false
  }
}

const onModeChange = async (value: string) => {
  try {
    if (value === 'system') {
      await invoke('set_system_proxy')
      message.success('已切换到系统代理模式')
    } else {
      await invoke('set_tun_proxy')
      message.success('已切换到TUN模式')
    }
  } catch (error) {
    message.error(error as string)
  }
}
</script>
