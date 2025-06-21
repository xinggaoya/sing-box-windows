<template>
  <div class="rules-view">
    <!-- 英雄式页面头部 -->
    <div class="hero-header">
      <div class="hero-content">
        <div class="hero-icon">
          <n-icon size="48">
            <search-outline />
          </n-icon>
        </div>
        <div class="hero-text">
          <h1 class="hero-title">{{ t('rules.title') }}</h1>
          <p class="hero-subtitle">
            {{ rules.length }} {{ t('rules.totalRules') }} •
            {{ t('rules.subtitle') }}
          </p>
        </div>
        <div class="hero-action">
          <n-button
            @click="fetchRules"
            :loading="loading"
            size="large"
            type="primary"
            round
            class="hero-btn"
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

    <!-- 搜索筛选卡片 -->
    <div class="filter-card">
      <div class="filter-header">
        <h3 class="filter-title">{{ t('rules.searchAndFilter') }}</h3>
        <div class="filter-stats">
          <n-tag type="info" size="small" round>
            {{ t('rules.totalRulesLabel', { count: rules.length }) }}
          </n-tag>
          <n-tag v-if="searchQuery || typeFilter || proxyFilter" type="success" size="small" round>
            {{ t('rules.matchingLabel', { count: filteredRules.length }) }}
          </n-tag>
        </div>
      </div>

      <div class="filter-controls">
        <n-input
          v-model:value="searchQuery"
          :placeholder="t('rules.searchPlaceholder')"
          clearable
          size="large"
          class="search-input"
        >
          <template #prefix>
            <n-icon size="16">
              <search-outline />
            </n-icon>
          </template>
        </n-input>

        <div class="filter-selects">
          <n-select
            v-model:value="typeFilter"
            :options="typeOptions"
            :placeholder="t('rules.filterByType')"
            clearable
            size="large"
            class="filter-select"
          />
          <n-select
            v-model:value="proxyFilter"
            :options="proxyOptions"
            :placeholder="t('rules.filterByProxy')"
            clearable
            size="large"
            class="filter-select"
          />
        </div>
      </div>
    </div>

    <!-- 规则表格卡片 -->
    <div class="table-card">
      <n-spin :show="loading">
        <div v-if="filteredRules.length > 0" class="table-container">
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
          <div class="empty-icon">
            <n-icon size="64">
              <search-outline />
            </n-icon>
          </div>
          <h3 class="empty-title">
            {{
              searchQuery || typeFilter || proxyFilter
                ? t('rules.noMatchingRulesFound')
                : t('rules.noRulesData')
            }}
          </h3>
          <p class="empty-description">
            {{
              searchQuery || typeFilter || proxyFilter
                ? t('rules.adjustSearchConditions')
                : t('rules.clickRefreshToGetRules')
            }}
          </p>
          <n-button
            v-if="!searchQuery && !typeFilter && !proxyFilter"
            @click="fetchRules"
            type="primary"
            size="large"
            round
            class="empty-action"
          >
            <template #icon>
              <n-icon>
                <refresh-outline />
              </n-icon>
            </template>
            {{ t('rules.getRules') }}
          </n-button>
        </div>
      </n-spin>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed, nextTick } from 'vue'
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

interface RulesResponse {
  rules: Rule[]
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
  // 防止同时多次调用
  if (loading.value) return

  loading.value = true
  try {
    // 添加超时机制防止请求挂起
    const timeout = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('请求超时')), 10000),
    )

    const response = (await Promise.race([tauriApi.proxy.getRules(), timeout])) as RulesResponse

    console.log(t('rules.logData'), response)
    if (response && response.rules && Array.isArray(response.rules)) {
      rules.value = response.rules
      message.success(t('rules.fetchSuccess', { count: rules.value.length }))
    } else {
      console.warn('getRules返回数据格式异常:', response)
      rules.value = []
      message.warning(t('rules.fetchErrorFormat'))
    }
  } catch (error) {
    console.error(t('rules.fetchError'), error)
    rules.value = []
    // 区分不同类型的错误
    if (error instanceof Error && error.message === '请求超时') {
      message.error('获取规则超时，请检查内核是否正常运行')
    } else {
      message.error(`${t('rules.fetchError')}: ${error}`)
    }
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  // 使用nextTick确保组件完全挂载后再获取数据
  nextTick(() => {
    // 延迟一点时间再调用，确保其他依赖都已初始化
    setTimeout(() => {
      fetchRules()
    }, 100)
  })
})
</script>

<style scoped>
.rules-view {
  min-height: 100vh;
  background: var(--n-color-embedded);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 1400px;
  margin: 0 auto;
  animation: fadeIn 0.4s ease-out;
}

/* 英雄式页面头部 */
.hero-header {
  background: var(--n-card-color);
  border-radius: 20px;
  padding: 24px 32px;
  box-shadow: var(--n-box-shadow-2);
  border: 1px solid var(--n-border-color);
  position: relative;
  overflow: hidden;
}

.hero-header::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(90deg, #4080ff 0%, #2266dd 50%, #909399 100%);
  border-radius: 20px 20px 0 0;
}

.hero-content {
  display: flex;
  align-items: center;
  gap: 24px;
}

.hero-icon {
  width: 72px;
  height: 72px;
  border-radius: 18px;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 12px 32px rgba(64, 128, 255, 0.3);
}

.hero-text {
  flex: 1;
}

.hero-title {
  font-size: 2rem;
  font-weight: 800;
  margin: 0 0 8px 0;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 100%);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  line-height: 1.2;
}

.hero-subtitle {
  font-size: 1.1rem;
  color: var(--n-text-color-3);
  margin: 0;
  line-height: 1.5;
  font-weight: 500;
}

.hero-action {
  flex-shrink: 0;
}

.hero-btn {
  height: 48px;
  padding: 0 24px;
  font-size: 1rem;
  font-weight: 600;
  border-radius: 24px;
  box-shadow:
    0 8px 32px rgba(64, 128, 255, 0.25),
    0 2px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.hero-btn:hover {
  transform: translateY(-3px);
  box-shadow:
    0 16px 48px rgba(64, 128, 255, 0.4),
    0 4px 12px rgba(0, 0, 0, 0.15);
}

/* 搜索筛选卡片 */
.filter-card {
  background: var(--n-card-color);
  border-radius: 16px;
  padding: 24px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.filter-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.filter-title {
  font-size: 1.25rem;
  font-weight: 700;
  margin: 0;
  color: var(--n-text-color-1);
}

.filter-stats {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-controls {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
  align-items: stretch;
}

.search-input {
  flex: 1;
  min-width: 300px;
}

.search-input :deep(.n-input) {
  border-radius: 12px;
  border: 2px solid var(--n-border-color);
  transition: all 0.3s ease;
}

.search-input :deep(.n-input:hover) {
  border-color: rgba(64, 128, 255, 0.3);
}

.search-input :deep(.n-input.n-input--focus) {
  border-color: #4080ff;
  box-shadow: 0 0 0 3px rgba(64, 128, 255, 0.1);
}

.filter-selects {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-select {
  min-width: 180px;
}

.filter-select :deep(.n-base-selection) {
  border-radius: 12px;
  border: 2px solid var(--n-border-color);
  transition: all 0.3s ease;
}

.filter-select :deep(.n-base-selection:hover) {
  border-color: rgba(64, 128, 255, 0.3);
}

.filter-select :deep(.n-base-selection.n-base-selection--focus) {
  border-color: #4080ff;
  box-shadow: 0 0 0 3px rgba(64, 128, 255, 0.1);
}

/* 规则表格卡片 */
.table-card {
  background: var(--n-card-color);
  border-radius: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  overflow: hidden;
}

.table-container {
  padding: 16px;
  min-height: 400px;
}

.rules-table {
  border-radius: 0;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  padding: 48px 24px;
  text-align: center;
}

.empty-icon {
  color: var(--n-text-color-disabled);
  margin-bottom: 24px;
}

.empty-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: var(--n-text-color-1);
  margin: 0 0 12px 0;
}

.empty-description {
  font-size: 1rem;
  color: var(--n-text-color-3);
  margin: 0 0 24px 0;
  line-height: 1.6;
  max-width: 400px;
}

.empty-action {
  height: 48px;
  padding: 0 24px;
  font-size: 1rem;
  font-weight: 600;
  border-radius: 24px;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.empty-action:hover {
  transform: translateY(-3px);
  box-shadow: 0 12px 32px rgba(64, 128, 255, 0.3);
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
@media (max-width: 1024px) {
  .rules-view {
    padding: 16px;
    gap: 16px;
  }
}

@media (max-width: 768px) {
  .rules-view {
    padding: 12px;
    gap: 16px;
  }

  .hero-header {
    padding: 20px;
    border-radius: 16px;
  }

  .hero-content {
    flex-direction: column;
    text-align: center;
    gap: 16px;
  }

  .hero-icon {
    width: 64px;
    height: 64px;
  }

  .hero-title {
    font-size: 1.75rem;
  }

  .hero-subtitle {
    font-size: 1rem;
  }

  .filter-card {
    padding: 20px;
  }

  .filter-controls {
    flex-direction: column;
    gap: 12px;
  }

  .search-input {
    min-width: unset;
  }

  .filter-selects {
    flex-direction: column;
    gap: 8px;
  }

  .filter-select {
    min-width: unset;
  }

  .table-card {
    border-radius: 14px;
  }

  .table-container {
    padding: 12px;
  }
}

@media (max-width: 480px) {
  .rules-view {
    padding: 8px;
  }

  .hero-header {
    padding: 16px;
  }

  .hero-title {
    font-size: 1.5rem;
  }

  .hero-btn {
    height: 44px;
    padding: 0 20px;
    font-size: 0.875rem;
  }

  .filter-card {
    padding: 16px;
  }

  .filter-header {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
    text-align: center;
  }

  .filter-stats {
    justify-content: center;
  }

  .empty-state {
    padding: 32px 16px;
    min-height: 320px;
  }

  .empty-title {
    font-size: 1.25rem;
  }

  .empty-description {
    font-size: 0.875rem;
  }

  .empty-action {
    height: 44px;
    padding: 0 20px;
    font-size: 0.875rem;
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
