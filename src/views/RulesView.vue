<template>
  <div class="page-shell rules-page">
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
          <n-button
            v-if="activeTab === 'custom'"
            type="primary"
            secondary
            @click="openCreateCustomRule"
          >
            <template #icon>
              <n-icon><AddOutline /></n-icon>
            </template>
            {{ customLabels.add }}
          </n-button>
        </n-space>
      </template>
    </PageHeader>

    <ToolbarBar>
      <template #tabs>
        <n-tabs v-model:value="activeTab" type="segment" size="small">
          <n-tab-pane name="rules" :tab="providerLabels.rulesTab" />
          <n-tab-pane name="providers" :tab="providerLabels.providersTab" />
          <n-tab-pane name="custom" :tab="customLabels.tab" />
        </n-tabs>
      </template>
      <template #filters>
        <n-input
          v-model:value="searchQuery"
          :placeholder="t('rules.searchPlaceholder')"
          clearable
          size="small"
          class="search-input"
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
        <n-select
          v-if="activeTab === 'rules'"
          v-model:value="typeFilter"
          :options="typeOptions"
          clearable
          size="small"
          :placeholder="t('rules.type')"
          class="type-select"
        />
      </template>
    </ToolbarBar>

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

      <EmptyState v-else :title="t('rules.noRulesData')" :icon="FilterOutline" />
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

      <EmptyState v-else :title="providerLabels.noProviders" :icon="FilterOutline" />
    </div>

    <div v-if="activeTab === 'custom'" class="card-list">
      <div class="custom-hint">{{ customLabels.hint }}</div>
      <div v-if="rulesStore.customRules.length" class="rules-grid">
        <div v-for="rule in rulesStore.customRules" :key="rule.id" class="rule-card">
          <div class="rule-head">
            <div class="rule-meta">
              <n-tag round size="small" :bordered="false">{{ matchTypeLabel(rule.match_type) }}</n-tag>
              <n-tag size="small" round :type="actionTagType(rule.action)">{{ actionLabel(rule.action) }}</n-tag>
            </div>
            <n-switch
              :value="rule.enabled"
              :loading="rulesStore.customRuleUpdating[rule.id]"
              @update:value="onToggleCustomRule(rule.id)"
            />
          </div>
          <div class="rule-payload">{{ rule.payload || '-' }}</div>
          <div v-if="rule.note" class="custom-note">{{ rule.note }}</div>
          <div class="rule-footer">
            <n-space size="small">
              <n-button size="tiny" secondary @click="openEditCustomRule(rule)">
                {{ customLabels.edit }}
              </n-button>
              <n-popconfirm @positive-click="onDeleteCustomRule(rule.id)">
                <template #trigger>
                  <n-button size="tiny" secondary type="error">{{ customLabels.delete }}</n-button>
                </template>
                {{ customLabels.deleteConfirm }}
              </n-popconfirm>
            </n-space>
            <n-tag
              size="small"
              round
              :bordered="false"
              :type="rule.enabled ? 'success' : 'warning'"
            >
              {{ rule.enabled ? providerLabels.enabled : providerLabels.disabled }}
            </n-tag>
          </div>
        </div>
      </div>

      <EmptyState v-else :title="customLabels.empty" :icon="AddOutline">
        <template #action>
          <n-button type="primary" secondary @click="openCreateCustomRule">
            {{ customLabels.add }}
          </n-button>
        </template>
      </EmptyState>
    </div>

    <!-- 自定义规则编辑表单 -->
    <n-modal
      v-model:show="customModalShow"
      preset="card"
      :title="editingCustomRule ? customLabels.editTitle : customLabels.addTitle"
      style="max-width: 520px"
    >
      <n-form label-placement="top">
        <n-form-item :label="customLabels.matchType">
          <n-select v-model:value="customForm.matchType" :options="matchTypeOptions" />
        </n-form-item>
        <n-form-item :label="customLabels.payload">
          <n-input
            v-model:value="customForm.payload"
            type="textarea"
            :rows="3"
            :placeholder="customLabels.payloadPlaceholder"
          />
        </n-form-item>
        <n-form-item :label="customLabels.action">
          <n-select v-model:value="customForm.action" :options="actionOptions" />
        </n-form-item>
        <n-form-item :label="customLabels.note">
          <n-input v-model:value="customForm.note" :placeholder="customLabels.notePlaceholder" />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-space justify="end">
          <n-button @click="customModalShow = false">{{ customLabels.cancel }}</n-button>
          <n-button type="primary" :loading="customSubmitting" @click="submitCustomRule">
            {{ customLabels.confirm }}
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { useMessage } from 'naive-ui'
import { AddOutline, FilterOutline, RefreshOutline, SearchOutline } from '@vicons/ionicons5'
import PageHeader from '@/components/common/PageHeader.vue'
import ToolbarBar from '@/components/common/ToolbarBar.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import { useRulesStore } from '@/stores/kernel/RulesStore'
import { useI18n } from 'vue-i18n'
import type { RuleItem } from '@/types/controller'
import type {
  CustomRule,
  CustomRuleAction,
  CustomRuleMatchType,
} from '@/types/generated'

defineOptions({
  name: 'RulesView',
})

const { t, locale } = useI18n()
const message = useMessage()
const rulesStore = useRulesStore()
const activeTab = ref<'rules' | 'providers' | 'custom'>('rules')
const searchQuery = ref('')
const typeFilter = ref<string | null>(null)

// 自定义规则表单状态
const customModalShow = ref(false)
const customSubmitting = ref(false)
const editingCustomRule = ref<CustomRule | null>(null)
const customForm = reactive({
  matchType: 'domain_suffix' as CustomRuleMatchType,
  payload: '',
  action: 'direct' as CustomRuleAction,
  note: '',
})

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

const customLabels = computed(() => {
  const zh = locale.value.startsWith('zh')
  return {
    tab: zh ? '自定义' : 'Custom',
    add: zh ? '新增规则' : 'Add Rule',
    edit: zh ? '编辑' : 'Edit',
    delete: zh ? '删除' : 'Delete',
    deleteConfirm: zh ? '确认删除这条自定义规则？' : 'Delete this custom rule?',
    addTitle: zh ? '新增自定义规则' : 'Add Custom Rule',
    editTitle: zh ? '编辑自定义规则' : 'Edit Custom Rule',
    cancel: zh ? '取消' : 'Cancel',
    confirm: zh ? '保存' : 'Save',
    matchType: zh ? '匹配类型' : 'Match Type',
    payload: zh ? '匹配内容' : 'Payload',
    payloadPlaceholder: zh
      ? '每行一个，或用逗号分隔（如 example.com, *.test.com）'
      : 'One per line, or comma-separated (e.g. example.com, *.test.com)',
    action: zh ? '动作' : 'Action',
    note: zh ? '备注' : 'Note',
    notePlaceholder: zh ? '可选' : 'Optional',
    empty: zh ? '暂无自定义规则' : 'No custom rules',
    hint: zh
      ? '自定义规则持久化在本地，重启内核后生效。与上方“规则列表”（内核默认规则，仅本会话生效）不同。'
      : 'Custom rules are persisted locally and take effect after kernel restart. Different from the built-in rules above (runtime only).',
    addSuccess: zh ? '已新增规则，重启内核后生效' : 'Rule added (effective after kernel restart)',
    updateSuccess: zh ? '已更新规则，重启内核后生效' : 'Rule updated (effective after kernel restart)',
    deleteSuccess: zh ? '已删除规则，重启内核后生效' : 'Rule deleted (effective after kernel restart)',
  }
})

const matchTypeOptions = computed(() => {
  const zh = locale.value.startsWith('zh')
  return [
    { label: zh ? '域名后缀' : 'Domain Suffix', value: 'domain_suffix' as CustomRuleMatchType },
    { label: zh ? '精确域名' : 'Domain', value: 'domain' as CustomRuleMatchType },
    { label: zh ? '域名关键字' : 'Domain Keyword', value: 'domain_keyword' as CustomRuleMatchType },
    { label: 'IP CIDR', value: 'ip_cidr' as CustomRuleMatchType },
  ]
})

const actionOptions = computed(() => {
  const zh = locale.value.startsWith('zh')
  return [
    { label: zh ? '直连' : 'Direct', value: 'direct' as CustomRuleAction },
    { label: zh ? '走代理' : 'Proxy', value: 'proxy' as CustomRuleAction },
    { label: zh ? '拦截' : 'Block', value: 'block' as CustomRuleAction },
  ]
})

const matchTypeLabel = (mt: CustomRuleMatchType) =>
  matchTypeOptions.value.find((o) => o.value === mt)?.label || mt

const actionLabel = (act: CustomRuleAction) =>
  actionOptions.value.find((o) => o.value === act)?.label || act

const actionTagType = (act: CustomRuleAction): 'success' | 'info' | 'error' => {
  if (act === 'direct') return 'success'
  if (act === 'proxy') return 'info'
  return 'error'
}

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

const resetCustomForm = () => {
  customForm.matchType = 'domain_suffix'
  customForm.payload = ''
  customForm.action = 'direct'
  customForm.note = ''
}

const openCreateCustomRule = () => {
  editingCustomRule.value = null
  resetCustomForm()
  customModalShow.value = true
}

const openEditCustomRule = (rule: CustomRule) => {
  editingCustomRule.value = rule
  customForm.matchType = rule.match_type
  customForm.payload = rule.payload
  customForm.action = rule.action
  customForm.note = rule.note ?? ''
  customModalShow.value = true
}

const submitCustomRule = async () => {
  if (!customForm.payload.trim()) {
    message.error(customLabels.value.payload)
    return
  }
  customSubmitting.value = true
  try {
    if (editingCustomRule.value) {
      await rulesStore.updateCustomRule(
        editingCustomRule.value.id,
        customForm.matchType,
        customForm.payload,
        customForm.action,
        customForm.note || undefined,
      )
      message.success(customLabels.value.updateSuccess)
    } else {
      await rulesStore.addCustomRule(
        customForm.matchType,
        customForm.payload,
        customForm.action,
        customForm.note || undefined,
      )
      message.success(customLabels.value.addSuccess)
    }
    customModalShow.value = false
  } catch (error) {
    message.error(String(error))
  } finally {
    customSubmitting.value = false
  }
}

const onToggleCustomRule = (id: string) => async () => {
  try {
    await rulesStore.toggleCustomRule(id)
  } catch (error) {
    message.error(String(error))
  }
}

const onDeleteCustomRule = async (id: string) => {
  try {
    await rulesStore.deleteCustomRule(id)
    message.success(customLabels.value.deleteSuccess)
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

if (!rulesStore.rules.length && !rulesStore.providers.length && !rulesStore.customRules.length) {
  refreshAll()
}
</script>

<style scoped>
.rules-page {
  max-width: var(--content-max-width, 1440px);
  margin: 0 auto;
}

.search-input {
  flex: 1 1 240px;
  min-width: 200px;
}

.type-select {
  width: 180px;
}

.card-list {
  display: flex;
  flex-direction: column;
}

.rules-grid,
.providers-grid {
  display: grid;
  gap: var(--space-4);
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
}

.rule-card,
.provider-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  box-shadow: var(--panel-shadow);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  transition:
    transform var(--transition-fast),
    box-shadow var(--transition-fast);
}

.rule-card:hover,
.provider-card:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.rule-head,
.provider-head,
.rule-footer,
.provider-row {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
  align-items: center;
}

.rule-meta {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.rule-payload {
  margin: var(--space-4) 0;
  color: var(--text-primary);
  font-size: var(--text-sm);
  line-height: 1.5;
  word-break: break-word;
  font-family: var(--font-mono);
}

.custom-note {
  margin: calc(-1 * var(--space-2)) 0 var(--space-4);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  word-break: break-word;
}

.custom-hint {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: 1.6;
  margin-bottom: var(--space-3);
  padding: var(--space-3);
  background: var(--bg-surface-2);
  border-radius: var(--radius-md);
}

.rule-footer,
.provider-meta,
.provider-row {
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.provider-name {
  font-weight: 600;
  color: var(--text-primary);
  font-size: var(--text-md);
}

.provider-meta {
  margin-top: 2px;
  font-size: var(--text-xs);
}
</style>
