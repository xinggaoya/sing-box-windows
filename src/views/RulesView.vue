<template>
  <div class="page-container">
    <PageHeader :title="t('rules.title')" :subtitle="t('rules.subtitle')">
      <template #actions>
        <n-button
          @click="fetchRules"
          :loading="loading"
          type="primary"
          secondary
          round
        >
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          {{ t('rules.getRules') }}
        </n-button>
      </template>
    </PageHeader>

    <!-- Stats -->
    <div class="stats-grid">
      <StatusCard
        v-for="stat in ruleStats"
        :key="stat.label"
        :label="stat.label"
        :value="stat.value"
        :type="stat.type"
      >
        <template #icon>
          <n-icon><component :is="stat.icon" /></n-icon>
        </template>
      </StatusCard>
    </div>

    <!-- Filters -->
    <div class="filter-section">
      <div class="filter-bar">
        <n-input
          v-model:value="searchQuery"
          :placeholder="t('rules.searchPlaceholder')"
          clearable
          round
          class="search-input"
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
        
        <n-select
          v-model:value="typeFilter"
          :options="typeOptions"
          :placeholder="t('rules.type')"
          clearable
          class="filter-select"
        />
        
        <n-select
          v-model:value="proxyFilter"
          :options="proxyOptions"
          :placeholder="t('rules.targetProxy')"
          clearable
          class="filter-select"
        />
      </div>
    </div>

    <!-- Rules List -->
    <div class="rules-section">
      <n-spin :show="loading">
        <div v-if="filteredRules.length > 0" class="rules-grid">
          <div
            v-for="(rule, index) in filteredRules"
            :key="index"
            class="rule-card"
          >
            <div class="rule-header">
              <n-tag :type="getRuleTypeTagType(rule.type)" size="small" round :bordered="false">
                {{ rule.type }}
              </n-tag>
              <span class="rule-index">#{{ index + 1 }}</span>
            </div>
            
            <div class="rule-body">
              <div class="rule-row">
                <span class="label">{{ t('rules.content') }}</span>
                <span class="value" :title="rule.payload">{{ rule.payload }}</span>
              </div>
              <div class="rule-row">
                <span class="label">{{ t('rules.targetProxy') }}</span>
                <span class="value proxy">
                  <n-tag :type="getProxyTagType(rule.proxy)" size="small" :bordered="false">
                    {{ getProxyLabel(rule.proxy) }}
                  </n-tag>
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Empty State -->
        <div v-else class="empty-state">
          <div class="empty-icon">
            <n-icon size="48"><FilterOutline /></n-icon>
          </div>
          <h3 class="empty-title">
            {{ searchQuery || typeFilter || proxyFilter ? t('rules.noMatchingRules') : t('rules.noRulesData') }}
          </h3>
          <n-button
            v-if="!searchQuery && !typeFilter && !proxyFilter"
            @click="fetchRules"
            type="primary"
          >
            {{ t('rules.getRules') }}
          </n-button>
          <n-button
            v-else
            @click="clearFilters"
            secondary
          >
            {{ t('rules.clearFilters') }}
          </n-button>
        </div>
      </n-spin>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { onMounted, ref, computed } from 'vue'
import { useMessage } from 'naive-ui'
import {
  RefreshOutline,
  FilterOutline,
  SearchOutline,
  CheckmarkCircleOutline,
  ExtensionPuzzleOutline,
  GlobeOutline,
} from '@vicons/ionicons5'
import { proxyService } from '@/services/proxy-service'
import { useI18n } from 'vue-i18n'
import PageHeader from '@/components/common/PageHeader.vue'
import StatusCard from '@/components/common/StatusCard.vue'

interface Rule {
  type: string
  payload: string
  proxy: string
}

interface RulesResponse {
  rules: Rule[]
}

defineOptions({
  name: 'RulesView'
})

const message = useMessage()
const { t } = useI18n()
const loading = ref(false)
const rules = ref<Rule[]>([])
const searchQuery = ref('')
const typeFilter = ref<string | null>(null)
const proxyFilter = ref<string | null>(null)

// Computed
const filteredRules = computed(() => {
  return rules.value.filter((rule) => {
    const matchSearch =
      !searchQuery.value ||
      rule.payload.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      rule.proxy.toLowerCase().includes(searchQuery.value.toLowerCase())
    const matchType = !typeFilter.value || rule.type === typeFilter.value
    const matchProxy = !proxyFilter.value || rule.proxy === proxyFilter.value
    return matchSearch && matchType && matchProxy
  })
})

const typeOptions = computed(() => {
  const types = new Set(rules.value.map((r) => r.type))
  return Array.from(types).map((type) => ({ label: type, value: type }))
})

const proxyOptions = computed(() => {
  const proxies = new Set<string>()
  rules.value.forEach((rule) => {
    let proxyName = rule.proxy
    if (proxyName.startsWith('[') && proxyName.endsWith(']')) {
      proxyName = proxyName.substring(1, proxyName.length - 1)
    }
    if (proxyName.startsWith('Proxy(') && proxyName.endsWith(')')) {
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
    value: rules.value.length,
    icon: FilterOutline,
    type: 'primary' as const,
  },
  {
    label: t('rules.matchingRules'),
    value: filteredRules.value.length,
    icon: CheckmarkCircleOutline,
    type: 'success' as const,
  },
  {
    label: t('rules.ruleTypes'),
    value: typeOptions.value.length,
    icon: ExtensionPuzzleOutline,
    type: 'warning' as const,
  },
  {
    label: t('rules.proxyTargets'),
    value: proxyOptions.value.length,
    icon: GlobeOutline,
    type: 'default' as const,
  },
])

// Methods
const getProxyLabel = (proxy: string): string => {
  if (proxy === 'direct') return t('rules.directConnect')
  if (proxy === 'reject') return t('rules.blockAction')
  return proxy
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

const clearFilters = () => {
  searchQuery.value = ''
  typeFilter.value = null
  proxyFilter.value = null
}

const fetchRules = async () => {
  loading.value = true
  try {
    const response = await proxyService.getRules() as RulesResponse
    rules.value = response.rules
    message.success(t('rules.fetchSuccess', { count: response.rules.length }))
  } catch (error) {
    message.error(t('rules.fetchError', { error: String(error) }))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchRules()
})
</script>

<style scoped>
.page-container {
  padding: 24px 32px;
  max-width: 1400px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.filter-section {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 16px;
}

.filter-bar {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.search-input {
  flex: 2;
  min-width: 200px;
}

.filter-select {
  flex: 1;
  min-width: 160px;
}

.rules-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
}

.rule-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 16px;
  transition: all 0.2s ease;
}

.rule-card:hover {
  border-color: var(--border-hover);
  transform: translateY(-2px);
  box-shadow: var(--panel-shadow);
}

.rule-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.rule-index {
  font-size: 12px;
  color: var(--text-tertiary);
}

.rule-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.rule-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.rule-row .label {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--text-tertiary);
  letter-spacing: 0.05em;
}

.rule-row .value {
  font-size: 14px;
  color: var(--text-primary);
  font-family: monospace;
  word-break: break-all;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px 0;
  color: var(--text-secondary);
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 16px;
  color: var(--text-primary);
}
</style>
