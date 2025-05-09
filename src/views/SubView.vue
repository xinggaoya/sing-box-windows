<template>
  <div class="sub-container">
    <!-- 订阅管理卡片 -->
    <n-card class="sub-card" :bordered="false">
      <div class="card-header">
        <div class="header-left">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <link-outline />
            </n-icon>
            {{ t('sub.title') }}
          </n-h3>
          <n-tag :bordered="false" type="info" size="small" class="sub-count-tag">
            {{ subStore.list.length }} {{ t('sub.count') }}
          </n-tag>
        </div>
        <n-button
          quaternary
          circle
          size="small"
          @click="showAddModal = true"
          :disabled="isLoading"
          class="add-button"
        >
          <template #icon>
            <n-icon>
              <add-outline />
            </n-icon>
          </template>
        </n-button>
      </div>

      <div class="sub-grid">
        <div v-for="(item, index) in subStore.list" :key="index" class="sub-grid-item">
          <n-card
            :class="{
              'sub-node-card': true,
              'sub-node-card-active': subStore.activeIndex === index,
            }"
            :bordered="false"
            size="small"
            hoverable
          >
            <n-space vertical :size="6">
              <n-flex justify="space-between" align="center">
                <div class="name-container">
                  <n-icon size="16" :color="subStore.activeIndex === index ? '#18a058' : '#4080ff'">
                    <link-outline />
                  </n-icon>
                  <n-text strong class="sub-name text-ellipsis">{{ item.name }}</n-text>
                  <div class="tag-container">
                    <n-tag
                      v-if="subStore.activeIndex === index"
                      type="success"
                      size="tiny"
                      :bordered="false"
                      class="active-tag"
                    >
                      {{ t('sub.inUse') }}
                    </n-tag>
                    <n-tag
                      v-if="item.isManual"
                      type="warning"
                      size="tiny"
                      :bordered="false"
                      class="manual-tag"
                    >
                      {{ t('sub.manual') }}
                    </n-tag>
                  </div>
                </div>
                <n-space :size="4">
                  <n-tooltip trigger="hover" :delay="500" placement="top">
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="tiny"
                        @click="copyUrl(item.url)"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon><copy-outline /></n-icon>
                        </template>
                      </n-button>
                    </template>
                    {{ t('sub.copyUrl') }}
                  </n-tooltip>

                  <n-tooltip trigger="hover" :delay="500" placement="top">
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="tiny"
                        @click="handleEdit(index, item)"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon><create-outline /></n-icon>
                        </template>
                      </n-button>
                    </template>
                    {{ t('sub.edit') }}
                  </n-tooltip>

                  <n-tooltip v-if="subStore.activeIndex === index" trigger="hover" :delay="500" placement="top">
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="tiny"
                        type="info"
                        @click="editCurrentConfig()"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon><code-outline /></n-icon>
                        </template>
                      </n-button>
                    </template>
                    {{ t('sub.editCurrentConfig') }}
                  </n-tooltip>

                  <n-popconfirm
                    @positive-click="deleteSubscription(index)"
                    :positive-text="t('common.delete')"
                    :negative-text="t('common.cancel')"
                  >
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="tiny"
                        type="error"
                        :disabled="subStore.activeIndex === index"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon><trash-outline /></n-icon>
                        </template>
                      </n-button>
                    </template>
                    {{ t('sub.confirmDelete') }}
                  </n-popconfirm>
                </n-space>
              </n-flex>

              <div class="url-container">
                <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                  {{ item.url }}
                </n-ellipsis>
              </div>

              <n-flex justify="space-between" align="center">
                <n-text depth="3" class="update-time">
                  {{ item.lastUpdate ? formatTime(item.lastUpdate) : t('sub.neverUsed') }}
                </n-text>
                <n-button
                  secondary
                  size="tiny"
                  :loading="item.isLoading"
                  @click="useSubscription(item.url, index)"
                  :type="subStore.activeIndex === index ? 'success' : 'primary'"
                  :ghost="subStore.activeIndex !== index"
                  class="use-button"
                >
                  <template #icon>
                    <n-icon>
                      <checkmark-circle-outline v-if="subStore.activeIndex === index" />
                      <play-circle-outline v-else />
                    </n-icon>
                  </template>
                  {{ subStore.activeIndex === index ? t('sub.useAgain') : t('sub.use') }}
                </n-button>
              </n-flex>
            </n-space>
          </n-card>
        </div>
      </div>

      <n-empty v-if="!subStore.list.length" :description="t('sub.noSubs')" class="empty-container">
        <template #extra>
          <n-button size="small" type="primary" @click="showAddModal = true" class="add-sub-button">
            <template #icon>
              <n-icon><add-outline /></n-icon>
            </template>
            {{ t('sub.add') }}
          </n-button>
        </template>
      </n-empty>
    </n-card>
  </div>

  <!-- 添加/编辑订阅对话框 -->
  <n-modal
    v-model:show="showAddModal"
    :mask-closable="false"
    preset="dialog"
    :title="editIndex === null ? t('sub.add') : t('sub.edit')"
    :bordered="false"
    style="width: 500px"
    class="sub-modal"
  >
    <n-form
      ref="formRef"
      :model="formValue"
      :rules="rules"
      label-placement="left"
      label-width="70"
      require-mark-placement="right-hanging"
      size="small"
    >
      <n-form-item :label="t('sub.name')" path="name">
        <n-input
          v-model:value="formValue.name"
          :placeholder="t('sub.namePlaceholder')"
          @keydown.enter.prevent
          class="form-input"
        />
      </n-form-item>

      <n-tabs type="line" animated v-model:value="activeTab" class="sub-tabs" size="small">
        <n-tab-pane :name="'url'" :tab="t('sub.urlAdd')">
          <n-form-item :label="t('sub.url')" path="url">
            <n-input
              v-model:value="formValue.url"
              type="textarea"
              :placeholder="t('sub.urlPlaceholder')"
              :autosize="{ minRows: 2, maxRows: 4 }"
              class="form-input"
            />
          </n-form-item>
        </n-tab-pane>
        <n-tab-pane :name="'manual'" :tab="t('sub.manualAdd')">
          <n-form-item :label="t('sub.content')" path="manualContent">
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

      <!-- 移除规则集开关，仅保留原始订阅开关 -->
      <n-form-item :label="t('sub.useOriginal')" label-placement="left">
        <n-space align="center">
          <n-switch v-model:value="formValue.useOriginalConfig" size="small" />
          <n-text depth="3" style="font-size: 12px">{{
            formValue.useOriginalConfig ? t('sub.useOriginalConfig') : t('sub.useExtractedNodes')
          }}</n-text>
        </n-space>
      </n-form-item>
    </n-form>
    <template #action>
      <n-space justify="end">
        <n-button size="small" @click="handleCancel" class="modal-button">{{ t('common.cancel') }}</n-button>
        <n-button size="small" type="primary" @click="handleConfirm" :loading="isLoading" class="modal-button">
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
    style="width: 700px"
    class="config-modal"
  >
    <n-input
      v-model:value="currentConfig"
      type="textarea"
      :placeholder="t('sub.configContentPlaceholder')"
      :autosize="{ minRows: 15, maxRows: 25 }"
      class="form-input code-input"
    />
    <template #action>
      <n-space justify="end">
        <n-button size="small" @click="showConfigModal = false" class="modal-button">{{
          t('common.cancel')
        }}</n-button>
        <n-button
          size="small"
          type="primary"
          @click="saveCurrentConfig"
          :loading="isConfigLoading"
          class="modal-button"
        >
          {{ t('sub.saveAndApply') }}
        </n-button>
      </n-space>
    </template>
  </n-modal>
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
const { width } = useWindowSize()
const activeTab = ref('url')
const { t } = useI18n()

// 当前配置编辑相关变量
const showConfigModal = ref(false)
const currentConfig = ref('')
const isConfigLoading = ref(false)

// 根据窗口宽度调整网格列数
const gridCols = computed(() => {
  if (width.value < 768) return 1
  if (width.value < 1200) return 2
  return 3
})

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
              formValue.value.useOriginalConfig
            )
          }
        } else if (!isManual) {
          // 如果是URL模式且是新建订阅
          if (editIndex.value === null) {
            await tauriApi.subscription.downloadSubscription(
              formValue.value.url,
              formValue.value.useOriginalConfig
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
      await tauriApi.subscription.addManualSubscription(
        item.manualContent,
        item.useOriginalConfig
      )
    } else {
      // 否则从URL下载内容
      await tauriApi.subscription.downloadSubscription(
        url, 
        item.useOriginalConfig
      )
    }

    // 更新订阅状态
    subStore.list[index].lastUpdate = Date.now()
    subStore.activeIndex = index
    message.success(t('sub.useSuccess'))
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
    await tauriApi.subscription.addManualSubscription(
      currentConfig.value,
      false
    )

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
  subStore.resetLoadingState();
});
</script>

<style scoped>
.sub-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 12px 8px;
  animation: slide-up 0.3s ease;
}

.sub-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0;
  margin-bottom: 12px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0;
  font-weight: 600;
  font-size: 16px;
}

.card-icon {
  color: var(--primary-color);
}

.sub-count-tag {
  font-size: 12px;
  padding: 0 8px;
  height: 22px;
  border-radius: 11px;
}

.sub-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
}

.name-container {
  display: flex;
  align-items: center;
  gap: 6px;
  overflow: hidden;
  white-space: nowrap;
}

.sub-node-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  border-left: 2px solid transparent;
  height: 100%;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
}

.sub-node-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.sub-node-card :deep(.n-card__content) {
  padding: 12px 14px;
}

.sub-node-card-active {
  border-left: 2px solid var(--success-color);
  background: linear-gradient(135deg, rgba(var(--success-color-rgb), 0.05), transparent);
}

.sub-name {
  font-size: 14px;
  font-weight: 500;
  max-width: 120px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.text-ellipsis {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.tag-container {
  display: flex;
  flex-wrap: nowrap;
  gap: 4px;
  min-width: 0;
}

.active-tag, .manual-tag {
  font-size: 10px;
  padding: 0 4px;
  height: 18px;
  white-space: nowrap;
}

.url-container {
  padding: 6px 8px;
  border-radius: 6px;
  font-family: monospace;
  font-size: 12px;
  word-break: break-all;
  border: 1px solid var(--n-border-color);
  background-color: rgba(0, 0, 0, 0.02);
  margin: 4px 0;
}

:deep(.dark) .url-container {
  background-color: rgba(255, 255, 255, 0.05);
}

.update-time {
  font-size: 11px;
}

.use-button {
  border-radius: 6px;
  font-size: 12px;
  padding: 0 8px;
  height: 24px;
}

.action-button {
  height: 22px;
  width: 22px;
}

.action-button:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

:deep(.dark) .action-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.empty-container {
  margin: 40px 0;
  opacity: 0.8;
}

.add-sub-button {
  border-radius: 6px;
  padding: 0 12px;
}

.sub-modal {
  border-radius: 12px;
}

.form-input {
  font-size: 13px;
}

.code-input {
  font-family: monospace;
  font-size: 12px;
  background-color: rgba(0, 0, 0, 0.02);
}

:deep(.dark) .code-input {
  background-color: rgba(255, 255, 255, 0.05);
}

.sub-tabs {
  margin-top: 8px;
}

.modal-button {
  min-width: 70px;
  border-radius: 4px;
}

@media (max-width: 768px) {
  .sub-grid {
    grid-template-columns: 1fr;
  }
  
  .sub-name {
    max-width: 160px;
  }
}

@keyframes slide-up {
  0% {
    opacity: 0;
    transform: translateY(10px);
  }
  100% {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
