<template>
  <n-card class="stats-card" :bordered="false">
    <template #header>
      <div class="stats-header">
        <h3 class="stats-title">
          <n-icon size="18" class="stats-icon">
            <analytics-outline />
          </n-icon>
          {{ t('home.traffic.title') }}
        </h3>
      </div>
    </template>

    <div class="traffic-content">
      <!-- 实时流量统计 -->
      <div class="traffic-stats">
        <div class="traffic-row">
          <div class="traffic-item">
            <div class="traffic-label">
              <n-icon size="16" class="traffic-icon upload-icon">
                <arrow-up-outline />
              </n-icon>
              <span>{{ t('home.traffic.uploadSpeed') }}</span>
            </div>
            <div class="traffic-value">{{ formattedUploadSpeed }}</div>
          </div>

          <div class="traffic-item">
            <div class="traffic-label">
              <n-icon size="16" class="traffic-icon download-icon">
                <arrow-down-outline />
              </n-icon>
              <span>{{ t('home.traffic.downloadSpeed') }}</span>
            </div>
            <div class="traffic-value">{{ formattedDownloadSpeed }}</div>
          </div>

          <div class="traffic-item">
            <div class="traffic-label">
              <n-icon size="16" class="traffic-icon cloud-up-icon">
                <cloud-upload-outline />
              </n-icon>
              <span>{{ t('home.traffic.uploadTotal') }}</span>
            </div>
            <div class="traffic-value">{{ formattedTotalUpload }}</div>
          </div>

          <div class="traffic-item">
            <div class="traffic-label">
              <n-icon size="16" class="traffic-icon cloud-down-icon">
                <cloud-download-outline />
              </n-icon>
              <span>{{ t('home.traffic.downloadTotal') }}</span>
            </div>
            <div class="traffic-value">{{ formattedTotalDownload }}</div>
          </div>

          <div class="traffic-item">
            <div class="traffic-label">
              <n-icon size="16" class="traffic-icon memory-icon">
                <hardware-chip-outline />
              </n-icon>
              <span>{{ t('home.traffic.memory') }}</span>
            </div>
            <div class="traffic-value">{{ formattedMemory }}</div>
          </div>
          
          <div class="traffic-item">
            <div class="traffic-label">
              <n-icon size="16" class="traffic-icon connections-icon">
                <git-network-outline />
              </n-icon>
              <span>{{ t('home.traffic.connectionsLabel') }}</span>
            </div>
            <div class="traffic-value">{{ activeConnectionsCount }}</div>
          </div>
        </div>
      </div>

      <!-- 流量图表 -->
      <div class="chart-container">
        <TrafficChart
          :upload-speed="trafficUp"
          :download-speed="trafficDown"
          class="traffic-chart"
        />
      </div>
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { formatBandwidth } from '@/utils'
import TrafficChart from '@/components/layout/TrafficChart.vue'
import {
  AnalyticsOutline,
  GitNetworkOutline,
  ArrowUpOutline,
  ArrowDownOutline,
  CloudUploadOutline,
  CloudDownloadOutline,
  HardwareChipOutline,
} from '@vicons/ionicons5'

defineOptions({
  name: 'TrafficStatsCard'
})

const props = defineProps({
  activeConnectionsCount: {
    type: String,
    default: '0'
  },
  trafficUp: {
    type: Number,
    default: 0
  },
  trafficDown: {
    type: Number,
    default: 0
  },
  totalUp: {
    type: Number,
    default: 0
  },
  totalDown: {
    type: Number,
    default: 0
  },
  memory: {
    type: Number,
    default: 0
  },
  isRouteActive: {
    type: Boolean,
    default: true
  }
})

const { t } = useI18n()

// 格式化上传和下载速度
const formattedUploadSpeed = computed(() => {
  if (!props.isRouteActive) return '0 B/s'
  return formatBandwidth(Number(props.trafficUp) || 0)
})

const formattedDownloadSpeed = computed(() => {
  if (!props.isRouteActive) return '0 B/s'
  return formatBandwidth(Number(props.trafficDown) || 0)
})

// 格式化总上传和下载流量
const formattedTotalUpload = computed(() => {
  if (!props.isRouteActive) return '0 B'
  return formatBandwidth(Number(props.totalUp) || 0)
})

const formattedTotalDownload = computed(() => {
  if (!props.isRouteActive) return '0 B'
  return formatBandwidth(Number(props.totalDown) || 0)
})

// 格式化内存使用
const formattedMemory = computed(() => {
  if (!props.isRouteActive) return '0 B'
  return formatBandwidth(props.memory || 0)
})
</script>

<style scoped>
.stats-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.06);
  overflow: hidden;
  backdrop-filter: blur(8px);
  border: 1px solid rgba(128, 128, 128, 0.1);
}


.stats-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.stats-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  display: flex;
  align-items: center;
  gap: 8px;
}

.stats-icon {
  color: var(--primary-color);
}

.traffic-content {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.traffic-stats {
  width: 100%;
}

.traffic-row {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: 12px;
  margin-bottom: 2px;
}

.traffic-item {
  padding: 10px 14px;
  border-radius: 10px;
}

.traffic-item:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  border-color: rgba(128, 128, 128, 0.15);
}

.traffic-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--n-text-color-2);
  margin-bottom: 4px;
}

.traffic-icon {
  flex-shrink: 0;
}

.upload-icon {
  color: var(--success-color);
}

.download-icon {
  color: var(--primary-color);
}

.cloud-up-icon {
  color: #2a9d8f;
}

.cloud-down-icon {
  color: #4c6ef5;
}

.memory-icon {
  color: var(--error-color);
}

.connections-icon {
  color: #9c27b0;
}

.traffic-value {
  font-size: 18px;
  font-weight: 600;
  color: var(--n-text-color-1);
  padding-left: 4px;
}

.chart-container {
  width: 100%;
  height: 160px;
  border-radius: 12px;
  overflow: hidden;
  background-color: rgba(var(--n-body-color-rgb), 0.4);
  margin-top: 4px;
  backdrop-filter: blur(4px);
  border: 1px solid rgba(128, 128, 128, 0.08);
}

.traffic-chart {
  width: 100%;
  height: 100%;
}

@media (max-width: 1200px) {
  .traffic-row {
    grid-template-columns: repeat(3, 1fr);
  }
  
  .traffic-item {
    padding: 10px 14px;
  }
}

@media (max-width: 768px) {
  .chart-container {
    height: 150px;
  }

  .traffic-value {
    font-size: 16px;
  }

  .traffic-row {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 480px) {
  .traffic-row {
    grid-template-columns: 1fr;
  }
  
  .traffic-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
  }
  
  .traffic-item:nth-child(n) {
    grid-column: auto;
  }
  
  .traffic-value {
    font-size: 15px;
  }
}
</style> 