<template>
  <div class="rules-container">
    <n-card class="rules-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <h2>{{ $t('rules.title') }}</h2>
          <n-space>
            <n-button type="primary" @click="fetchRules" :loading="loading">
              <template #icon>
                <n-icon><refresh-outline /></n-icon>
              </template>
              {{ $t('rules.refresh') }}
            </n-button>
          </n-space>
        </div>
      </template>

      <n-spin :show="loading">
        <div v-if="rules.length > 0" class="rules-list">
          <n-data-table
            :columns="columns"
            :data="rules"
            :pagination="pagination"
            :bordered="false"
            :max-height="600"
            striped
          />
        </div>
        <n-empty v-else :description="$t('rules.no_data')" />
      </n-spin>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue'
import { useMessage, NTag, DataTableColumns } from 'naive-ui'
import { RefreshOutline } from '@vicons/ionicons5'
import { tauriApi } from '@/services/tauri-api'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const message = useMessage()
const loading = ref(false)

interface Rule {
  type: string
  payload: string
  proxy: string
}

const rules = ref<Rule[]>([])

// Определяем колонки таблицы
const columns: DataTableColumns<Rule> = [
  {
    title: t('rules.columns.type'),
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
    title: t('rules.columns.payload'),
    key: 'payload',
    render(row: Rule) {
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
      return row.payload
    },
  },
  {
    title: t('rules.columns.proxy'),
    key: 'proxy',
    width: 180,
    render(row: Rule) {
      let proxyName = row.proxy
      if (proxyName.startsWith('route(') && proxyName.endsWith(')')) {
        proxyName = proxyName.substring(6, proxyName.length - 1)
      }
      let color: 'default' | 'error' | 'primary' | 'success' | 'info' | 'warning' = 'default'
      if (proxyName === 'reject') {
        color = 'error'
      } else if (proxyName === t('rules.proxy_names.direct')) {
        color = 'success'
      } else if (proxyName === 'hijack-dns' || proxyName === 'sniff') {
        color = 'info'
      } else if (proxyName.includes('Google') || proxyName.includes('YouTube')) {
        color = 'warning'
      } else if (proxyName.includes(t('rules.proxy_names.manual')) || proxyName.includes(t('rules.proxy_names.auto'))) {
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

// Параметры пагинации
const pagination = {
  pageSize: 10,
}

// Получение списка правил
const fetchRules = async () => {
  loading.value = true
  try {
    const response = await tauriApi.proxy.getRules()
    console.log('规则数据:', response)
    if (response && response.rules) {
      rules.value = response.rules
      message.success(`成功获取 ${rules.value.length} 条规则`)
    } else {
      message.error('获取规则失败：返回数据格式错误')
    }
  } catch (error) {
    console.error('获取规则失败:', error)
    message.error(`获取规则失败: ${error}`)
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

.rules-list {
  margin-top: 12px;
}
</style>
