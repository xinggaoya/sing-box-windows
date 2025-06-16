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
import { ref, defineProps, onMounted, onUnmounted, watch, computed } from 'vue'
import { useThemeVars } from 'naive-ui'
import { formatBandwidth } from '@/utils/index'
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

// 计算最大值，使用动态变化的最大值，确保图表更平滑
const maxValue = computed(() => {
  const uploadMax = Math.max(...uploadData.value, 0.1)
  const downloadMax = Math.max(...downloadData.value, 0.1)
  const currentMax = Math.max(uploadMax, downloadMax)
  // 使用平滑变化的最大值，避免图表剧烈跳动
  return Math.max(currentMax * 1.2, 0.1) // 留出20%的空间，确保最小值不为0
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

  // 减小内边距使图表更紧凑，但增加左侧内边距确保文本不被截断
  const padding = {
    top: 24 * dpr,
    right: 15 * dpr,
    bottom: 28 * dpr,
    left: 65 * dpr,
  }

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
  ctx.font = `${11 * dpr}px sans-serif`
  ctx.textAlign = 'right'
  ctx.textBaseline = 'middle'
  ctx.fillStyle = textColor

  // 绘制Y轴标签和网格线 - 精简网格线数量提高可读性
  const yAxisSteps = 4
  for (let i = 0; i <= yAxisSteps; i++) {
    const y = padding.top + chartHeight - (i / yAxisSteps) * chartHeight
    const value = (i / yAxisSteps) * maxValue.value

    // 将值从B/s转换为适当单位的字符串
    const formattedValue = formatBandwidth(value * 1024 * 1024)

    // 使用更紧凑的标签格式
    let speedLabel = `${formattedValue}/s`

    // 简化大单位的显示
    speedLabel = speedLabel
      .replace(' MB/s', 'MB/s')
      .replace(' KB/s', 'KB/s')
      .replace(' B/s', 'B/s')
      .replace(' GB/s', 'GB/s')

    // 绘制网格线 - 使用虚线样式并降低不透明度提高视觉效果
    ctx.beginPath()
    ctx.strokeStyle = `${gridColor}30` // 增加透明度
    ctx.lineWidth = 0.5 * dpr
    ctx.setLineDash([4 * dpr, 4 * dpr]) // 设置虚线样式
    ctx.moveTo(padding.left, y)
    ctx.lineTo(padding.left + chartWidth, y)
    ctx.stroke()
    ctx.setLineDash([]) // 重置虚线样式

    // 绘制Y轴标签
    ctx.fillText(speedLabel, padding.left - 10 * dpr, y)
  }

  // 绘制X轴 - 使用实线样式，稍微加粗提高可读性
  ctx.beginPath()
  ctx.strokeStyle = `${gridColor}80`
  ctx.lineWidth = 0.8 * dpr
  ctx.moveTo(padding.left, padding.top + chartHeight)
  ctx.lineTo(padding.left + chartWidth, padding.top + chartHeight)
  ctx.stroke()

  // 只绘制较少的X轴标签以减少视觉复杂度
  const labelInterval = Math.ceil(MAX_DATA_POINTS / 5) // 减少标签数量
  ctx.font = `${10 * dpr}px sans-serif`
  ctx.textAlign = 'center'
  ctx.textBaseline = 'top'
  for (let i = MAX_DATA_POINTS - 1; i >= 0; i -= labelInterval) {
    if (timeLabels.value[i]) {
      const x = padding.left + (i / (MAX_DATA_POINTS - 1)) * chartWidth
      ctx.fillText(timeLabels.value[i], x, padding.top + chartHeight + 10 * dpr)
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

  // 绘制曲线渐变区域
  ctx.beginPath()

  // 移动到第一个点的位置
  const firstX = padding.left
  const firstY = padding.top + chartHeight - (data[0] / max) * chartHeight
  ctx.moveTo(firstX, firstY)

  // 使用贝塞尔曲线绘制平滑曲线
  for (let i = 1; i < data.length; i++) {
    const x = padding.left + (i / (MAX_DATA_POINTS - 1)) * chartWidth
    const y = padding.top + chartHeight - (data[i] / max) * chartHeight

    const prevX = padding.left + ((i - 1) / (MAX_DATA_POINTS - 1)) * chartWidth
    const prevY = padding.top + chartHeight - (data[i - 1] / max) * chartHeight

    // 控制点 - 使曲线更平滑
    const cpX1 = prevX + (x - prevX) / 3
    const cpX2 = prevX + ((x - prevX) * 2) / 3

    ctx.bezierCurveTo(cpX1, prevY, cpX2, y, x, y)
  }

  // 完成渐变区域路径
  ctx.lineTo(padding.left + chartWidth, padding.top + chartHeight)
  ctx.lineTo(padding.left, padding.top + chartHeight)
  ctx.closePath()

  // 绘制填充渐变
  const gradient = ctx.createLinearGradient(0, padding.top, 0, padding.top + chartHeight)
  gradient.addColorStop(0, `${color}30`) // 顶部适当透明
  gradient.addColorStop(1, `${color}05`) // 底部更透明

  ctx.fillStyle = gradient
  ctx.fill()

  // 绘制曲线
  ctx.beginPath()
  ctx.strokeStyle = color
  ctx.lineWidth = 2.5 * dpr
  ctx.lineJoin = 'round'
  ctx.lineCap = 'round'

  ctx.moveTo(firstX, firstY)

  // 再次绘制贝塞尔曲线（只绘制线条）
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

  // 绘制结束点高亮
  const lastIndex = data.length - 1
  const lastX = padding.left + (lastIndex / (MAX_DATA_POINTS - 1)) * chartWidth
  const lastY = padding.top + chartHeight - (data[lastIndex] / max) * chartHeight

  // 外圈光晕
  ctx.beginPath()
  ctx.fillStyle = `${color}30`
  ctx.arc(lastX, lastY, 6 * dpr, 0, Math.PI * 2)
  ctx.fill()

  // 内圈实心点
  ctx.beginPath()
  ctx.fillStyle = color
  ctx.arc(lastX, lastY, 3 * dpr, 0, Math.PI * 2)
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

  // 使用requestAnimationFrame优化性能
  requestAnimationFrame(() => {
    drawChart()
  })
}

let updateTimer: number | null = null

// 启动定时更新
const startUpdates = () => {
  if (updateTimer !== null) {
    clearInterval(updateTimer)
  }

  // 使用1秒的更新频率，足够流畅同时不会造成性能问题
  updateTimer = setInterval(() => {
    updateData()
  }, 1000) as unknown as number
}

// 重置并刷新图表
const resetAndRefresh = () => {
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
  requestAnimationFrame(() => {
    drawChart()
    // 确保定时更新器在运行
    if (updateTimer === null) {
      startUpdates()
    }
  })
}

// 组件挂载时初始化
onMounted(() => {
  // 使用requestAnimationFrame确保DOM已完全渲染
  requestAnimationFrame(() => {
    initChart()
    startUpdates()
  })

  // 添加窗口大小变化事件监听器，使用防抖处理
  let resizeTimeout: number | null = null
  const handleResize = () => {
    if (resizeTimeout) {
      clearTimeout(resizeTimeout)
    }

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

// 组件卸载时清理
onUnmounted(() => {
  // 清理更新定时器
  if (updateTimer !== null) {
    clearInterval(updateTimer)
    updateTimer = null
  }

  // 清理窗口事件监听器
  window.removeEventListener('resize', handleResize)

  // 清理所有画布引用，帮助垃圾回收
  if (chartCanvas.value) {
    const ctx = chartCanvas.value.getContext('2d')
    if (ctx) {
      ctx.clearRect(0, 0, chartCanvas.value.width, chartCanvas.value.height)
    }
  }

  // 清空数据数组
  uploadData.value = []
  downloadData.value = []
  timeLabels.value = []
})

// 监听主题变化
watch(themeVars, () => {
  drawChart()
})

// 处理窗口大小变化，使用防抖优化
const handleResize = () => {
  if (chartContainer.value && chartCanvas.value) {
    const { width, height } = chartContainer.value.getBoundingClientRect()
    const dpr = window.devicePixelRatio || 1

    chartCanvas.value.width = width * dpr
    chartCanvas.value.height = height * dpr
    chartCanvas.value.style.width = `${width}px`
    chartCanvas.value.style.height = `${height}px`

    drawChart()
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
  filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.05));
}

.chart-legend {
  position: absolute;
  top: 12px;
  right: 16px;
  display: flex;
  gap: 16px;
  z-index: 1;
  background-color: rgba(var(--n-body-color-rgb), 0.6);
  border-radius: 10px;
  padding: 8px 14px;
  backdrop-filter: blur(8px);
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  border: 1px solid rgba(128, 128, 128, 0.1);
  transition: all 0.25s ease;
  transform: translateZ(0);
}

.chart-legend:hover {
  background-color: rgba(var(--n-body-color-rgb), 0.8);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.15);
  transform: translateY(-2px);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--n-text-color-1);
}

.legend-color {
  width: 12px;
  height: 12px;
  border-radius: 3px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.upload .legend-color {
  background-color: #18a058;
}

.download .legend-color {
  background-color: #2080f0;
}

/* 深色模式样式会通过CSS变量自动应用，删除手动适配代码 */

@media (max-width: 768px) {
  .chart-legend {
    top: auto;
    bottom: 12px;
    right: 12px;
    padding: 6px 12px;
  }
}
</style>
