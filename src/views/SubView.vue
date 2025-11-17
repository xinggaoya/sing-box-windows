<template>
  <div class="subscription-container">
    <!-- 页面标题区域 -->
    <div class="page-header">
      <div class="header-content">
        <div class="header-left">
          <div class="header-icon">
            <n-icon size="24">
              <LinkOutline />
            </n-icon>
          </div>
          <div class="header-info">
            <h1 class="page-title">{{ t('sub.title') }}</h1>
            <p class="page-subtitle">{{ t('sub.subtitle') }}</p>
          </div>
        </div>
        <div class="header-right">
          <div class="stats-info">
            <div class="stat-item">
              <span class="stat-label">{{ t('sub.total') }}</span>
              <span class="stat-value">{{ subStore.list.length }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">{{ t('sub.active') }}</span>
              <span class="stat-value">{{ (subStore.activeIndex !== null && subStore.activeIndex >= 0) ? 1 : 0 }}</span>
            </div>
          </div>
          <n-button
            @click="showAddModal = true"
            type="primary"
            size="medium"
            class="add-button"
          >
            <template #icon>
              <n-icon size="16">
                <AddOutline />
              </n-icon>
            </template>
            {{ t('sub.add') }}
          </n-button>
        </div>
      </div>
    </div>

    <!-- 订阅内容区域 -->
    <div class="subscription-content">
      <!-- 有订阅时显示卡片网格 -->
      <div v-if="subStore.list.length" class="subscription-grid">
        <n-grid
          :cols="24"
          :x-gap="12"
          :y-gap="12"
          responsive="screen"
        >
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
              class="subscription-card"
              :class="{
                'card-active': subStore.activeIndex === index,
                'card-loading': item.isLoading,
              }"
              :bordered="false"
              hoverable
            >
              <!-- 卡片头部 -->
              <div class="card-header">
                <div class="card-icon" :class="{ 'icon-active': subStore.activeIndex === index }">
                  <n-icon size="20">
                    <LinkOutline />
                  </n-icon>
                </div>
                <div class="card-info">
                  <h3 class="card-title" :title="item.name">{{ item.name }}</h3>
                  <div class="card-meta">
                    <n-tag v-if="item.isManual" size="small" type="warning" round>
                      {{ t('sub.manual') }}
                    </n-tag>
                    <n-tag v-else size="small" type="info" round>
                      {{ t('sub.urlSubscription') }}
                    </n-tag>
                    <div v-if="subStore.activeIndex === index" class="active-badge">
                      <n-icon size="12">
                        <CheckmarkCircleOutline />
                      </n-icon>
                      <span>{{ t('sub.inUse') }}</span>
                    </div>
                  </div>
                </div>
                <div class="card-actions">
                  <n-dropdown trigger="hover" placement="bottom-end" :options="getDropdownOptions(index)">
                    <n-button quaternary circle size="small" class="more-button">
                      <n-icon size="16">
                        <EllipsisVerticalOutline />
                      </n-icon>
                    </n-button>
                  </n-dropdown>
                </div>
              </div>

              <!-- 卡片内容 -->
              <div class="card-body">
                <!-- URL/内容预览 -->
                <div class="content-preview">
                  <div class="preview-text" :title="item.url || t('sub.manualContent')">
                    <n-icon size="14" class="preview-icon">
                      <GlobeOutline />
                    </n-icon>
                    <span>{{ item.url || t('sub.manualContent') }}</span>
                  </div>
                  <div class="update-info">
                    <n-icon size="12" class="time-icon">
                      <TimeOutline />
                    </n-icon>
                    <span>{{ item.lastUpdate ? formatTime(item.lastUpdate) : t('sub.neverUsed') }}</span>
                  </div>
                </div>

                <!-- 配置选项 -->
                <div class="config-options">
                  <div class="option-item">
                    <span class="option-label">{{ t('sub.useOriginalConfig') }}</span>
                    <n-tag
                      :type="item.useOriginalConfig ? 'success' : 'default'"
                      size="small"
                      round
                    >
                      {{ item.useOriginalConfig ? t('sub.useOriginal') : t('sub.useExtractedNodes') }}
                    </n-tag>
                  </div>
                </div>
              </div>

              <!-- 卡片底部 -->
              <div class="card-footer">
                <n-button
                  @click="useSubscription(item.url, index)"
                  :loading="item.isLoading"
                  :type="subStore.activeIndex === index ? 'success' : 'primary'"
                  block
                  size="medium"
                >
                  <template #icon>
                    <n-icon size="16">
                      <CheckmarkCircleOutline v-if="subStore.activeIndex === index" />
                      <PlayCircleOutline v-else />
                    </n-icon>
                  </template>
                  {{ subStore.activeIndex === index ? t('sub.useAgain') : t('sub.use') }}
                </n-button>
              </div>

              <!-- 加载遮罩 -->
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
          <n-button
            @click="showAddModal = true"
            type="primary"
            size="large"
            class="empty-action"
          >
            <template #icon>
              <n-icon size="18">
                <AddOutline />
              </n-icon>
            </template>
            {{ t('sub.addFirstSubscription') }}
          </n-button>
        </div>
      </div>
    </div>

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

          <n-form-item :label="t('sub.useOriginalConfig')" label-placement="left" class="switch-item">
            <n-space align="center">
              <n-switch v-model:value="formValue.useOriginalConfig" size="medium" />
              <n-text depth="3" style="font-size: 13px">
                {{
                  formValue.useOriginalConfig
                    ? t('sub.useOriginal')
                    : t('sub.useExtractedNodes')
                }}
              </n-text>
            </n-space>
          </n-form-item>
        </n-form>
      </div>

      <template #action>
        <n-space justify="end" size="medium">
          <n-button @click="handleCancel">{{ t('common.cancel') }}</n-button>
          <n-button
            type="primary"
            @click="handleConfirm"
            :loading="isLoading"
          >
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
          <n-button
            type="primary"
            @click="saveCurrentConfig"
            :loading="isConfigLoading"
          >
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

defineOptions({
  name: 'SubscriptionPage'
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
  padding: 16px;
  background: transparent;
  min-height: 100vh;
  animation: fadeInUp 0.4s ease-out;
}

/* 页面标题区域 */
.page-header {
  margin-bottom: 24px;
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.9)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border-radius: 16px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  box-shadow: 0 4px 20px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.03)"');
  padding: 16px 20px;
  transition: all 0.3s ease;
}

.page-header:hover {
  box-shadow: 0 8px 32px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.08)"');
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  background: linear-gradient(135deg, #10b981 0%, #059669 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 8px 24px rgba(16, 185, 129, 0.3);
  transition: all 0.3s ease;
}

.header-icon:hover {
  transform: translateY(-2px);
  box-shadow: 0 12px 32px rgba(16, 185, 129, 0.4);
}

.header-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.page-title {
  font-size: 1.5rem;
  font-weight: 700;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0;
  letter-spacing: -0.02em;
}

.page-subtitle {
  font-size: 0.9rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0;
  font-weight: 400;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 20px;
}

.stats-info {
  display: flex;
  gap: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 16px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
  border-radius: 8px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

.stat-label {
  font-size: 0.7rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
  text-transform: uppercase;
}

.stat-value {
  font-size: 1.2rem;
  font-weight: 700;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
}

.add-button {
  height: 40px;
  padding: 0 16px;
  font-weight: 500;
}

/* 订阅内容区域 */
.subscription-content {
  min-height: calc(100vh - 200px);
}

/* 订阅网格 */
.subscription-grid {
  min-height: 100%;
}

/* 订阅卡片 */
.subscription-card {
  height: 100%;
  transition: all 0.3s ease;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  position: relative;
  overflow: hidden;
}

.subscription-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 40px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.1)"');
}

.subscription-card.card-active {
  border-color: #10b981;
  background: v-bind('themeStore.isDark ? "rgba(16, 185, 129, 0.05)" : "rgba(16, 185, 129, 0.03)"');
}

.subscription-card.card-loading {
  pointer-events: none;
  opacity: 0.7;
}

/* 卡片头部 */
.card-header {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

.card-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  display: flex;
  align-items: center;
  justify-content: center;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.card-icon.icon-active {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.card-info {
  flex: 1;
  min-width: 0;
}

.card-title {
  font-size: 1rem;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0 0 4px 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.card-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.active-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border-radius: 12px;
  font-size: 0.7rem;
  font-weight: 600;
}

.card-actions {
  flex-shrink: 0;
}

.more-button {
  transition: all 0.2s ease;
}

.more-button:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

/* 卡片内容 */
.card-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
}

.content-preview {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preview-text {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.8rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.03)" : "rgba(0, 0, 0, 0.02)"');
  padding: 6px 10px;
  border-radius: 6px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.preview-icon {
  flex-shrink: 0;
}

.update-info {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 0.7rem;
  color: v-bind('themeStore.isDark ? "#6b7280" : "#9ca3af"');
  font-weight: 500;
}

.time-icon {
  flex-shrink: 0;
}

.config-options {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.option-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.03)" : "rgba(0, 0, 0, 0.02)"');
  border-radius: 6px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
}

.option-label {
  font-size: 0.8rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
}

/* 卡片底部 */
.card-footer {
  padding-top: 12px;
  border-top: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

/* 加载遮罩 */
.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(4px);
  border-radius: inherit;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
}

.empty-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 40px 16px;
  background: v-bind('themeStore.isDark ? "rgba(24, 24, 28, 0.8)" : "rgba(255, 255, 255, 0.9)"');
  backdrop-filter: blur(20px) saturate(180%);
  -webkit-backdrop-filter: blur(20px) saturate(180%);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  border-radius: 16px;
  box-shadow: 0 4px 20px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.1)" : "rgba(0, 0, 0, 0.03)"');
  transition: all 0.3s ease;
}

.empty-content:hover {
  box-shadow: 0 8px 32px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.08)"');
}

.empty-icon {
  color: v-bind('themeStore.isDark ? "#6b7280" : "#9ca3af"');
  margin-bottom: 16px;
  opacity: 0.6;
}

.empty-title {
  font-size: 1.2rem;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0 0 8px 0;
}

.empty-description {
  font-size: 0.9rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0 0 24px 0;
  max-width: 400px;
}

.empty-action {
  height: 40px;
  padding: 0 24px;
  font-weight: 500;
}

/* 模态框样式 */
.modern-modal {
  border-radius: 16px;
}

.modal-content {
  padding: 8px 0;
}

.subscription-form {
  margin-top: 8px;
}

.sub-tabs {
  margin: 16px 0;
}

.tab-form-item {
  margin-top: 12px;
}

.form-input {
  border-radius: 8px;
}

.code-input {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 13px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
}

.switch-item {
  margin-top: 12px;
  padding-top: 16px;
  border-top: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

/* 配置编辑器 */
.config-editor {
  margin: 16px 0;
}

.config-input {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 13px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
  border-radius: 8px;
}

/* 移除 Naive UI 下拉菜单样式覆盖 */

.dropdown-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  margin-right: 8px;
}

.delete-icon {
  color: #ef4444;
}

/* 动画效果 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .subscription-container {
    padding: 16px;
  }

  .page-header {
    padding: 16px 20px;
    margin-bottom: 20px;
  }

  .header-content {
    flex-direction: column;
    gap: 16px;
    align-items: flex-start;
  }

  .header-left {
    gap: 12px;
  }

  .header-icon {
    width: 40px;
    height: 40px;
  }

  .page-title {
    font-size: 1.25rem;
  }

  .page-subtitle {
    font-size: 0.85rem;
  }

  .header-right {
    width: 100%;
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .stats-info {
    justify-content: center;
  }

  .add-button {
    width: 100%;
  }

  .card-header {
    gap: 10px;
    margin-bottom: 12px;
  }

  .card-icon {
    width: 36px;
    height: 36px;
  }

  .card-title {
    font-size: 0.9rem;
  }

  .preview-text {
    font-size: 0.7rem;
  }

  .empty-content {
    padding: 40px 16px;
  }

  .empty-title {
    font-size: 1.1rem;
  }

  .empty-description {
    font-size: 0.85rem;
  }
}

@media (max-width: 480px) {
  .subscription-container {
    padding: 12px;
  }

  .page-header {
    padding: 12px 16px;
    margin-bottom: 16px;
  }

  .header-icon {
    width: 36px;
    height: 36px;
  }

  .page-title {
    font-size: 1.1rem;
  }

  .page-subtitle {
    font-size: 0.8rem;
  }

  .stats-info {
    gap: 12px;
  }

  .stat-item {
    padding: 6px 12px;
  }

  .stat-value {
    font-size: 1rem;
  }

  .card-header {
    gap: 8px;
  }

  .card-icon {
    width: 32px;
    height: 32px;
  }

  .card-title {
    font-size: 0.85rem;
  }

  .preview-text {
    font-size: 0.65rem;
  }

  .update-info {
    font-size: 0.65rem;
  }

  .option-item {
    padding: 4px 8px;
  }

  .option-label {
    font-size: 0.7rem;
  }
}

/* 移除 Naive UI 组件内部样式覆盖，使用官方主题系统 */
</style>