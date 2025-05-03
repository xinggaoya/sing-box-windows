<template>
  <div class="rules-container">
    <n-card class="rules-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <h2>{{ t('rules.title') }}</h2>
          <n-space>
            <n-button type="primary" @click="fetchRules" :loading="loading">
              <template #icon>
                <n-icon><refresh-outline /></n-icon>
              </template>
              {{ t('common.refresh') }}
            </n-button>
          </n-space>
        </div>
      </template>

      <n-spin :show="loading">
        <div class="search-filter-bar">
          <n-input
            v-model:value="searchQuery"
            placeholder="搜索规则..."
            clearable
            :style="{ width: '300px' }"
          >
            <template #prefix>
              <n-icon><search-outline /></n-icon>
            </template>
          </n-input>
          <n-select
            v-model:value="typeFilter"
            :options="typeOptions"
            placeholder="按类型筛选"
            clearable
            :style="{ width: '180px' }"
          />
          <n-select
            v-model:value="proxyFilter"
            :options="proxyOptions"
            placeholder="按代理筛选"
            clearable
            :style="{ width: '180px' }"
          />
        </div>

        <div v-if="filteredRules.length > 0" class="rules-list">
          <n-data-table
            :columns="columns"
            :data="filteredRules"
            :pagination="pagination"
            :bordered="false"
            :max-height="600"
            striped
          />
        </div>
        <n-empty v-else :description="searchQuery || typeFilter || proxyFilter ? t('rules.noMatchingRules') : t('rules.noRules')" />
        
        <div class="stats-bar">
          <n-tag type="info" size="small">{{ t('rules.totalRules') }}: {{ rules.length }}</n-tag>
          <n-tag v-if="searchQuery || typeFilter || proxyFilter" type="success" size="small">
            {{ t('rules.matchingRules') }}: {{ filteredRules.length }}
          </n-tag>
        </div>
      </n-spin>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed } from 'vue'
import { useMessage, NTag, DataTableColumns, SelectOption } from 'naive-ui'
import { RefreshOutline, SearchOutline } from '@vicons/ionicons5'
import { tauriApi } from '@/services/tauri-api'
import { useI18n } from 'vue-i18n'

const message = useMessage()
const loading = ref(false)
const { t } = useI18n()

interface Rule {
  type: string
  payload: string
  proxy: string
}

const rules = ref<Rule[]>([])
const searchQuery = ref('')
const typeFilter = ref(null)
const proxyFilter = ref(null)

// 计算筛选后的规则
const filteredRules = computed(() => {
  return rules.value.filter(rule => {
    const matchesSearch = !searchQuery.value || 
      rule.payload.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      rule.proxy.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      rule.type.toLowerCase().includes(searchQuery.value.toLowerCase())
    
    const matchesType = !typeFilter.value || rule.type === typeFilter.value
    
    const matchesProxy = !proxyFilter.value || 
      (proxyFilter.value === 'direct' && rule.proxy === t('rules.directConnect')) ||
      (proxyFilter.value === 'reject' && rule.proxy === 'reject') ||
      (proxyFilter.value !== 'direct' && proxyFilter.value !== 'reject' && rule.proxy.includes(proxyFilter.value))
    
    return matchesSearch && matchesType && matchesProxy
  })
})

// 类型过滤选项
const typeOptions = computed(() => {
  const types = [...new Set(rules.value.map(rule => rule.type))]
  return types.map(type => ({ label: type, value: type }))
})

// 代理过滤选项
const proxyOptions = computed(() => {
  const proxies = new Set<string>()
  
  // 添加常见特殊代理
  proxies.add('direct')
  proxies.add('reject')
  
  // 添加其他代理
  rules.value.forEach(rule => {
    let proxyName = rule.proxy
    if (proxyName.startsWith('route(') && proxyName.endsWith(')')) {
      proxyName = proxyName.substring(6, proxyName.length - 1)
    }
    
    if (proxyName !== t('rules.directConnect') && proxyName !== 'reject') {
      proxies.add(proxyName)
    }
  })
  
  return [
    { label: t('rules.directConnect'), value: 'direct' },
    { label: '拦截', value: 'reject' },
    ...Array.from(proxies)
      .filter(proxy => proxy !== 'direct' && proxy !== 'reject')
      .map(proxy => ({ label: proxy, value: proxy }))
  ]
})

// 定义表格列
const columns: DataTableColumns<Rule> = [
  {
    title: t('rules.type'),
    key: 'type',
    width: 120,
    render(row: Rule) {
      const type = row.type
      let color: 'default' | 'error' | 'primary' | 'success' | 'info' | 'warning' = 'default'

      if (type === 'logical') {
        color = 'warning'
      } else if (type === 'default') {
        color = 'info'
      }

      return h(
        NTag,
        {
          style: {
            marginRight: '0',
          },
          type: color,
          bordered: false,
        },
        { default: () => type },
      )
    },
  },
  {
    title: t('rules.content'),
    key: 'payload',
    render(row: Rule) {
      // 判断是否包含规则集
      if (row.payload.includes('rule_set=')) {
        const parts = row.payload.split('rule_set=')[1].replace(/\[|\]/g, '').split(' ')
        return h('div', {}, [
          parts.map((part: string) => {
            return h(
              NTag,
              {
                style: {
                  marginRight: '8px',
                  marginBottom: '4px',
                },
                type: 'success',
                bordered: false,
                size: 'small',
              },
              { default: () => part },
            )
          }),
        ])
      }
      
      // 高亮搜索关键字
      if (searchQuery.value && row.payload.toLowerCase().includes(searchQuery.value.toLowerCase())) {
        const index = row.payload.toLowerCase().indexOf(searchQuery.value.toLowerCase())
        const beforeMatch = row.payload.substring(0, index)
        const match = row.payload.substring(index, index + searchQuery.value.length)
        const afterMatch = row.payload.substring(index + searchQuery.value.length)
        
        return h('div', {}, [
          beforeMatch,
          h('span', { style: { backgroundColor: 'rgba(var(--primary-color), 0.1)', fontWeight: 'bold' } }, match),
          afterMatch
        ])
      }
      
      return row.payload
    },
  },
  {
    title: t('rules.targetProxy'),
    key: 'proxy',
    width: 180,
    render(row: Rule) {
      // 提取代理名称
      let proxyName = row.proxy
      if (proxyName.startsWith('route(') && proxyName.endsWith(')')) {
        proxyName = proxyName.substring(6, proxyName.length - 1)
      }

      let color: 'default' | 'error' | 'primary' | 'success' | 'info' | 'warning' = 'default'
      if (proxyName === 'reject') {
        color = 'error'
      } else if (proxyName === t('rules.directConnect')) {
        color = 'success'
      } else if (proxyName === 'hijack-dns' || proxyName === 'sniff') {
        color = 'info'
      } else if (proxyName.includes('Google') || proxyName.includes('YouTube')) {
        color = 'warning'
      } else if (
        proxyName.includes(t('rules.manualSwitch')) ||
        proxyName.includes(t('rules.autoSelect'))
      ) {
        color = 'primary'
      }

      return h(
        NTag,
        {
          style: {
            marginRight: '0',
          },
          type: color,
          bordered: false,
        },
        { default: () => proxyName },
      )
    },
  },
]

// 分页设置
const pagination = {
  pageSize: 15,
}

// 获取规则列表
const fetchRules = async () => {
  loading.value = true
  try {
    const response = await tauriApi.proxy.getRules()
    console.log(t('rules.logData'), response)
    if (response && response.rules) {
      rules.value = response.rules
      message.success(t('rules.fetchSuccess', { count: rules.value.length }))
    } else {
      message.error(t('rules.fetchErrorFormat'))
    }
  } catch (error) {
    console.error(t('rules.fetchError'), error)
    message.error(`${t('rules.fetchError')}: ${error}`)
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchRules()
})
</script>

<style scoped>
.rules-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 12px 8px;
}

.rules-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.05);
}

.rules-card :deep(.n-card__content) {
  padding: 16px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header h2 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 500;
}

.search-filter-bar {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.rules-list {
  margin-top: 12px;
}

.stats-bar {
  display: flex;
  gap: 8px;
  margin-top: 12px;
  padding: 8px 0;
}
</style>
