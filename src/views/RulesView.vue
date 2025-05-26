<template>
  <div class="rules-container">
    <!-- 顶部标题区 -->
    <div class="header-section">
      <div class="header-content">
        <div class="header-left">
          <div class="title-wrapper">
            <div class="title-icon">
              <n-icon size="20">
                <search-outline />
              </n-icon>
            </div>
            <h2 class="page-title">{{ t('rules.title') }}</h2>
            <n-badge
              :value="rules.length"
              :max="999"
              show-zero
              type="info"
              size="small"
              class="count-badge"
            />
          </div>
        </div>
        <div class="header-actions">
          <n-button
            @click="fetchRules"
            :loading="loading"
            size="small"
            type="primary"
            class="refresh-btn"
            round
          >
            <template #icon>
              <n-icon>
                <refresh-outline />
              </n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
        </div>
      </div>
    </div>

    <!-- 规则内容区 -->
    <div class="rules-content">
      <n-spin :show="loading">
        <!-- 搜索筛选区 -->
        <div class="search-section">
          <div class="search-filters">
            <div class="search-input-wrapper">
              <n-input
                v-model:value="searchQuery"
                placeholder="搜索规则..."
                clearable
                class="search-input"
              >
                <template #prefix>
                  <n-icon size="16">
                    <search-outline />
                  </n-icon>
                </template>
              </n-input>
            </div>

            <div class="filter-group">
              <n-select
                v-model:value="typeFilter"
                :options="typeOptions"
                placeholder="按类型筛选"
                clearable
                class="filter-select"
                size="small"
              />
              <n-select
                v-model:value="proxyFilter"
                :options="proxyOptions"
                placeholder="按代理筛选"
                clearable
                class="filter-select"
                size="small"
              />
            </div>
          </div>

          <!-- 统计信息 -->
          <div class="stats-section">
            <div class="stats-item">
              <span class="stats-label">总规则</span>
              <span class="stats-value">{{ rules.length }}</span>
            </div>
            <div v-if="searchQuery || typeFilter || proxyFilter" class="stats-item filtered">
              <span class="stats-label">匹配</span>
              <span class="stats-value">{{ filteredRules.length }}</span>
            </div>
          </div>
        </div>

        <!-- 规则表格区 -->
        <div class="table-section">
          <div v-if="filteredRules.length > 0" class="table-wrapper">
            <n-data-table
              :columns="columns"
              :data="filteredRules"
              :pagination="pagination"
              :bordered="false"
              :max-height="600"
              striped
              class="rules-table"
            />
          </div>

          <!-- 空状态 -->
          <div v-else class="empty-state">
            <div class="empty-content">
              <div class="empty-icon">
                <n-icon size="48" color="#d1d5db">
                  <search-outline />
                </n-icon>
              </div>
              <h3 class="empty-title">
                {{ searchQuery || typeFilter || proxyFilter ? '没有匹配的规则' : '暂无规则数据' }}
              </h3>
              <p class="empty-description">
                {{
                  searchQuery || typeFilter || proxyFilter
                    ? '请尝试调整搜索条件或筛选条件'
                    : '点击刷新按钮获取规则数据'
                }}
              </p>
              <n-button
                v-if="!searchQuery && !typeFilter && !proxyFilter"
                @click="fetchRules"
                type="primary"
                size="medium"
                class="empty-action-btn"
                round
              >
                <template #icon>
                  <n-icon>
                    <refresh-outline />
                  </n-icon>
                </template>
                获取规则
              </n-button>
            </div>
          </div>
        </div>
      </n-spin>
    </div>
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
  return rules.value.filter((rule) => {
    const matchesSearch =
      !searchQuery.value ||
      rule.payload.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      rule.proxy.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      rule.type.toLowerCase().includes(searchQuery.value.toLowerCase())

    const matchesType = !typeFilter.value || rule.type === typeFilter.value

    const matchesProxy =
      !proxyFilter.value ||
      (proxyFilter.value === 'direct' && rule.proxy === t('rules.directConnect')) ||
      (proxyFilter.value === 'reject' && rule.proxy === 'reject') ||
      (proxyFilter.value !== 'direct' &&
        proxyFilter.value !== 'reject' &&
        rule.proxy.includes(proxyFilter.value))

    return matchesSearch && matchesType && matchesProxy
  })
})

// 类型过滤选项
const typeOptions = computed(() => {
  const types = [...new Set(rules.value.map((rule) => rule.type))]
  return types.map((type) => ({ label: type, value: type }))
})

// 代理过滤选项
const proxyOptions = computed(() => {
  const proxies = new Set<string>()

  // 添加常见特殊代理
  proxies.add('direct')
  proxies.add('reject')

  // 添加其他代理
  rules.value.forEach((rule) => {
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
      .filter((proxy) => proxy !== 'direct' && proxy !== 'reject')
      .map((proxy) => ({ label: proxy, value: proxy })),
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
      if (
        searchQuery.value &&
        row.payload.toLowerCase().includes(searchQuery.value.toLowerCase())
      ) {
        const index = row.payload.toLowerCase().indexOf(searchQuery.value.toLowerCase())
        const beforeMatch = row.payload.substring(0, index)
        const match = row.payload.substring(index, index + searchQuery.value.length)
        const afterMatch = row.payload.substring(index + searchQuery.value.length)

        return h('div', {}, [
          beforeMatch,
          h(
            'span',
            { style: { backgroundColor: 'rgba(var(--primary-color), 0.1)', fontWeight: 'bold' } },
            match,
          ),
          afterMatch,
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
  min-height: calc(100vh - 120px);
  padding: 20px;
  background: var(--n-color-embedded);
  animation: fadeIn 0.4s ease-out;
}

/* 顶部标题区 */
.header-section {
  margin-bottom: 20px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: var(--n-card-color);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  border: 1px solid var(--n-border-color);
  box-shadow: var(--n-box-shadow-1);
  transition: all 0.3s ease;
}

.header-content:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
}

.title-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, #4080ff, #2266dd);
  border-radius: 10px;
  color: white;
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.3);
}

.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color-1);
  background: linear-gradient(135deg, #4080ff, #2266dd);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.count-badge {
  margin-left: 8px;
}

.refresh-btn {
  height: 36px;
  font-weight: 500;
  transition: all 0.3s ease;
}

.refresh-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(64, 128, 255, 0.2);
}

/* 规则内容区 */
.rules-content {
  background: var(--n-card-color);
  backdrop-filter: blur(12px);
  border-radius: 20px;
  border: 1px solid var(--n-border-color);
  padding: 20px;
  box-shadow: var(--n-box-shadow-2);
  transition: all 0.3s ease;
}

.rules-content:hover {
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.08);
}

/* 搜索筛选区 */
.search-section {
  margin-bottom: 20px;
  padding: 16px 20px;
  background: var(--n-color-embedded-popover);
  border-radius: 16px;
  border: 1px solid var(--n-border-color);
}

.search-filters {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  align-items: center;
}

.search-input-wrapper {
  flex: 1;
  min-width: 280px;
  max-width: 400px;
}

.search-input {
  width: 100%;
  border-radius: 12px;
  transition: all 0.3s ease;
}

.search-input:hover {
  box-shadow: 0 2px 8px rgba(64, 128, 255, 0.1);
}

.search-input:focus-within {
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.2);
}

.filter-group {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-select {
  min-width: 160px;
  border-radius: 10px;
  transition: all 0.3s ease;
}

.filter-select:hover {
  box-shadow: 0 2px 8px rgba(64, 128, 255, 0.1);
}

/* 统计信息区 */
.stats-section {
  display: flex;
  gap: 16px;
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid rgba(64, 128, 255, 0.1);
}

.stats-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: var(--n-card-color);
  border-radius: 12px;
  border: 1px solid var(--n-border-color);
  box-shadow: var(--n-box-shadow-1);
  transition: all 0.2s ease;
}

.stats-item:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.stats-item.filtered {
  background: linear-gradient(135deg, rgba(0, 180, 42, 0.1), rgba(0, 154, 26, 0.1));
  border-color: rgba(0, 180, 42, 0.2);
}

.stats-label {
  font-size: 11px;
  color: var(--text-color-3);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.stats-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-color-1);
}

.stats-item.filtered .stats-value {
  color: #009a1a;
}

/* 表格区域 */
.table-section {
  margin-top: 0;
}

.table-wrapper {
  background: var(--n-card-color);
  border-radius: 16px;
  border: 1px solid var(--n-border-color);
  overflow: hidden;
  box-shadow: var(--n-box-shadow-1);
}

.rules-table {
  border-radius: 16px;
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  padding: 40px 20px;
  background: var(--n-card-color);
  border-radius: 16px;
  border: 1px solid var(--n-border-color);
  box-shadow: var(--n-box-shadow-1);
}

.empty-content {
  text-align: center;
  max-width: 300px;
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.6;
}

.empty-title {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color-1);
}

.empty-description {
  margin: 0 0 20px 0;
  font-size: 14px;
  color: var(--text-color-3);
  line-height: 1.5;
}

.empty-action-btn {
  height: 40px;
  font-weight: 500;
  transition: all 0.3s ease;
}

.empty-action-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(64, 128, 255, 0.2);
}

/* 暗黑模式样式会通过CSS变量自动应用 */

/* 动画效果 */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .rules-container {
    padding: 12px;
  }

  .header-content {
    padding: 12px 16px;
    flex-direction: column;
    gap: 12px;
  }

  .title-wrapper {
    justify-content: center;
  }

  .rules-content {
    padding: 16px;
  }

  .search-section {
    padding: 12px 16px;
  }

  .search-filters {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .search-input-wrapper {
    min-width: auto;
    max-width: none;
  }

  .filter-group {
    flex-direction: column;
    gap: 8px;
  }

  .filter-select {
    min-width: auto;
  }

  .stats-section {
    flex-direction: column;
    gap: 8px;
  }

  .stats-item {
    justify-content: space-between;
  }
}

@media (max-width: 480px) {
  .search-section {
    padding: 10px 12px;
  }

  .stats-section {
    margin-top: 12px;
    padding-top: 12px;
  }

  .empty-state {
    padding: 30px 15px;
    min-height: 250px;
  }

  .empty-title {
    font-size: 16px;
  }

  .empty-description {
    font-size: 13px;
  }
}

/* Naive UI 组件样式覆盖 */
:deep(.n-data-table) {
  background: transparent;
}

:deep(.n-data-table-thead) {
  background: var(--n-color-embedded);
}

:deep(.n-data-table-th) {
  background: var(--n-color-embedded);
  border-bottom: 1px solid var(--n-border-color);
  font-weight: 600;
}

:deep(.n-data-table-td) {
  border-bottom: 1px solid var(--n-border-color);
}

:deep(.n-data-table-tr:hover .n-data-table-td) {
  background: var(--n-color-embedded-popover);
}

/* 删除暗色模式自定义样式，现在通过CSS变量自动应用 */

:deep(.n-badge) {
  --n-font-size: 10px;
}

:deep(.n-spin-container) {
  min-height: 200px;
}
</style>
