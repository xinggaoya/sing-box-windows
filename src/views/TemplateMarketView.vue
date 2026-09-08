<template>
  <div class="page-shell template-market-page">
    <PageHeader :title="t('templateMarket.title')" :subtitle="t('templateMarket.subtitle')">
      <template #actions>
        <n-button round @click="openSettingsModal">
          <template #icon>
            <n-icon><CloudOutline /></n-icon>
          </template>
          {{ t('templateMarket.serviceSettings') }}
        </n-button>
        <n-button type="primary" round @click="openCreateModal">
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          {{ t('templateMarket.newTemplate') }}
        </n-button>
      </template>
    </PageHeader>

    <n-card :bordered="false" class="market-card">
      <n-tabs v-model:value="activeTab" type="line" animated>
        <!-- 当前模板 -->
        <n-tab-pane name="current" :tab="t('templateMarket.tabCurrent')">
          <n-alert
            v-if="store.isActiveOfficial"
            type="default"
            :show-icon="false"
            class="active-hint"
          >
            {{ t('templateMarket.officialActiveHint') }}
          </n-alert>
          <n-alert v-else type="info" :show-icon="false" class="active-hint">
            {{ t('templateMarket.templateActiveHint') }}
            <b>{{ activeDisplayName(store.activeTemplate) }}</b>
          </n-alert>

          <div class="template-grid">
            <div
              v-for="tpl in store.templates"
              :key="tpl.id"
              class="template-card"
              :class="{ active: tpl.id === store.settings.active_template_id }"
            >
              <div class="template-card-header">
                <div class="template-icon">
                  <n-icon size="20">
                    <ShieldCheckmarkOutline v-if="tpl.id === OFFICIAL_TEMPLATE_ID" />
                    <DocumentTextOutline v-else />
                  </n-icon>
                </div>
                <div class="template-info">
                  <div class="template-name" :title="tpl.name">{{ activeDisplayName(tpl) }}</div>
                  <div class="template-tags">
                    <n-tag size="small" :bordered="false" round>
                      {{ sourceLabel(tpl.source) }}
                    </n-tag>
                    <n-tag
                      v-if="tpl.market_status"
                      size="small"
                      :bordered="false"
                      round
                      :type="marketStatusType(tpl.market_status)"
                    >
                      {{ marketStatusLabel(tpl.market_status) }}
                    </n-tag>
                    <n-tag
                      v-if="tpl.id === store.settings.active_template_id"
                      type="success"
                      size="small"
                      :bordered="false"
                      round
                    >
                      {{ t('templateMarket.active') }}
                    </n-tag>
                  </div>
                </div>
              </div>

              <div class="template-card-body">
                <div class="info-row">
                  <n-icon size="14"><TimeOutline /></n-icon>
                  <span class="info-text">
                    {{ tpl.updated_at ? formatTime(tpl.updated_at) : t('templateMarket.builtin') }}
                  </span>
                </div>
                <div class="info-row desc-row" :title="tpl.description || ''">
                  <n-icon size="14"><InformationCircleOutline /></n-icon>
                  <span class="info-text">{{
                    tpl.description || t('templateMarket.noDescription')
                  }}</span>
                </div>
              </div>

              <div class="template-card-footer">
                <n-button
                  v-if="tpl.id !== store.settings.active_template_id"
                  size="small"
                  type="primary"
                  secondary
                  block
                  :loading="activatingId === tpl.id"
                  @click="activateTemplate(tpl)"
                >
                  {{ t('templateMarket.activate') }}
                </n-button>
                <n-button v-else size="small" type="success" secondary block disabled>
                  {{ t('templateMarket.inUse') }}
                </n-button>
                <div class="card-actions">
                  <n-button
                    v-if="tpl.id !== OFFICIAL_TEMPLATE_ID"
                    text
                    size="small"
                    @click="openEditModal(tpl)"
                  >
                    {{ t('templateMarket.edit') }}
                  </n-button>
                  <n-button
                    v-if="tpl.id !== OFFICIAL_TEMPLATE_ID && !tpl.remote_id"
                    text
                    size="small"
                    @click="openPublishModal(tpl)"
                  >
                    {{ t('templateMarket.publish') }}
                  </n-button>
                  <n-button
                    v-if="tpl.id !== OFFICIAL_TEMPLATE_ID"
                    text
                    size="small"
                    type="error"
                    @click="confirmDeleteTemplate(tpl)"
                  >
                    {{ t('templateMarket.delete') }}
                  </n-button>
                </div>
              </div>
            </div>
          </div>
        </n-tab-pane>

        <!-- 模板市场 -->
        <n-tab-pane name="market" :tab="t('templateMarket.tabMarket')">
          <template v-if="store.settings.service_url">
            <div class="market-toolbar">
              <n-input
                v-model:value="marketSearch"
                :placeholder="t('templateMarket.marketSearch')"
                clearable
                @keyup.enter="loadMarket(true)"
                @clear="loadMarket(true)"
              />
              <n-select
                v-model:value="marketSort"
                :options="sortOptions"
                class="sort-select"
                @update:value="loadMarket(true)"
              />
              <n-button secondary @click="loadMarket(true)">
                <template #icon>
                  <n-icon><RefreshOutline /></n-icon>
                </template>
                {{ t('templateMarket.refresh') }}
              </n-button>
            </div>

            <n-spin :show="marketLoading">
              <div v-if="marketItems.length > 0" class="template-grid">
                <div v-for="item in marketItems" :key="item.id" class="template-card">
                  <div class="template-card-header">
                    <div class="template-icon">
                      <n-icon size="20"><CloudDownloadOutline /></n-icon>
                    </div>
                    <div class="template-info">
                      <div class="template-name" :title="item.name">{{ item.name }}</div>
                      <div class="template-tags">
                        <n-tag size="small" :bordered="false" round>
                          {{ item.author_name || t('templateMarket.anonymous') }}
                        </n-tag>
                      </div>
                    </div>
                  </div>
                  <div class="template-card-body">
                    <div class="info-row desc-row" :title="item.description">
                      <n-icon size="14"><InformationCircleOutline /></n-icon>
                      <span class="info-text">{{
                        item.description || t('templateMarket.noDescription')
                      }}</span>
                    </div>
                    <div class="info-row">
                      <n-icon size="14"><DownloadOutline /></n-icon>
                      <span class="info-text">{{ item.downloads }}</span>
                      <n-icon size="14" class="gap-left"><TimeOutline /></n-icon>
                      <span class="info-text">{{
                        item.updated_at ? formatIsoTime(item.updated_at) : '-'
                      }}</span>
                    </div>
                  </div>
                  <div class="template-card-footer">
                    <n-button
                      size="small"
                      type="primary"
                      secondary
                      block
                      :loading="downloadingId === item.id"
                      @click="downloadMarketTemplate(item)"
                    >
                      {{ t('templateMarket.downloadUse') }}
                    </n-button>
                    <div class="card-actions">
                      <n-button text size="small" @click="openMarketDetail(item.id)">
                        {{ t('templateMarket.detail') }}
                      </n-button>
                    </div>
                  </div>
                </div>
              </div>
              <EmptyState
                v-else-if="!marketLoading"
                :title="t('templateMarket.marketEmpty')"
                :description="t('templateMarket.marketEmptyDesc')"
                :icon="CloudDownloadOutline"
              />
            </n-spin>
          </template>
          <EmptyState
            v-else
            :title="t('templateMarket.serviceNotConfigured')"
            :description="t('templateMarket.serviceNotConfiguredDesc')"
            :icon="CloudOutline"
          >
            <template #action>
              <n-button type="primary" @click="openSettingsModal">
                {{ t('templateMarket.configureService') }}
              </n-button>
            </template>
          </EmptyState>
        </n-tab-pane>

        <!-- 我的发布 -->
        <n-tab-pane name="mine" :tab="t('templateMarket.tabMine')">
          <template v-if="store.myPublishedTemplates.length > 0">
            <div class="market-toolbar">
              <n-button secondary :loading="refreshingMine" @click="refreshMine">
                <template #icon>
                  <n-icon><RefreshOutline /></n-icon>
                </template>
                {{ t('templateMarket.refreshStatus') }}
              </n-button>
            </div>
            <div class="mine-list">
              <div v-for="tpl in store.myPublishedTemplates" :key="tpl.id" class="mine-item">
                <div class="mine-info">
                  <div class="template-name">{{ tpl.name }}</div>
                  <n-alert
                    v-if="tpl.market_status === 'rejected' && tpl.market_reject_reason"
                    type="error"
                    :show-icon="false"
                    class="reject-reason"
                  >
                    {{ t('templateMarket.rejectReason') }}: {{ tpl.market_reject_reason }}
                  </n-alert>
                </div>
                <div class="mine-actions">
                  <n-tag
                    size="small"
                    :bordered="false"
                    round
                    :type="marketStatusType(tpl.market_status || '')"
                  >
                    {{ marketStatusLabel(tpl.market_status || '') }}
                  </n-tag>
                  <n-button
                    size="tiny"
                    secondary
                    :loading="updatingId === tpl.id"
                    @click="updateMarketTemplate(tpl)"
                  >
                    {{ t('templateMarket.updatePublish') }}
                  </n-button>
                  <n-button
                    size="tiny"
                    secondary
                    type="error"
                    :loading="removingId === tpl.id"
                    @click="removeMarketTemplate(tpl)"
                  >
                    {{ t('templateMarket.deleteFromMarket') }}
                  </n-button>
                </div>
              </div>
            </div>
            <n-alert type="warning" :show-icon="false" class="token-hint">
              {{ t('templateMarket.tokenHint') }}
            </n-alert>
          </template>
          <EmptyState
            v-else
            :title="t('templateMarket.mineEmpty')"
            :description="t('templateMarket.mineEmptyDesc')"
            :icon="CloudUploadOutline"
          />
        </n-tab-pane>
      </n-tabs>
    </n-card>

    <!-- 模板编辑弹窗 -->
    <n-modal
      v-model:show="showEditModal"
      preset="card"
      class="template-modal"
      :title="editingId ? t('templateMarket.editTemplate') : t('templateMarket.newTemplate')"
    >
      <n-form :model="editForm" label-placement="top">
        <n-form-item :label="t('templateMarket.name')" required>
          <n-input v-model:value="editForm.name" maxlength="80" />
        </n-form-item>
        <n-form-item :label="t('templateMarket.description')">
          <n-input
            v-model:value="editForm.description"
            type="textarea"
            :rows="2"
            maxlength="2000"
          />
        </n-form-item>
        <div class="form-row">
          <n-form-item :label="t('templateMarket.author')">
            <n-input v-model:value="editForm.author" maxlength="60" />
          </n-form-item>
          <n-form-item :label="t('templateMarket.schemaVersion')">
            <n-input v-model:value="editForm.schema_version" placeholder="1.14" />
          </n-form-item>
        </div>
      </n-form>

      <n-alert v-if="!editorSupported" type="warning" :show-icon="false" class="mode-hint">
        {{ t('templateMarket.visualUnsupported') }}
      </n-alert>
      <n-tabs v-else v-model:value="editorMode" type="segment" size="small" animated>
        <n-tab-pane name="visual" :tab="t('templateMarket.modeVisual')">
          <div class="visual-form">
            <n-form label-placement="top" size="small">
              <div class="form-row">
                <n-form-item :label="t('templateMarket.defaultOutbound')">
                  <n-select v-model:value="formModel.default_outbound" :options="outboundOptions" />
                </n-form-item>
                <n-form-item :label="t('templateMarket.urltestUrl')">
                  <n-input
                    v-model:value="formModel.urltest_url"
                    :placeholder="t('templateMarket.followAppSetting')"
                  />
                </n-form-item>
              </div>

              <div class="section-title">{{ t('templateMarket.sectionGroups') }}</div>
              <n-checkbox-group v-model:value="formModel.app_groups" class="group-checks">
                <n-checkbox
                  v-for="g in groupOptions"
                  :key="g.value"
                  :value="g.value"
                  :label="g.label"
                />
              </n-checkbox-group>

              <div class="section-title">{{ t('templateMarket.sectionDns') }}</div>
              <div class="switch-grid">
                <div class="switch-item">
                  <n-checkbox v-model:checked="formModel.block_ads">{{
                    t('templateMarket.blockAds')
                  }}</n-checkbox>
                </div>
                <div class="switch-item">
                  <n-checkbox v-model:checked="formModel.dns_hijack">{{
                    t('templateMarket.dnsHijack')
                  }}</n-checkbox>
                </div>
                <div class="switch-item">
                  <n-checkbox v-model:checked="formModel.dns_use_mdns">{{
                    t('templateMarket.dnsMdns')
                  }}</n-checkbox>
                </div>
                <div class="switch-item">
                  <n-checkbox v-model:checked="formModel.fake_dns_enabled">{{
                    t('templateMarket.fakeDns')
                  }}</n-checkbox>
                </div>
              </div>
              <div class="form-row">
                <n-form-item :label="t('templateMarket.dnsProxy')">
                  <n-input
                    v-model:value="formModel.dns_proxy"
                    :placeholder="t('templateMarket.followAppSetting')"
                  />
                </n-form-item>
                <n-form-item :label="t('templateMarket.dnsCn')">
                  <n-input
                    v-model:value="formModel.dns_cn"
                    :placeholder="t('templateMarket.followAppSetting')"
                  />
                </n-form-item>
              </div>
              <n-form-item :label="t('templateMarket.dnsResolver')">
                <n-input
                  v-model:value="formModel.dns_resolver"
                  :placeholder="t('templateMarket.followAppSetting')"
                />
              </n-form-item>

              <div class="section-title">{{ t('templateMarket.sectionRules') }}</div>
              <div v-for="(rule, idx) in formModel.custom_rules" :key="rule.id" class="rule-row">
                <n-select
                  v-model:value="rule.match_type"
                  :options="matchTypeOptions"
                  class="rule-match"
                  size="small"
                />
                <n-input
                  v-model:value="rule.payload"
                  size="small"
                  :placeholder="t('templateMarket.rulePayloadHint')"
                />
                <n-select
                  v-model:value="rule.action"
                  :options="actionOptions"
                  class="rule-action"
                  size="small"
                />
                <n-button text type="error" size="small" @click="removeRule(idx)">
                  {{ t('templateMarket.delete') }}
                </n-button>
              </div>
              <n-button size="small" secondary @click="addRule">
                <template #icon>
                  <n-icon><AddOutline /></n-icon>
                </template>
                {{ t('templateMarket.addRule') }}
              </n-button>
              <p class="form-hint">{{ t('templateMarket.rulePriorityHint') }}</p>
            </n-form>
          </div>
        </n-tab-pane>
        <n-tab-pane name="json" :tab="t('templateMarket.modeJson')">
          <n-form label-placement="top">
            <n-form-item :label="t('templateMarket.content')" required>
              <n-input
                v-model:value="editForm.content"
                type="textarea"
                class="content-editor"
                :rows="16"
                monospace
                :placeholder="t('templateMarket.contentPlaceholder', { marker: NODES_MARKER })"
              />
            </n-form-item>
          </n-form>
        </n-tab-pane>
      </n-tabs>
      <template #footer>
        <div class="modal-footer">
          <n-button v-if="!editingId" secondary @click="fillFromOfficial">
            {{ t('templateMarket.exportOfficial') }}
          </n-button>
          <div class="footer-spacer" />
          <n-button @click="showEditModal = false">{{ t('templateMarket.cancel') }}</n-button>
          <n-button type="primary" :loading="saving" @click="saveTemplate">
            {{ t('templateMarket.save') }}
          </n-button>
        </div>
      </template>
    </n-modal>

    <!-- 发布弹窗 -->
    <n-modal
      v-model:show="showPublishModal"
      preset="dialog"
      type="info"
      :title="t('templateMarket.publishToMarket')"
      :positive-text="t('templateMarket.publish')"
      :negative-text="t('templateMarket.cancel')"
      :loading="publishing"
      @positive-click="submitPublish"
      @negative-click="showPublishModal = false"
    >
      <n-form label-placement="top">
        <n-form-item :label="t('templateMarket.author')">
          <n-input
            v-model:value="publishAuthor"
            maxlength="60"
            :placeholder="t('templateMarket.authorPlaceholder')"
          />
        </n-form-item>
      </n-form>
      <n-alert type="warning" :show-icon="false">
        {{ t('templateMarket.publishWarning') }}
      </n-alert>
    </n-modal>

    <!-- 市场模板详情弹窗 -->
    <n-modal
      v-model:show="showDetailModal"
      preset="card"
      class="template-modal"
      :title="marketDetail?.name || t('templateMarket.detail')"
    >
      <template v-if="marketDetail">
        <div class="detail-meta">
          <n-tag size="small" :bordered="false" round>
            {{ marketDetail.author_name || t('templateMarket.anonymous') }}
          </n-tag>
          <n-tag size="small" :bordered="false" round>
            {{ marketDetail.downloads }} {{ t('templateMarket.downloadsUnit') }}
          </n-tag>
          <n-tag v-if="marketDetail.schema_version" size="small" :bordered="false" round>
            sing-box {{ marketDetail.schema_version }}
          </n-tag>
        </div>
        <p class="detail-desc">
          {{ marketDetail.description || t('templateMarket.noDescription') }}
        </p>
        <n-input
          :value="marketDetail.content || ''"
          type="textarea"
          class="content-editor"
          :rows="14"
          readonly
        />
      </template>
      <n-spin v-else style="width: 100%" />
    </n-modal>

    <!-- 服务设置弹窗 -->
    <n-modal
      v-model:show="showSettingsModal"
      preset="dialog"
      type="info"
      :title="t('templateMarket.serviceSettings')"
      :positive-text="t('templateMarket.save')"
      :negative-text="t('templateMarket.cancel')"
      :loading="savingSettings"
      @positive-click="submitSettings"
      @negative-click="showSettingsModal = false"
    >
      <n-form label-placement="top">
        <n-form-item :label="t('templateMarket.serviceUrl')">
          <n-input
            v-model:value="settingsForm.service_url"
            :placeholder="t('templateMarket.serviceUrlPlaceholder')"
          />
        </n-form-item>
        <p class="form-hint">{{ t('templateMarket.serviceUrlHint') }}</p>
      </n-form>
      <n-button secondary :loading="testing" @click="testService">
        {{ t('templateMarket.testConnection') }}
      </n-button>
      <n-alert
        v-if="testResult"
        :type="testResult.ok ? 'success' : 'error'"
        :show-icon="false"
        class="test-result"
      >
        {{ testResult.text }}
      </n-alert>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDialog, useMessage } from 'naive-ui'
import {
  AddOutline,
  CloudDownloadOutline,
  CloudOutline,
  CloudUploadOutline,
  DocumentTextOutline,
  DownloadOutline,
  InformationCircleOutline,
  RefreshOutline,
  ShieldCheckmarkOutline,
  TimeOutline,
} from '@vicons/ionicons5'
import PageHeader from '@/components/common/PageHeader.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import { templateMarketService } from '@/services/template-market-service'
import { OFFICIAL_TEMPLATE_ID, useTemplateStore } from '@/stores/template/TemplateStore'
import type {
  ConfigTemplate,
  CustomRule,
  MarketTemplate,
  TemplateFormOptions,
} from '@/types/generated'

const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()

/** 节点占位符标记（i18n 消息中以 {marker} 插值引用，避免消息内出现字面量大括号） */
const NODES_MARKER = '"{{NODES}}"'

const store = useTemplateStore()

const activeTab = ref('current')

// --- 当前模板 ---
const activatingId = ref<string | null>(null)

// --- 模板市场 ---
const marketItems = ref<MarketTemplate[]>([])
const marketLoading = ref(false)
const marketSearch = ref('')
const marketSort = ref<'newest' | 'downloads'>('newest')
const downloadingId = ref<string | null>(null)
const sortOptions = [
  { label: t('templateMarket.marketSortNewest'), value: 'newest' },
  { label: t('templateMarket.marketSortDownloads'), value: 'downloads' },
]

// --- 我的发布 ---
const refreshingMine = ref(false)
const updatingId = ref<string | null>(null)
const removingId = ref<string | null>(null)

// --- 编辑弹窗 ---
const showEditModal = ref(false)
const editingId = ref<string | null>(null)
const saving = ref(false)
/** 编辑模式：visual = 可视化表单，json = 原始 JSON */
const editorMode = ref<'visual' | 'json'>('visual')
/** 当前模板是否可用可视化表单表达（解析失败时仅 JSON 模式） */
const editorSupported = ref(true)
const formModel = ref<TemplateFormOptions>(defaultFormModel())

function defaultFormModel(): TemplateFormOptions {
  return {
    default_outbound: 'manual',
    urltest_url: '',
    block_ads: false,
    dns_hijack: true,
    fake_dns_enabled: false,
    dns_use_mdns: true,
    app_groups: ['Telegram', 'YouTube', 'Netflix', 'OpenAI', 'Google'],
    dns_proxy: '',
    dns_cn: '',
    dns_resolver: '',
    custom_rules: [],
  }
}

const outboundOptions = [
  { label: '手动切换', value: 'manual' },
  { label: '自动选择', value: 'auto' },
]
const groupOptions = ['Telegram', 'YouTube', 'Netflix', 'OpenAI', 'Google'].map((g) => ({
  label: g,
  value: g,
}))
const matchTypeOptions = [
  { label: '域名后缀', value: 'domain_suffix' },
  { label: '精确域名', value: 'domain' },
  { label: '域名关键字', value: 'domain_keyword' },
  { label: 'IP / CIDR', value: 'ip_cidr' },
]
const actionOptions = [
  { label: '走代理', value: 'proxy' },
  { label: '直连', value: 'direct' },
  { label: '拦截', value: 'block' },
]

function addRule() {
  const now = new Date().toISOString()
  const rule: CustomRule = {
    id: crypto.randomUUID
      ? crypto.randomUUID()
      : `${Date.now()}-${formModel.value.custom_rules.length}`,
    enabled: true,
    match_type: 'domain_suffix',
    payload: '',
    action: 'proxy',
    outbound: null,
    note: null,
    created_at: now,
    updated_at: now,
  }
  formModel.value.custom_rules.push(rule)
}

function removeRule(idx: number) {
  formModel.value.custom_rules.splice(idx, 1)
}
const editForm = ref({
  id: '',
  name: '',
  description: '',
  author: '',
  schema_version: '',
  content: '',
})

// --- 发布弹窗 ---
const showPublishModal = ref(false)
const publishing = ref(false)
const publishTargetId = ref('')
const publishAuthor = ref('')

// --- 详情弹窗 ---
const showDetailModal = ref(false)
const marketDetail = ref<MarketTemplate | null>(null)

// --- 设置弹窗 ---
const showSettingsModal = ref(false)
const savingSettings = ref(false)
const testing = ref(false)
const testResult = ref<{ ok: boolean; text: string } | null>(null)
const settingsForm = ref({ service_url: '' })

onMounted(() => {
  store.initializeStore().catch((e) => message.error(String(e)))
})

// === 展示辅助 ===

function activeDisplayName(tpl: ConfigTemplate | null): string {
  if (!tpl) return t('templateMarket.officialName')
  return tpl.id === OFFICIAL_TEMPLATE_ID ? t('templateMarket.officialName') : tpl.name
}

function sourceLabel(source: string): string {
  if (source === 'builtin') return t('templateMarket.sourceBuiltin')
  if (source === 'market') return t('templateMarket.sourceMarket')
  return t('templateMarket.sourceLocal')
}

function marketStatusLabel(status: string): string {
  const map: Record<string, string> = {
    pending: t('templateMarket.marketStatusPending'),
    approved: t('templateMarket.marketStatusApproved'),
    rejected: t('templateMarket.marketStatusRejected'),
    deleted: t('templateMarket.marketStatusDeleted'),
  }
  return map[status] ?? status
}

function marketStatusType(status: string): 'default' | 'warning' | 'success' | 'error' {
  if (status === 'approved') return 'success'
  if (status === 'pending') return 'warning'
  if (status === 'rejected') return 'error'
  return 'default'
}

function formatTime(ms: number): string {
  return new Date(ms).toLocaleString()
}

function formatIsoTime(iso: string): string {
  const date = new Date(iso)
  return Number.isNaN(date.getTime()) ? iso : date.toLocaleString()
}

// === 当前模板 ===

async function activateTemplate(tpl: ConfigTemplate) {
  activatingId.value = tpl.id
  try {
    await store.setActiveTemplate(tpl.id)
    message.success(
      t('templateMarket.activatedToast', {
        name: activeDisplayName(tpl),
      }),
    )
    message.info(t('templateMarket.activateEffectHint'), { duration: 5000 })
  } catch (e) {
    message.error(String(e))
  } finally {
    activatingId.value = null
  }
}

function confirmDeleteTemplate(tpl: ConfigTemplate) {
  dialog.warning({
    title: t('templateMarket.deleteConfirmTitle'),
    content: t('templateMarket.deleteConfirmBody', { name: tpl.name }),
    positiveText: t('templateMarket.delete'),
    negativeText: t('templateMarket.cancel'),
    onPositiveClick: async () => {
      try {
        await store.deleteTemplate(tpl.id)
        message.success(t('templateMarket.deletedToast'))
      } catch (e) {
        message.error(String(e))
      }
    },
  })
}

// === 编辑 ===

function openCreateModal() {
  editingId.value = null
  editForm.value = {
    id: '',
    name: '',
    description: '',
    author: '',
    schema_version: '',
    content: '',
  }
  formModel.value = defaultFormModel()
  editorMode.value = 'visual'
  editorSupported.value = true
  showEditModal.value = true
}

async function openEditModal(tpl: ConfigTemplate) {
  editingId.value = tpl.id
  editForm.value = {
    id: tpl.id,
    name: tpl.name,
    description: tpl.description ?? '',
    author: tpl.author ?? '',
    schema_version: tpl.schema_version ?? '',
    content: tpl.content,
  }
  // 尝试解析回可视化表单；失败则仅提供 JSON 模式
  try {
    formModel.value = await templateMarketService.parseTemplateForm(tpl.content)
    editorSupported.value = true
    editorMode.value = 'visual'
  } catch {
    formModel.value = defaultFormModel()
    editorSupported.value = false
    editorMode.value = 'json'
  }
  showEditModal.value = true
}

async function fillFromOfficial() {
  try {
    editForm.value.content = await templateMarketService.exportOfficialTemplate()
  } catch (e) {
    message.error(String(e))
  }
}

function validateContent(content: string): boolean {
  try {
    const parsed = JSON.parse(content) as unknown
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      message.error(t('templateMarket.contentMustBeObject'))
      return false
    }
    const outbounds = (parsed as Record<string, unknown>).outbounds
    if (!Array.isArray(outbounds) || outbounds.length === 0) {
      message.error(t('templateMarket.contentNeedsOutbounds'))
      return false
    }
    return true
  } catch {
    message.error(t('templateMarket.contentInvalid'))
    return false
  }
}

async function saveTemplate() {
  if (!editForm.value.name.trim()) {
    message.error(t('templateMarket.nameRequired'))
    return
  }

  let content = editForm.value.content
  if (editorSupported.value && editorMode.value === 'visual') {
    // 可视化模式：表单 → 官方骨架 JSON（生成的配置天然合法，无需再手写校验）
    try {
      content = await templateMarketService.generateTemplateFromForm(formModel.value)
    } catch (e) {
      message.error(String(e))
      return
    }
  } else if (!validateContent(content)) {
    return
  }

  saving.value = true
  try {
    await store.saveTemplate({
      id: editingId.value ?? '',
      name: editForm.value.name.trim(),
      kernel_type: 'singbox',
      content,
      source: 'local',
      description: editForm.value.description || null,
      author: editForm.value.author || null,
      remote_id: null,
      edit_token: null,
      market_status: null,
      market_reject_reason: null,
      schema_version: editForm.value.schema_version || null,
      revision: 1,
      created_at: 0,
      updated_at: 0,
    })
    showEditModal.value = false
    message.success(t('templateMarket.savedToast'))
  } catch (e) {
    message.error(String(e))
  } finally {
    saving.value = false
  }
}

// === 发布 ===

function openPublishModal(tpl: ConfigTemplate) {
  publishTargetId.value = tpl.id
  publishAuthor.value = tpl.author ?? ''
  showPublishModal.value = true
}

async function submitPublish() {
  publishing.value = true
  try {
    await store.publishTemplate(publishTargetId.value, publishAuthor.value || undefined)
    showPublishModal.value = false
    message.success(t('templateMarket.publishedToast'))
  } catch (e) {
    message.error(String(e))
    return false
  } finally {
    publishing.value = false
  }
  return true
}

// === 市场列表 ===

async function loadMarket(reset = false) {
  marketLoading.value = true
  try {
    const result = await templateMarketService.marketListTemplates({
      search: marketSearch.value || undefined,
      sort: marketSort.value,
      page: reset ? 1 : undefined,
      pageSize: 50,
    })
    marketItems.value = result.items
  } catch (e) {
    message.error(String(e))
  } finally {
    marketLoading.value = false
  }
}

async function downloadMarketTemplate(item: MarketTemplate) {
  downloadingId.value = item.id
  try {
    await store.downloadMarketTemplate(item.id)
    message.success(t('templateMarket.downloadedToast', { name: item.name }))
    activeTab.value = 'current'
  } catch (e) {
    message.error(String(e))
  } finally {
    downloadingId.value = null
  }
}

async function openMarketDetail(remoteId: string) {
  showDetailModal.value = true
  marketDetail.value = null
  try {
    marketDetail.value = await templateMarketService.marketGetTemplateDetail(remoteId)
  } catch (e) {
    showDetailModal.value = false
    message.error(String(e))
  }
}

// === 我的发布 ===

async function refreshMine() {
  refreshingMine.value = true
  try {
    await store.refreshMyTemplates()
    message.success(t('templateMarket.statusRefreshed'))
  } catch (e) {
    message.error(String(e))
  } finally {
    refreshingMine.value = false
  }
}

async function updateMarketTemplate(tpl: ConfigTemplate) {
  updatingId.value = tpl.id
  try {
    await store.updateMarketTemplate(tpl.id)
    message.success(t('templateMarket.resubmittedToast'))
  } catch (e) {
    message.error(String(e))
  } finally {
    updatingId.value = null
  }
}

function removeMarketTemplate(tpl: ConfigTemplate) {
  dialog.warning({
    title: t('templateMarket.deleteFromMarketTitle'),
    content: t('templateMarket.deleteFromMarketBody', { name: tpl.name }),
    positiveText: t('templateMarket.delete'),
    negativeText: t('templateMarket.cancel'),
    onPositiveClick: async () => {
      removingId.value = tpl.id
      try {
        await store.deleteMarketTemplate(tpl.id)
        message.success(t('templateMarket.removedFromMarketToast'))
      } catch (e) {
        message.error(String(e))
      } finally {
        removingId.value = null
      }
    },
  })
}

// === 服务设置 ===

function openSettingsModal() {
  settingsForm.value.service_url = store.settings.service_url
  testResult.value = null
  showSettingsModal.value = true
}

async function testService() {
  testing.value = true
  testResult.value = null
  try {
    const health = await templateMarketService.marketCheckService(
      settingsForm.value.service_url || undefined,
    )
    testResult.value = {
      ok: true,
      text: t('templateMarket.testOk', { version: health.version }),
    }
  } catch (e) {
    testResult.value = { ok: false, text: String(e) }
  } finally {
    testing.value = false
  }
}

async function submitSettings() {
  savingSettings.value = true
  try {
    await store.saveSettings({
      service_url: settingsForm.value.service_url.trim(),
      active_template_id: store.settings.active_template_id,
    })
    showSettingsModal.value = false
    message.success(t('templateMarket.settingsSaved'))
    if (activeTab.value === 'market') await loadMarket(true)
  } catch (e) {
    message.error(String(e))
    return false
  } finally {
    savingSettings.value = false
  }
  return true
}

// 切到市场 Tab 时懒加载列表
watch(activeTab, (tab) => {
  if (tab === 'market' && marketItems.value.length === 0 && store.settings.service_url) {
    loadMarket(true)
  }
})
</script>

<style scoped>
.template-market-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow-y: auto;
}

.market-card {
  flex: 1;
}

.active-hint {
  margin-bottom: 12px;
}

.template-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 14px;
}

.template-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 14px;
  border: 1px solid var(--n-border-color, rgba(128, 128, 128, 0.2));
  border-radius: 10px;
  transition: border-color 0.2s;
}

.template-card.active {
  border-color: rgba(99, 102, 241, 0.6);
}

.template-card-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.template-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: rgba(99, 102, 241, 0.12);
  color: rgba(99, 102, 241, 0.9);
  flex-shrink: 0;
}

.template-info {
  min-width: 0;
}

.template-name {
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.template-tags {
  display: flex;
  gap: 6px;
  margin-top: 4px;
  flex-wrap: wrap;
}

.template-card-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-height: 40px;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--n-text-color-3, #888);
  min-width: 0;
}

.info-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.desc-row .info-text {
  white-space: normal;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.gap-left {
  margin-left: 8px;
}

.template-card-footer {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: auto;
}

.card-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.market-toolbar {
  display: flex;
  gap: 10px;
  margin-bottom: 14px;
}

.sort-select {
  width: 150px;
}

.mine-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 12px;
}

.mine-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border: 1px solid var(--n-border-color, rgba(128, 128, 128, 0.2));
  border-radius: 10px;
}

.mine-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.mine-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.reject-reason {
  font-size: 12px;
}

.token-hint {
  font-size: 12px;
}

.template-modal {
  width: 720px;
  max-width: 92vw;
}

.content-editor :deep(textarea) {
  font-family: 'Cascadia Code', Consolas, monospace;
  font-size: 12px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.mode-hint {
  margin-bottom: 10px;
}

.visual-form {
  max-height: 54vh;
  overflow-y: auto;
  padding-right: 4px;
}

.section-title {
  font-weight: 600;
  font-size: 13px;
  margin: 14px 0 8px;
}

.group-checks {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 4px;
}

.switch-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  margin-bottom: 10px;
}

.switch-item {
  border: 1px solid rgba(128, 128, 128, 0.25);
  border-radius: 8px;
  padding: 8px 10px;
}

.rule-row {
  display: grid;
  grid-template-columns: 130px 1fr 104px auto;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
}

.form-hint {
  font-size: 12px;
  color: var(--n-text-color-3, #888);
  margin: 8px 0 0;
}

.modal-footer {
  display: flex;
  align-items: center;
  gap: 10px;
}

.footer-spacer {
  flex: 1;
}

.detail-meta {
  display: flex;
  gap: 8px;
  margin-bottom: 10px;
}

.detail-desc {
  color: var(--n-text-color-3, #888);
  font-size: 13px;
  margin: 0 0 10px;
}

.test-result {
  margin-top: 10px;
  font-size: 12px;
}
</style>
