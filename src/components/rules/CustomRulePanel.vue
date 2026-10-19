<template>
  <div class="custom-rule-panel">
    <div class="panel-toolbar">
      <n-button size="small" type="primary" secondary @click="openCreate">
        <template #icon>
          <n-icon :size="14"><AddOutline /></n-icon>
        </template>
        {{ t('rules.custom.add') }}
      </n-button>
      <n-button size="small" secondary :loading="store.loading" @click="store.fetchRules">
        <template #icon>
          <n-icon :size="14"><RefreshOutline /></n-icon>
        </template>
        {{ t('common.refresh') }}
      </n-button>
      <span class="toolbar-hint">{{ t('rules.custom.restartHint') }}</span>
    </div>

    <n-alert v-if="usesOriginalConfig" type="warning" :bordered="false" size="small">
      {{ t('rules.custom.originalConfigHint') }}
    </n-alert>

    <n-empty
      v-if="!store.loading && store.rules.length === 0"
      :description="t('rules.custom.empty')"
    />

    <n-data-table
      v-else
      :columns="columns"
      :data="store.rules"
      :pagination="pagination"
      :bordered="false"
      :max-height="520"
      size="small"
    />

    <!-- 新增 / 编辑弹窗 -->
    <n-modal
      v-model:show="showModal"
      preset="card"
      style="width: 520px"
      :title="editingId ? t('rules.custom.edit') : t('rules.custom.add')"
      :mask-closable="!submitting"
    >
      <n-form label-placement="top" :disabled="submitting">
        <n-form-item :label="t('rules.custom.matchType')">
          <n-select
            v-model:value="form.match_type"
            :options="matchTypeOptions"
            :consistent-menu-width="false"
          />
        </n-form-item>

        <n-form-item :label="t('rules.custom.payload')">
          <n-input
            v-model:value="form.payload"
            type="textarea"
            :placeholder="t('rules.custom.payloadPlaceholder')"
            :autosize="{ minRows: 3, maxRows: 8 }"
          />
        </n-form-item>

        <n-form-item :label="t('rules.custom.action')">
          <n-select v-model:value="form.action" :options="actionOptions" />
        </n-form-item>

        <n-form-item :label="t('rules.custom.note')">
          <n-input v-model:value="form.note" :placeholder="t('rules.custom.notePlaceholder')" />
        </n-form-item>
      </n-form>

      <template #footer>
        <div class="modal-footer">
          <n-button :disabled="submitting" @click="showModal = false">
            {{ t('common.cancel') }}
          </n-button>
          <n-button type="primary" :loading="submitting" @click="submit">
            {{ t('common.save') }}
          </n-button>
        </div>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { computed, h, onMounted, ref } from 'vue'
import { AddOutline, RefreshOutline } from '@vicons/ionicons5'
import { NButton, NIcon, NPopconfirm, NSpace, NSwitch, NTag, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { useCustomRuleStore } from '@/stores/kernel/CustomRuleStore'
import { useSubStore } from '@/stores/subscription/SubStore'
import type { CustomRule } from '@/types/generated/CustomRule'
import type { CustomRuleAction } from '@/types/generated/CustomRuleAction'
import type { CustomRuleMatchType } from '@/types/generated/CustomRuleMatchType'

defineOptions({ name: 'CustomRulePanel' })

const { t } = useI18n()
const message = useMessage()
const store = useCustomRuleStore()
const subStore = useSubStore()

const showModal = ref(false)
const submitting = ref(false)
const editingId = ref<string | null>(null)

const form = ref<{
  match_type: CustomRuleMatchType
  payload: string
  action: CustomRuleAction
  note: string
}>({
  match_type: 'domain',
  payload: '',
  action: 'direct',
  note: '',
})

const pagination = ref({ pageSize: 20 })

const matchTypeOptions = computed(() => [
  { label: t('rules.custom.matchTypeOptions.domain'), value: 'domain' },
  { label: t('rules.custom.matchTypeOptions.domainSuffix'), value: 'domain_suffix' },
  { label: t('rules.custom.matchTypeOptions.domainKeyword'), value: 'domain_keyword' },
  { label: t('rules.custom.matchTypeOptions.ipCidr'), value: 'ip_cidr' },
])

const actionOptions = computed(() => [
  { label: t('rules.custom.actionOptions.direct'), value: 'direct' },
  { label: t('rules.custom.actionOptions.proxy'), value: 'proxy' },
  { label: t('rules.custom.actionOptions.block'), value: 'block' },
])

/**
 * 当前生效的订阅是否使用「原始配置」。
 * 自定义规则只会被注入到本程序生成的配置里，原始配置模式下不生效——UI 需要明确提示。
 */
const usesOriginalConfig = computed(() => {
  const idx = subStore.activeIndex
  if (idx === null || idx < 0) return false
  return Boolean(subStore.list[idx]?.useOriginalConfig)
})

const actionTagType = (action: string): 'success' | 'warning' | 'error' => {
  if (action === 'direct') return 'success'
  if (action === 'block') return 'error'
  return 'warning'
}

const columns = computed(() => [
  {
    title: t('rules.custom.columnEnabled'),
    key: 'enabled',
    width: 70,
    render: (row: CustomRule) =>
      h(NSwitch, {
        size: 'small',
        value: row.enabled,
        loading: store.pendingId === row.id,
        'onUpdate:value': () => void toggle(row),
      }),
  },
  {
    title: t('rules.custom.columnMatchType'),
    key: 'match_type',
    width: 130,
    render: (row: CustomRule) =>
      h(
        NTag,
        { size: 'small', type: 'info', bordered: false, round: true },
        { default: () => matchTypeLabel(row.match_type) },
      ),
  },
  {
    title: t('rules.custom.columnPayload'),
    key: 'payload',
    ellipsis: { tooltip: true },
  },
  {
    title: t('rules.custom.columnAction'),
    key: 'action',
    width: 90,
    render: (row: CustomRule) =>
      h(
        NTag,
        { size: 'small', type: actionTagType(row.action), bordered: false, round: true },
        { default: () => actionLabel(row.action) },
      ),
  },
  {
    title: t('rules.custom.columnNote'),
    key: 'note',
    width: 160,
    ellipsis: { tooltip: true },
    render: (row: CustomRule) => row.note || '—',
  },
  {
    title: t('rules.custom.columnActions'),
    key: 'actions',
    width: 120,
    render: (row: CustomRule) =>
      h(
        NSpace,
        { size: 6 },
        {
          default: () => [
            h(
              NButton,
              { size: 'tiny', secondary: true, onClick: () => openEdit(row) },
              { default: () => t('common.edit') },
            ),
            h(
              NPopconfirm,
              {
                positiveText: t('common.confirm'),
                negativeText: t('common.cancel'),
                onPositiveClick: () => void remove(row),
              },
              {
                trigger: () =>
                  h(
                    NButton,
                    { size: 'tiny', type: 'error', secondary: true },
                    { default: () => t('common.delete') },
                  ),
                default: () => t('rules.custom.deleteConfirm'),
              },
            ),
          ],
        },
      ),
  },
])

const matchTypeLabel = (value: string) =>
  matchTypeOptions.value.find((o) => o.value === value)?.label ?? value

const actionLabel = (value: string) =>
  actionOptions.value.find((o) => o.value === value)?.label ?? value

const openCreate = () => {
  editingId.value = null
  form.value = { match_type: 'domain', payload: '', action: 'direct', note: '' }
  showModal.value = true
}

const openEdit = (rule: CustomRule) => {
  editingId.value = rule.id
  form.value = {
    match_type: rule.match_type,
    payload: rule.payload,
    action: rule.action,
    note: rule.note ?? '',
  }
  showModal.value = true
}

const submit = async () => {
  if (submitting.value) return
  if (!form.value.payload.trim()) {
    message.warning(t('rules.custom.payloadRequired'))
    return
  }
  submitting.value = true
  try {
    const input = {
      match_type: form.value.match_type,
      payload: form.value.payload,
      action: form.value.action,
      note: form.value.note.trim() || null,
    }
    if (editingId.value) {
      await store.updateRule(editingId.value, input)
      message.success(t('rules.custom.saved'))
    } else {
      await store.addRule(input)
      message.success(t('rules.custom.saved'))
    }
    showModal.value = false
  } catch (error) {
    message.error(`${t('rules.custom.saveFailed')}: ${String(error)}`)
  } finally {
    submitting.value = false
  }
}

const toggle = async (rule: CustomRule) => {
  try {
    await store.toggleRule(rule.id)
  } catch (error) {
    message.error(`${t('rules.custom.saveFailed')}: ${String(error)}`)
  }
}

const remove = async (rule: CustomRule) => {
  try {
    await store.removeRule(rule.id)
    message.success(t('rules.custom.deleted'))
  } catch (error) {
    message.error(`${t('rules.custom.deleteFailed')}: ${String(error)}`)
  }
}

onMounted(() => {
  void store.fetchRules()
})
</script>

<style scoped>
.custom-rule-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.panel-toolbar {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.toolbar-hint {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
}
</style>
