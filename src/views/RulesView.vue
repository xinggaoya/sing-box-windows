<template>
  <div class="ultra-rules">
    <!-- 紧凑工具栏 -->
    <div class="rules-toolbar">
      <div class="toolbar-left">
        <div class="toolbar-icon">
          <n-icon size="16">
            <FilterOutline />
          </n-icon>
        </div>
        <div class="toolbar-info">
          <span class="toolbar-title">{{ t('rules.title') }}</span>
          <span class="toolbar-stats">{{ filteredRules.length }} {{ t('rules.totalRulesLabel', { count: filteredRules.length }) }}</span>
        </div>
      </div>

      <div class="toolbar-right">
        <n-button
          @click="fetchRules"
          :loading="loading"
          type="primary"
          size="small"
          class="refresh-btn"
        >
          <template #icon>
            <n-icon size="12"><RefreshOutline /></n-icon>
          </template>
          {{ t('common.refresh') }}
        </n-button>
      </div>
    </div>

    <!-- 搜索筛选区域 -->
    <div class="rules-content">
      <div class="search-section">
        <div class="search-input-group">
          <n-input
            v-model:value="searchQuery"
            :placeholder="t('rules.searchPlaceholder')"
            clearable
            size="small"
            class="search-input"
          >
            <template #prefix>
              <n-icon size="14">
                <SearchOutline />
              </n-icon>
            </template>
          </n-input>

          <div class="filter-selects">
            <n-select
              v-model:value="typeFilter"
              :options="typeOptions"
              :placeholder="t('rules.type')"
              clearable
              size="small"
              class="filter-select"
            />
            <n-select
              v-model:value="proxyFilter"
              :options="proxyOptions"
              :placeholder="t('rules.targetProxy')"
              clearable
              size="small"
              class="filter-select"
            />
          </div>
        </div>

        <div class="filter-tags" v-if="searchQuery || typeFilter || proxyFilter">
          <n-tag v-if="searchQuery" size="tiny" round class="filter-tag">
            {{ t('common.search') }}: {{ searchQuery }}
          </n-tag>
          <n-tag v-if="typeFilter" size="tiny" round class="filter-tag">
            {{ t('rules.type') }}: {{ typeFilter }}
          </n-tag>
          <n-tag v-if="proxyFilter" size="tiny" round class="filter-tag">
            {{ t('rules.targetProxy') }}: {{ proxyFilter }}
          </n-tag>
        </div>
      </div>

      <!-- 规则列表 -->
      <div class="rules-list">
        <n-spin :show="loading">
          <div v-if="filteredRules.length > 0" class="rules-grid">
            <div
              v-for="(rule, index) in filteredRules"
              :key="index"
              class="rule-item"
              :class="{ 'rule-highlight': isRuleHighlighted(rule) }"
            >
              <!-- 规则类型 -->
              <div class="rule-type">
                <div class="type-badge" :class="getTypeClass(rule.type)">
                  {{ rule.type }}
                </div>
              </div>

              <!-- 规则内容 -->
              <div class="rule-content">
                <div class="rule-text" :title="rule.payload">
                  {{ getHighlightedText(rule.payload) }}
                </div>
              </div>

              <!-- 代理目标 -->
              <div class="rule-proxy">
                <div class="proxy-badge" :class="getProxyClass(rule.proxy)">
                  {{ getProxyName(rule.proxy) }}
                </div>
              </div>

              <!-- 规则序号 -->
              <div class="rule-index">
                #{{ index + 1 }}
              </div>
            </div>
          </div>

          <!-- 空状态 -->
          <div v-else class="empty-state">
            <div class="empty-icon">
              <n-icon size="32">
                <FilterOutline />
              </n-icon>
            </div>
            <div class="empty-title">
              {{
                searchQuery || typeFilter || proxyFilter
                  ? t('rules.noMatchingRulesFound')
                  : t('rules.noRulesData')
              }}
            </div>
            <div class="empty-desc">
              {{
                searchQuery || typeFilter || proxyFilter
                  ? t('rules.adjustSearchConditions')
                  : t('rules.clickRefreshToGetRules')
              }}
            </div>
            <n-button
              v-if="!searchQuery && !typeFilter && !proxyFilter"
              @click="fetchRules"
              type="primary"
              size="medium"
              class="empty-btn"
            >
              <template #icon>
                <n-icon size="14"><RefreshOutline /></n-icon>
              </template>
              {{ t('rules.getRules') }}
            </n-button>
          </div>
        </n-spin>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h, computed, nextTick } from 'vue'
import { useMessage, NTag, DataTableColumns, SelectOption } from 'naive-ui'
import { RefreshOutline, SearchOutline, FilterOutline } from '@vicons/ionicons5'
import { tauriApi } from '@/services/tauri-api'
import { useI18n } from 'vue-i18n'
import { useThemeStore } from '@/stores/app/ThemeStore'

const message = useMessage()
const loading = ref(false)
const { t } = useI18n()
const themeStore = useThemeStore()

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
    { label: t('rules.blockAction'), value: 'reject' },
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

// 辅助方法
const getProxyName = (proxy: string): string => {
  if (proxy.startsWith('route(') && proxy.endsWith(')')) {
    return proxy.substring(6, proxy.length - 1)
  }
  return proxy
}

const getTypeClass = (type: string): string => {
  if (type === 'logical') return 'type-logical'
  if (type === 'default') return 'type-default'
  return 'type-normal'
}

const getProxyClass = (proxy: string): string => {
  if (proxy === 'reject') return 'proxy-reject'
  if (proxy === t('rules.directConnect')) return 'proxy-direct'
  if (proxy === 'hijack-dns' || proxy === 'sniff') return 'proxy-info'
  return 'proxy-normal'
}

const isRuleHighlighted = (rule: Rule): boolean => {
  if (!searchQuery.value) return false

  const query = searchQuery.value.toLowerCase()
  const payload = String(rule.payload || '').toLowerCase()
  const proxy = String(rule.proxy || '').toLowerCase()
  const type = String(rule.type || '').toLowerCase()

  return payload.includes(query) || proxy.includes(query) || type.includes(query)
}

const getHighlightedText = (text: string): string => {
  if (!searchQuery.value) return text

  const index = text.toLowerCase().indexOf(searchQuery.value.toLowerCase())
  if (index === -1) return text

  const beforeMatch = text.substring(0, index)
  const match = text.substring(index, index + searchQuery.value.length)
  const afterMatch = text.substring(index + searchQuery.value.length)

  return `${beforeMatch}<mark>${match}</mark>${afterMatch}`
}

// 获取规则列表
const fetchRules = async () => {
  // 防止同时多次调用
  if (loading.value) return

  loading.value = true
  try {
    // 添加超时机制防止请求挂起
    const timeout = new Promise((_, reject) =>
      setTimeout(() => reject(new Error(t('common.requestTimeout'))), 10000),
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
    if (error instanceof Error && error.message === t('common.requestTimeout')) {
      message.error(t('rules.fetchTimeout'))
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
.ultra-rules {
  padding: 16px;
  background: var(--n-color-embedded);
  min-height: calc(100vh - 36px);
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: slideFadeIn 0.4s ease-out;
}

/* 紧凑工具栏 */
.rules-toolbar {
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toolbar-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.3);
}

.toolbar-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toolbar-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0;
}

.toolbar-stats {
  font-size: 0.75rem;
  color: var(--n-text-color-3);
  margin: 0;
}

.toolbar-right {
  display: flex;
  gap: 8px;
}

.refresh-btn {
  height: 32px;
  padding: 0 12px;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.refresh-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.3);
}

/* 搜索筛选区域 */
.rules-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 0;
}

.search-section {
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
}

.search-input-group {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1;
  min-width: 280px;
}

.search-input :deep(.n-input) {
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
}

.search-input :deep(.n-input:hover) {
  border-color: #4080ff;
}

.search-input :deep(.n-input.n-input--focus) {
  border-color: #4080ff;
  box-shadow: 0 0 0 2px rgba(64, 128, 255, 0.1);
}

.filter-selects {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-select {
  min-width: 140px;
}

.filter-select :deep(.n-base-selection) {
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
}

.filter-select :deep(.n-base-selection:hover) {
  border-color: #4080ff;
}

.filter-select :deep(.n-base-selection.n-base-selection--focus) {
  border-color: #4080ff;
  box-shadow: 0 0 0 2px rgba(64, 128, 255, 0.1);
}

.filter-tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-tag {
  font-size: 0.75rem;
  font-weight: 500;
}

/* 规则列表 */
.rules-list {
  flex: 1;
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 16px;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  min-height: 0;
}

.rules-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rule-item {
  display: grid;
  grid-template-columns: 80px 1fr 120px 40px;
  gap: 12px;
  align-items: center;
  padding: 12px 16px;
  background: var(--n-color-embedded);
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.rule-item:hover {
  background: var(--n-color-embedded-modal);
  border-color: #4080ff;
  transform: translateX(2px);
  box-shadow: 0 2px 8px rgba(64, 128, 255, 0.1);
}

.rule-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: transparent;
  transition: background 0.2s ease;
}

.rule-item:hover::before {
  background: #4080ff;
}

.rule-highlight {
  background: rgba(64, 128, 255, 0.05);
  border-color: rgba(64, 128, 255, 0.2);
}

.rule-highlight::before {
  background: #4080ff;
}

.rule-type {
  display: flex;
  align-items: center;
}

.type-badge {
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 500;
  text-align: center;
  white-space: nowrap;
}

.type-logical {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.2);
}

.type-default {
  background: rgba(99, 102, 241, 0.1);
  color: #6366f1;
  border: 1px solid rgba(99, 102, 241, 0.2);
}

.type-normal {
  background: rgba(107, 114, 128, 0.1);
  color: #6b7280;
  border: 1px solid rgba(107, 114, 128, 0.2);
}

.rule-content {
  flex: 1;
  min-width: 0;
}

.rule-text {
  font-size: 0.875rem;
  color: var(--n-text-color-1);
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.rule-proxy {
  display: flex;
  align-items: center;
}

.proxy-badge {
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 0.75rem;
  font-weight: 500;
  text-align: center;
  white-space: nowrap;
}

.proxy-reject {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border: 1px solid rgba(239, 68, 68, 0.2);
}

.proxy-direct {
  background: rgba(16, 185, 129, 0.1);
  color: #10b981;
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.proxy-info {
  background: rgba(99, 102, 241, 0.1);
  color: #6366f1;
  border: 1px solid rgba(99, 102, 241, 0.2);
}

.proxy-normal {
  background: rgba(107, 114, 128, 0.1);
  color: #6b7280;
  border: 1px solid rgba(107, 114, 128, 0.2);
}

.rule-index {
  font-size: 0.75rem;
  color: var(--n-text-color-3);
  font-weight: 500;
  text-align: right;
}

/* 高亮标记 */
.rule-text :deep(mark) {
  background: rgba(64, 128, 255, 0.2);
  color: var(--n-text-color-1);
  padding: 1px 2px;
  border-radius: 2px;
  font-weight: 600;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  padding: 40px 20px;
  text-align: center;
}

.empty-icon {
  color: var(--n-text-color-disabled);
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0 0 8px 0;
}

.empty-desc {
  font-size: 0.875rem;
  color: var(--n-text-color-3);
  margin: 0 0 20px 0;
  line-height: 1.5;
  max-width: 300px;
}

.empty-btn {
  height: 36px;
  padding: 0 16px;
  font-size: 0.875rem;
  font-weight: 500;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.empty-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.3);
}

/* 动画效果 */
@keyframes slideFadeIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .ultra-rules {
    padding: 12px;
    gap: 12px;
  }

  .rules-toolbar {
    padding: 10px 12px;
  }

  .toolbar-icon {
    width: 28px;
    height: 28px;
  }

  .toolbar-title {
    font-size: 0.875rem;
  }

  .toolbar-stats {
    font-size: 0.7rem;
  }

  .search-input-group {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .search-input {
    min-width: unset;
  }

  .filter-selects {
    justify-content: space-between;
  }

  .filter-select {
    flex: 1;
    min-width: 120px;
  }

  .rule-item {
    grid-template-columns: 60px 1fr 100px 30px;
    gap: 8px;
    padding: 10px 12px;
  }

  .type-badge,
  .proxy-badge {
    font-size: 0.7rem;
    padding: 3px 6px;
  }

  .rule-text {
    font-size: 0.8rem;
  }

  .rule-index {
    font-size: 0.7rem;
  }
}

@media (max-width: 480px) {
  .ultra-rules {
    padding: 8px;
    gap: 8px;
  }

  .rules-toolbar {
    padding: 8px 10px;
  }

  .toolbar-left {
    gap: 8px;
  }

  .toolbar-icon {
    width: 24px;
    height: 24px;
  }

  .toolbar-title {
    font-size: 0.8rem;
  }

  .search-section {
    padding: 12px;
  }

  .rules-list {
    padding: 12px;
  }

  .rule-item {
    grid-template-columns: 1fr;
    gap: 6px;
    padding: 8px 10px;
  }

  .rule-type,
  .rule-proxy,
  .rule-index {
    display: none;
  }

  .rule-text {
    font-size: 0.8rem;
    white-space: normal;
    overflow: visible;
    text-overflow: unset;
    line-height: 1.4;
  }

  .empty-state {
    padding: 32px 16px;
    min-height: 250px;
  }

  .empty-title {
    font-size: 1rem;
  }

  .empty-desc {
    font-size: 0.8rem;
  }
}

/* Naive UI 组件优化 */
:deep(.n-spin-container) {
  min-height: 200px;
}

:deep(.n-input__input-el) {
  font-size: 0.875rem !important;
}

:deep(.n-base-selection-label) {
  font-size: 0.875rem !important;
}

:deep(.n-button__content) {
  font-size: 0.875rem !important;
}
</style>
