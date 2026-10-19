<template>
  <div class="diagnostics">
    <h3 class="setting-section-title">{{ t('setting.diagnostics.title') }}</h3>

    <!-- 网络诊断 -->
    <div class="setting-row top-align">
      <div class="setting-info">
        <div class="setting-label">{{ t('setting.diagnostics.networkTest') }}</div>
        <div class="setting-desc">{{ t('setting.diagnostics.networkTestDesc') }}</div>
      </div>
      <n-button
        size="small"
        secondary
        :loading="testing"
        :disabled="!kernelRunning"
        @click="runNetworkTest"
      >
        <template #icon>
          <n-icon><PulseOutline /></n-icon>
        </template>
        {{ t('setting.diagnostics.runTest') }}
      </n-button>
    </div>

    <div v-if="!kernelRunning" class="hint">{{ t('setting.diagnostics.kernelStopped') }}</div>

    <div v-else-if="networkResult" class="metric-grid">
      <div class="metric-item">
        <div class="metric-label">{{ t('setting.diagnostics.tcpRtt') }}</div>
        <div class="metric-value">{{ formatLatency(networkResult.tcp_rtt_ms) }}</div>
      </div>
      <div class="metric-item">
        <div class="metric-label">{{ t('setting.diagnostics.downloadSpeed') }}</div>
        <div class="metric-value">{{ formatSpeed(networkResult.download_speed_bps) }}</div>
      </div>
      <div class="metric-item">
        <div class="metric-label">{{ t('setting.diagnostics.downloadLatency') }}</div>
        <div class="metric-value">{{ formatLatency(networkResult.download_latency_ms) }}</div>
      </div>
      <div class="metric-item">
        <div class="metric-label">{{ t('setting.diagnostics.uploadSpeed') }}</div>
        <div class="metric-value">{{ formatSpeed(networkResult.upload_speed_bps) }}</div>
      </div>
      <div class="metric-item">
        <div class="metric-label">{{ t('setting.diagnostics.uploadLatency') }}</div>
        <div class="metric-value">{{ formatLatency(networkResult.upload_latency_ms) }}</div>
      </div>
    </div>

    <!-- 服务状态 -->
    <div class="setting-row top-align">
      <div class="setting-info">
        <div class="setting-label">{{ t('setting.diagnostics.services') }}</div>
        <div class="setting-desc">{{ t('setting.diagnostics.servicesDesc') }}</div>
      </div>
      <n-button
        size="small"
        secondary
        :loading="servicesLoading"
        :disabled="!kernelRunning"
        @click="refreshServices"
      >
        <template #icon>
          <n-icon><RefreshOutline /></n-icon>
        </template>
        {{ t('common.refresh') }}
      </n-button>
    </div>

    <div v-if="serviceStatus" class="service-status">
      <n-tag size="small" round :bordered="false" :type="serviceStatusType">
        {{ serviceStatusLabel }}
      </n-tag>
      <span v-if="serviceError" class="service-error">{{ serviceError }}</span>
    </div>

    <div v-if="services.length === 0" class="hint">
      {{ t('setting.diagnostics.noServices') }}
    </div>

    <div v-else class="service-list">
      <div v-for="svc in services" :key="svc.name" class="service-row">
        <span class="service-name">{{ svc.name }}</span>
        <n-tag size="tiny" round :bordered="false" type="info">{{ svc.type }}</n-tag>
        <n-tag size="tiny" round :bordered="false" :type="svc.running ? 'success' : 'default'">
          {{ svc.running ? t('setting.diagnostics.running') : t('setting.diagnostics.stopped') }}
        </n-tag>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { PulseOutline, RefreshOutline } from '@vicons/ionicons5'
import { NIcon, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { proxyService } from '@/services/proxy-service'
import type { NetworkQualityResult } from '@/types/generated/NetworkQualityResult'
import type { Service } from '@/types/generated/Service'
import type { ServiceStatus } from '@/types/generated/ServiceStatus'
import { useKernelStore } from '@/stores/kernel/KernelStore'

defineOptions({ name: 'SettingsDiagnosticsCard' })

const { t } = useI18n()
const kernelStore = useKernelStore()

const networkResult = ref<NetworkQualityResult | null>(null)
const testing = ref(false)
const services = ref<Service[]>([])
const servicesLoading = ref(false)
const serviceStatus = ref<ServiceStatus | ''>('')
const serviceError = ref('')

const kernelRunning = computed(() => kernelStore.isRunning)

const serviceStatusType = computed<'success' | 'warning' | 'error' | 'default'>(() => {
  switch (serviceStatus.value) {
    case 'started':
      return 'success'
    case 'starting':
    case 'stopping':
      return 'warning'
    case 'fatal':
      return 'error'
    default:
      return 'default'
  }
})

/**
 * 服务状态文案。
 *
 * 刻意用 `switch` + 静态 `t(...)` 调用而不是模板字符串拼 key：
 * `scripts/check-i18n.mjs` 的门禁只扫静态字面量的翻译函数调用，动态 key 会绕过校验——
 * 某个语言包漏了某个状态值时，界面会直接显示裸 key，而门禁放行。
 */
const serviceStatusLabel = computed(() => {
  switch (serviceStatus.value) {
    case 'starting':
      return t('setting.diagnostics.status.starting')
    case 'started':
      return t('setting.diagnostics.status.started')
    case 'stopping':
      return t('setting.diagnostics.status.stopping')
    case 'fatal':
      return t('setting.diagnostics.status.fatal')
    case 'idle':
    default:
      return t('setting.diagnostics.status.idle')
  }
})

const runNetworkTest = async () => {
  if (testing.value) return
  testing.value = true
  try {
    networkResult.value = await proxyService.networkQualityTest()
  } catch (error) {
    console.warn('[SettingsDiagnosticsCard] network_quality_test failed:', error)
    networkResult.value = null
  } finally {
    testing.value = false
  }
}

const refreshServices = async () => {
  if (servicesLoading.value) return
  servicesLoading.value = true
  try {
    const [list, status] = await Promise.allSettled([
      proxyService.getServices(),
      proxyService.probeServiceStatus(),
    ])
    services.value = list.status === 'fulfilled' ? (list.value?.services ?? []) : []
    if (status.status === 'fulfilled') {
      serviceStatus.value = status.value?.status ?? ''
      serviceError.value = status.value?.error_message ?? ''
    } else {
      serviceStatus.value = ''
      serviceError.value = ''
    }
  } finally {
    servicesLoading.value = false
  }
}

const formatLatency = (ms: number) => {
  if (!ms || ms < 0) return '—'
  return `${ms}ms`
}

const formatSpeed = (bps: number) => {
  if (!bps || bps <= 0) return '—'
  if (bps >= 1_000_000) return `${(bps / 1_000_000).toFixed(1)} Mbps`
  if (bps >= 1_000) return `${(bps / 1_000).toFixed(0)} Kbps`
  return `${bps} bps`
}

onMounted(() => {
  if (kernelRunning.value) void refreshServices()
})
</script>

<style scoped>
.diagnostics {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.metric-item {
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  background: var(--bg-surface-2);
  border: 1px solid var(--panel-border);
}

.metric-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-bottom: var(--space-1);
}

.metric-value {
  font-size: var(--text-md);
  font-weight: 700;
  color: var(--text-primary);
  font-family: var(--font-mono);
}

.service-status {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.service-error {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.service-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.service-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-sm);
}

.service-row:hover {
  background: var(--bg-surface-2);
}

.service-name {
  flex: 1;
  min-width: 0;
  font-size: var(--text-sm);
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hint {
  padding: var(--space-1) 0 var(--space-2);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
</style>
