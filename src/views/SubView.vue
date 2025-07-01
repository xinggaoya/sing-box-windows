<template>
  <!-- 单一根元素包装器，修复Transition警告 -->
  <div class="sub-view-wrapper">
    <div class="modern-sub">
      <!-- 英雄式头部 -->
      <div class="sub-hero">
        <div class="hero-content">
          <div class="hero-info">
            <div class="hero-icon">
              <n-icon size="28">
                <LinkOutline />
              </n-icon>
            </div>
            <div class="hero-text">
              <h1 class="hero-title">{{ t('sub.title') }}</h1>
              <p class="hero-subtitle">
                {{ t('sub.subtitle') }}
                <n-tag size="small" type="info" round class="count-tag">
                  {{ subStore.list.length }} {{ t('sub.count') }}
                </n-tag>
              </p>
            </div>
          </div>

          <div class="hero-actions">
            <n-button
              @click="showAddModal = true"
              type="primary"
              size="large"
              round
              class="add-subscription-btn"
            >
              <template #icon>
                <n-icon>
                  <AddOutline />
                </n-icon>
              </template>
              {{ t('sub.add') }}
            </n-button>
          </div>
        </div>
      </div>

      <!-- 订阅内容区 -->
      <div class="sub-main">
        <!-- 有订阅时显示网格 -->
        <div v-if="subStore.list.length" class="subscriptions-grid">
          <div
            v-for="(item, index) in subStore.list"
            :key="`sub-${index}`"
            class="subscription-card"
            :class="{
              'card-active': subStore.activeIndex === index,
              'card-loading': item.isLoading,
            }"
          >
            <!-- 卡片头部 -->
            <div class="card-header">
              <div class="card-title-section">
                <div class="title-row">
                  <div class="sub-icon" :class="{ 'icon-active': subStore.activeIndex === index }">
                    <n-icon size="16">
                      <LinkOutline />
                    </n-icon>
                  </div>
                  <div class="sub-name" :title="item.name">{{ item.name }}</div>
                  <div v-if="subStore.activeIndex === index" class="active-badge">
                    <n-icon size="10">
                      <CheckmarkCircleOutline />
                    </n-icon>
                    <span>{{ t('sub.inUse') }}</span>
                  </div>
                </div>

                <div class="subtitle-row">
                  <div class="sub-tags">
                    <n-tag v-if="item.isManual" size="tiny" type="warning" round>
                      {{ t('sub.manualConfig') }}
                    </n-tag>
                    <n-tag v-else size="tiny" type="info" round>
                      {{ t('sub.urlSubscription') }}
                    </n-tag>
                  </div>

                  <div class="card-actions">
                    <n-tooltip trigger="hover" placement="top">
                      <template #trigger>
                        <n-button
                          @click="copyUrl(item.url)"
                          size="tiny"
                          quaternary
                          circle
                          class="action-btn"
                        >
                          <n-icon size="14">
                            <CopyOutline />
                          </n-icon>
                        </n-button>
                      </template>
                      {{ t('sub.copyLink') }}
                    </n-tooltip>

                    <n-tooltip trigger="hover" placement="top">
                      <template #trigger>
                        <n-button
                          @click="handleEdit(index, item)"
                          size="tiny"
                          quaternary
                          circle
                          class="action-btn"
                        >
                          <n-icon size="14">
                            <CreateOutline />
                          </n-icon>
                        </n-button>
                      </template>
                      {{ t('sub.edit') }}
                    </n-tooltip>

                    <n-tooltip
                      v-if="subStore.activeIndex === index"
                      trigger="hover"
                      placement="top"
                    >
                      <template #trigger>
                        <n-button
                          @click="editCurrentConfig()"
                          size="tiny"
                          quaternary
                          circle
                          class="action-btn"
                        >
                          <n-icon size="14">
                            <CodeOutline />
                          </n-icon>
                        </n-button>
                      </template>
                      {{ t('sub.editConfig') }}
                    </n-tooltip>

                    <n-popconfirm
                      @positive-click="deleteSubscription(index)"
                      :positive-text="t('common.delete')"
                      :negative-text="t('common.cancel')"
                      placement="top"
                    >
                      <template #trigger>
                        <n-tooltip trigger="hover" placement="top">
                          <template #trigger>
                            <n-button
                              size="tiny"
                              quaternary
                              circle
                              type="error"
                              :disabled="subStore.activeIndex === index"
                              class="action-btn delete-btn"
                            >
                              <n-icon size="14">
                                <TrashOutline />
                              </n-icon>
                            </n-button>
                          </template>
                          {{ t('common.delete') }}
                        </n-tooltip>
                      </template>
                      {{ t('sub.deleteConfirm') }}
                    </n-popconfirm>
                  </div>
                </div>
              </div>
            </div>

            <!-- URL 预览区 -->
            <div class="url-preview">
              <div class="url-content" :title="item.url || t('sub.manualContent')">
                {{ item.url || t('sub.manualContent') }}
              </div>
            </div>

            <!-- 卡片底部 -->
            <div class="card-footer">
              <div class="status-info">
                <div class="last-update">
                  {{ item.lastUpdate ? formatTime(item.lastUpdate) : t('sub.neverUsed') }}
                </div>
              </div>

              <n-button
                @click="useSubscription(item.url, index)"
                :loading="item.isLoading"
                :type="subStore.activeIndex === index ? 'success' : 'primary'"
                size="medium"
                round
                class="use-btn"
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

            <!-- 活跃指示器 -->
            <div v-if="subStore.activeIndex === index" class="active-indicator"></div>

            <!-- 加载遮罩 -->
            <div v-if="item.isLoading" class="loading-overlay">
              <n-spin size="small" />
            </div>
          </div>
        </div>

        <!-- 空状态 -->
        <div v-else class="empty-state">
          <div class="empty-container">
            <div class="empty-icon">
              <n-icon size="64">
                <LinkOutline />
              </n-icon>
            </div>
            <h3 class="empty-title">{{ t('sub.noSubs') }}</h3>
            <p class="empty-description">
              {{ t('sub.noSubscriptionsYet') }}<br />
              {{ t('sub.startAddingFirst') }}
            </p>
            <n-button
              @click="showAddModal = true"
              type="primary"
              size="large"
              round
              class="empty-action-btn"
            >
              <template #icon>
                <n-icon>
                  <AddOutline />
                </n-icon>
              </template>
              {{ t('sub.add') }}
            </n-button>
          </div>
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
      style="width: 540px"
      class="modern-modal"
    >
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

        <n-card size="small" class="tab-container">
          <n-tabs type="segment" animated v-model:value="activeTab" class="sub-tabs" size="medium">
            <n-tab-pane :name="'url'" :tab="t('sub.urlAdd')">
              <n-form-item :label="t('sub.url')" path="url" class="tab-form-item">
                <n-input
                  v-model:value="formValue.url"
                  type="textarea"
                  :placeholder="t('sub.urlPlaceholder')"
                  :autosize="{ minRows: 3, maxRows: 5 }"
                  class="form-input"
                />
              </n-form-item>
            </n-tab-pane>
            <n-tab-pane :name="'manual'" :tab="t('sub.manualAdd')">
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
        </n-card>

        <n-form-item :label="t('sub.useOriginal')" label-placement="left" class="switch-item">
          <n-space align="center">
            <n-switch v-model:value="formValue.useOriginalConfig" size="medium" />
            <n-text depth="3" style="font-size: 13px"
              >{{
                formValue.useOriginalConfig
                  ? t('sub.useOriginalConfig')
                  : t('sub.useExtractedNodes')
              }}
            </n-text>
          </n-space>
        </n-form-item>
      </n-form>

      <template #action>
        <n-space justify="end" size="large">
          <n-button size="medium" @click="handleCancel" class="modal-btn">
            {{ t('common.cancel') }}
          </n-button>
          <n-button
            size="medium"
            type="primary"
            @click="handleConfirm"
            :loading="isLoading"
            class="modal-btn"
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
      style="width: 720px"
      class="modern-modal config-modal"
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
        <n-space justify="end" size="large">
          <n-button size="medium" @click="showConfigModal = false" class="modal-btn">
            {{ t('common.cancel') }}
          </n-button>
          <n-button
            size="medium"
            type="primary"
            @click="saveCurrentConfig"
            :loading="isConfigLoading"
            class="modal-btn"
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
import { ref, computed, onMounted } from 'vue'
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
} from '@vicons/ionicons5'
import type { FormInst, FormRules } from 'naive-ui'
import { useWindowSize } from '@vueuse/core'
import { tauriApi } from '@/services/tauri-api'
import { useI18n } from 'vue-i18n'
import { useAppStore } from '@/stores'

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
          subStore.activeIndex = subStore.list.length - 1
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

const deleteSubscription = (index: number) => {
  if (subStore.activeIndex === index) {
    message.warning(t('sub.cannotDeleteActive'))
    return
  }
  subStore.list.splice(index, 1)
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
    subStore.activeIndex = index
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
/* 包装器样式，确保单一根元素用于Transition */
.sub-view-wrapper {
  width: 100%;
  height: 100%;
}

.modern-sub {
  display: flex;
  flex-direction: column;
  gap: 32px;
  min-height: 100%;
  padding: 0;
}

/* 英雄式头部 */
.sub-hero {
  background: var(--n-card-color);
  border-radius: 24px;
  border: 1px solid var(--n-border-color);
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.sub-hero:hover {
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.12);
  transform: translateY(-2px);
}

.hero-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 32px 40px;
}

.hero-info {
  display: flex;
  align-items: center;
  gap: 20px;
}

.hero-icon {
  width: 56px;
  height: 56px;
  border-radius: 20px;
  background: linear-gradient(135deg, #10b981, #059669);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 6px 20px rgba(16, 185, 129, 0.3);
}

.hero-text {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.hero-title {
  font-size: 28px;
  font-weight: 800;
  margin: 0;
  color: var(--n-text-color);
  line-height: 1.2;
}

.hero-subtitle {
  font-size: 16px;
  color: var(--n-text-color-2);
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
}

.count-tag {
  font-weight: 600;
}

.add-subscription-btn {
  height: 48px;
  min-width: 140px;
  font-size: 16px;
  font-weight: 600;
}

/* 主要内容区 */
.sub-main {
  flex: 1;
}

/* 订阅网格 */
.subscriptions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

/* 订阅卡片 */
.subscription-card {
  position: relative;
  background: var(--n-card-color);
  border: 1px solid var(--n-border-color);
  border-radius: 16px;
  padding: 18px;
  height: 160px; /* 固定卡片高度 */
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  display: flex;
  flex-direction: column;
  gap: 12px;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.05);
}

.subscription-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.1);
  border-color: var(--n-primary-color);
}

.subscription-card.card-active {
  background: rgba(16, 185, 129, 0.06);
  border-color: #10b981;
  box-shadow: 0 8px 24px rgba(16, 185, 129, 0.2);
}

.subscription-card.card-loading {
  pointer-events: none;
}

/* 卡片头部 */
.card-header {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.card-title-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.title-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.sub-icon {
  width: 28px;
  height: 28px;
  border-radius: 10px;
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
}

.sub-icon.icon-active {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.sub-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color);
  flex: 1;
  line-height: 1.4;
  overflow: hidden;
  display: -webkit-box;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 1; /* 限制为1行 */
  word-break: break-all;
}

.active-badge {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 2px 6px;
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border-radius: 8px;
  font-size: 10px;
  font-weight: 600;
  flex-shrink: 0;
}

.subtitle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.sub-tags {
  display: flex;
  gap: 6px;
}

.card-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.action-btn:hover {
  transform: scale(1.1);
  background: rgba(59, 130, 246, 0.1);
}

.delete-btn:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* URL 预览 */
.url-preview {
  flex: 1;
  min-height: 0;
  margin: 4px 0;
}

.url-content {
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.02);
  border: 1px solid var(--n-border-color);
  border-radius: 8px;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 11px;
  color: var(--n-text-color-2);
  line-height: 1.4;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  height: 32px;
  display: flex;
  align-items: center;
  overflow: hidden;
}

.url-content:hover {
  background: rgba(0, 0, 0, 0.04);
  border-color: var(--n-primary-color);
}

/* 卡片底部 */
.card-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin-top: auto;
}

.status-info {
  flex: 1;
  min-width: 0;
}

.last-update {
  font-size: 10px;
  color: var(--n-text-color-3);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}

.use-btn {
  height: 32px;
  min-width: 80px;
  font-weight: 600;
  font-size: 12px;
  flex-shrink: 0;
}

/* 活跃指示器 */
.active-indicator {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(90deg, #10b981, #059669);
  border-radius: 20px 20px 0 0;
  animation: pulse 2s ease-in-out infinite;
}

/* 加载遮罩 */
.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(2px);
  border-radius: 20px;
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
  padding: 60px 20px;
}

.empty-container {
  text-align: center;
  max-width: 360px;
}

.empty-icon {
  color: var(--n-text-color-3);
  margin-bottom: 24px;
  opacity: 0.6;
}

.empty-title {
  font-size: 24px;
  font-weight: 700;
  color: var(--n-text-color);
  margin: 0 0 12px 0;
}

.empty-description {
  font-size: 16px;
  color: var(--n-text-color-2);
  line-height: 1.6;
  margin: 0 0 32px 0;
}

.empty-action-btn {
  height: 48px;
  min-width: 140px;
  font-size: 16px;
  font-weight: 600;
}

/* 对话框样式 */
.modern-modal {
  border-radius: 20px;
}

.subscription-form {
  margin-top: 20px;
}

.tab-container {
  margin: 16px 0;
  border-radius: 12px;
}

.sub-tabs {
  margin: 16px 0;
}

.tab-form-item {
  margin-top: 16px;
}

.form-input {
  border-radius: 10px;
  font-size: 14px;
}

.code-input {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 13px;
  background: rgba(0, 0, 0, 0.02);
}

.switch-item {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--n-border-color);
}

.modal-btn {
  min-width: 80px;
  border-radius: 10px;
  font-weight: 600;
}

/* 配置编辑器 */
.config-editor {
  margin: 20px 0;
}

.config-input {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 13px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 12px;
}

/* 深色模式适配 */
[data-theme='dark'] .url-content {
  background: rgba(255, 255, 255, 0.04);
}

[data-theme='dark'] .url-content:hover {
  background: rgba(255, 255, 255, 0.08);
}

[data-theme='dark'] .code-input {
  background: rgba(255, 255, 255, 0.04);
}

[data-theme='dark'] .loading-overlay {
  background: rgba(0, 0, 0, 0.6);
}

/* 动画 */
@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .hero-content {
    flex-direction: column;
    gap: 24px;
    padding: 24px;
    text-align: center;
  }

  .subscriptions-grid {
    grid-template-columns: 1fr;
    gap: 20px;
  }

  .subscription-card {
    padding: 16px;
    height: 140px; /* 移动端调整高度 */
  }

  .title-row {
    flex-wrap: wrap;
  }

  .subtitle-row {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .card-actions {
    justify-content: flex-end;
  }

  .card-footer {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }

  .use-btn {
    width: 100%;
  }
}

@media (max-width: 480px) {
  .modern-sub {
    gap: 20px;
  }

  .hero-icon {
    width: 48px;
    height: 48px;
  }

  .hero-title {
    font-size: 22px;
  }

  .hero-subtitle {
    font-size: 14px;
  }

  .subscription-card {
    padding: 14px;
    gap: 10px;
    height: 120px; /* 小屏幕更紧凑 */
  }

  .sub-name {
    font-size: 13px;
  }

  .use-btn {
    height: 28px;
    font-size: 11px;
  }
}
</style>
