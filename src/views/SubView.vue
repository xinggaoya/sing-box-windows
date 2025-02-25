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
              订阅管理
            </n-h3>
            <n-tag :bordered="false" type="info" size="medium" class="sub-count-tag">
              {{ subStore.list.length }} 个订阅
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
            添加订阅
          </n-tooltip>
        </div>
      </template>

      <n-grid :x-gap="16" :y-gap="16" :cols="gridCols" responsive="screen">
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
                <n-space align="center" :size="10">
                  <n-icon size="20" :color="subStore.activeIndex === index ? '#18a058' : '#4080ff'">
                    <link-outline />
                  </n-icon>
                  <n-text strong class="sub-name">{{ item.name }}</n-text>
                  <n-tag
                    v-if="subStore.activeIndex === index"
                    type="success"
                    size="small"
                    :bordered="false"
                    class="active-tag"
                  >
                    使用中
                  </n-tag>
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
                    复制链接
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
                    编辑订阅
                  </n-tooltip>

                  <n-popconfirm
                    @positive-click="deleteSubscription(index)"
                    positive-text="删除"
                    negative-text="取消"
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
                    确定要删除这个订阅吗？
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
                  {{ item.lastUpdate ? formatTime(item.lastUpdate) : '从未使用' }}
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
                  {{ subStore.activeIndex === index ? '重新使用' : '使用' }}
                </n-button>
              </n-flex>
            </n-space>
          </n-card>
        </n-grid-item>
      </n-grid>

      <n-empty v-if="!subStore.list.length" description="暂无订阅" class="empty-container">
        <template #extra>
          <n-button type="primary" @click="showAddModal = true" class="add-sub-button">
            <template #icon>
              <n-icon><add-outline /></n-icon>
            </template>
            添加订阅
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
    :title="editIndex === null ? '添加订阅' : '编辑订阅'"
    :bordered="false"
    style="width: 500px"
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
      <n-form-item label="名称" path="name">
        <n-input
          v-model:value="formValue.name"
          placeholder="请输入订阅名称"
          @keydown.enter.prevent
          class="form-input"
        />
      </n-form-item>
      <n-form-item label="链接" path="url">
        <n-input
          v-model:value="formValue.url"
          type="textarea"
          placeholder="请输入订阅链接"
          :autosize="{ minRows: 2, maxRows: 4 }"
          class="form-input"
        />
      </n-form-item>
    </n-form>
    <template #action>
      <n-space justify="end">
        <n-button @click="handleCancel" class="modal-button">取消</n-button>
        <n-button type="primary" @click="handleConfirm" :loading="isLoading" class="modal-button">
          确认
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
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
} from '@vicons/ionicons5'
import type { FormInst, FormRules } from 'naive-ui'
import { useWindowSize } from '@vueuse/core'

interface Subscription {
  name: string
  url: string
  lastUpdate?: number
  isLoading: boolean
}

const message = useMessage()
const subStore = useSubStore()
const showAddModal = ref(false)
const editIndex = ref<number | null>(null)
const formRef = ref<FormInst | null>(null)
const isLoading = ref(false)
const { width } = useWindowSize()

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
})

const rules: FormRules = {
  name: [{ required: true, message: '请输入订阅名称', trigger: 'blur' }],
  url: [
    { required: true, message: '请输入订阅链接', trigger: 'blur' },
    { type: 'url', message: '请输入有效的URL', trigger: 'blur' },
  ],
}

const resetForm = () => {
  formValue.value = {
    name: '',
    url: '',
    isLoading: false,
  }
  editIndex.value = null
}

const handleEdit = (index: number, item: Subscription) => {
  editIndex.value = index
  formValue.value = {
    name: item.name,
    url: item.url,
    isLoading: item.isLoading,
  }
  showAddModal.value = true
}

const handleConfirm = () => {
  formRef.value?.validate(async (errors) => {
    if (!errors) {
      if (editIndex.value === null) {
        // 添加新订阅
        subStore.list.push({
          ...formValue.value,
          lastUpdate: undefined,
        })
        message.success('订阅添加成功')
      } else {
        // 更新订阅
        subStore.list[editIndex.value] = {
          ...subStore.list[editIndex.value],
          name: formValue.value.name,
          url: formValue.value.url,
        }
        message.success('订阅更新成功')
      }
      showAddModal.value = false
      resetForm()
    }
  })
}

const handleCancel = () => {
  showAddModal.value = false
  resetForm()
}

const deleteSubscription = (index: number) => {
  if (subStore.activeIndex === index) {
    message.warning('不能删除当前正在使用的订阅')
    return
  }
  subStore.list.splice(index, 1)
  message.success('订阅已删除')
}

const useSubscription = async (url: string, index: number) => {
  try {
    // 标记正在加载
    subStore.list[index].isLoading = true

    // 调用后端API下载订阅
    await invoke('download_subscription', { url })

    // 更新订阅状态
    subStore.list[index].lastUpdate = Date.now()
    subStore.activeIndex = index
    message.success('订阅使用成功')
  } catch (error) {
    message.error('订阅使用失败：' + error)
  } finally {
    subStore.list[index].isLoading = false
  }
}

const copyUrl = (url: string) => {
  navigator.clipboard.writeText(url)
  message.success('链接已复制到剪贴板')
}

const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp)
  return `最后更新: ${date.toLocaleString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })}`
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
}

.active-tag {
  font-weight: 500;
  padding: 2px 8px;
}

.url-container {
  padding: 8px 10px;
  border-radius: 8px;
  font-family: monospace;
  font-size: 13px;
  color: var(--n-text-color-2);
  word-break: break-all;
  border: 1px solid var(--n-border-color);
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
</style>
