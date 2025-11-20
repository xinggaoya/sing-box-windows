<template>
  <div class="traffic-chart-container" ref="chartContainer">
    <canvas ref="chartCanvas" class="chart-canvas"></canvas>
    <div class="chart-legend">
      <div class="legend-item upload">
        <div class="legend-color"></div>
        <span>{{ t('home.traffic.uploadSpeed') }}</span>
      </div>
      <div class="legend-item download">
        <div class="legend-color"></div>
        <span>{{ t('home.traffic.downloadSpeed') }}</span>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, defineProps, onMounted, onUnmounted, onBeforeUnmount, watch, computed } from 'vue'
import { useThemeVars } from 'naive-ui'
import { formatBandwidth } from '@/utils/index'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'

defineOptions({
  name: 'TrafficChart',
})

const props = defineProps({
  uploadSpeed: {
    type: Number,
    default: 0,
  },
  downloadSpeed: {
    type: Number,
    default: 0,
  },
})

const chartContainer = ref<HTMLDivElement | null>(null)
const chartCanvas = ref<HTMLCanvasElement | null>(null)
const themeVars = useThemeVars()
const { t } = useI18n()

// Chart Config
const MAX_DATA_POINTS = 60
const uploadData = ref<number[]>([])
const downloadData = ref<number[]>([])
const timeLabels = ref<string[]>([])

// Colors
const getColors = () => {
  const style = getComputedStyle(document.body)
  return {
    upload: style.getPropertyValue('--success-color').trim() || '#10b981',
    download: style.getPropertyValue('--primary-color').trim() || '#6366f1',
    grid: style.getPropertyValue('--border-color').trim() || 'rgba(128, 128, 128, 0.2)',
    text: style.getPropertyValue('--text-secondary').trim() || '#666',
  }
}

const maxValue = computed(() => {
  const uploadMax = Math.max(...uploadData.value, 0.1)
  const downloadMax = Math.max(...downloadData.value, 0.1)
  const currentMax = Math.max(uploadMax, downloadMax)
  return Math.max(currentMax * 1.2, 0.1)
})

const initChart = () => {
  if (!chartCanvas.value || !chartContainer.value) return

  const canvas = chartCanvas.value
  const container = chartContainer.value
  const { width, height } = container.getBoundingClientRect()
  const dpr = window.devicePixelRatio || 1

  canvas.width = width * dpr
  canvas.height = height * dpr
  canvas.style.width = `${width}px`
  canvas.style.height = `${height}px`

  uploadData.value = Array(MAX_DATA_POINTS).fill(0)
  downloadData.value = Array(MAX_DATA_POINTS).fill(0)
  timeLabels.value = Array(MAX_DATA_POINTS).fill('')

  drawChart()
}

const drawChart = () => {
  if (!chartCanvas.value) return

  const canvas = chartCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const width = canvas.width
  const height = canvas.height
  const colors = getColors()

  const padding = {
    top: 24 * dpr,
    right: 15 * dpr,
    bottom: 28 * dpr,
    left: 65 * dpr,
  }

  ctx.clearRect(0, 0, width, height)

  const chartWidth = width - padding.left - padding.right
  const chartHeight = height - padding.top - padding.bottom

  ctx.font = `${11 * dpr}px sans-serif`
  ctx.textAlign = 'right'
  ctx.textBaseline = 'middle'
  ctx.fillStyle = colors.text

  // Y Axis
  const yAxisSteps = 4
  for (let i = 0; i <= yAxisSteps; i++) {
    const y = padding.top + chartHeight - (i / yAxisSteps) * chartHeight
    const value = (i / yAxisSteps) * maxValue.value
    const formattedValue = formatBandwidth(value * 1024 * 1024)
    const speedLabel = `${formattedValue}/s`
      .replace(' MB/s', 'MB/s')
      .replace(' KB/s', 'KB/s')
      .replace(' B/s', 'B/s')
      .replace(' GB/s', 'GB/s')

    ctx.beginPath()
    ctx.strokeStyle = colors.grid
    ctx.lineWidth = 0.5 * dpr
    ctx.setLineDash([4 * dpr, 4 * dpr])
    ctx.moveTo(padding.left, y)
    ctx.lineTo(padding.left + chartWidth, y)
    ctx.stroke()
    ctx.setLineDash([])

    ctx.fillText(speedLabel, padding.left - 10 * dpr, y)
  }

  // X Axis
  ctx.beginPath()
  ctx.strokeStyle = colors.grid
  ctx.lineWidth = 0.8 * dpr
  ctx.moveTo(padding.left, padding.top + chartHeight)
  ctx.lineTo(padding.left + chartWidth, padding.top + chartHeight)
  ctx.stroke()

  const labelInterval = Math.ceil(MAX_DATA_POINTS / 5)
  ctx.font = `${10 * dpr}px sans-serif`
  ctx.textAlign = 'center'
  ctx.textBaseline = 'top'
  for (let i = MAX_DATA_POINTS - 1; i >= 0; i -= labelInterval) {
    if (timeLabels.value[i]) {
      const x = padding.left + (i / (MAX_DATA_POINTS - 1)) * chartWidth
      ctx.fillText(timeLabels.value[i], x, padding.top + chartHeight + 10 * dpr)
    }
  }

  if (uploadData.value.some((v) => v > 0)) {
    drawCurve(ctx, uploadData.value, colors.upload, padding, chartWidth, chartHeight, dpr)
  }

  if (downloadData.value.some((v) => v > 0)) {
    drawCurve(ctx, downloadData.value, colors.download, padding, chartWidth, chartHeight, dpr)
  }
}

const drawCurve = (
  ctx: CanvasRenderingContext2D,
  data: number[],
  color: string,
  padding: { top: number; right: number; bottom: number; left: number },
  chartWidth: number,
  chartHeight: number,
  dpr: number,
) => {
  const max = maxValue.value || 0.1

  ctx.beginPath()
  const firstX = padding.left
  const firstY = padding.top + chartHeight - (data[0] / max) * chartHeight
  ctx.moveTo(firstX, firstY)

  for (let i = 1; i < data.length; i++) {
    const x = padding.left + (i / (MAX_DATA_POINTS - 1)) * chartWidth
    const y = padding.top + chartHeight - (data[i] / max) * chartHeight
    const prevX = padding.left + ((i - 1) / (MAX_DATA_POINTS - 1)) * chartWidth
    const prevY = padding.top + chartHeight - (data[i - 1] / max) * chartHeight
    const cpX1 = prevX + (x - prevX) / 3
    const cpX2 = prevX + ((x - prevX) * 2) / 3
    ctx.bezierCurveTo(cpX1, prevY, cpX2, y, x, y)
  }

  ctx.lineTo(padding.left + chartWidth, padding.top + chartHeight)
  ctx.lineTo(padding.left, padding.top + chartHeight)
  ctx.closePath()

  const gradient = ctx.createLinearGradient(0, padding.top, 0, padding.top + chartHeight)
  gradient.addColorStop(0, `${color}40`)
  gradient.addColorStop(1, `${color}05`)
  ctx.fillStyle = gradient
  ctx.fill()

  ctx.beginPath()
  ctx.strokeStyle = color
  ctx.lineWidth = 2.5 * dpr
  ctx.lineJoin = 'round'
  ctx.lineCap = 'round'
  ctx.moveTo(firstX, firstY)

  for (let i = 1; i < data.length; i++) {
    const x = padding.left + (i / (MAX_DATA_POINTS - 1)) * chartWidth
    const y = padding.top + chartHeight - (data[i] / max) * chartHeight
    const prevX = padding.left + ((i - 1) / (MAX_DATA_POINTS - 1)) * chartWidth
    const prevY = padding.top + chartHeight - (data[i - 1] / max) * chartHeight
    const cpX1 = prevX + (x - prevX) / 3
    const cpX2 = prevX + ((x - prevX) * 2) / 3
    ctx.bezierCurveTo(cpX1, prevY, cpX2, y, x, y)
  }
  ctx.stroke()

  const lastIndex = data.length - 1
  const lastX = padding.left + (lastIndex / (MAX_DATA_POINTS - 1)) * chartWidth
  const lastY = padding.top + chartHeight - (data[lastIndex] / max) * chartHeight

  ctx.beginPath()
  ctx.fillStyle = `${color}40`
  ctx.arc(lastX, lastY, 6 * dpr, 0, Math.PI * 2)
  ctx.fill()

  ctx.beginPath()
  ctx.fillStyle = color
  ctx.arc(lastX, lastY, 3 * dpr, 0, Math.PI * 2)
  ctx.fill()
}

const updateData = () => {
  const uploadSpeed = props.uploadSpeed
  const downloadSpeed = props.downloadSpeed

  uploadData.value.shift()
  downloadData.value.shift()
  timeLabels.value.shift()

  uploadData.value.push(uploadSpeed / 1024 / 1024)
  downloadData.value.push(downloadSpeed / 1024 / 1024)

  const now = new Date()
  const timeStr = `${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`
  timeLabels.value.push(timeStr)

  requestAnimationFrame(() => {
    drawChart()
  })
}

let updateTimer: number | null = null

const startUpdates = () => {
  if (updateTimer !== null) clearInterval(updateTimer)
  updateTimer = setInterval(() => updateData(), 1000) as unknown as number
}

onMounted(() => {
  requestAnimationFrame(() => {
    initChart()
    startUpdates()
  })

  let resizeTimeout: number | null = null
  const handleResize = () => {
    if (resizeTimeout) clearTimeout(resizeTimeout)
    resizeTimeout = window.setTimeout(() => {
      if (chartContainer.value && chartCanvas.value) {
        const { width, height } = chartContainer.value.getBoundingClientRect()
        const dpr = window.devicePixelRatio || 1
        chartCanvas.value.width = width * dpr
        chartCanvas.value.height = height * dpr
        chartCanvas.value.style.width = `${width}px`
        chartCanvas.value.style.height = `${height}px`
        drawChart()
      }
      resizeTimeout = null
    }, 100) as unknown as number
  }
  window.addEventListener('resize', handleResize)
})

onUnmounted(() => {
  if (updateTimer !== null) clearInterval(updateTimer)
  // Cleanup logic...
})

watch(themeVars, () => drawChart())
</script>

<style scoped>
.traffic-chart-container {
  position: relative;
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
}

.chart-canvas {
  flex-grow: 1;
  width: 100%;
  height: 100%;
}

.chart-legend {
  position: absolute;
  top: 12px;
  right: 16px;
  display: flex;
  gap: 16px;
  z-index: 1;
  background-color: var(--glass-bg);
  border-radius: 10px;
  padding: 8px 14px;
  backdrop-filter: blur(8px);
  box-shadow: var(--glass-shadow);
  border: 1px solid var(--glass-border);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.legend-color {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.upload .legend-color {
  background-color: var(--success-color);
  box-shadow: 0 0 8px var(--success-color);
}

.download .legend-color {
  background-color: var(--primary-color);
  box-shadow: 0 0 8px var(--primary-color);
}
</style>
