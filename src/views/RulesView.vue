<template>
  <div class="page-shell rules-page" :style="pageThemeStyle">
    <section class="page-hero">
      <div class="hero-row">
        <div class="hero-left">
          <div class="hero-icon">
            <n-icon size="26">
              <FilterOutline />
            </n-icon>
          </div>
          <div class="hero-meta">
            <p class="hero-subtitle">{{ t('rules.subtitle') }}</p>
            <h2 class="hero-title">{{ t('rules.title') }}</h2>
          </div>
        </div>
        <div class="hero-actions">
          <n-button
            @click="fetchRules"
            :loading="loading"
            type="primary"
            size="large"
          >
            <template #icon>
              <n-icon size="18">
                <RefreshOutline />
              </n-icon>
            </template>
            {{ t('rules.getRules') }}
          </n-button>
        </div>
      </div>
      <div class="hero-stats">
        <div
          v-for="stat in ruleStats"
          :key="stat.label"
          class="stat-card"
          :data-accent="stat.accent"
        >
          <div class="stat-icon">
            <n-icon :size="20">
              <component :is="stat.icon" />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stat.value }}</div>
            <div class="stat-label">{{ stat.label }}</div>
          </div>
        </div>
      </div>
    </section>

    <section class="page-section">
      <n-card class="surface-card filter-panel" :bordered="false">
        <div class="filter-content">
          <div class="filter-row">
            <n-input
              v-model:value="searchQuery"
              :placeholder="t('rules.searchPlaceholder')"
              clearable
              size="large"
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
              size="large"
              class="filter-select"
            />
            <n-select
              v-model:value="proxyFilter"
              :options="proxyOptions"
              :placeholder="t('rules.targetProxy')"
              clearable
              size="large"
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

      <n-card class="surface-card rules-card" :bordered="false">
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
    </section>
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
import { tauriApi } from '@/services/tauri'
import { useI18n } from 'vue-i18n'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { usePageTheme } from '@/composables/usePageTheme'

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
const pageThemeStyle = usePageTheme(themeStore)

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

const ruleStats = computed(() => [
  {
    label: t('rules.totalRules'),
    value: totalRules.value,
    icon: FilterOutline,
    accent: 'purple',
  },
  {
    label: t('rules.matchingRules'),
    value: filteredRules.value.length,
    icon: CheckmarkCircleOutline,
    accent: 'pink',
  },
  {
    label: t('rules.ruleTypes'),
    value: typeOptions.value.length,
    icon: ExtensionPuzzleOutline,
    accent: 'amber',
  },
  {
    label: t('rules.proxyTargets'),
    value: proxyOptions.value.length,
    icon: GlobeOutline,
    accent: 'blue',
  },
])

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
    const response = await tauriApi.getRules() as RulesResponse
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
  animation: fadeIn 0.4s ease both;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(12px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.filter-panel {
  border-radius: 28px;
}

.filter-content {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.filter-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.search-input {
  flex: 1;
}

.filter-select {
  min-width: 180px;
  flex: 1;
}

.active-filters {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.rules-card {
  border-radius: 32px;
}

.rules-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
}

.rule-item {
  position: relative;
  border-radius: 24px;
  padding: 18px;
  border: 1px solid var(--panel-border);
  background: rgba(15, 23, 42, 0.02);
  transition: border-color 0.2s ease, transform 0.2s ease;
}

.rule-item:hover {
  border-color: rgba(91, 76, 253, 0.3);
  transform: translateY(-2px);
}

.rule-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  gap: 12px;
}

.rule-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.rule-payload,
.rule-proxy {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.content-label,
.proxy-label {
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.content-value,
.proxy-value {
  font-weight: 600;
  color: var(--text-primary);
  word-break: break-word;
}

.rule-indicator {
  position: absolute;
  inset: 0;
  border-radius: 24px;
  border: 1px solid transparent;
  pointer-events: none;
}

.rule-domain .rule-indicator {
  border-color: rgba(99, 102, 241, 0.4);
}

.rule-ipv4 .rule-indicator {
  border-color: rgba(16, 185, 129, 0.45);
}

.rule-port .rule-indicator {
  border-color: rgba(251, 146, 60, 0.45);
}

@media (max-width: 768px) {
  .rules-grid {
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  }
}
</style>
