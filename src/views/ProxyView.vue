<template>
  <n-space vertical>
    <n-card>
      <n-space align="center">
        <n-h3 style="margin: 0">代理设置</n-h3>
        <n-button
          quaternary
          circle
          type="primary"
          @click="init"
          :loading="isLoading"
        >
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
        </n-button>
      </n-space>
    </n-card>

    <n-spin :show="isLoading">
      <n-card>
        <n-tabs type="segment">
          <n-tab-pane
            v-for="(item, index) in list"
            :key="index"
            :name="index"
            :tab="item.name"
          >
            <n-grid :x-gap="12" :y-gap="12" :cols="4">
              <n-grid-item v-for="(proxy, i) in item.all" :key="i">
                <n-card
                  :class="{ 'proxy-card-active': item.now === proxy.name }"
                  class="proxy-card"
                  hoverable
                >
                  <n-space vertical :size="12">
                    <n-flex justify="space-between" align="center">
                      <n-ellipsis style="max-width: 150px">
                        {{ proxy.name }}
                      </n-ellipsis>
                      <n-tag :type="getDelayType(proxy.delay)" size="small">
                        {{ proxy.delay }}
                      </n-tag>
                    </n-flex>
                    
                    <n-flex justify="space-between">
                      <n-button
                        @click="changeProxy(item.name, proxy.name, index)"
                        :type="item.now === proxy.name ? 'default' : 'primary'"
                        size="small"
                        :disabled="item.now === proxy.name"
                      >
                        <template #icon>
                          <n-icon>
                            <CheckmarkCircleOutline v-if="item.now === proxy.name" />
                            <SwapHorizontalOutline v-else />
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
                      >
                        <template #icon>
                          <n-icon><SpeedometerOutline /></n-icon>
                        </template>
                        测速
                      </n-button>
                    </n-flex>
                  </n-space>
                </n-card>
              </n-grid-item>
            </n-grid>
          </n-tab-pane>
        </n-tabs>
      </n-card>
    </n-spin>
  </n-space>
</template>

<script lang="ts" setup>
import { onMounted, ref } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  CheckmarkCircleOutline,
  SwapHorizontalOutline,
  SpeedometerOutline
} from '@vicons/ionicons5'
import TunModeSwitch from '../components/TunModeSwitch.vue'

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
    const data = await res.json()
    const info: ProxyGroup[] = []
    
    Object.entries(data.proxies).forEach(([key, item]: [string, any]) => {
      if (item.type === 'Selector' || item.type === 'URLTest') {
        info.push({
          name: key,
          now: item.now,
          delay: '0ms',
          all: item.all.map((name: string) => ({
            name,
            delay: '0ms',
            isTestingDelay: false
          }))
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
  server: string = 'https://www.gstatic.com/generate_204'
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
      }
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
.proxy-card {
  transition: all 0.3s ease;
}

.proxy-card-active {
  border: 2px solid var(--primary-color);
}

:deep(.n-card.proxy-card:hover) {
  transform: translateY(-2px);
}
</style>
