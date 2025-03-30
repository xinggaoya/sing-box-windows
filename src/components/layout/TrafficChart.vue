<template>
  <div class="traffic-chart-container" ref="chartContainer">
    <canvas ref="chartCanvas" class="chart-canvas"></canvas>
    <div class="chart-labels">
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
import { ref, defineProps, onMounted, onUnmounted, watch, computed } from 'vue'
import { useThemeVars } from 'naive-ui'
import { formatBandwidth } from '@/utils/index' // 导入formatBandwidth函数
import { useI18n } from 'vue-i18n'

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

// 图表配置
const MAX_DATA_POINTS = 60 // 最大数据点数量
const uploadData = ref<number[]>([]) // 上传速度数据
const downloadData = ref<number[]>([]) // 下载速度数据
const timeLabels = ref<string[]>([]) // 时间标签

// 计算最大值
const maxValue = computed(() => {
  const uploadMax = Math.max(...uploadData.value, 0.1)
  const downloadMax = Math.max(...downloadData.value, 0.1)
  return Math.max(uploadMax, downloadMax) * 1.2 // 留出20%的空间
})

// 初始化图表
const initChart = () => {
  if (!chartCanvas.value || !chartContainer.value) return

  const canvas = chartCanvas.value
  const container = chartContainer.value
  const { width, height } = container.getBoundingClientRect()

  // 设置canvas大小，考虑设备像素比以保持清晰度
  const dpr = window.devicePixelRatio || 1
  canvas.width = width * dpr
  canvas.height = height * dpr
  canvas.style.width = `${width}px`
  canvas.style.height = `${height}px`

  // 初始清空数据数组
  uploadData.value = Array(MAX_DATA_POINTS).fill(0)
  downloadData.value = Array(MAX_DATA_POINTS).fill(0)
  timeLabels.value = Array(MAX_DATA_POINTS).fill('')

  // 立即绘制空图表
  drawChart()
}

// 绘制图表
const drawChart = () => {
  if (!chartCanvas.value) return

  const canvas = chartCanvas.value
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const dpr = window.devicePixelRatio || 1
  const width = canvas.width
  const height = canvas.height
  const padding = { top: 30 * dpr, right: 20 * dpr, bottom: 40 * dpr, left: 80 * dpr } // 增加左侧padding

  // 清除画布
  ctx.clearRect(0, 0, width, height)

  // 绘制区域
  const chartWidth = width - padding.left - padding.right
  const chartHeight = height - padding.top - padding.bottom

  // 获取当前主题颜色
  const bgColor = themeVars.value.bodyColor
  const textColor = themeVars.value.textColor2
  const gridColor = themeVars.value.borderColor
  const uploadColor = '#18A058' // 绿色
  const downloadColor = '#2080F0' // 蓝色

  // 设置字体
  ctx.font = `${12 * dpr}px sans-serif`
  ctx.textAlign = 'right'
  ctx.textBaseline = 'middle'
  ctx.fillStyle = textColor

  // 绘制Y轴标签和网格线
  const yAxisSteps = 5
  for (let i = 0; i <= yAxisSteps; i++) {
    const y = padding.top + chartHeight - (i / yAxisSteps) * chartHeight
    const value = (i / yAxisSteps) * maxValue.value

    // 将值从B/s转换为适当单位的字符串（传入参数需要转换为KB）
    const formattedValue = formatBandwidth(value * 1024 * 1024) // 转换为KB后传给formatBandwidth

    // 添加"/s"表示速率单位
    const speedLabel = `${formattedValue}/s`

    // 绘制网格线
    ctx.beginPath()
    ctx.strokeStyle = gridColor
    ctx.lineWidth = 1
    ctx.moveTo(padding.left, y)
    ctx.lineTo(padding.left + chartWidth, y)
    ctx.stroke()

    // 绘制Y轴标签
    ctx.fillText(speedLabel, padding.left - 10 * dpr, y)
  }

  // 绘制X轴
  ctx.beginPath()
  ctx.strokeStyle = gridColor
  ctx.lineWidth = 1
  ctx.moveTo(padding.left, padding.top + chartHeight)
  ctx.lineTo(padding.left + chartWidth, padding.top + chartHeight)
  ctx.stroke()

  // 绘制X轴标签（只显示部分时间点以避免拥挤）
  const labelInterval = Math.ceil(MAX_DATA_POINTS / 6) // 显示约6个标签
  for (let i = 0; i < MAX_DATA_POINTS; i += labelInterval) {
    if (timeLabels.value[i]) {
      const x = padding.left + (i / (MAX_DATA_POINTS - 1)) * chartWidth
      ctx.textAlign = 'center'
      ctx.fillText(timeLabels.value[i], x, padding.top + chartHeight + 20 * dpr)
    }
  }

  // 绘制上传速度曲线
  if (uploadData.value.some((v) => v > 0)) {
    drawCurve(ctx, uploadData.value, uploadColor, padding, chartWidth, chartHeight, dpr)
  }

  // 绘制下载速度曲线
  if (downloadData.value.some((v) => v > 0)) {
    drawCurve(ctx, downloadData.value, downloadColor, padding, chartWidth, chartHeight, dpr)
  }
}

// 绘制曲线函数
const drawCurve = (
  ctx: CanvasRenderingContext2D,
  data: number[],
  color: string,
  padding: { top: number; right: number; bottom: number; left: number },
  chartWidth: number,
  chartHeight: number,
  dpr: number,
) => {
  const max = maxValue.value || 0.1 // 避免除以零

  // 绘制曲线
  ctx.beginPath()
  ctx.strokeStyle = color
  ctx.lineWidth = 2 * dpr
  ctx.lineJoin = 'round'

  data.forEach((value, index) => {
    const x = padding.left + (index / (MAX_DATA_POINTS - 1)) * chartWidth
    const y = padding.top + chartHeight - (value / max) * chartHeight

    if (index === 0) {
      ctx.moveTo(x, y)
    } else {
      ctx.lineTo(x, y)
    }
  })

  ctx.stroke()

  // 绘制渐变区域
  ctx.lineTo(padding.left + chartWidth, padding.top + chartHeight)
  ctx.lineTo(padding.left, padding.top + chartHeight)
  ctx.closePath()

  const gradient = ctx.createLinearGradient(0, padding.top, 0, padding.top + chartHeight)
  gradient.addColorStop(0, `${color}40`) // 40 为透明度的十六进制
  gradient.addColorStop(1, `${color}05`) // 05 为透明度的十六进制

  ctx.fillStyle = gradient
  ctx.fill()
}

// 更新数据
const updateData = () => {
  // 直接使用字节单位保存数据，方便后续处理
  const uploadSpeed = props.uploadSpeed
  const downloadSpeed = props.downloadSpeed

  // 移除最旧的数据
  uploadData.value.shift()
  downloadData.value.shift()
  timeLabels.value.shift()

  // 添加新数据（存储MB值保持现有逻辑一致）
  uploadData.value.push(uploadSpeed / 1024 / 1024)
  downloadData.value.push(downloadSpeed / 1024 / 1024)

  const now = new Date()
  const timeStr = `${now.getMinutes().toString().padStart(2, '0')}:${now.getSeconds().toString().padStart(2, '0')}`
  timeLabels.value.push(timeStr)

  // 重绘图表
  drawChart()
}

let updateTimer: number | null = null

// 启动定时更新
const startUpdates = () => {
  if (updateTimer !== null) {
    clearInterval(updateTimer)
  }

  console.log(t('chart.startUpdateTimer'))
  updateTimer = setInterval(() => {
    updateData()
  }, 1000) as unknown as number
}

// 重置并刷新图表
const resetAndRefresh = () => {
  console.log(t('chart.resetRefresh'))

  // 清除所有数据
  uploadData.value = Array(MAX_DATA_POINTS).fill(0)
  downloadData.value = Array(MAX_DATA_POINTS).fill(0)
  timeLabels.value = Array(MAX_DATA_POINTS).fill('')

  // 确保重新获取容器大小
  if (chartContainer.value && chartCanvas.value) {
    const { width, height } = chartContainer.value.getBoundingClientRect()
    const dpr = window.devicePixelRatio || 1

    // 重置画布大小强制重新渲染
    chartCanvas.value.width = width * dpr
    chartCanvas.value.height = height * dpr
    chartCanvas.value.style.width = `${width}px`
    chartCanvas.value.style.height = `${height}px`
  }

  // 重新初始化图表
  setTimeout(() => {
    initChart()
    // 立即更新一次数据以显示当前状态
    updateData()
    // 确保定时更新器在运行
    if (updateTimer === null) {
      startUpdates()
    }
  }, 50)
}

// 组件挂载时初始化
onMounted(() => {
  // 延迟执行以确保DOM已完全渲染
  setTimeout(() => {
    initChart()
    startUpdates()
  }, 100)

  // 添加窗口大小变化事件监听器
  window.addEventListener('resize', handleResize)
})

// 组件卸载时清理
onUnmounted(() => {
  if (updateTimer !== null) {
    clearInterval(updateTimer)
    updateTimer = null
  }

  window.removeEventListener('resize', handleResize)
})

// 监听主题变化
watch(themeVars, () => {
  console.log(t('chart.themeChanged'))
  drawChart()
})

// 处理窗口大小变化
const handleResize = () => {
  if (chartContainer.value && chartCanvas.value) {
    console.log(t('chart.windowResized'))
    initChart()
  }
}
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

.chart-labels {
  position: absolute;
  top: 10px;
  right: 20px;
  display: flex;
  gap: 20px;
  z-index: 1;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--n-text-color-1);
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 3px;
}

.upload .legend-color {
  background-color: #18a058;
}

.download .legend-color {
  background-color: #2080f0;
}
</style>
