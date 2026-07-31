<template>
  <div class="page-shell sub-page">
    <PageHeader :title="t('sub.title')" :subtitle="t('sub.subtitle')">
      <template #actions>
        <n-button type="primary" @click="showAddModal = true" round>
          <template #icon>
            <n-icon><AddOutline /></n-icon>
          </template>
          {{ t('sub.add') }}
        </n-button>
      </template>
    </PageHeader>

    <!-- Subscription List -->
    <div class="subscription-section">
      <div v-if="subStore.list.length > 0" class="subscription-grid">
        <div
          v-for="(item, index) in subStore.list"
          :key="index"
          class="sub-card"
          :class="{ active: subStore.activeIndex === index }"
        >
          <div class="sub-card-header">
            <div class="sub-icon" :class="{ active: subStore.activeIndex === index }">
              <n-icon size="20"><LinkOutline /></n-icon>
            </div>
            <div class="sub-info">
              <div class="sub-name" :title="item.name">{{ item.name }}</div>
              <div class="sub-tags">
                <n-tag size="small" :bordered="false" round>
                  {{ item.isManual ? t('sub.manual') : t('sub.urlSubscription') }}
                </n-tag>
                <n-tag
                  v-if="subStore.activeIndex === index"
                  type="success"
                  size="small"
                  :bordered="false"
                  round
                >
                  {{ t('sub.inUse') }}
                </n-tag>
                <n-tag
                  v-if="!item.isManual && (item.autoUpdateIntervalMinutes ?? DEFAULT_AUTO_UPDATE_MINUTES) > 0"
                  size="small"
                  round
                  :bordered="false"
                  type="info"
                >
                  <template #icon>
                    <n-icon size="14"><TimerOutline /></n-icon>
                  </template>
                  {{ formatIntervalLabel(item.autoUpdateIntervalMinutes) }}
                </n-tag>
              </div>
            </div>
            <n-dropdown
              trigger="hover"
              placement="bottom-end"
              :options="getDropdownOptions(index)"
            >
              <n-button text class="more-btn">
                <n-icon size="20"><EllipsisVerticalOutline /></n-icon>
              </n-button>
            </n-dropdown>
          </div>

          <div class="sub-card-body">
            <div class="info-row" :title="item.url || t('sub.manualContent')">
              <n-icon size="14"><GlobeOutline /></n-icon>
              <span class="info-text">{{ item.url || t('sub.manualContent') }}</span>
            </div>
            <div class="info-row">
              <n-icon size="14"><TimeOutline /></n-icon>
              <span class="info-text">
                {{ item.lastUpdate ? formatTime(item.lastUpdate) : t('sub.neverUsed') }}
              </span>
            </div>
            <div v-if="hasSubscriptionTraffic(item)" class="info-row">
              <n-icon size="14"><StatsChartOutline /></n-icon>
              <span class="info-text">{{ formatTrafficSummary(item) }}</span>
            </div>
            <div v-if="item.subscriptionExpire" class="info-row">
              <n-icon size="14"><CalendarOutline /></n-icon>
              <span class="info-text">{{ formatExpireTime(item.subscriptionExpire) }}</span>
            </div>
            <div v-if="item.autoUpdateFailCount && item.autoUpdateFailCount > 0" class="info-row warn">
              <n-icon size="14"><AlertCircleOutline /></n-icon>
              <span class="info-text">{{ formatAutoUpdateHealth(item) }}</span>
            </div>
          </div>

          <div class="sub-card-footer">
            <n-button
              block
              :type="subStore.activeIndex === index ? 'success' : 'primary'"
              secondary
              :loading="item.isLoading"
              @click="useSubscription(index)"
            >
              <template #icon>
                <n-icon>
                  <CheckmarkCircleOutline v-if="subStore.activeIndex === index" />
                  <PlayCircleOutline v-else />
                </n-icon>
              </template>
              {{ subStore.activeIndex === index ? t('sub.useAgain') : t('sub.use') }}
            </n-button>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <EmptyState v-else :title="t('sub.noSubs')" :description="t('sub.noSubscriptionsYet')" :icon="LinkOutline">
        <template #action>
          <n-button type="primary" @click="showAddModal = true">
            {{ t('sub.addFirstSubscription') }}
          </n-button>
        </template>
      </EmptyState>
    </div>

    <!-- Add/Edit Modal -->
    <n-modal
      v-model:show="showAddModal"
      preset="dialog"
      :title="editIndex === null ? t('sub.add') : t('sub.edit')"
      class="modern-modal"
      :style="{ width: '500px' }"
      :mask-closable="false"
    >
      <n-form
        ref="formRef"
        :model="formValue"
        :rules="rules"
        label-placement="top"
        class="sub-form"
      >
        <n-form-item :label="t('sub.name')" path="name">
          <n-input v-model:value="formValue.name" :placeholder="t('sub.namePlaceholder')" />
        </n-form-item>

        <n-tabs type="segment" animated v-model:value="activeTab">
          <n-tab-pane name="url" :tab="t('sub.urlSubscription')">
            <n-form-item label="URL" path="url">
              <n-input
                v-model:value="formValue.url"
                type="textarea"
                :rows="3"
                :placeholder="t('sub.urlPlaceholder')"
              />
            </n-form-item>
            <p class="form-hint">{{ t('sub.urlHint') }}</p>
          </n-tab-pane>
          <n-tab-pane name="manual" :tab="t('sub.manualConfig')">
            <n-form-item :label="t('sub.content')" path="manualContent">        
              <n-input
                v-model:value="formValue.manualContent"
                type="textarea"
                :rows="6"
                :placeholder="t('sub.manualContentPlaceholder')"
                class="code-input"
              />
            </n-form-item>
            <p class="form-hint">{{ t('sub.manualHint') }}</p>
          </n-tab-pane>
          <n-tab-pane name="uri" :tab="t('sub.uriList')">
            <n-form-item :label="t('sub.uriContent')" path="uriContent">        
              <n-input
                v-model:value="formValue.uriContent"
                type="textarea"
                :rows="6"
                :placeholder="t('sub.uriContentPlaceholder')"
                class="code-input"
              />
            </n-form-item>
            <p class="form-hint">{{ t('sub.uriHint') }}</p>
          </n-tab-pane>
        </n-tabs>

        <div v-if="activeTab !== 'uri'" class="form-switch">
          <div class="switch-label">
            <span>{{ t('sub.useOriginalConfig') }}</span>
            <span class="switch-desc">{{ formValue.useOriginalConfig ? t('sub.useOriginal') : t('sub.useExtractedNodes') }}</span>
          </div>
          <n-switch v-model:value="formValue.useOriginalConfig" @update:value="markUseOriginalTouched" />
        </div>
        <p v-if="formValue.useOriginalConfig" class="form-hint warning">
          {{ t('sub.originalConfigWarning') }}
        </p>

        <n-form-item :label="t('sub.autoUpdate')" path="autoUpdateIntervalMinutes">
          <n-select
            v-model:value="formValue.autoUpdateIntervalMinutes"
            :options="autoUpdateOptions"
            size="small"
            :disabled="autoUpdateDisabled"
          />
          <p v-if="autoUpdateDisabled" class="form-hint">
            {{ t('sub.autoUpdateManualHint') }}
          </p>
        </n-form-item>
      </n-form>

      <template #action>
        <n-space justify="end">
          <n-button @click="handleCancel">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" @click="handleConfirm" :loading="isLoading">
            {{ t('common.confirm') }}
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- Config Editor Modal -->
    <n-modal
      v-model:show="showConfigModal"
      preset="dialog"
      :title="t('sub.editCurrentConfig')"
      class="modern-modal"
      :style="{ width: '800px' }"
      :mask-closable="false"
    >
      <n-input
        v-model:value="currentConfig"
        type="textarea"
        :rows="20"
        class="code-input"
        placeholder="JSON Config..."
      />
      <template #action>
        <n-space justify="end">
          <n-button @click="showConfigModal = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" @click="saveCurrentConfig" :loading="isConfigLoading">
            {{ t('sub.saveAndApply') }}
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, h, watch } from 'vue'
import { useMessage } from 'naive-ui'
import { useSubStore } from '@/stores/subscription/SubStore'
import { useAppStore } from '@/stores'
import { subscriptionService } from '@/services/subscription-service'
import type { SubscriptionPersistResult } from '@/services/subscription-service'
import { kernelService } from '@/services/kernel-service'
import { useI18n } from 'vue-i18n'
import { DEFAULT_AUTO_UPDATE_MINUTES, type FrontendSubscription } from '@/stores/subscription/types'
import {
  formatExpireTime as formatExpireTimeText,
  formatIntervalLabel as formatIntervalLabelText,
  formatAutoUpdateHealth as formatAutoUpdateHealthText,
  formatLocalTime,
  formatTrafficSummary as formatTrafficSummaryText,
  generateConfigFileName,
  hasSubscriptionTraffic,
  isJsonContent,
} from '@/views/sub/subscription-utils'
import { useSubscriptionAutoUpdate } from '@/views/sub/useSubscriptionAutoUpdate'
import {
  AddOutline,
  LinkOutline,
  CopyOutline,
  CreateOutline,
  TrashOutline,
  CheckmarkCircleOutline,
  PlayCircleOutline,
  CodeOutline,
  EllipsisVerticalOutline,
  GlobeOutline,
  TimeOutline,
  StatsChartOutline,
  CalendarOutline,
  AlertCircleOutline,
  RefreshOutline,
  ArrowUndoOutline,
  TimerOutline,
} from '@vicons/ionicons5'
import type { FormInst, FormRules, DropdownOption } from 'naive-ui'
import PageHeader from '@/components/common/PageHeader.vue'
import EmptyState from '@/components/common/EmptyState.vue'

defineOptions({
  name: 'SubView'
})

type Subscription = FrontendSubscription

interface SubscriptionForm extends Subscription {
  uriContent?: string
}

const message = useMessage()
const subStore = useSubStore()
const appStore = useAppStore()
const { t } = useI18n()

const showAddModal = ref(false)
const editIndex = ref<number | null>(null)
const formRef = ref<FormInst | null>(null)
const isLoading = ref(false)
const activeTab = ref('url')
const useOriginalTouched = ref(false)

const showConfigModal = ref(false)
const currentConfig = ref('')
const isConfigLoading = ref(false)

const formValue = ref<SubscriptionForm>({
  name: '',
  url: '',
  isLoading: false,
  isManual: false,
  manualContent: '',
  uriContent: '',
  useOriginalConfig: false,
  autoUpdateIntervalMinutes: DEFAULT_AUTO_UPDATE_MINUTES,
})

const autoUpdateOptions = computed(() => [
  { label: t('sub.autoUpdateOff'), value: 0 },
  { label: t('sub.autoUpdate6h'), value: 360 },
  { label: t('sub.autoUpdate12h'), value: 720 },
  { label: t('sub.autoUpdate1d'), value: 1440 },
])

const autoUpdateDisabled = computed(() => activeTab.value !== 'url')

const markUseOriginalTouched = () => {
  useOriginalTouched.value = true
}

const rules: FormRules = {
  name: [{ required: true, message: t('sub.nameRequired'), trigger: 'blur' }],
  url: [
    {
      required: true,
      message: t('sub.urlRequired'),
      trigger: 'blur',
      validator: (rule, value) => activeTab.value === 'url' ? !!value : true
    }
  ],
  manualContent: [
    {
      required: true,
      message: t('sub.contentRequired'),
      trigger: 'blur',
      validator: (rule, value) => activeTab.value === 'manual' ? !!value : true
    }
  ],
  uriContent: [
    {
      required: true,
      message: t('sub.uriContentRequired'),
      trigger: 'blur',
      validator: (rule, value) => activeTab.value === 'uri' ? !!value : true
    }
  ],
}

const resolvePersistOptionsFor = (item: Subscription) => {
  if (item.configPath && item.configPath.length > 0) {
    return { configPath: item.configPath }
  }
  return { fileName: generateConfigFileName(item.name || 'sub') }
}

const formatIntervalLabel = (minutes?: number) =>
  formatIntervalLabelText(minutes, t, DEFAULT_AUTO_UPDATE_MINUTES)

const getDropdownOptions = (index: number): DropdownOption[] => [
  {
    label: t('sub.copyLink'),
    key: 'copy',
    icon: () => h('span', { class: 'icon' }, [h(CopyOutline)]),
    props: { onClick: () => copyUrl(subStore.list[index].url) }
  },
  {
    label: t('sub.edit'),
    key: 'edit',
    icon: () => h('span', { class: 'icon' }, [h(CreateOutline)]),
    props: { onClick: () => handleEdit(index, subStore.list[index]) }
  },
  {
    label: t('sub.editConfig'),
    key: 'edit-config',
    icon: () => h('span', { class: 'icon' }, [h(CodeOutline)]),
    show: subStore.activeIndex === index,
    props: { onClick: editCurrentConfig }
  },
  {
    label: t('sub.refreshNow'),
    key: 'refresh',
    icon: () => h('span', { class: 'icon' }, [h(RefreshOutline)]),
    props: { onClick: () => refreshSubscription(index, subStore.activeIndex === index && appStore.isRunning) }
  },
  {
    label: t('sub.rollback'),
    key: 'rollback',
    icon: () => h('span', { class: 'icon' }, [h(ArrowUndoOutline)]),
    props: { onClick: () => rollbackSubscription(index) }
  },
  { type: 'divider', key: 'd1' },
  {
    label: t('common.delete'),
    key: 'delete',
    icon: () => h('span', { class: 'icon delete' }, [h(TrashOutline)]),
    disabled: subStore.activeIndex === index,
    props: { onClick: () => deleteSubscription(index) }
  }
]

const resetForm = () => {
  formValue.value = {
    name: '',
    url: '',
    isLoading: false,
    isManual: false,
    manualContent: '',
    uriContent: '',
    useOriginalConfig: false,
    autoUpdateIntervalMinutes: DEFAULT_AUTO_UPDATE_MINUTES,
  }
  useOriginalTouched.value = false
  editIndex.value = null
  activeTab.value = 'url'
}

watch(activeTab, (tab) => {
  if (tab === 'uri') {
    formValue.value.useOriginalConfig = false
  }
  if (tab !== 'url') {
    formValue.value.autoUpdateIntervalMinutes = 0
  }
  if (editIndex.value !== null || useOriginalTouched.value) return
  formValue.value.useOriginalConfig = tab === 'manual'
})

const handleEdit = (index: number, item: Subscription) => {
  const manualContent = item.manualContent ?? ''
  const isJson = isJsonContent(manualContent)
  const isNodeList = !isJson && manualContent.trim().length > 0
  editIndex.value = index
  formValue.value = {
    ...item,
    manualContent: item.isManual && isJson ? manualContent : '',
    uriContent: item.isManual && !isJson && isNodeList ? manualContent : '',
  }
  activeTab.value = item.isManual ? (isNodeList ? 'uri' : 'manual') : 'url'
  showAddModal.value = true
}

const handleConfirm = () => {
  formRef.value?.validate(async (errors) => {
    if (errors) return
    try {
      isLoading.value = true
      const isUrlTab = activeTab.value === 'url'
      const isJsonTab = activeTab.value === 'manual'
      const isUriTab = activeTab.value === 'uri'
      const urlInput = formValue.value.url?.trim() ?? ''
      const manualInput = formValue.value.manualContent?.trim() ?? ''
      const uriInput = formValue.value.uriContent?.trim() ?? ''
      const resolvedManualContent = isJsonTab ? manualInput : isUriTab ? uriInput : ''
      const isManual = !isUrlTab
      const useOriginalConfig = isUriTab ? false : formValue.value.useOriginalConfig
      const persistOptions = { fileName: generateConfigFileName(formValue.value.name || 'sub') }
      let savedResult: SubscriptionPersistResult | null = null

      if (editIndex.value === null) {
        if (isManual && resolvedManualContent) {
          if (useOriginalConfig && !isJsonContent(resolvedManualContent)) {
            message.warning(t('sub.originalConfigJsonOnly'))
            return
          }
          savedResult = await subscriptionService.addManualSubscription(
            resolvedManualContent,
            useOriginalConfig,
            { ...persistOptions, applyRuntime: false },
          )
        } else if (!isManual) {
          savedResult = await subscriptionService.downloadSubscription(
            urlInput,
            useOriginalConfig,
            { ...persistOptions, applyRuntime: false },
          )
        }
        const savedPath = savedResult?.configPath ?? null

        const { uriContent, ...base } = formValue.value
        const newItem: Subscription = {
          ...base,
          url: isManual ? '' : urlInput,
          lastUpdate: Date.now(),
          isManual,
          manualContent: isManual ? resolvedManualContent : undefined,
          useOriginalConfig,
          autoUpdateIntervalMinutes: isManual ? 0 : base.autoUpdateIntervalMinutes,
          configPath: savedPath || undefined,
          backupPath: savedPath ? `${savedPath}.bak` : undefined,
        }
        applySubscriptionUserinfo(newItem, savedResult, isManual)

        subStore.list.push(newItem)
        await subStore.saveToBackend()
        await subStore.setActiveIndex(subStore.list.length - 1)

        if (savedPath) {
          await subscriptionService.setActiveConfig(savedPath, { useOriginalConfig })
          await appStore.setActiveConfigPath(savedPath)
        }

        message.success(t('sub.addAndUseSuccess'))
      } else {
        if (isManual && resolvedManualContent && useOriginalConfig) {
          if (!isJsonContent(resolvedManualContent)) {
            message.warning(t('sub.originalConfigJsonOnly'))
            return
          }
        }
        const { uriContent, ...base } = formValue.value
        subStore.list[editIndex.value] = {
          ...subStore.list[editIndex.value],
          ...base,
          url: isManual ? '' : urlInput,
          isManual,
          manualContent: isManual ? resolvedManualContent : undefined,
          useOriginalConfig,
          autoUpdateIntervalMinutes: isManual ? 0 : base.autoUpdateIntervalMinutes,
        }
        if (isManual) {
          applySubscriptionUserinfo(subStore.list[editIndex.value], null, true)
        }
        await subStore.saveToBackend()
        message.success(t('sub.updateSuccess'))
      }

      showAddModal.value = false
      resetForm()
    } catch (error) {
      message.error(t('sub.operationFailed') + error)
    } finally {
      isLoading.value = false
    }
  })
}

const handleCancel = () => {
  showAddModal.value = false
  resetForm()
}

const deleteSubscription = async (index: number) => {
  if (subStore.activeIndex === index) {
    message.warning(t('sub.cannotDeleteActive'))
    return
  }
  const target = subStore.list[index]
  try {
    if (target?.configPath) {
      await subscriptionService.deleteConfig(target.configPath)
    }
    subStore.list.splice(index, 1)
    if (subStore.activeIndex !== null && subStore.activeIndex > index) {
      await subStore.setActiveIndex(subStore.activeIndex - 1)
    }
    message.success(t('sub.deleteSuccess'))
  } catch (error) {
    message.error(t('sub.operationFailed') + error)
  }
}

const refreshSubscription = async (index: number, applyRuntime = false, silent = false) => {
  const item = subStore.list[index]
  if (!item) return

  if (item.isManual && !item.manualContent) {
    message.error(t('sub.manualContentMissing'))
    return
  }
  if (item.isManual && item.useOriginalConfig && !isJsonContent(item.manualContent ?? '')) {
    message.warning(t('sub.originalConfigJsonOnly'))
    return
  }

  const persistOptions = {
    ...resolvePersistOptionsFor(item),
    applyRuntime,
  }

  try {
    subStore.list[index].isLoading = true
    const savedResult = item.isManual
      ? await subscriptionService.addManualSubscription(
        item.manualContent || '',
        item.useOriginalConfig,
        persistOptions,
      )
      : await subscriptionService.downloadSubscription(
        item.url,
        item.useOriginalConfig,
        persistOptions,
      )

    const savedPath = savedResult.configPath
    if (savedPath) {
      subStore.list[index].configPath = savedPath
      subStore.list[index].backupPath = `${savedPath}.bak`
    }
    applySubscriptionUserinfo(subStore.list[index], savedResult, item.isManual)
    subStore.list[index].lastUpdate = Date.now()
    await subStore.saveToBackend()

    if (savedPath && applyRuntime) {
      await subscriptionService.setActiveConfig(savedPath, {
        useOriginalConfig: item.useOriginalConfig,
      })
      await appStore.setActiveConfigPath(savedPath)
    }

    if (!silent) {
      message.success(applyRuntime ? t('sub.refreshAndApplied') : t('sub.refreshSuccess'))
    }

  } catch (error) {
    message.error(t('sub.refreshFailed') + error)
  } finally {
    if (index >= 0 && index < subStore.list.length) {
      subStore.list[index].isLoading = false
    }
  }
}

const rollbackSubscription = async (index: number) => {
  const item = subStore.list[index]
  if (!item?.configPath) {
    message.error(t('sub.missingConfigFile'))
    return
  }
  try {
    await subscriptionService.rollbackConfig(item.configPath)
    message.success(t('sub.rollbackSuccess'))
    if (subStore.activeIndex === index) {
      await subscriptionService.setActiveConfig(item.configPath, {
        useOriginalConfig: item.useOriginalConfig,
      })
      await appStore.setActiveConfigPath(item.configPath)
      if (appStore.isRunning) {
        await kernelService.restartKernel()
      }
    }
  } catch (error) {
    message.error(t('sub.rollbackFailed') + error)
  }
}

const useSubscription = async (index: number) => {
  const item = subStore.list[index]
  if (!item) return
  if (!item.configPath) {
    message.warning(t('sub.missingConfigFile'))
    return
  }

  try {
    subStore.list[index].isLoading = true
    const activePath = appStore.activeConfigPath
    if (activePath && item.configPath === activePath && subStore.activeIndex !== index) {
      message.warning(t('sub.configPathCollision'))
      const regeneratedPath = await regenerateConfigFor(item)
      if (regeneratedPath) {
        subStore.list[index].configPath = regeneratedPath
        subStore.list[index].backupPath = `${regeneratedPath}.bak`
        subStore.list[index].lastUpdate = Date.now()
        item.configPath = regeneratedPath
      }
    }
    if (!item.configPath) {
      throw new Error(t('sub.missingConfigFile'))
    }
    await subStore.saveToBackend()
    await subscriptionService.setActiveConfig(item.configPath, {
      useOriginalConfig: item.useOriginalConfig,
    })
    await appStore.setActiveConfigPath(item.configPath)
    await subStore.setActiveIndex(index)
    subStore.list[index].lastUpdate = Date.now()
    message.success(t('sub.useSuccess'))
  } catch (error) {
    message.error(t('sub.useFailed') + error)
  } finally {
    if (index >= 0 && index < subStore.list.length) {
      subStore.list[index].isLoading = false
    }
  }
}

const copyUrl = (url: string) => {
  navigator.clipboard.writeText(url)
  message.success(t('sub.linkCopied'))
}

const applySubscriptionUserinfo = (
  target: Subscription,
  result: SubscriptionPersistResult | null,
  isManual: boolean,
) => {
  if (isManual || !result) {
    target.subscriptionUpload = undefined
    target.subscriptionDownload = undefined
    target.subscriptionTotal = undefined
    target.subscriptionExpire = undefined
    return
  }

  target.subscriptionUpload = result.subscriptionUpload
  target.subscriptionDownload = result.subscriptionDownload
  target.subscriptionTotal = result.subscriptionTotal
  target.subscriptionExpire = result.subscriptionExpire
}

const formatTrafficSummary = (item: Subscription) => formatTrafficSummaryText(item, t)
const formatExpireTime = (timestamp?: number) => formatExpireTimeText(timestamp, t)
const formatTime = (timestamp: number): string => formatLocalTime(timestamp)
const formatAutoUpdateHealth = (item: Subscription) => formatAutoUpdateHealthText(item, t)

const regenerateConfigFor = async (item: Subscription) => {
  const persistOptions = { fileName: generateConfigFileName(item.name || 'sub'), applyRuntime: false }
  if (item.isManual) {
    const content = item.manualContent?.trim() ?? ''
    if (!content) {
      throw new Error(t('sub.manualContentMissing'))
    }
    const result = await subscriptionService.addManualSubscription(
      content,
      item.useOriginalConfig,
      persistOptions,
    )
    return result.configPath
  }
  const result = await subscriptionService.downloadSubscription(
    item.url,
    item.useOriginalConfig,
    persistOptions,
  )
  return result.configPath
}

const editCurrentConfig = async () => {
  try {
    isConfigLoading.value = true
    const config = await subscriptionService.getCurrentConfig()
    if (typeof config === 'string') {
      currentConfig.value = config
      showConfigModal.value = true
    }
  } catch (error) {
    message.error(t('sub.readConfigFailed') + error)
  } finally {
    isConfigLoading.value = false
  }
}

const saveCurrentConfig = async () => {
  try {
    isConfigLoading.value = true
    const activeItem = subStore.getActiveSubscription()
    const persistOptions = activeItem?.configPath
      ? { configPath: activeItem.configPath, applyRuntime: true }
      : { fileName: generateConfigFileName(activeItem?.name || 'sub'), applyRuntime: true }

    const savedResult = await subscriptionService.addManualSubscription(
      currentConfig.value,
      false,
      persistOptions,
    )
    const savedPath = savedResult.configPath

    if (activeItem) {
      if (activeItem.isManual) {
        activeItem.manualContent = currentConfig.value
      }
      activeItem.lastUpdate = Date.now()
      activeItem.configPath = savedPath || activeItem.configPath
      activeItem.backupPath = savedPath ? `${savedPath}.bak` : activeItem.backupPath
    }
    message.success(t('sub.configSaved'))
    showConfigModal.value = false
  } catch (error) {
    message.error(t('sub.saveConfigFailed') + error)
  } finally {
    isConfigLoading.value = false
  }
}

const { startAutoUpdateLoop, stopAutoUpdateLoop } = useSubscriptionAutoUpdate({
  getSubscriptions: () => subStore.list,
  getActiveIndex: () => subStore.activeIndex,
  isKernelRunning: () => appStore.isRunning,
  defaultIntervalMinutes: DEFAULT_AUTO_UPDATE_MINUTES,
  onRefresh: refreshSubscription,
})

onMounted(() => {
  subStore.resetLoadingState()
  startAutoUpdateLoop()
})

onUnmounted(() => {
  stopAutoUpdateLoop()
})

</script>

<style scoped>
.sub-page {
  max-width: var(--content-max-width, 1440px);
  margin: 0 auto;
}

.subscription-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--space-4);
}

.sub-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-lg);
  padding: var(--space-5);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  transition:
    transform var(--transition-fast),
    box-shadow var(--transition-fast),
    border-color var(--transition-fast);
}

.sub-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
  border-color: var(--border-hover);
}

.sub-card.active {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 1px var(--primary-color), var(--shadow-md);
}

.sub-card-header {
  display: flex;
  gap: var(--space-3);
  align-items: flex-start;
}

.sub-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  background: var(--bg-surface-2);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  flex-shrink: 0;
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
}

.sub-icon.active {
  background: var(--primary-color);
  color: var(--primary-contrast);
}

.sub-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.sub-name {
  font-weight: 600;
  font-size: var(--text-md);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sub-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.more-btn {
  color: var(--text-tertiary);
}

.more-btn:hover {
  color: var(--text-primary);
}

.sub-card-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-3);
  background: var(--bg-surface-2);
  border-radius: var(--radius-sm);
}

.info-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.info-row.warn {
  color: var(--warning-color);
}

.info-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sub-card-footer {
  margin-top: auto;
}

.form-switch {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-3);
  background: var(--bg-surface-2);
  border-radius: var(--radius-sm);
  margin-top: var(--space-4);
}

.form-hint {
  margin-top: 6px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.form-hint.warning {
  color: var(--warning-color);
}

.switch-label {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: var(--text-base);
  color: var(--text-primary);
}

.switch-desc {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.code-input {
  font-family: var(--font-mono);
}

/* Dropdown Icon Styles */
:deep(.icon) {
  display: flex;
  align-items: center;
  justify-content: center;
}

:deep(.icon.delete) {
  color: var(--error-color);
}
</style>
