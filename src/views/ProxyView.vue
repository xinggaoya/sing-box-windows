<template>
  <div class="proxy-container">
    <!-- 顶部标题卡片 -->
    <n-card class="proxy-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <swap-horizontal-outline />
            </n-icon>
            代理设置
          </n-h3>
          <n-button quaternary circle type="primary" @click="init" :loading="isLoading">
            <template #icon>
              <n-icon><refresh-outline /></n-icon>
            </template>
          </n-button>
        </div>
      </template>
    </n-card>

    <!-- 代理列表卡片 -->
    <n-spin :show="isLoading">
      <n-card class="proxy-card" :bordered="false">
        <n-tabs type="segment" animated>
          <n-tab-pane v-for="(item, index) in list" :key="index" :name="index" :tab="item.name">
            <div class="proxy-group-info">
              <n-space align="center">
                <n-tag :bordered="false" type="success" size="small"
                  >当前节点：{{ item.now }}</n-tag
                >
                <n-tag :bordered="false" type="info" size="small"
                  >{{ item.all.length }} 个节点</n-tag
                >
              </n-space>
            </div>

            <n-grid :x-gap="12" :y-gap="12" :cols="4">
              <n-grid-item v-for="(proxy, i) in item.all" :key="i">
                <n-card
                  :class="{
                    'proxy-node-card': true,
                    'proxy-node-card-active': item.now === proxy.name,
                  }"
                  :bordered="false"
                  hoverable
                >
                  <n-space vertical :size="12">
                    <n-flex justify="space-between" align="center">
                      <n-ellipsis style="max-width: 150px" :tooltip="{ width: 'trigger' }">
                        {{ proxy.name }}
                      </n-ellipsis>
                      <n-tag :type="getDelayType(proxy.delay)" size="small" :bordered="false" round>
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

            <n-empty v-if="!item.all.length" description="暂无代理节点" />
          </n-tab-pane>
        </n-tabs>
      </n-card>
    </n-spin>
  </div>
</template>

<script lang="ts" setup>
import { onMounted, ref } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  CheckmarkCircleOutline,
  SwapHorizontalOutline,
  SpeedometerOutline,
} from '@vicons/ionicons5'

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

onMounted(() => {
  init()
})

const init = async () => {
  isLoading.value = true
  try {
    const res = await fetch('http://127.0.0.1:9090/proxies', {
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
      `http://127.0.0.1:9090/proxies/${name}/delay?url=${server}&timeout=5000`,
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
    const res = await fetch(`http://127.0.0.1:9090/proxies/${type}`, {
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
  padding: 16px;
}

.proxy-card {
  margin-bottom: 16px;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.proxy-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.card-title {
  display: flex;
  align-items: center;
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.card-icon {
  margin-right: 8px;
  color: var(--primary-color);
}

.proxy-group-info {
  margin-bottom: 16px;
}

.proxy-node-card {
  transition: all 0.3s ease;
  border-radius: 8px;
}

.proxy-node-card:hover {
  transform: translateY(-2px);
}

.proxy-node-card-active {
  background-color: rgba(var(--primary-color-rgb), 0.1);
  border: 1px solid var(--primary-color);
}

:deep(.n-tabs .n-tab-pane) {
  padding: 12px 0;
}

:deep(.n-card.proxy-node-card) {
  background-color: var(--card-color);
}

:deep(.n-card.proxy-node-card:hover) {
  background-color: var(--card-color-hover);
}

:deep(.n-button.n-button--ghost) {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

:deep(.n-button.n-button--ghost:hover) {
  background-color: var(--primary-color);
  color: white;
}
</style>
