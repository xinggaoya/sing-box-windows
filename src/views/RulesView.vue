<template>
  <div class="page-container">
    <PageHeader :title="t('rules.title')" :subtitle="t('rules.subtitle')">
      <template #actions>
        <n-space>
          <n-button secondary @click="refreshAll">
            <template #icon>
              <n-icon><RefreshOutline /></n-icon>
            </template>
            {{ t('common.refresh') }}
          </n-button>
          <n-button
            v-if="activeTab === 'providers'"
            type="primary"
            secondary
            :loading="bulkUpdating"
            @click="updateAllProviders"
          >
            {{ providerLabels.updateAll }}
          </n-button>
        </n-space>
      </template>
    </PageHeader>

    <div class="toolbar-card">
      <n-tabs v-model:value="activeTab" type="segment" size="small">
        <n-tab-pane name="rules" :tab="providerLabels.rulesTab" />
        <n-tab-pane name="providers" :tab="providerLabels.providersTab" />
      </n-tabs>

      <div class="toolbar-row">
        <n-input v-model:value="searchQuery" :placeholder="t('rules.searchPlaceholder')" clearable>
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>

        <n-select
          v-if="activeTab === 'rules'"
          v-model:value="typeFilter"
          :options="typeOptions"
          clearable
          :placeholder="t('rules.type')"
        />
      </div>
    </div>

    <div v-if="activeTab === 'rules'" class="card-list">
      <div v-if="filteredRules.length" class="rules-grid">
        <div v-for="rule in filteredRules" :key="rule.index" class="rule-card">
          <div class="rule-head">
            <div class="rule-meta">
              <n-tag round size="small" :bordered="false">{{ rule.type }}</n-tag>
              <n-tag v-if="typeof rule.index === 'number'" size="small" round>{{ rule.index }}</n-tag>
            </div>
            <n-switch
              :value="!rule.extra?.disabled"
              :loading="typeof rule.index === 'number' ? rulesStore.ruleUpdatingMap[rule.index] : false"
              @update:value="toggleRule(rule)"
            />
          </div>
          <div class="rule-payload">{{ rule.payload || '-' }}</div>
          <div class="rule-footer">
            <span>{{ t('rules.targetProxy') }}: {{ getProxyLabel(rule.proxy) }}</span>
            <n-tag
              size="small"
              round
              :bordered="false"
              :type="rule.extra?.disabled ? 'warning' : 'success'"
            >
              {{ rule.extra?.disabled ? providerLabels.disabled : providerLabels.enabled }}
            </n-tag>
          </div>
        </div>
      </div>

      <div v-else class="empty-state">
        <div class="empty-icon">
          <n-icon size="48"><FilterOutline /></n-icon>
        </div>
        <h3 class="empty-title">{{ t('rules.noRulesData') }}</h3>
      </div>
    </div>

    <div v-else class="card-list">
      <div v-if="filteredProviders.length" class="providers-grid">
        <div v-for="provider in filteredProviders" :key="provider.name" class="provider-card">
          <div class="provider-head">
            <div>
              <div class="provider-name">{{ provider.name }}</div>
              <div class="provider-meta">
                {{ provider.behavior || '-' }} · {{ provider.vehicleType || '-' }}
              </div>
            </div>
            <n-button
              size="small"
              secondary
              :loading="rulesStore.providerUpdatingMap[provider.name]"
              @click="updateProvider(provider.name)"
            >
              {{ t('common.refresh') }}
            </n-button>
          </div>
          <div class="provider-row">
            <span>{{ providerLabels.count }}</span>
            <strong>{{ provider.count ?? '-' }}</strong>
          </div>
          <div class="provider-row">
            <span>{{ providerLabels.updatedAt }}</span>
            <strong>{{ formatProviderTime(provider.updatedAt || provider.updateAt) }}</strong>
          </div>
        </div>
      </div>

      <div v-else class="empty-state">
        <div class="empty-icon">
          <n-icon size="48"><FilterOutline /></n-icon>
        </div>
        <h3 class="empty-title">{{ providerLabels.noProviders }}</h3>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useMessage } from 'naive-ui'
import { FilterOutline, RefreshOutline, SearchOutline } from '@vicons/ionicons5'
import PageHeader from '@/components/common/PageHeader.vue'
import { useRulesStore } from '@/stores/kernel/RulesStore'
import { useI18n } from 'vue-i18n'
import type { RuleItem } from '@/types/controller'

defineOptions({
  name: 'RulesView',
})

const { t, locale } = useI18n()
const message = useMessage()
const rulesStore = useRulesStore()
const activeTab = ref<'rules' | 'providers'>('rules')
const searchQuery = ref('')
const typeFilter = ref<string | null>(null)

const providerLabels = computed(() => ({
  rulesTab: locale.value.startsWith('zh') ? '规则列表' : 'Rules',
  providersTab: locale.value.startsWith('zh') ? '规则 Providers' : 'Providers',
  updateAll: locale.value.startsWith('zh') ? '刷新全部 Providers' : 'Update All',
  count: locale.value.startsWith('zh') ? '条目数' : 'Count',
  updatedAt: locale.value.startsWith('zh') ? '更新时间' : 'Updated',
  noProviders: locale.value.startsWith('zh') ? '暂无规则 Providers' : 'No rule providers',
  enabled: locale.value.startsWith('zh') ? '已启用' : 'Enabled',
  disabled: locale.value.startsWith('zh') ? '已停用' : 'Disabled',
}))

const filteredRules = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return rulesStore.rules.filter((rule) => {
    const matchesQuery =
      !query ||
      rule.type.toLowerCase().includes(query) ||
      rule.payload.toLowerCase().includes(query) ||
      rule.proxy.toLowerCase().includes(query)

    const matchesType = !typeFilter.value || rule.type === typeFilter.value
    return matchesQuery && matchesType
  })
})

const filteredProviders = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return rulesStore.providers.filter((provider) => {
    return (
      !query ||
      provider.name.toLowerCase().includes(query) ||
      (provider.behavior || '').toLowerCase().includes(query) ||
      (provider.vehicleType || '').toLowerCase().includes(query)
    )
  })
})

const typeOptions = computed(() =>
  rulesStore.ruleTypes.map((item) => ({
    label: item,
    value: item,
  })),
)

const bulkUpdating = computed(() =>
  Object.values(rulesStore.providerUpdatingMap).some(Boolean),
)

const refreshAll = async () => {
  try {
    await rulesStore.fetchAll()
    message.success(t('rules.fetchSuccess', { count: rulesStore.rules.length }))
  } catch (error) {
    message.error(t('rules.fetchError', { error: String(error) }))
  }
}

const updateProvider = async (providerName: string) => {
  try {
    await rulesStore.updateProvider(providerName)
    message.success(providerName)
  } catch (error) {
    message.error(String(error))
  }
}

const updateAllProviders = async () => {
  try {
    await rulesStore.updateAllProviders()
    message.success(providerLabels.value.updateAll)
  } catch (error) {
    message.error(String(error))
  }
}

const toggleRule = async (rule: RuleItem) => {
  try {
    await rulesStore.toggleDisabled(rule)
  } catch (error) {
    message.error(String(error))
  }
}

const getProxyLabel = (proxy: string) => {
  if (proxy === 'direct') return t('rules.directConnect')
  if (proxy === 'reject') return t('rules.blockAction')
  return proxy
}

const formatProviderTime = (value?: string) => {
  if (!value) return '-'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

if (!rulesStore.rules.length && !rulesStore.providers.length) {
  refreshAll()
}
</script>

<style scoped>
.page-container {
  padding: var(--layout-page-padding-y, 16px) var(--layout-page-padding-x, 24px);
  max-width: var(--layout-page-max-width, 1400px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--layout-page-gap, 16px);
}

.toolbar-card,
.rule-card,
.provider-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
}

.toolbar-card {
  padding: 16px;
}

.toolbar-row {
  margin-top: 12px;
  display: grid;
  grid-template-columns: minmax(220px, 1fr) 220px;
  gap: 12px;
}

.card-list {
  display: flex;
  flex-direction: column;
}

.rules-grid,
.providers-grid {
  display: grid;
  gap: 16px;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
}

.rule-card,
.provider-card {
  padding: 16px;
}

.rule-head,
.provider-head,
.rule-footer,
.provider-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.rule-meta {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.rule-payload {
  margin: 14px 0;
  color: var(--text-primary);
  line-height: 1.5;
  word-break: break-word;
}

.rule-footer,
.provider-meta,
.provider-row {
  color: var(--text-secondary);
  font-size: 13px;
}

.provider-name {
  font-weight: 600;
  color: var(--text-primary);
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
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
}

@media (max-width: 900px) {
  .toolbar-row {
    grid-template-columns: 1fr;
  }
}
</style>
