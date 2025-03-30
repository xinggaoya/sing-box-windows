<template>
  <div class="sub-container">
    <!-- 订阅管理卡片 -->
    <n-card class="sub-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <n-h3 class="card-title">
              <n-icon size="24" class="card-icon">
                <link-outline />
              </n-icon>
              {{ t('sub.title') }}
            </n-h3>
            <n-tag :bordered="false" type="info" size="medium" class="sub-count-tag">
              {{ subStore.list.length }} {{ t('sub.count') }}
            </n-tag>
          </div>
          <n-tooltip trigger="hover" placement="top">
            <template #trigger>
              <n-button
                quaternary
                circle
                size="medium"
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
            </template>
            {{ t('sub.add') }}
          </n-tooltip>
        </div>
      </template>

      <n-grid
        :x-gap="16"
        :y-gap="16"
        :cols="gridCols"
        responsive="screen"
        item-responsive
        :collapsed="false"
        :collapsed-rows="1"
      >
        <n-grid-item v-for="(item, index) in subStore.list" :key="index">
          <n-card
            :class="{
              'sub-node-card': true,
              'sub-node-card-active': subStore.activeIndex === index,
            }"
            :bordered="false"
            hoverable
          >
            <n-space vertical :size="14">
              <n-flex justify="space-between" align="center">
                <n-space align="center" :size="8" style="flex-wrap: nowrap; overflow: hidden">
                  <n-icon size="20" :color="subStore.activeIndex === index ? '#18a058' : '#4080ff'">
                    <link-outline />
                  </n-icon>
                  <n-text strong class="sub-name text-ellipsis">{{ item.name }}</n-text>
                  <div class="tag-container">
                    <n-tag
                      v-if="subStore.activeIndex === index"
                      type="success"
                      size="small"
                      :bordered="false"
                      class="active-tag"
                    >
                      {{ t('sub.inUse') }}
                    </n-tag>
                    <n-tag
                      v-if="item.isManual"
                      type="warning"
                      size="small"
                      :bordered="false"
                      class="manual-tag"
                    >
                      {{ t('sub.manual') }}
                    </n-tag>
                  </div>
                </n-space>
                <n-space :size="10">
                  <n-tooltip trigger="hover" placement="top">
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="small"
                        @click="copyUrl(item.url)"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon>
                            <copy-outline />
                          </n-icon>
                        </template>
                      </n-button>
                    </template>
                    {{ t('sub.copyLink') }}
                  </n-tooltip>

                  <n-tooltip trigger="hover" placement="top">
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="small"
                        @click="handleEdit(index, item)"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon>
                            <create-outline />
                          </n-icon>
                        </template>
                      </n-button>
                    </template>
                    {{ t('sub.edit') }}
                  </n-tooltip>

                  <!-- 新增：查看/编辑当前配置按钮 -->
                  <n-tooltip v-if="subStore.activeIndex === index" trigger="hover" placement="top">
                    <template #trigger>
                      <n-button
                        quaternary
                        circle
                        size="small"
                        type="info"
                        @click="editCurrentConfig()"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon>
                            <code-outline />
                          </n-icon>
                        </template>
                      </n-button>
                    </template>
                    {{ t('sub.editConfig') }}
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
                        size="small"
                        type="error"
                        :disabled="subStore.activeIndex === index"
                        class="action-button"
                      >
                        <template #icon>
                          <n-icon>
                            <trash-outline />
                          </n-icon>
                        </template>
                      </n-button>
                    </template>
                    {{ t('sub.deleteConfirm') }}
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
                  size="small"
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
        </n-grid-item>
      </n-grid>

      <n-empty v-if="!subStore.list.length" :description="t('sub.noSubs')" class="empty-container">
        <template #extra>
          <n-button type="primary" @click="showAddModal = true" class="add-sub-button">
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
    style="width: 600px"
    class="sub-modal"
  >
    <n-form
      ref="formRef"
      :model="formValue"
      :rules="rules"
      label-placement="left"
      label-width="80"
      require-mark-placement="right-hanging"
    >
      <n-form-item :label="t('sub.name')" path="name">
        <n-input
          v-model:value="formValue.name"
          :placeholder="t('sub.namePlaceholder')"
          @keydown.enter.prevent
          class="form-input"
        />
      </n-form-item>

      <n-tabs type="line" animated v-model:value="activeTab" class="sub-tabs">
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
              :autosize="{ minRows: 8, maxRows: 20 }"
              class="form-input code-input"
            />
          </n-form-item>
        </n-tab-pane>
      </n-tabs>
    </n-form>
    <template #action>
      <n-space justify="end">
        <n-button @click="handleCancel" class="modal-button">{{ t('common.cancel') }}</n-button>
        <n-button type="primary" @click="handleConfirm" :loading="isLoading" class="modal-button">
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
    style="width: 800px"
    class="config-modal"
  >
    <n-input
      v-model:value="currentConfig"
      type="textarea"
      :placeholder="t('sub.configContentPlaceholder')"
      :autosize="{ minRows: 15, maxRows: 30 }"
      class="form-input code-input"
    />
    <template #action>
      <n-space justify="end">
        <n-button @click="showConfigModal = false" class="modal-button">{{
          t('common.cancel')
        }}</n-button>
        <n-button
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
import { ref, computed } from 'vue'
import { useSubStore } from '@/stores/SubStore'
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
            await tauriApi.subscription.addManualSubscription(formValue.value.manualContent)
          }
        } else if (!isManual) {
          // 如果是URL模式且是新建订阅
          if (editIndex.value === null) {
            await tauriApi.subscription.downloadSubscription(formValue.value.url)
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
          })

          // 如果是新添加的手动配置，自动设为当前活跃订阅
          if (isManual) {
            subStore.activeIndex = subStore.list.length - 1
          }

          message.success(t('sub.addSuccess'))
        } else {
          // 更新订阅
          subStore.list[editIndex.value] = {
            ...subStore.list[editIndex.value],
            name: formValue.value.name,
            url: formValue.value.url,
            isManual: isManual,
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
      await tauriApi.subscription.addManualSubscription(item.manualContent)
    } else {
      // 否则从URL下载内容
      await tauriApi.subscription.downloadSubscription(url)
    }

    // 更新订阅状态
    subStore.list[index].lastUpdate = Date.now()
    subStore.activeIndex = index
    message.success(t('sub.useSuccess'))
  } catch (error) {
    message.error(t('sub.useFailed') + error)
  } finally {
    subStore.list[index].isLoading = false
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
    await tauriApi.subscription.addManualSubscription(currentConfig.value)

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
</script>

<style scoped>
.sub-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 8px;
  animation: slide-up 0.4s ease;
}

.sub-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.sub-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-weight: 600;
}

.card-icon {
  color: var(--primary-color);
}

.sub-count-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
  background-color: rgba(144, 147, 153, 0.12);
  color: var(--n-text-color-2);
}

.add-button {
  transition: all 0.3s ease;
}

.add-button:hover:not(:disabled) {
  transform: translateY(-2px);
  color: var(--primary-color);
  background-color: rgba(64, 128, 255, 0.1);
}

.sub-node-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  border-left: 3px solid transparent;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.sub-node-card:hover {
  transform: translateY(-3px);
  box-shadow: var(--shadow-medium);
}

.sub-node-card-active {
  border-left: 3px solid var(--success-color);
  background-color: rgba(0, 180, 42, 0.05);
}

.sub-name {
  font-size: 15px;
  font-weight: 600;
  color: var(--n-text-color-1);
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

.active-tag {
  font-weight: 500;
  padding: 2px 8px;
  white-space: nowrap;
}

.manual-tag {
  font-weight: 500;
  padding: 2px 8px;
  white-space: nowrap;
}

.url-container {
  padding: 8px 10px;
  border-radius: 8px;
  font-family: monospace;
  font-size: 13px;
  color: var(--n-text-color-2);
  word-break: break-all;
  border: 1px solid var(--n-border-color);
  flex-grow: 1;
  margin: 8px 0;
  display: flex;
  align-items: center;
}

.update-time {
  font-size: 12px;
  color: var(--n-text-color-3);
}

.use-button {
  border-radius: 8px;
  font-weight: 500;
  transition: all 0.25s ease;
}

.use-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

:deep(.dark) .use-button:hover:not(:disabled) {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.25);
}

.action-button {
  transition: all 0.3s ease;
}

.action-button:hover {
  transform: translateY(-1px);
}

.empty-container {
  margin: 60px 0;
  opacity: 0.8;
}

.add-sub-button {
  font-weight: 500;
  border-radius: 8px;
  padding: 0 20px;
  height: 36px;
  transition: all 0.3s ease;
}

.add-sub-button:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.sub-modal {
  border-radius: 16px;
}

.form-input {
  transition: all 0.3s ease;
}

.form-input:hover {
  box-shadow: var(--shadow-focus);
}

.modal-button {
  min-width: 80px;
  border-radius: 8px;
  font-weight: 500;
}

/* 新增样式 */
.sub-tabs {
  margin-top: 10px;
}

.code-input {
  font-family: monospace;
  font-size: 13px;
  background-color: rgba(0, 0, 0, 0.02);
}

:deep(.dark) .code-input {
  background-color: rgba(255, 255, 255, 0.05);
}

.manual-icon {
  margin-right: 4px;
  color: #ff9800;
}

.sub-node-card > :deep(.n-card__content) {
  display: flex;
  flex-direction: column;
  flex-grow: 1;
}

.sub-node-card > :deep(.n-card__content) > .n-space {
  height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
