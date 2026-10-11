<template>
  <div class="page-shell rules-page">
    <PageHeader :title="t('rules.title')" :subtitle="t('rules.subtitle')" />

    <n-space vertical>
      <!-- 规则表（GetRules 已在 sing-box 1.14 gRPC API 暴露） -->
      <n-card :title="t('rules.routeRules')" size="small">
        <template #header-extra>
          <n-space :size="8" align="center">
            <n-tag :type="kernelRunning ? 'success' : 'default'" size="small">
              {{ kernelRunning ? t('common.running') : t('common.stopped') }}
            </n-tag>
            <n-button size="small" :loading="loading" @click="refresh">
              <template #icon><n-icon :size="14"><ReloadOutline /></n-icon></template>
              {{ t('common.refresh') }}
            </n-button>
          </n-space>
        </template>

        <n-empty
          v-if="!loading && rules.length === 0"
          :description="t('rules.empty')"
        />
        <n-data-table
          v-else
          :columns="columns"
          :data="rules"
          :pagination="pagination"
          :bordered="false"
          :max-height="520"
          size="small"
        />
      </n-card>

      <!-- 规则说明 -->
      <n-card :title="t('rules.about')" size="small">
        <p class="rules-hint">
          {{ t('rules.aboutBody') }}
        </p>
      </n-card>
    </n-space>
  </div>
</template>

<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { ReloadOutline } from '@vicons/ionicons5'
import { NTag } from 'naive-ui'
import PageHeader from '@/components/common/PageHeader.vue'
import { useI18n } from 'vue-i18n'
import { invokeWithAppContext } from '@/services/invoke-client'

defineOptions({ name: 'RulesView' })
const { t } = useI18n()

interface Rule {
  rule_type: string
  payload: string
  outbound: string
}

interface RuleList {
  rules: Rule[]
}

const rules = ref<Rule[]>([])
const loading = ref(false)
const kernelRunning = ref(true)

const pagination = ref({ pageSize: 20 })

const typeColorMap: Record<string, 'default' | 'info' | 'success' | 'warning' | 'error'> = {
  default: 'default',
  logical: 'info',
  rule_set: 'success',
  inbound: 'warning',
}

const columns = computed(() => [
  {
    title: t('rules.columnType'),
    key: 'rule_type',
    width: 110,
    render(row: Rule) {
      const type = String(row.rule_type || 'default')
      return h(
        NTag,
        { size: 'small', type: typeColorMap[type] || 'default', bordered: false },
        { default: () => type },
      )
    },
  },
  {
    title: t('rules.columnPayload'),
    key: 'payload',
    ellipsis: { tooltip: true },
  },
  {
    title: t('rules.columnOutbound'),
    key: 'outbound',
    width: 180,
    ellipsis: { tooltip: true },
  },
])

const refresh = async () => {
  if (loading.value) return
  loading.value = true
  try {
    const result = await invokeWithAppContext<RuleList>('get_rules')
    rules.value = Array.isArray(result?.rules) ? result.rules : []
  } catch (error) {
    console.warn('[RulesView] get_rules failed:', error)
    kernelRunning.value = false
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  refresh()
})
</script>

<style scoped>
.rules-hint {
  margin: 0;
  color: var(--text-color-secondary, #6b7280);
  font-size: 13px;
  line-height: 1.6;
}
</style>
