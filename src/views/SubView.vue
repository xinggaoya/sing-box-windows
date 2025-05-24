<template>
  <div class="sub-container">
    <!-- 顶部标题区 -->
    <div class="header-section">
      <div class="header-content">
        <div class="header-left">
          <div class="title-wrapper">
            <div class="title-icon">
              <n-icon size="20">
                <link-outline />
              </n-icon>
            </div>
            <h2 class="page-title">{{ t('sub.title') }}</h2>
            <n-badge
              :value="subStore.list.length"
              :max="99"
              show-zero
              type="info"
              size="small"
              class="count-badge"
            />
          </div>
        </div>
        <div class="header-actions">
          <n-button
            @click="showAddModal = true"
            :disabled="isLoading"
            size="small"
            type="primary"
            class="add-btn"
            round
          >
            <template #icon>
              <n-icon>
                <add-outline />
              </n-icon>
            </template>
            {{ t('sub.add') }}
          </n-button>
        </div>
      </div>
    </div>

    <!-- 订阅内容区 -->
    <div class="sub-content">
      <div v-if="subStore.list.length" class="sub-grid">
        <div
          v-for="(item, index) in subStore.list"
          :key="index"
          class="sub-card"
          :class="{ 'sub-active': subStore.activeIndex === index }"
        >
          <div class="sub-card-content">
            <!-- 卡片头部 -->
            <div class="card-header">
              <div class="name-section">
                <div class="name-wrapper">
                  <div class="sub-icon">
                    <n-icon
                      size="14"
                      :color="subStore.activeIndex === index ? '#10b981' : '#3b82f6'"
                    >
                      <link-outline />
                    </n-icon>
                  </div>
                  <div class="sub-name">
                    <n-ellipsis :tooltip="{ width: 'trigger' }">
                      {{ item.name }}
                    </n-ellipsis>
                  </div>
                </div>
                <div class="status-tags">
                  <div v-if="subStore.activeIndex === index" class="status-tag active">
                    {{ t('sub.inUse') }}
                  </div>
                  <div v-if="item.isManual" class="status-tag manual">
                    {{ t('sub.manual') }}
                  </div>
                </div>
              </div>
              <div class="action-buttons">
                <n-button
                  @click="copyUrl(item.url)"
                  size="tiny"
                  quaternary
                  circle
                  class="action-btn"
                >
                  <template #icon>
                    <n-icon size="12">
                      <copy-outline />
                    </n-icon>
                  </template>
                </n-button>
                <n-button
                  @click="handleEdit(index, item)"
                  size="tiny"
                  quaternary
                  circle
                  class="action-btn"
                >
                  <template #icon>
                    <n-icon size="12">
                      <create-outline />
                    </n-icon>
                  </template>
                </n-button>
                <n-button
                  v-if="subStore.activeIndex === index"
                  @click="editCurrentConfig()"
                  size="tiny"
                  quaternary
                  circle
                  class="action-btn"
                >
                  <template #icon>
                    <n-icon size="12">
                      <code-outline />
                    </n-icon>
                  </template>
                </n-button>
                <n-popconfirm
                  @positive-click="deleteSubscription(index)"
                  :positive-text="t('common.delete')"
                  :negative-text="t('common.cancel')"
                >
                  <template #trigger>
                    <n-button
                      size="tiny"
                      quaternary
                      circle
                      type="error"
                      :disabled="subStore.activeIndex === index"
                      class="action-btn delete-btn"
                    >
                      <template #icon>
                        <n-icon size="12">
                          <trash-outline />
                        </n-icon>
                      </template>
                    </n-button>
                  </template>
                  {{ t('sub.confirmDelete') }}
                </n-popconfirm>
              </div>
            </div>

            <!-- URL 显示区 -->
            <div class="url-section">
              <div class="url-display">
                <n-ellipsis :tooltip="{ width: 'trigger' }">
                  {{ item.url }}
                </n-ellipsis>
              </div>
            </div>

            <!-- 卡片底部 -->
            <div class="card-footer">
              <div class="update-info">
                <span class="update-time">
                  {{ item.lastUpdate ? formatTime(item.lastUpdate) : t('sub.neverUsed') }}
                </span>
              </div>
              <n-button
                @click="useSubscription(item.url, index)"
                :loading="item.isLoading"
                :type="subStore.activeIndex === index ? 'success' : 'primary'"
                size="small"
                class="use-btn"
                round
              >
                <template #icon>
                  <n-icon size="14">
                    <checkmark-circle-outline v-if="subStore.activeIndex === index" />
                    <play-circle-outline v-else />
                  </n-icon>
                </template>
                {{ subStore.activeIndex === index ? t('sub.useAgain') : t('sub.use') }}
              </n-button>
            </div>
          </div>

          <!-- 活跃指示器 -->
          <div v-if="subStore.activeIndex === index" class="active-indicator"></div>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-else class="empty-state">
        <div class="empty-content">
          <div class="empty-icon">
            <n-icon size="48" color="#d1d5db">
              <link-outline />
            </n-icon>
          </div>
          <h3 class="empty-title">{{ t('sub.noSubs') }}</h3>
          <p class="empty-description">添加您的第一个订阅开始使用</p>
          <n-button
            @click="showAddModal = true"
            type="primary"
            size="medium"
            class="empty-action-btn"
            round
          >
            <template #icon>
              <n-icon>
                <add-outline />
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
        <n-button size="small" @click="handleCancel" class="modal-button">{{
          t('common.cancel')
        }}</n-button>
        <n-button
          size="small"
          type="primary"
          @click="handleConfirm"
          :loading="isLoading"
          class="modal-button"
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
.sub-container {
  min-height: calc(100vh - 120px);
  padding: 20px;
  background: linear-gradient(135deg, rgba(64, 128, 255, 0.02), rgba(144, 147, 153, 0.02));
  animation: fadeIn 0.4s ease-out;
}

/* 顶部标题区 */
.header-section {
  margin-bottom: 20px;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border-radius: 16px;
  border: 1px solid rgba(255, 255, 255, 0.2);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.04);
  transition: all 0.3s ease;
}

.header-content:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.08);
}

.title-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: linear-gradient(135deg, #4080ff, #2266dd);
  border-radius: 10px;
  color: white;
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.3);
}

.page-title {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color-1);
  background: linear-gradient(135deg, #4080ff, #2266dd);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.count-badge {
  margin-left: 8px;
}

.add-btn {
  height: 36px;
  font-weight: 500;
  transition: all 0.3s ease;
}

.add-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(64, 128, 255, 0.2);
}

/* 订阅内容区 */
.sub-content {
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(12px);
  border-radius: 20px;
  border: 1px solid rgba(255, 255, 255, 0.3);
  padding: 20px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.06);
  transition: all 0.3s ease;
}

.sub-content:hover {
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.08);
}

/* 订阅网格 */
.sub-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 20px;
}

.sub-card {
  position: relative;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 16px;
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  backdrop-filter: blur(8px);
  min-height: 180px;
  display: flex;
  flex-direction: column;
}

.sub-card:hover {
  transform: translateY(-3px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.08);
  border-color: rgba(99, 102, 241, 0.2);
}

.sub-card.sub-active {
  background: linear-gradient(135deg, rgba(0, 180, 42, 0.08), rgba(0, 154, 26, 0.08));
  border-color: rgba(0, 180, 42, 0.3);
  box-shadow: 0 6px 18px rgba(0, 180, 42, 0.12);
}

.sub-card.sub-active:hover {
  box-shadow: 0 8px 24px rgba(0, 180, 42, 0.16);
}

.sub-card-content {
  padding: 16px;
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 0;
}

/* 卡片头部 */
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.name-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}

.name-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.sub-icon {
  flex-shrink: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  background: rgba(59, 130, 246, 0.1);
}

.sub-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-color-1);
  line-height: 1.4;
  min-width: 0;
  flex: 1;
  word-break: break-word;
  overflow-wrap: break-word;
}

.status-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.status-tag {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 6px;
  border-radius: 8px;
  white-space: nowrap;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.status-tag.active {
  background: linear-gradient(135deg, rgba(0, 180, 42, 0.15), rgba(0, 154, 26, 0.15));
  color: #009a1a;
  font-weight: 600;
}

.status-tag.manual {
  background: linear-gradient(135deg, rgba(255, 125, 0, 0.15), rgba(214, 102, 0, 0.15));
  color: #d66600;
  font-weight: 600;
}

.action-buttons {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.action-btn {
  width: 24px;
  height: 24px;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.action-btn:hover {
  transform: scale(1.1);
  background: rgba(99, 102, 241, 0.1);
}

.delete-btn:hover {
  background: rgba(239, 68, 68, 0.1);
}

/* URL 显示区 */
.url-section {
  margin-bottom: 12px;
  flex: 1;
  display: flex;
  align-items: center;
  min-height: 0;
}

.url-display {
  padding: 10px 12px;
  background: linear-gradient(135deg, rgba(248, 250, 252, 0.8), rgba(241, 245, 249, 0.8));
  border: 1px solid rgba(226, 232, 240, 0.8);
  border-radius: 8px;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 11px;
  color: var(--text-color-2);
  line-height: 1.5;
  word-break: break-all;
  transition: all 0.2s ease;
  width: 100%;
  min-height: 40px;
  display: flex;
  align-items: center;
}

.url-display:hover {
  background: linear-gradient(135deg, rgba(236, 254, 255, 0.8), rgba(225, 245, 254, 0.8));
  border-color: rgba(186, 230, 253, 0.8);
}

/* 卡片底部 */
.card-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
  margin-top: auto;
}

.update-info {
  flex: 1;
  min-width: 0;
}

.update-time {
  font-size: 10px;
  color: var(--text-color-3);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.use-btn {
  height: 32px;
  font-size: 11px;
  font-weight: 500;
  transition: all 0.2s ease;
  flex-shrink: 0;
  min-width: 80px;
  padding: 0 12px;
}

.use-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
}

/* 活跃指示器 */
.active-indicator {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 3px;
  background: linear-gradient(135deg, rgba(236, 254, 255, 0.8), rgba(225, 245, 254, 0.8));
  border-radius: 16px;
  animation: pulse 2s ease-in-out infinite;
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 300px;
  padding: 40px 20px;
}

.empty-content {
  text-align: center;
  max-width: 300px;
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.6;
}

.empty-title {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-color-1);
}

.empty-description {
  margin: 0 0 20px 0;
  font-size: 14px;
  color: var(--text-color-3);
  line-height: 1.5;
}

.empty-action-btn {
  height: 40px;
  font-weight: 500;
  transition: all 0.3s ease;
}

.empty-action-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(64, 128, 255, 0.2);
}

/* 对话框样式 */
.sub-modal,
.config-modal {
  border-radius: 16px;
}

.form-input {
  font-size: 13px;
  border-radius: 8px;
}

.code-input {
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
  font-size: 12px;
  background: linear-gradient(135deg, rgba(248, 250, 252, 0.8), rgba(241, 245, 249, 0.8));
  border-radius: 8px;
}

.sub-tabs {
  margin: 16px 0;
}

.modal-button {
  min-width: 70px;
  border-radius: 8px;
  font-weight: 500;
}

/* 暗黑模式适配 */
:deep(.dark) .header-content {
  background: rgba(24, 24, 28, 0.8);
  border-color: rgba(255, 255, 255, 0.1);
}

:deep(.dark) .sub-content {
  background: linear-gradient(135deg, rgba(248, 250, 252, 0.8), rgba(241, 245, 249, 0.8));
  border-color: rgba(255, 255, 255, 0.1);
}

:deep(.dark) .sub-card {
  background: rgba(40, 40, 48, 0.9);
  border-color: rgba(255, 255, 255, 0.1);
}

:deep(.dark) .sub-card:hover {
  border-color: rgba(99, 102, 241, 0.3);
}

:deep(.dark) .url-display {
  background: linear-gradient(135deg, rgba(30, 41, 59, 0.8), rgba(15, 23, 42, 0.8));
  border-color: rgba(71, 85, 105, 0.8);
}

:deep(.dark) .url-display:hover {
  background: linear-gradient(135deg, rgba(51, 65, 85, 0.8), rgba(30, 41, 59, 0.8));
  border-color: rgba(100, 116, 139, 0.8);
}

:deep(.dark) .code-input {
  background: linear-gradient(135deg, rgba(30, 41, 59, 0.8), rgba(15, 23, 42, 0.8));
}

:deep(.dark) .modal-content {
  background: linear-gradient(135deg, rgba(30, 41, 59, 0.8), rgba(15, 23, 42, 0.8));
}

:deep(.dark) .active-indicator {
  background: linear-gradient(135deg, rgba(51, 65, 85, 0.8), rgba(30, 41, 59, 0.8));
}

:deep(.dark) .sub-content:hover {
  background: linear-gradient(135deg, rgba(30, 41, 59, 0.8), rgba(15, 23, 42, 0.8));
}

/* 动画效果 */
@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .sub-container {
    padding: 12px;
  }

  .header-content {
    padding: 12px 16px;
    flex-direction: column;
    gap: 12px;
  }

  .title-wrapper {
    justify-content: center;
  }

  .sub-grid {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  .sub-card {
    min-height: 160px;
  }

  .sub-content {
    padding: 16px;
  }

  .card-header {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .action-buttons {
    justify-content: flex-end;
  }

  .url-display {
    padding: 8px 10px;
    min-height: 36px;
    font-size: 10px;
  }
}

@media (max-width: 480px) {
  .sub-card {
    min-height: 140px;
  }

  .sub-card-content {
    padding: 14px;
  }

  .card-footer {
    flex-direction: column;
    gap: 8px;
    align-items: stretch;
  }

  .use-btn {
    width: 100%;
    min-width: auto;
    height: 28px;
  }

  .name-wrapper {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }

  .status-tags {
    justify-content: flex-start;
  }

  .url-display {
    padding: 6px 8px;
    min-height: 32px;
    font-size: 9px;
    line-height: 1.4;
  }
}

@media (min-width: 1200px) {
  .sub-grid {
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 24px;
  }

  .sub-card {
    min-height: 190px;
  }
}

@media (min-width: 1600px) {
  .sub-grid {
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 28px;
  }

  .sub-card {
    min-height: 200px;
  }
}

/* Naive UI 组件样式覆盖 */
:deep(.n-badge) {
  --n-font-size: 10px;
}

:deep(.n-modal) {
  border-radius: 16px;
}

:deep(.n-form-item-label) {
  font-weight: 500;
}
</style>
