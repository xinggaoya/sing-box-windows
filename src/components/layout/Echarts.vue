<template>
  <v-chart class="chart" :option="option" />
</template>

<script lang="ts" setup>
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import {
  TitleComponent,
  TooltipComponent,
  GridComponent,
  ToolboxComponent,
  DataZoomComponent,
} from 'echarts/components'
import VChart, { THEME_KEY } from 'vue-echarts'
import { ref, provide, watch, defineProps, onMounted } from 'vue'

defineOptions({
  name: 'SpeedChart',
})

use([
  CanvasRenderer,
  LineChart,
  TitleComponent,
  TooltipComponent,
  GridComponent,
  ToolboxComponent,
  DataZoomComponent,
])

provide(THEME_KEY, 'light')

onMounted(() => {
  startTheTask()
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

const MAX_DATA_LENGTH = 20 // 设置最大数据长度

const option = ref<any>({
  tooltip: {
    trigger: 'axis',
    formatter: function (params: any) {
      let tooltipText = ''
      params.forEach((param: any) => {
        tooltipText += `${param.seriesName}: ${param.value.toFixed(2)} MB/s<br/>`
      })
      return tooltipText
    },
  },
  legend: {
    data: ['上传速度', '下载速度'],
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    containLabel: true,
  },
  toolbox: {
    feature: {
      saveAsImage: {},
    },
  },
  xAxis: {
    type: 'category',
    boundaryGap: false,
    data: [],
  },
  yAxis: {
    type: 'value',
    name: '速度 (MB/s)',
  },
  series: [
    {
      name: '上传速度',
      type: 'line',
      data: [],
    },
    {
      name: '下载速度',
      type: 'line',
      data: [],
    },
  ],
})

// 安装定时器
const startTheTask = () => {
  setInterval(() => {
    // 将传入的毫秒值转换为 MB/s
    const mbUploadSpeed = props.uploadSpeed / 1024 / 1024
    const mbDownloadSpeed = props.downloadSpeed / 1024 / 1024

    // 如果数据超出最大长度，则删除最旧的数据
    if (option.value.series[0].data.length >= MAX_DATA_LENGTH) {
      option.value.series[0].data.shift()
      option.value.series[1].data.shift()
      option.value.xAxis.data.shift()
    }

    // 更新数据
    option.value.series[0].data.push(mbUploadSpeed)
    option.value.series[1].data.push(mbDownloadSpeed)

    // 更新 X 轴标签
    const date = new Date()
    option.value.xAxis.data.push(date.getMinutes() + ':' + date.getSeconds())
  }, 1000)
}
</script>

<style scoped>
.chart {
  height: 100%;
  width: 100%;
}
</style>
