<template>
  <div class="rules-page">
    <!-- 页面标题和统计 -->
    <div class="page-header">
      <div class="header-content">
        <div class="header-left">
          <div class="header-icon">
            <n-icon size="20">
              <FilterOutline />
            </n-icon>
          </div>
          <div class="header-info">
            <h1 class="page-title">{{ t('rules.title') }}</h1>
            <p class="page-subtitle">{{ t('rules.subtitle') }}</p>
          </div>
        </div>
        <div class="header-actions">
          <n-button
            @click="fetchRules"
            :loading="loading"
            type="primary"
            size="medium"
            class="refresh-btn"
          >
            <template #icon>
              <n-icon size="16">
                <RefreshOutline />
              </n-icon>
            </template>
            {{ t('rules.getRules') }}
          </n-button>
        </div>
      </div>
    </div>

    <!-- 统计卡片 -->
    <div class="stats-grid">
      <n-card class="stat-card total-card" :bordered="false">
        <div class="stat-content">
          <div class="stat-icon">
            <n-icon size="24">
              <FilterOutline />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ totalRules }}</div>
            <div class="stat-label">{{ t('rules.totalRules') }}</div>
          </div>
        </div>
      </n-card>

      <n-card class="stat-card matched-card" :bordered="false">
        <div class="stat-content">
          <div class="stat-icon">
            <n-icon size="24">
              <CheckmarkCircleOutline />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ filteredRules.length }}</div>
            <div class="stat-label">{{ t('rules.matchingRules') }}</div>
          </div>
        </div>
      </n-card>

      <n-card class="stat-card types-card" :bordered="false">
        <div class="stat-content">
          <div class="stat-icon">
            <n-icon size="24">
              <ExtensionPuzzleOutline />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ typeOptions.length }}</div>
            <div class="stat-label">{{ t('rules.ruleTypes') }}</div>
          </div>
        </div>
      </n-card>

      <n-card class="stat-card proxies-card" :bordered="false">
        <div class="stat-content">
          <div class="stat-icon">
            <n-icon size="24">
              <GlobeOutline />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ proxyOptions.length }}</div>
            <div class="stat-label">{{ t('rules.proxyTargets') }}</div>
          </div>
        </div>
      </n-card>
    </div>

    <!-- 搜索和筛选 -->
    <n-card class="filter-card" :bordered="false">
      <div class="filter-content">
        <div class="filter-row">
          <n-input
            v-model:value="searchQuery"
            :placeholder="t('rules.searchPlaceholder')"
            clearable
            size="medium"
            class="search-input"
          >
            <template #prefix>
              <n-icon size="16">
                <SearchOutline />
              </n-icon>
            </template>
          </n-input>
          <n-select
            v-model:value="typeFilter"
            :options="typeOptions"
            :placeholder="t('rules.type')"
            clearable
            size="medium"
            class="filter-select"
          />
          <n-select
            v-model:value="proxyFilter"
            :options="proxyOptions"
            :placeholder="t('rules.targetProxy')"
            clearable
            size="medium"
            class="filter-select"
          />
        </div>
        <div class="active-filters" v-if="searchQuery || typeFilter || proxyFilter">
          <n-tag v-if="searchQuery" size="small" round closable @close="searchQuery = ''">
            {{ t('common.search') }}: {{ searchQuery }}
          </n-tag>
          <n-tag v-if="typeFilter" size="small" round closable @close="typeFilter = null">
            {{ t('rules.type') }}: {{ typeFilter }}
          </n-tag>
          <n-tag v-if="proxyFilter" size="small" round closable @close="proxyFilter = null">
            {{ t('rules.targetProxy') }}: {{ getProxyLabel(proxyFilter) }}
          </n-tag>
        </div>
      </div>
    </n-card>

    <!-- 规则列表 -->
    <n-card class="rules-card" :bordered="false">
      <n-spin :show="loading">
        <div v-if="filteredRules.length > 0" class="rules-grid">
          <div
            v-for="(rule, index) in filteredRules"
            :key="index"
            class="rule-item"
            :class="{ 'rule-highlight': isRuleHighlighted(rule) }"
          >
            <!-- 规则头部 -->
            <div class="rule-header">
              <div class="rule-type">
                <n-tag :type="getRuleTypeTagType(rule.type)" size="small">
                  {{ rule.type }}
                </n-tag>
              </div>
              <div class="rule-index">
                #{{ index + 1 }}
              </div>
            </div>

            <!-- 规则内容 -->
            <div class="rule-content">
              <div class="rule-payload">
                <div class="content-label">{{ t('rules.content') }}</div>
                <div class="content-value">
                  <HighlightText
                    v-if="searchQuery"
                    :text="rule.payload"
                    :keyword="searchQuery"
                  />
                  <span v-else>{{ rule.payload }}</span>
                </div>
              </div>
              <div class="rule-proxy">
                <div class="proxy-label">{{ t('rules.targetProxy') }}</div>
                <div class="proxy-value">
                  <HighlightText
                    v-if="searchQuery"
                    :text="getProxyLabel(rule.proxy)"
                    :keyword="searchQuery"
                  />
                  <span v-else>
                    <n-tag :type="getProxyTagType(rule.proxy)" size="small">
                      {{ getProxyLabel(rule.proxy) }}
                    </n-tag>
                  </span>
                </div>
              </div>
            </div>

            <!-- 规则指示器 -->
            <div class="rule-indicator" :class="getRuleClass(rule.type)"></div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else class="empty-state">
          <div class="empty-icon">
            <n-icon size="48">
              <FilterOutline />
            </n-icon>
          </div>
          <div class="empty-title">
            {{ searchQuery || typeFilter || proxyFilter ? t('rules.noMatchingRules') : t('rules.noRulesData') }}
          </div>
          <div class="empty-desc">
            {{
              searchQuery || typeFilter || proxyFilter
                ? t('rules.adjustSearchOrFilters')
                : t('rules.clickRefreshToGetRules')
            }}
          </div>
          <n-button
            v-if="!searchQuery && !typeFilter && !proxyFilter"
            @click="fetchRules"
            type="primary"
            size="large"
            class="empty-btn"
          >
            <template #icon>
              <n-icon size="18">
                <RefreshOutline />
              </n-icon>
            </template>
            {{ t('rules.getRules') }}
          </n-button>
          <n-button
            v-else
            @click="clearFilters"
            type="default"
            size="medium"
            class="empty-btn"
          >
            {{ t('rules.clearFilters') }}
          </n-button>
        </div>
      </n-spin>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, defineComponent, h } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  SearchOutline,
  FilterOutline,
  CheckmarkCircleOutline,
  ExtensionPuzzleOutline,
  GlobeOutline,
} from '@vicons/ionicons5'
import { tauriApi } from '@/services/tauri-api'
import { useI18n } from 'vue-i18n'
import { useThemeStore } from '@/stores/app/ThemeStore'

defineOptions({
  name: 'RulesView'
})

interface Rule {
  type: string
  payload: string
  proxy: string
}

interface RulesResponse {
  rules: Rule[]
}

// 高亮文本组件
const HighlightText = defineComponent({
  name: 'HighlightText',
  props: {
    text: {
      type: String,
      required: true,
    },
    keyword: {
      type: String,
      required: true,
    },
  },
  setup(props) {
    return () => {
      if (!props.keyword) {
        return props.text
      }

      const parts = props.text.split(new RegExp(`(${props.keyword})`, 'gi'))

      return h('span', {}, parts.map((part, index) => {
        if (part.toLowerCase() === props.keyword.toLowerCase()) {
          return h('mark', {
            key: index,
            style: {
              backgroundColor: 'rgba(91, 76, 253, 0.2)',
              color: '#5b4cfd',
              padding: '2px 4px',
              borderRadius: '4px',
              fontWeight: '600'
            }
          }, part)
        }
        return h('span', { key: index }, part)
      }))
    }
  }
})

const message = useMessage()
const loading = ref(false)
const { t } = useI18n()
const themeStore = useThemeStore()

const rules = ref<Rule[]>([])
const searchQuery = ref('')
const typeFilter = ref<string | null>(null)
const proxyFilter = ref<string | null>(null)

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

// 统计数据
const totalRules = computed(() => rules.value.length)

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

// 辅助方法
const getProxyLabel = (proxy: string): string => {
  if (proxy === 'direct') return t('rules.directConnect')
  if (proxy === 'reject') return t('rules.blockAction')
  return proxy
}

const getRuleClass = (type: string): string => {
  const typeLower = type.toLowerCase()
  if (typeLower.includes('domain')) return 'rule-domain'
  if (typeLower.includes('ipcidr')) return 'rule-ipv4'
  if (typeLower.includes('source')) return 'rule-source'
  if (typeLower.includes('port')) return 'rule-port'
  if (typeLower.includes('process')) return 'rule-process'
  return 'rule-default'
}

const getRuleTypeTagType = (type: string): 'info' | 'warning' | 'error' | 'success' | 'default' => {
  const typeLower = type.toLowerCase()
  if (typeLower.includes('domain')) return 'info'
  if (typeLower.includes('ipcidr')) return 'success'
  if (typeLower.includes('source')) return 'warning'
  if (typeLower.includes('port')) return 'error'
  return 'default'
}

const getProxyTagType = (proxy: string): 'success' | 'error' | 'info' | 'warning' => {
  if (proxy === t('rules.directConnect')) return 'success'
  if (proxy === 'reject') return 'error'
  return 'info'
}

const isRuleHighlighted = (rule: Rule): boolean => {
  if (!searchQuery.value) return false

  const searchText = searchQuery.value.toLowerCase()
  return (
    rule.payload.toLowerCase().includes(searchText) ||
    rule.proxy.toLowerCase().includes(searchText) ||
    rule.type.toLowerCase().includes(searchText)
  )
}

const clearFilters = () => {
  searchQuery.value = ''
  typeFilter.value = null
  proxyFilter.value = null
}

const fetchRules = async () => {
  loading.value = true
  try {
    const response: RulesResponse = await tauriApi.getRules()
    rules.value = response.rules
    message.success(t('rules.fetchSuccess', { count: response.rules.length }))
  } catch (error) {
    console.error('获取规则失败:', error)
    message.error(t('rules.fetchError', { error: String(error) }))
  } finally {
    loading.value = false
  }
}

// 生命周期
onMounted(() => {
  fetchRules()
})
</script>

<style scoped>
.rules-page {
  padding: 16px;
  min-height: calc(100vh - 48px);
  background: v-bind('themeStore.isDark ? "#18181b" : "#f8fafc"');
}

/* 页面标题 */
.page-header {
  margin-bottom: 24px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border-radius: 16px;
  padding: 24px 28px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3);
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.page-title {
  font-size: 24px;
  font-weight: 700;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0;
  letter-spacing: -0.02em;
}

.page-subtitle {
  font-size: 14px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0;
  font-weight: 400;
}

.header-actions {
  display: flex;
  gap: 16px;
}

.refresh-btn {
  height: 42px;
  padding: 0 16px;
  font-weight: 600;
  border-radius: 10px;
  transition: all 0.2s ease;
}

.refresh-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3);
}

/* 统计卡片 */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 20px;
  margin-bottom: 24px;
}

.stat-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
  transition: all 0.3s ease;
  overflow: hidden;
  position: relative;
}

.stat-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
}

.total-card::before {
  background: linear-gradient(90deg, #f59e0b 0%, #d97706 100%);
}

.matched-card::before {
  background: linear-gradient(90deg, #10b981 0%, #059669 100%);
}

.types-card::before {
  background: linear-gradient(90deg, #3b82f6 0%, #2563eb 100%);
}

.proxies-card::before {
  background: linear-gradient(90deg, #8b5cf6 0%, #7c3aed 100%);
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.1)"');
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.total-card .stat-icon {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%);
}

.matched-card .stat-icon {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
}

.types-card .stat-icon {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
}

.proxies-card .stat-icon {
  background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: 700;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  line-height: 1.2;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
}

/* 筛选卡片 */
.filter-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
  margin-bottom: 24px;
}

.filter-content {
  padding: 8px;
}

.filter-row {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
}

.filter-row:last-child {
  margin-bottom: 0;
}

.search-input {
  flex: 2;
}

.filter-select {
  flex: 1;
}

.active-filters {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  padding-top: 8px;
  border-top: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

/* 规则卡片 */
.rules-card {
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 16px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.05)"');
}

.rules-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.rule-item {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.02)" : "rgba(0, 0, 0, 0.02)"');
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  border-radius: 12px;
  padding: 16px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.rule-item:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.04)" : "rgba(0, 0, 0, 0.04)"');
  border-color: v-bind('themeStore.isDark ? "rgba(245, 158, 11, 0.3)" : "rgba(245, 158, 11, 0.2)"');
  transform: translateX(4px);
}

.rule-highlight {
  background: rgba(245, 158, 11, 0.05);
  border-color: rgba(245, 158, 11, 0.3);
}

.rule-indicator {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
}

.rule-domain {
  background: #3b82f6;
}

.rule-ipv4 {
  background: #10b981;
}

.rule-source {
  background: #f59e0b;
}

.rule-port {
  background: #ef4444;
}

.rule-process {
  background: #8b5cf6;
}

.rule-default {
  background: #6b7280;
}

.rule-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.rule-index {
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "#64748b" : "#9ca3af"');
  font-weight: 500;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
}

.rule-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.rule-payload,
.rule-proxy {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.content-label,
.proxy-label {
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.content-value,
.proxy-value {
  font-size: 13px;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  font-weight: 500;
  line-height: 1.4;
  word-break: break-all;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  padding: 40px 16px;
  text-align: center;
}

.empty-icon {
  color: v-bind('themeStore.isDark ? "#4b5563" : "#9ca3af"');
  margin-bottom: 12px;
  opacity: 0.6;
}

.empty-title {
  font-size: 20px;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0 0 12px 0;
}

.empty-desc {
  font-size: 14px;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0 0 24px 0;
  line-height: 1.5;
  max-width: 400px;
}

.empty-btn {
  height: 42px;
  padding: 0 24px;
  font-weight: 600;
  border-radius: 10px;
  transition: all 0.2s ease;
}

.empty-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(245, 158, 11, 0.3);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .rules-page {
    padding: 16px;
  }

  .header-content {
    flex-direction: column;
    gap: 20px;
    padding: 16px;
  }

  .header-left {
    width: 100%;
  }

  .header-actions {
    width: 100%;
    justify-content: center;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
  }

  .stat-content {
    padding: 16px;
    gap: 12px;
  }

  .stat-icon {
    width: 40px;
    height: 40px;
  }

  .stat-value {
    font-size: 20px;
  }

  .filter-row {
    flex-direction: column;
    gap: 12px;
  }

  .search-input,
  .filter-select {
    width: 100%;
  }

  .rule-content {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .rule-item {
    padding: 12px;
  }
}

@media (max-width: 480px) {
  .rules-page {
    padding: 12px;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .header-content {
    padding: 16px;
  }

  .page-title {
    font-size: 20px;
  }

  .page-subtitle {
    font-size: 13px;
  }

  .rule-item {
    padding: 10px 12px;
  }

  .rule-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .content-value,
  .proxy-value {
    font-size: 12px;
  }

  .empty-state {
    padding: 40px 16px;
    min-height: 300px;
  }

  .empty-title {
    font-size: 18px;
  }

  .empty-desc {
    font-size: 13px;
  }
}

/* Naive UI 组件优化 */
:deep(.n-spin-container) {
  min-height: 200px;
}

:deep(.n-input) {
  border-radius: 10px;
}

:deep(.n-base-selection) {
  border-radius: 10px;
}

:deep(.n-tag) {
  border-radius: 6px;
  font-weight: 500;
}

:deep(.n-button) {
  border-radius: 8px;
  font-weight: 500;
}

/* 高亮样式 */
mark {
  background: rgba(245, 158, 11, 0.2) !important;
  color: #f59e0b !important;
  padding: 2px 4px !important;
  border-radius: 4px !important;
  font-weight: 600 !important;
}
</style>