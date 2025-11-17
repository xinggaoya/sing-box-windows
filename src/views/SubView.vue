<template>
  <div class="page-shell subscription-container" :style="pageThemeStyle">
    <section class="page-hero">
      <div class="hero-row">
        <div class="hero-left">
          <div class="hero-icon">
            <n-icon size="26">
              <LinkOutline />
            </n-icon>
          </div>
          <div class="hero-meta">
            <p class="hero-subtitle">{{ t('sub.subtitle') }}</p>
            <h2 class="hero-title">{{ t('sub.title') }}</h2>
          </div>
        </div>
        <div class="hero-actions">
          <n-button @click="showAddModal = true" type="primary" size="large" class="add-button">
            <template #icon>
              <n-icon size="18">
                <AddOutline />
              </n-icon>
            </template>
            {{ t('sub.add') }}
          </n-button>
        </div>
      </div>
      <div class="hero-stats">
        <div
          v-for="stat in subscriptionStats"
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

    <!-- 订阅内容区域 -->
    <section class="page-section subscription-content">
      <!-- 有订阅时显示卡片网格 -->
      <div v-if="subStore.list.length" class="subscription-grid">
        <n-grid :cols="24" :x-gap="12" :y-gap="12" responsive="screen">
          <n-grid-item
            v-for="(item, index) in subStore.list"
            :key="`sub-${index}`"
            :span="24"
            :s="24"
            :m="12"
            :l="8"
            :xl="6"
            :xxl="6"
          >
            <n-card
              class="surface-card subscription-card compact-card"
              :class="{
                'is-active': subStore.activeIndex === index,
                'is-loading': item.isLoading,
              }"
              :bordered="false"
              hoverable
            >
              <div class="sub-card-compact">
                <div class="compact-header">
                  <div
                    class="compact-icon"
                    :class="{ 'icon-active': subStore.activeIndex === index }"
                  >
                    <n-icon size="16">
                      <LinkOutline />
                    </n-icon>
                  </div>
                  <div class="compact-meta">
                    <p class="compact-title" :title="item.name">{{ item.name }}</p>
                    <div class="compact-tags">
                      <span class="chip tiny" v-if="item.isManual">{{ t('sub.manual') }}</span>
                      <span class="chip tiny" v-else>{{ t('sub.urlSubscription') }}</span>
                      <span class="chip tiny success" v-if="subStore.activeIndex === index">
                        {{ t('sub.inUse') }}
                      </span>
                    </div>
                  </div>
                  <n-dropdown
                    trigger="hover"
                    placement="bottom-end"
                    :options="getDropdownOptions(index)"
                  >
                    <n-button text size="tiny" class="more-button">
                      <n-icon size="14">
                        <EllipsisVerticalOutline />
                      </n-icon>
                    </n-button>
                  </n-dropdown>
                </div>

                <div class="compact-info">
                  <div class="info-line" :title="item.url || t('sub.manualContent')">
                    <n-icon size="12">
                      <GlobeOutline />
                    </n-icon>
                    <span>{{ item.url || t('sub.manualContent') }}</span>
                  </div>
                  <div class="info-line">
                    <n-icon size="12">
                      <TimeOutline />
                    </n-icon>
                    <span>{{
                      item.lastUpdate ? formatTime(item.lastUpdate) : t('sub.neverUsed')
                    }}</span>
                  </div>
                </div>

                <div class="compact-actions">
                  <n-button
                    @click="useSubscription(item.url, index)"
                    :loading="item.isLoading"
                    :type="subStore.activeIndex === index ? 'success' : 'primary'"
                    size="small"
                    block
                  >
                    <template #icon>
                      <n-icon size="14">
                        <CheckmarkCircleOutline v-if="subStore.activeIndex === index" />
                        <PlayCircleOutline v-else />
                      </n-icon>
                    </template>
                    {{ subStore.activeIndex === index ? t('sub.useAgain') : t('sub.use') }}
                  </n-button>
                </div>
              </div>

              <div v-if="item.isLoading" class="loading-overlay">
                <n-spin size="large" />
              </div>
            </n-card>
          </n-grid-item>
        </n-grid>
      </div>

      <!-- 空状态 -->
      <div v-else class="empty-state">
        <div class="empty-content">
          <div class="empty-icon">
            <n-icon size="64">
              <LinkOutline />
            </n-icon>
          </div>
          <h3 class="empty-title">{{ t('sub.noSubs') }}</h3>
          <p class="empty-description">{{ t('sub.noSubscriptionsYet') }}</p>
          <n-button @click="showAddModal = true" type="primary" size="large" class="empty-action">
            <template #icon>
              <n-icon size="18">
                <AddOutline />
              </n-icon>
            </template>
            {{ t('sub.addFirstSubscription') }}
          </n-button>
        </div>
      </div>
    </section>

    <!-- 添加/编辑订阅对话框 -->
    <n-modal
      v-model:show="showAddModal"
      :mask-closable="false"
      preset="dialog"
      :title="editIndex === null ? t('sub.add') : t('sub.edit')"
      :bordered="false"
      class="modern-modal"
      :style="{ width: '600px' }"
    >
      <div class="modal-content">
        <n-form
          ref="formRef"
          :model="formValue"
          :rules="rules"
          label-placement="left"
          label-width="80"
          require-mark-placement="right-hanging"
          size="medium"
          class="subscription-form"
        >
          <n-form-item :label="t('sub.name')" path="name">
            <n-input
              v-model:value="formValue.name"
              :placeholder="t('sub.namePlaceholder')"
              @keydown.enter.prevent
              class="form-input"
            />
          </n-form-item>

          <n-tabs type="segment" animated v-model:value="activeTab" class="sub-tabs">
            <n-tab-pane name="url" :tab="t('sub.urlSubscription')">
              <n-form-item label="URL" path="url" class="tab-form-item">
                <n-input
                  v-model:value="formValue.url"
                  type="textarea"
                  :placeholder="t('sub.urlPlaceholder')"
                  :autosize="{ minRows: 3, maxRows: 5 }"
                  class="form-input"
                />
              </n-form-item>
            </n-tab-pane>
            <n-tab-pane name="manual" :tab="t('sub.manualConfig')">
              <n-form-item :label="t('sub.content')" path="manualContent" class="tab-form-item">
                <n-input
                  v-model:value="formValue.manualContent"
                  type="textarea"
                  :placeholder="t('sub.manualContentPlaceholder')"
                  :autosize="{ minRows: 8, maxRows: 20 }"
                  class="form-input code-input"
                />
              </n-form-item>
            </n-tab-pane>
          </n-tabs>

          <n-form-item
            :label="t('sub.useOriginalConfig')"
            label-placement="left"
            class="switch-item"
          >
            <n-space align="center">
              <n-switch v-model:value="formValue.useOriginalConfig" size="medium" />
              <n-text depth="3" style="font-size: 13px">
                {{
                  formValue.useOriginalConfig ? t('sub.useOriginal') : t('sub.useExtractedNodes')
                }}
              </n-text>
            </n-space>
          </n-form-item>
        </n-form>
      </div>

      <template #action>
        <n-space justify="end" size="medium">
          <n-button @click="handleCancel">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" @click="handleConfirm" :loading="isLoading">
            {{ t('common.confirm') }}
          </n-button>
        </n-space>
      </template>
    </n-modal>

    <!-- 编辑当前配置对话框 -->
    <n-modal
      v-model:show="showConfigModal"
      :mask-closable="false"
      preset="dialog"
      :title="t('sub.editCurrentConfig')"
      :bordered="false"
      class="modern-modal config-modal"
      :style="{ width: '800px' }"
    >
      <div class="config-editor">
        <n-input
          v-model:value="currentConfig"
          type="textarea"
          :placeholder="t('sub.configContentPlaceholder')"
          :autosize="{ minRows: 20, maxRows: 30 }"
          class="config-input"
        />
      </div>

      <template #action>
        <n-space justify="end" size="medium">
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
import { useMessage } from 'naive-ui'
import { ref, computed, onMounted, h } from 'vue'
import { useSubStore } from '@/stores/subscription/SubStore'
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
} from '@vicons/ionicons5'
import type { FormInst, FormRules, DropdownOption } from 'naive-ui'
import { tauriApi } from '@/services/tauri'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { usePageTheme } from '@/composables/usePageTheme'

defineOptions({
  name: 'SubscriptionPage',
})

interface Subscription {
  name: string
  url: string
  lastUpdate?: number
  isLoading: boolean
  isManual: boolean
  manualContent?: string
  useOriginalConfig: boolean
}

const message = useMessage()
const subStore = useSubStore()
const showAddModal = ref(false)
const editIndex = ref<number | null>(null)
const formRef = ref<FormInst | null>(null)
const isLoading = ref(false)
const appStore = useAppStore()
const themeStore = useThemeStore()
const pageThemeStyle = usePageTheme(themeStore)
const activeTab = ref('url')
const { t } = useI18n()

// 当前配置编辑相关变量
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
})

const subscriptionStats = computed(() => [
  {
    label: t('sub.total'),
    value: subStore.list.length,
    icon: LinkOutline,
    accent: 'purple',
  },
  {
    label: t('sub.active'),
    value: subStore.activeIndex !== null && subStore.activeIndex >= 0 ? 1 : 0,
    icon: CheckmarkCircleOutline,
    accent: 'blue',
  },
])

const rules: FormRules = {
  name: [{ required: true, message: t('sub.nameRequired'), trigger: 'blur' }],
  url: [
    {
      required: true,
      message: t('sub.urlRequired'),
      trigger: 'blur',
      validator: (rule, value) => {
        // 如果是URL模式，验证URL；如果是手动编辑模式，不验证URL
        return activeTab.value === 'url' ? !!value : true
      },
    },
    {
      type: 'url',
      message: t('sub.invalidUrl'),
      trigger: 'blur',
      validator: (rule, value) => {
        // 只在URL模式下验证URL格式
        return activeTab.value === 'url' ? true : true
      },
    },
  ],
  manualContent: [
    {
      required: true,
      message: t('sub.contentRequired'),
      trigger: 'blur',
      validator: (rule, value) => {
        // 如果是手动编辑模式，验证内容；如果是URL模式，不验证内容
        return activeTab.value === 'manual' ? !!value : true
      },
    },
  ],
}

// 获取下拉菜单选项
const getDropdownOptions = (index: number): DropdownOption[] => [
  {
    label: t('sub.copyLink'),
    key: 'copy',
    icon: () => h('span', { class: 'dropdown-icon' }, [h(CopyOutline)]),
    props: {
      onClick: () => copyUrl(subStore.list[index].url),
    },
  },
  {
    label: t('sub.edit'),
    key: 'edit',
    icon: () => h('span', { class: 'dropdown-icon' }, [h(CreateOutline)]),
    props: {
      onClick: () => handleEdit(index, subStore.list[index]),
    },
  },
  {
    label: t('sub.editConfig'),
    key: 'edit-config',
    icon: () => h('span', { class: 'dropdown-icon' }, [h(CodeOutline)]),
    show: subStore.activeIndex === index,
    props: {
      onClick: editCurrentConfig,
    },
  },
  {
    type: 'divider',
    key: 'divider1',
  },
  {
    label: t('common.delete'),
    key: 'delete',
    icon: () => h('span', { class: 'dropdown-icon delete-icon' }, [h(TrashOutline)]),
    disabled: subStore.activeIndex === index,
    props: {
      onClick: () => deleteSubscription(index),
    },
  },
]

const resetForm = () => {
  formValue.value = {
    name: '',
    url: '',
    isLoading: false,
    isManual: false,
    manualContent: '',
    useOriginalConfig: false,
  }
  editIndex.value = null
  activeTab.value = 'url'
}

const handleEdit = (index: number, item: Subscription) => {
  editIndex.value = index
  formValue.value = {
    name: item.name,
    url: item.url,
    isLoading: item.isLoading,
    isManual: item.isManual,
    manualContent: item.manualContent,
    useOriginalConfig: item.useOriginalConfig,
  }
  // 根据订阅类型设置activeTab
  activeTab.value = item.isManual ? 'manual' : 'url'
  showAddModal.value = true
}

const handleConfirm = () => {
  formRef.value?.validate(async (errors) => {
    if (!errors) {
      try {
        isLoading.value = true

        // 确定是否是手动编辑模式
        const isManual = activeTab.value === 'manual'

        if (isManual && formValue.value.manualContent) {
          // 如果是手动编辑模式且有内容，直接保存内容
          if (editIndex.value === null) {
            // 如果是新建订阅，同时使用这个内容
            await tauriApi.subscription.addManualSubscription(
              formValue.value.manualContent,
              formValue.value.useOriginalConfig,
            )
          }
        } else if (!isManual) {
          // 如果是URL模式且是新建订阅
          if (editIndex.value === null) {
            await tauriApi.subscription.downloadSubscription(
              formValue.value.url,
              formValue.value.useOriginalConfig,
            )
          }
        }

        if (editIndex.value === null) {
          // 添加新订阅
          subStore.list.push({
            name: formValue.value.name,
            url: formValue.value.url,
            lastUpdate: isManual ? Date.now() : undefined,
            isLoading: false,
            isManual: isManual,
            manualContent: isManual ? formValue.value.manualContent : undefined,
            useOriginalConfig: formValue.value.useOriginalConfig,
          })

          // 自动设置为当前活跃订阅（无论是手动还是URL订阅）
          await subStore.setActiveIndex(subStore.list.length - 1)
          message.success(t('sub.addAndUseSuccess'))
        } else {
          // 更新订阅
          subStore.list[editIndex.value] = {
            ...subStore.list[editIndex.value],
            name: formValue.value.name,
            url: formValue.value.url,
            isManual: isManual,
            manualContent: isManual ? formValue.value.manualContent : undefined,
            useOriginalConfig: formValue.value.useOriginalConfig,
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

  subStore.list.splice(index, 1)

  // 如果删除的索引小于当前激活索引，需要调整激活索引
  if (subStore.activeIndex !== null && subStore.activeIndex > index) {
    await subStore.setActiveIndex(subStore.activeIndex - 1)
  }

  message.success(t('sub.deleteSuccess'))
}

const useSubscription = async (url: string, index: number) => {
  try {
    // 标记正在加载
    subStore.list[index].isLoading = true

    const item = subStore.list[index]

    if (item.isManual && item.manualContent) {
      // 如果是手动配置，直接使用保存的内容
      await tauriApi.subscription.addManualSubscription(item.manualContent, item.useOriginalConfig)
    } else {
      // 否则从URL下载内容
      await tauriApi.subscription.downloadSubscription(url, item.useOriginalConfig)
    }

    // 更新订阅状态
    subStore.list[index].lastUpdate = Date.now()
    await subStore.setActiveIndex(index)
    message.success(t('sub.useSuccess'))

    // 判断内核是否在运行 重启内核
    if (appStore.isRunning) {
      await tauriApi.kernel.restartKernel()
    }
  } catch (error) {
    message.error(t('sub.useFailed') + error)
  } finally {
    // 确保一定会重置加载状态
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
  const date = new Date(timestamp)
  return `${t('sub.lastUpdate')}: ${date.toLocaleString(window.navigator.language, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })}`
}

const editCurrentConfig = async () => {
  try {
    isConfigLoading.value = true
    // 获取当前配置内容
    const config = await tauriApi.subscription.getCurrentConfig()
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

    // 保存配置内容
    await tauriApi.subscription.addManualSubscription(currentConfig.value, false)

    // 如果当前活跃订阅是手动配置，更新其内容
    if (subStore.activeIndex !== null) {
      const activeItem = subStore.list[subStore.activeIndex]
      if (activeItem.isManual) {
        subStore.list[subStore.activeIndex].manualContent = currentConfig.value
        subStore.list[subStore.activeIndex].lastUpdate = Date.now()
      }
    }

    message.success(t('sub.configSaved'))
    showConfigModal.value = false
  } catch (error) {
    message.error(t('sub.saveConfigFailed') + error)
  } finally {
    isConfigLoading.value = false
  }
}

// 在组件挂载时重置所有订阅的加载状态
onMounted(() => {
  // 使用SubStore提供的方法重置所有加载状态
  subStore.resetLoadingState()
})
</script>

<style scoped>
.subscription-container {
  padding: 24px;
  background: transparent;
  animation: fadeInUp 0.4s ease-out;
}

.subscription-content {
  min-height: calc(100vh - 200px);
}

.subscription-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}

.subscription-card {
  position: relative;
  border-radius: 24px;
  overflow: hidden;
  transition:
    transform 0.25s ease,
    border-color 0.25s ease,
    box-shadow 0.25s ease;
}

.compact-card {
  padding: 16px;
}

.sub-card-compact {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.compact-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.compact-icon {
  width: 32px;
  height: 32px;
  border-radius: 10px;
  background: rgba(99, 102, 241, 0.12);
  color: #6366f1;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.compact-icon.icon-active {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.compact-meta {
  flex: 1;
  min-width: 0;
}

.compact-title {
  margin: 0;
  font-size: 0.95rem;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#0f172a"');
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.compact-tags {
  display: flex;
  gap: 6px;
  margin-top: 4px;
}

.chip.tiny {
  padding: 2px 8px;
  font-size: 11px;
}

.compact-info {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 0.8rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#475569"');
}

.info-line {
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.compact-actions {
  margin-top: 4px;
}

.subscription-card.is-loading {
  pointer-events: none;
  opacity: 0.8;
}

@media (max-width: 768px) {
  .subscription-grid {
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  }

  .add-button {
    width: 100%;
  }
}

@media (max-width: 480px) {
  .subscription-grid {
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  }
}

/* 移除 Naive UI 组件内部样式覆盖，使用官方主题系统 */
</style>
