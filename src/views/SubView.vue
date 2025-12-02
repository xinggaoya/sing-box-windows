<template>
  <div class="page-container">
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

    <!-- Stats -->
    <div class="stats-grid">
      <StatusCard
        v-for="stat in subscriptionStats"
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
                  v-if="(item.autoUpdateIntervalMinutes ?? DEFAULT_AUTO_UPDATE_MINUTES) > 0"
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
      <div v-else class="empty-state">
        <div class="empty-icon">
          <n-icon size="48"><LinkOutline /></n-icon>
        </div>
        <h3 class="empty-title">{{ t('sub.noSubs') }}</h3>
        <p class="empty-desc">{{ t('sub.noSubscriptionsYet') }}</p>
        <n-button type="primary" @click="showAddModal = true">
          {{ t('sub.addFirstSubscription') }}
        </n-button>
      </div>
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
          </n-tab-pane>
        </n-tabs>

        <div class="form-switch">
          <div class="switch-label">
            <span>{{ t('sub.useOriginalConfig') }}</span>
            <span class="switch-desc">{{ formValue.useOriginalConfig ? t('sub.useOriginal') : t('sub.useExtractedNodes') }}</span>
          </div>
          <n-switch v-model:value="formValue.useOriginalConfig" />
        </div>

        <n-form-item :label="t('sub.autoUpdate')" path="autoUpdateIntervalMinutes">
          <n-select
            v-model:value="formValue.autoUpdateIntervalMinutes"
            :options="autoUpdateOptions"
            size="small"
          />
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
import { ref, computed, onMounted, onUnmounted, h } from 'vue'
import { useMessage } from 'naive-ui'
import { useSubStore } from '@/stores/subscription/SubStore'
import { useAppStore } from '@/stores'
import { subscriptionService } from '@/services/subscription-service'
import { kernelService } from '@/services/kernel-service'
import { useI18n } from 'vue-i18n'
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
  RefreshOutline,
  ArrowUndoOutline,
  TimerOutline,
} from '@vicons/ionicons5'
import type { FormInst, FormRules, DropdownOption } from 'naive-ui'
import PageHeader from '@/components/common/PageHeader.vue'
import StatusCard from '@/components/common/StatusCard.vue'

defineOptions({
  name: 'SubView'
})

interface Subscription {
  name: string
  url: string
  lastUpdate?: number
  isLoading: boolean
  isManual: boolean
  manualContent?: string
  useOriginalConfig: boolean
  configPath?: string
  backupPath?: string
  autoUpdateIntervalMinutes?: number
}

const DEFAULT_AUTO_UPDATE_MINUTES = 720

const message = useMessage()
const subStore = useSubStore()
const appStore = useAppStore()
const { t } = useI18n()

const showAddModal = ref(false)
const editIndex = ref<number | null>(null)
const formRef = ref<FormInst | null>(null)
const isLoading = ref(false)
const activeTab = ref('url')

const showConfigModal = ref(false)
const currentConfig = ref('')
const isConfigLoading = ref(false)

const formValue = ref<Subscription>({
  name: '',
  url: '',
  isLoading: false,
  isManual: false,
  manualContent: '',
  useOriginalConfig: false,
  autoUpdateIntervalMinutes: DEFAULT_AUTO_UPDATE_MINUTES,
})

const subscriptionStats = computed(() => [
  {
    label: t('sub.total'),
    value: subStore.list.length,
    icon: LinkOutline,
    type: 'primary' as const,
  },
  {
    label: t('sub.active'),
    value: subStore.activeIndex !== null && subStore.activeIndex >= 0 ? 1 : 0,
    icon: CheckmarkCircleOutline,
    type: 'success' as const,
  },
])

const autoUpdateOptions = computed(() => [
  { label: t('sub.autoUpdateOff'), value: 0 },
  { label: t('sub.autoUpdate6h'), value: 360 },
  { label: t('sub.autoUpdate12h'), value: 720 },
  { label: t('sub.autoUpdate1d'), value: 1440 },
])

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
  ]
}

const generateConfigFileName = (name: string) => {
  const safe = name
    .toLowerCase()
    .replace(/[^a-z0-9-_]/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '')
  return `${safe || 'subscription'}-${Date.now()}.json`
}

const resolvePersistOptionsFor = (item: Subscription) => {
  if (item.configPath && item.configPath.length > 0) {
    return { configPath: item.configPath }
  }
  return { fileName: generateConfigFileName(item.name || 'sub') }
}

const formatIntervalLabel = (minutes?: number) => {
  const value = minutes ?? DEFAULT_AUTO_UPDATE_MINUTES
  if (!value) return t('sub.autoUpdateOff')
  if (value % 1440 === 0) return t('sub.autoUpdate1d')
  if (value % 720 === 0) return t('sub.autoUpdate12h')
  if (value % 360 === 0) return t('sub.autoUpdate6h')
  return `${value} min`
}

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
    useOriginalConfig: false,
    autoUpdateIntervalMinutes: DEFAULT_AUTO_UPDATE_MINUTES,
  }
  editIndex.value = null
  activeTab.value = 'url'
}

const handleEdit = (index: number, item: Subscription) => {
  editIndex.value = index
  formValue.value = { ...item }
  activeTab.value = item.isManual ? 'manual' : 'url'
  showAddModal.value = true
}

const handleConfirm = () => {
  formRef.value?.validate(async (errors) => {
    if (errors) return
    try {
      isLoading.value = true
      const isManual = activeTab.value === 'manual'
      const persistOptions = { fileName: generateConfigFileName(formValue.value.name || 'sub') }
      let savedPath: string | null = null

      if (editIndex.value === null) {
        if (isManual && formValue.value.manualContent) {
          savedPath = await subscriptionService.addManualSubscription(
            formValue.value.manualContent,
            formValue.value.useOriginalConfig,
            { ...persistOptions, applyRuntime: false },
          )
        } else if (!isManual) {
          savedPath = await subscriptionService.downloadSubscription(
            formValue.value.url,
            formValue.value.useOriginalConfig,
            { ...persistOptions, applyRuntime: false },
          )
        }

        const newItem: Subscription = {
          ...formValue.value,
          lastUpdate: Date.now(),
          isManual,
          manualContent: isManual ? formValue.value.manualContent : undefined,
          configPath: savedPath || undefined,
          backupPath: savedPath ? `${savedPath}.bak` : undefined,
        }

        subStore.list.push(newItem)
        await subStore.setActiveIndex(subStore.list.length - 1)

        if (savedPath) {
          await subscriptionService.setActiveConfig(savedPath)
        }

        message.success(t('sub.addAndUseSuccess'))
        if (appStore.isRunning) {
          await kernelService.restartKernel()
        }
      } else {
        subStore.list[editIndex.value] = {
          ...subStore.list[editIndex.value],
          ...formValue.value,
          isManual,
          manualContent: isManual ? formValue.value.manualContent : undefined,
        }
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

  const persistOptions = {
    ...resolvePersistOptionsFor(item),
    applyRuntime,
  }

  try {
    subStore.list[index].isLoading = true
    const savedPath = item.isManual
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

    if (savedPath) {
      subStore.list[index].configPath = savedPath
      subStore.list[index].backupPath = `${savedPath}.bak`
      if (applyRuntime) {
        await subscriptionService.setActiveConfig(savedPath)
      }
    }
    subStore.list[index].lastUpdate = Date.now()

    if (!silent) {
      message.success(applyRuntime ? t('sub.refreshAndApplied') : t('sub.refreshSuccess'))
    }

    if (applyRuntime && appStore.isRunning) {
      await kernelService.restartKernel()
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
      await subscriptionService.setActiveConfig(item.configPath)
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
    await subscriptionService.setActiveConfig(item.configPath)
    await subStore.setActiveIndex(index)
    subStore.list[index].lastUpdate = Date.now()
    message.success(t('sub.useSuccess'))
    if (appStore.isRunning) {
      await kernelService.restartKernel()
    }
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

const formatTime = (timestamp: number): string => {
  return new Date(timestamp).toLocaleString()
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

    const savedPath = await subscriptionService.addManualSubscription(
      currentConfig.value,
      false,
      persistOptions,
    )

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
    if (appStore.isRunning) {
      await kernelService.restartKernel()
    }
  } catch (error) {
    message.error(t('sub.saveConfigFailed') + error)
  } finally {
    isConfigLoading.value = false
  }
}

const AUTO_UPDATE_CHECK_INTERVAL = 30 * 60 * 1000
let autoUpdateTimer: number | null = null

const runAutoUpdate = async () => {
  const now = Date.now()
  for (let i = 0; i < subStore.list.length; i += 1) {
    const item = subStore.list[i]
    const interval = item.autoUpdateIntervalMinutes ?? DEFAULT_AUTO_UPDATE_MINUTES
    const last = item.lastUpdate ?? 0
    if (interval > 0 && now - last >= interval * 60 * 1000 && !item.isLoading) {
      await refreshSubscription(i, subStore.activeIndex === i && appStore.isRunning, true)
    }
  }
}

const startAutoUpdateLoop = () => {
  stopAutoUpdateLoop()
  autoUpdateTimer = window.setInterval(runAutoUpdate, AUTO_UPDATE_CHECK_INTERVAL)
}

const stopAutoUpdateLoop = () => {
  if (autoUpdateTimer) {
    window.clearInterval(autoUpdateTimer)
    autoUpdateTimer = null
  }
}

onMounted(() => {
  subStore.resetLoadingState()
  startAutoUpdateLoop()
})

onUnmounted(() => {
  stopAutoUpdateLoop()
})
</script>

<style scoped>
.page-container {
  padding: var(--layout-page-padding-y, 24px) var(--layout-page-padding-x, 32px);
  max-width: var(--layout-page-max-width, 1400px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--layout-page-gap, 24px);
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--layout-row-gap, 16px);
}

.subscription-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: var(--layout-subscription-gap, 20px);
}

.sub-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  transition: all 0.2s ease;
}

.sub-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--panel-shadow);
  border-color: var(--border-hover);
}

.sub-card.active {
  border-color: var(--primary-color);
  background: var(--bg-secondary);
}

.sub-card-header {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.sub-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: var(--bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.sub-icon.active {
  background: var(--primary-color);
  color: white;
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
  font-size: 16px;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sub-tags {
  display: flex;
  gap: 6px;
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
  gap: 8px;
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: 8px;
}

.info-row {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-secondary);
  font-size: 13px;
}

.info-text {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sub-card-footer {
  margin-top: auto;
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
  margin: 0 0 8px;
  color: var(--text-primary);
}

.empty-desc {
  margin-bottom: 24px;
  color: var(--text-tertiary);
}

.form-switch {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: 8px;
  margin-top: 16px;
}

.switch-label {
  display: flex;
  flex-direction: column;
  gap: 2px;
  font-size: 14px;
  color: var(--text-primary);
}

.switch-desc {
  font-size: 12px;
  color: var(--text-tertiary);
}

.code-input {
  font-family: 'JetBrains Mono', monospace;
}

/* Dropdown Icon Styles */
:deep(.icon) {
  display: flex;
  align-items: center;
  justify-content: center;
}

:deep(.icon.delete) {
  color: #ef4444;
}
</style>
