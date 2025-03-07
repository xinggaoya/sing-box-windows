<template>
  <div class="proxy-container">
    <!-- 顶部标题卡片 -->
    <n-card class="proxy-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <n-h3 class="card-title">
              <n-icon size="24" class="card-icon">
                <swap-horizontal-outline />
              </n-icon>
              代理设置
            </n-h3>
          </div>
          <n-tooltip trigger="hover" placement="top">
            <template #trigger>
              <n-button
                quaternary
                circle
                size="medium"
                @click="init"
                :loading="isLoading"
                class="refresh-button"
              >
                <template #icon>
                  <n-icon><refresh-outline /></n-icon>
                </template>
              </n-button>
            </template>
            刷新代理列表
          </n-tooltip>
        </div>
      </template>
    </n-card>

    <!-- 代理列表卡片 -->
    <n-spin :show="isLoading">
      <n-card class="proxy-list-card" :bordered="false">
        <n-tabs type="segment" animated class="proxy-tabs">
          <n-tab-pane v-for="(item, index) in list" :key="index" :name="index" :tab="item.name">
            <div class="proxy-group-info">
              <n-space align="center" :size="12">
                <n-tag :bordered="false" type="success" size="medium" class="proxy-tag">
                  当前节点：{{ item.now }}
                </n-tag>
                <n-tag :bordered="false" type="info" size="medium" class="proxy-tag">
                  {{ item.all.length }} 个节点
                </n-tag>
              </n-space>
            </div>

            <n-grid :x-gap="16" :y-gap="16" :cols="gridCols" responsive="screen">
              <n-grid-item v-for="(proxy, i) in item.all" :key="i">
                <n-card
                  :class="{
                    'proxy-node-card': true,
                    'proxy-node-card-active': item.now === proxy.name,
                  }"
                  :bordered="false"
                  hoverable
                >
                  <n-space vertical :size="14">
                    <n-flex justify="space-between" align="center">
                      <div class="proxy-name-container">
                        <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                          {{ proxy.name }}
                        </n-ellipsis>
                      </div>
                      <n-tag
                        :type="getDelayType(proxy.delay)"
                        size="small"
                        :bordered="false"
                        round
                        class="delay-tag"
                      >
                        {{ proxy.delay === '0ms' ? '未测速' : proxy.delay }}
                      </n-tag>
                    </n-flex>

                    <n-flex justify="space-between" align="center">
                      <n-button
                        @click="changeProxy(item.name, proxy.name, index)"
                        :type="item.now === proxy.name ? 'default' : 'primary'"
                        size="small"
                        :disabled="item.now === proxy.name"
                        :ghost="item.now !== proxy.name"
                        class="proxy-button"
                      >
                        <template #icon>
                          <n-icon>
                            <checkmark-circle-outline v-if="item.now === proxy.name" />
                            <swap-horizontal-outline v-else />
                          </n-icon>
                        </template>
                        {{ item.now === proxy.name ? '使用中' : '切换' }}
                      </n-button>
                      <n-button
                        @click="getDelay(proxy.name, index, i)"
                        :loading="proxy.isTestingDelay"
                        secondary
                        size="small"
                        type="info"
                        ghost
                        class="proxy-button"
                      >
                        <template #icon>
                          <n-icon><speedometer-outline /></n-icon>
                        </template>
                        测速
                      </n-button>
                    </n-flex>
                  </n-space>
                </n-card>
              </n-grid-item>
            </n-grid>

            <n-empty v-if="!item.all.length" description="暂无代理节点" class="empty-container" />
          </n-tab-pane>
        </n-tabs>
      </n-card>
    </n-spin>
  </div>
</template>

<script lang="ts" setup>
import { onMounted, ref, computed } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  CheckmarkCircleOutline,
  SwapHorizontalOutline,
  SpeedometerOutline,
} from '@vicons/ionicons5'
import { useWindowSize } from '@vueuse/core'

interface ProxyItem {
  name: string
  delay: string
  isTestingDelay?: boolean
}

interface ProxyGroup {
  name: string
  now: string
  delay: string
  all: ProxyItem[]
}

interface ProxyData {
  type: string
  now: string
  all: string[]
}

interface Proxies {
  proxies: Record<string, ProxyData>
}

const list = ref<ProxyGroup[]>([])
const message = useMessage()
const isLoading = ref(false)
const { width } = useWindowSize()

// 根据窗口宽度调整网格列数
const gridCols = computed(() => {
  if (width.value < 640) return 1
  if (width.value < 960) return 2
  if (width.value < 1280) return 3
  return 4
})

onMounted(() => {
  init()
})

const init = async () => {
  isLoading.value = true
  try {
    const res = await fetch('http://127.0.0.1:12081/proxies', {
      headers: {
        'Content-Type': 'application/json',
      },
      method: 'GET',
    })
    const data = (await res.json()) as Proxies
    const info: ProxyGroup[] = []

    Object.entries(data.proxies).forEach(([key, item]: [string, ProxyData]) => {
      if (item.type === 'Selector' || item.type === 'URLTest') {
        info.push({
          name: key,
          now: item.now,
          delay: '0ms',
          all: item.all.map((name: string) => ({
            name,
            delay: '0ms',
            isTestingDelay: false,
          })),
        })
      }
    })

    list.value = info
    if (info.length > 0) {
      message.success('代理列表加载成功')
    }
  } catch (error) {
    message.error('获取代理列表失败')
  } finally {
    isLoading.value = false
  }
}

const getDelayType = (delay: string) => {
  const ms = parseInt(delay)
  if (ms === 0) return 'default'
  if (ms < 100) return 'success'
  if (ms < 200) return 'info'
  if (ms < 300) return 'warning'
  return 'error'
}

const getDelay = async (
  name: string,
  index: number,
  i: number,
  server: string = 'https://www.gstatic.com/generate_204',
) => {
  if (!list.value[index]?.all[i]) return
  list.value[index].all[i].isTestingDelay = true

  try {
    const res = await fetch(
      `http://127.0.0.1:12081/proxies/${name}/delay?url=${server}&timeout=5000`,
      {
        headers: {
          'Content-Type': 'application/json',
        },
        method: 'GET',
      },
    )
    const data = await res.json()
    list.value[index].all[i].delay = data.delay + 'ms'
  } catch (error) {
    message.error('测速失败')
  } finally {
    if (list.value[index]?.all[i]) {
      list.value[index].all[i].isTestingDelay = false
    }
  }
}

const changeProxy = async (type: string, name: string, index: number) => {
  try {
    const res = await fetch(`http://127.0.0.1:12081/proxies/${type}`, {
      method: 'PUT',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ name }),
    })

    if (res.status === 400) {
      message.error('切换失败，可能是不可切换的代理组')
      return
    }

    if (res.status === 204) {
      message.success('切换成功')
      await init()
    }
  } catch (error) {
    message.error('切换失败')
  }
}
</script>

<style scoped>
.proxy-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 8px;
  animation: slide-up 0.4s ease;
}

.proxy-card {
  margin-bottom: 16px;
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.proxy-card:hover,
.proxy-list-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.proxy-list-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-weight: 600;
}

.card-icon {
  color: var(--primary-color);
}

.refresh-button {
  transition: all 0.3s ease;
}

.refresh-button:hover:not(:disabled) {
  transform: translateY(-2px);
  color: var(--primary-color);
  background-color: rgba(64, 128, 255, 0.1);
}

.proxy-group-info {
  margin-bottom: 20px;
  padding: 0 4px;
}

.proxy-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
}

.proxy-node-card {
  transition: all 0.3s ease;
  border-radius: 12px;
  border-left: 3px solid transparent;
}

.proxy-node-card:hover {
  transform: translateY(-3px);
  box-shadow: var(--shadow-medium);
}

.proxy-node-card-active {
  border-left: 3px solid var(--success-color);
}

.proxy-name-container {
  font-weight: 500;
  flex: 1;
  overflow: hidden;
  color: var(--n-text-color-1);
}

.delay-tag {
  font-weight: 500;
  transition: all 0.3s ease;
}

.proxy-button {
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.25s ease;
}

.proxy-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

:deep(.dark) .proxy-button:hover:not(:disabled) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.25);
}

.empty-container {
  margin: 60px 0;
  opacity: 0.8;
}

:deep(.proxy-tabs .n-tabs-tab) {
  padding: 8px 16px;
  font-weight: 500;
  transition: all 0.3s ease;
}

:deep(.proxy-tabs .n-tabs-tab.n-tabs-tab--active) {
  font-weight: 600;
}

:deep(.proxy-tabs .n-tabs-tab-wrapper) {
  padding: 4px;
}

:deep(.n-tabs .n-tab-pane) {
  padding: 16px 0;
}

:deep(.n-card.proxy-node-card) {
  background-color: var(--card-color);
}

:deep(.n-card.proxy-node-card:hover) {
  background-color: var(--card-color-hover);
}
</style>
