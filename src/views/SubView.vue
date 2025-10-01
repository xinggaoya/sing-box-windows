<template>
  <div class="ultra-sub">
    <!-- 紧凑工具栏 -->
    <div class="sub-toolbar">
      <div class="toolbar-left">
        <div class="toolbar-icon">
          <n-icon size="16">
            <LinkOutline />
          </n-icon>
        </div>
        <div class="toolbar-info">
          <span class="toolbar-title">{{ t('sub.title') }}</span>
          <span class="toolbar-stats">{{ subStore.list.length }} {{ t('sub.count') }}</span>
        </div>
      </div>

      <div class="toolbar-right">
        <n-button
          @click="showAddModal = true"
          type="primary"
          size="small"
          class="add-btn"
        >
          <template #icon>
            <n-icon size="12"><AddOutline /></n-icon>
          </template>
          {{ t('sub.add') }}
        </n-button>
      </div>
    </div>

    <!-- 订阅列表 -->
    <div class="sub-content">
      <!-- 有订阅时显示列表 -->
      <div v-if="subStore.list.length" class="subscription-list">
        <div
          v-for="(item, index) in subStore.list"
          :key="`sub-${index}`"
          class="subscription-item"
          :class="{
            'item-active': subStore.activeIndex === index,
            'item-loading': item.isLoading,
          }"
        >
          <!-- 左侧信息 -->
          <div class="item-info">
            <div class="item-header">
              <div class="item-icon" :class="{ 'icon-active': subStore.activeIndex === index }">
                <n-icon size="14">
                  <LinkOutline />
                </n-icon>
              </div>
              <div class="item-title-section">
                <div class="item-name" :title="item.name">{{ item.name }}</div>
                <div class="item-meta">
                  <n-tag v-if="item.isManual" size="tiny" type="warning" round>
                    {{ t('sub.manual') }}
                  </n-tag>
                  <n-tag v-else size="tiny" type="info" round>
                    {{ t('sub.urlSubscription') }}
                  </n-tag>
                  <div v-if="subStore.activeIndex === index" class="active-indicator">
                    <n-icon size="10">
                      <CheckmarkCircleOutline />
                    </n-icon>
                    <span>{{ t('sub.inUse') }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- URL/内容预览 -->
            <div class="item-preview">
              <div class="preview-text" :title="item.url || t('sub.manualContent')">
                {{ item.url || t('sub.manualContent') }}
              </div>
              <div class="update-time">
                {{ item.lastUpdate ? formatTime(item.lastUpdate) : t('sub.neverUsed') }}
              </div>
            </div>
          </div>

          <!-- 右侧操作 -->
          <div class="item-actions">
            <div class="action-group">
              <n-tooltip trigger="hover" placement="top">
                <template #trigger>
                  <n-button
                    @click="copyUrl(item.url)"
                    size="tiny"
                    quaternary
                    circle
                    class="action-btn"
                  >
                    <n-icon size="12">
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
                    <n-icon size="12">
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
                    <n-icon size="12">
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
                        <n-icon size="12">
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

            <n-button
              @click="useSubscription(item.url, index)"
              :loading="item.isLoading"
              :type="subStore.activeIndex === index ? 'success' : 'primary'"
              size="small"
              class="use-btn"
            >
              <template #icon>
                <n-icon size="12">
                  <CheckmarkCircleOutline v-if="subStore.activeIndex === index" />
                  <PlayCircleOutline v-else />
                </n-icon>
              </template>
              {{ subStore.activeIndex === index ? t('sub.useAgain') : t('sub.use') }}
            </n-button>
          </div>

          <!-- 加载遮罩 -->
          <div v-if="item.isLoading" class="loading-overlay">
            <n-spin size="small" />
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="empty-state">
        <div class="empty-icon">
          <n-icon size="40">
            <LinkOutline />
          </n-icon>
        </div>
        <div class="empty-title">{{ t('sub.noSubs') }}</div>
        <div class="empty-desc">{{ t('sub.noSubscriptionsYet') }}</div>
        <n-button
          @click="showAddModal = true"
          type="primary"
          size="medium"
          class="empty-btn"
        >
          <template #icon>
            <n-icon size="14"><AddOutline /></n-icon>
          </template>
          {{ t('sub.noSubscriptionsYet') }}
        </n-button>
      </div>
    </div>

    <!-- 添加/编辑订阅对话框 -->
    <n-modal
      v-model:show="showAddModal"
      :mask-closable="false"
      preset="dialog"
      :title="editIndex === null ? t('sub.add') : t('sub.edit')"
      :bordered="false"
      style="width: 480px"
      class="compact-modal"
    >
      <n-form
        ref="formRef"
        :model="formValue"
        :rules="rules"
        label-placement="left"
        label-width="60"
        require-mark-placement="right-hanging"
        size="small"
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

        <n-tabs type="segment" animated v-model:value="activeTab" class="sub-tabs" size="small">
          <n-tab-pane name="url" :tab="t('sub.urlSubscription')">
            <n-form-item label="URL" path="url" class="tab-form-item">
              <n-input
                v-model:value="formValue.url"
                type="textarea"
                :placeholder="t('sub.urlPlaceholder')"
                :autosize="{ minRows: 3, maxRows: 4 }"
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
                :autosize="{ minRows: 6, maxRows: 15 }"
                class="form-input code-input"
              />
            </n-form-item>
          </n-tab-pane>
        </n-tabs>

        <n-form-item :label="t('sub.useOriginalConfig')" label-placement="left" class="switch-item">
          <n-space align="center">
            <n-switch v-model:value="formValue.useOriginalConfig" size="small" />
            <n-text depth="3" style="font-size: 12px">
              {{
                formValue.useOriginalConfig
                  ? t('sub.useOriginal')
                  : t('sub.useExtractedNodes')
              }}
            </n-text>
          </n-space>
        </n-form-item>
      </n-form>

      <template #action>
        <n-space justify="end" size="medium">
          <n-button size="small" @click="handleCancel">{{ t('common.cancel') }}</n-button>
          <n-button
            size="small"
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
      style="width: 640px"
      class="compact-modal config-modal"
    >
      <div class="config-editor">
        <n-input
          v-model:value="currentConfig"
          type="textarea"
          :placeholder="t('sub.configContentPlaceholder')"
          :autosize="{ minRows: 15, maxRows: 25 }"
          class="config-input"
        />
      </div>

      <template #action>
        <n-space justify="end" size="medium">
          <n-button size="small" @click="showConfigModal = false">{{ t('common.cancel') }}</n-button>
          <n-button
            size="small"
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
import { useThemeStore } from '@/stores/app/ThemeStore'

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
.ultra-sub {
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 100%;
  font-size: 13px;
}

/* 紧凑工具栏 */
.sub-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(12px);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  border-radius: 10px;
  box-shadow: 0 2px 8px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.05)"');
  transition: all 0.2s var(--ease-out);
}

.sub-toolbar:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.25)" : "rgba(0, 0, 0, 0.08)"');
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.toolbar-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: linear-gradient(135deg, #10b981, #059669);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
  flex-shrink: 0;
}

.toolbar-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toolbar-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--n-text-color);
}

.toolbar-stats {
  font-size: 11px;
  color: var(--n-text-color-3);
}

.add-btn {
  height: 28px;
  min-width: 60px;
  border-radius: 6px;
  font-weight: 500;
}

/* 主内容区 */
.sub-content {
  flex: 1;
}

/* 订阅列表 */
.subscription-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 订阅项 */
.subscription-item {
  position: relative;
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(12px);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  border-radius: 10px;
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  transition: all 0.2s var(--ease-out);
  min-height: 72px;
}

.subscription-item:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.2)" : "rgba(0, 0, 0, 0.05)"');
}

.subscription-item.item-active {
  background: v-bind('themeStore.isDark ? "rgba(16, 185, 129, 0.15)" : "rgba(16, 185, 129, 0.1)"');
  border-color: #10b981;
}

.subscription-item.item-loading {
  pointer-events: none;
  opacity: 0.7;
}

/* 项目信息 */
.item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.item-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.item-icon {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s var(--ease-out);
  flex-shrink: 0;
}

.item-icon.icon-active {
  background: rgba(16, 185, 129, 0.2);
  color: #10b981;
}

.item-title-section {
  flex: 1;
  min-width: 0;
}

.item-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--n-text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 2px;
}

.item-meta {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.active-indicator {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 1px 4px;
  background: rgba(16, 185, 129, 0.15);
  color: #10b981;
  border-radius: 4px;
  font-size: 9px;
  font-weight: 600;
  flex-shrink: 0;
}

/* 预览信息 */
.item-preview {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.preview-text {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 10px;
  color: var(--n-text-color-3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
  padding: 2px 6px;
  border-radius: 4px;
  max-width: 300px;
}

.update-time {
  font-size: 9px;
  color: var(--n-text-color-3);
  font-weight: 500;
}

/* 项目操作 */
.item-actions {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 6px;
  flex-shrink: 0;
}

.action-group {
  display: flex;
  gap: 4px;
}

.action-btn {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  transition: all 0.15s var(--ease-out);
}

.action-btn:hover {
  transform: scale(1.1);
  background: rgba(59, 130, 246, 0.1);
}

.action-btn.delete-btn:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.use-btn {
  height: 24px;
  min-width: 50px;
  border-radius: 5px;
  font-size: 11px;
  font-weight: 500;
}

/* 加载遮罩 */
.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(2px);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
  background: v-bind('themeStore.isDark ? "rgba(17, 24, 39, 0.6)" : "rgba(255, 255, 255, 0.8)"');
  backdrop-filter: blur(12px);
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  border-radius: 12px;
  margin: 8px 0;
}

.empty-icon {
  color: var(--n-text-color-3);
  margin-bottom: 12px;
  opacity: 0.5;
}

.empty-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--n-text-color-2);
  margin-bottom: 6px;
}

.empty-desc {
  font-size: 12px;
  color: var(--n-text-color-3);
  margin-bottom: 16px;
}

.empty-btn {
  height: 32px;
  min-width: 100px;
  border-radius: 6px;
  font-weight: 500;
}

/* 对话框样式 */
.compact-modal {
  border-radius: 12px;
}

.subscription-form {
  margin-top: 16px;
}

.sub-tabs {
  margin: 12px 0;
}

.tab-form-item {
  margin-top: 12px;
}

.form-input {
  border-radius: 6px;
  font-size: 13px;
}

.code-input {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 12px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
}

.switch-item {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--n-border-color);
}

/* 配置编辑器 */
.config-editor {
  margin: 16px 0;
}

.config-input {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 12px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.05)" : "rgba(0, 0, 0, 0.03)"');
  border-radius: 8px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .sub-toolbar {
    padding: 10px 12px;
  }

  .subscription-item {
    padding: 10px 12px;
    min-height: 64px;
  }

  .item-header {
    flex-wrap: wrap;
    gap: 6px;
  }

  .item-actions {
    align-items: center;
    gap: 4px;
  }

  .action-group {
    order: 2;
  }

  .use-btn {
    order: 1;
  }

  .preview-text {
    max-width: 200px;
  }
}

@media (max-width: 480px) {
  .subscription-item {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
    padding: 8px 10px;
    min-height: auto;
  }

  .item-info {
    gap: 4px;
  }

  .item-actions {
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
  }

  .action-group {
    order: 1;
  }

  .use-btn {
    order: 2;
  }

  .preview-text {
    max-width: none;
  }
}
</style>
