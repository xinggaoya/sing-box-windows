<template>
  <div class="sub-container">
    <!-- 订阅管理卡片 -->
    <n-card class="sub-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <div class="header-left">
            <n-h3 class="card-title">
              <n-icon size="20" class="card-icon">
                <link-outline />
              </n-icon>
              订阅管理
            </n-h3>
            <n-tag :bordered="false" type="info" size="small"
              >{{ subStore.list.length }} 个订阅</n-tag
            >
          </div>
          <n-button
            quaternary
            circle
            type="primary"
            @click="showAddModal = true"
            :disabled="isLoading"
          >
            <template #icon>
              <n-icon>
                <add-outline />
              </n-icon>
            </template>
          </n-button>
        </div>
      </template>

      <n-grid :x-gap="12" :y-gap="12" :cols="3">
        <n-grid-item v-for="(item, index) in subStore.list" :key="index">
          <n-card
            :class="{
              'sub-node-card': true,
              'sub-node-card-active': subStore.activeIndex === index,
            }"
            :bordered="false"
            hoverable
          >
            <n-space vertical :size="12">
              <n-flex justify="space-between" align="center">
                <n-space align="center">
                  <n-icon size="18" :color="subStore.activeIndex === index ? '#18a058' : '#2080f0'">
                    <link-outline />
                  </n-icon>
                  <n-text strong>{{ item.name }}</n-text>
                  <n-tag
                    v-if="subStore.activeIndex === index"
                    type="success"
                    size="small"
                    :bordered="false"
                    >使用中</n-tag
                  >
                </n-space>
                <n-space>
                  <n-button quaternary circle size="small" @click="copyUrl(item.url)">
                    <template #icon>
                      <n-icon>
                        <copy-outline />
                      </n-icon>
                    </template>
                  </n-button>
                  <n-button quaternary circle size="small" @click="handleEdit(index, item)">
                    <template #icon>
                      <n-icon>
                        <create-outline />
                      </n-icon>
                    </template>
                  </n-button>
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

              <n-ellipsis style="max-width: 100%" :tooltip="{ width: 'trigger' }">
                {{ item.url }}
              </n-ellipsis>

              <n-flex justify="space-between" align="center">
                <n-text depth="3" style="font-size: 12px">
                  {{ item.lastUpdate ? formatTime(item.lastUpdate) : '从未使用' }}
                </n-text>
                <n-button
                  secondary
                  size="small"
                  :loading="item.isLoading"
                  @click="useSubscription(item.url, index)"
                  :type="subStore.activeIndex === index ? 'success' : 'primary'"
                  :ghost="subStore.activeIndex !== index"
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

      <n-empty v-if="!subStore.list.length" description="暂无订阅">
        <template #extra>
          <n-button type="primary" @click="showAddModal = true" ghost>
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
        />
      </n-form-item>
      <n-form-item label="链接" path="url">
        <n-input
          v-model:value="formValue.url"
          type="textarea"
          placeholder="请输入订阅链接"
          :autosize="{ minRows: 2, maxRows: 4 }"
        />
      </n-form-item>
    </n-form>
    <template #action>
      <n-space justify="end">
        <n-button @click="handleCancel" ghost>取消</n-button>
        <n-button type="primary" @click="handleConfirm" :loading="isLoading"> 确认 </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { ref } from 'vue'
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
        // 添加
        subStore.add(formValue.value.name, formValue.value.url)
        message.success('添加成功')
      } else {
        // 编辑
        subStore.list[editIndex.value] = {
          ...subStore.list[editIndex.value],
          name: formValue.value.name,
          url: formValue.value.url,
        }
        message.success('修改成功')
      }
      showAddModal.value = false
      resetForm()
    }
  })
}

const handleCancel = () => {
  resetForm()
}

const deleteSubscription = (index: number) => {
  subStore.list.splice(index, 1)
  message.success('删除成功')
}

const useSubscription = async (url: string, index: number) => {
  // 设置加载状态
  const currentItem = subStore.list[index]
  currentItem.isLoading = true
  try {
    await invoke('download_subscription', { url })
    currentItem.lastUpdate = Date.now()
    subStore.activeIndex = index
    message.success('订阅已启用')
  } catch (error) {
    message.error('启用失败：' + error)
  } finally {
    currentItem.isLoading = false
  }
}

const copyUrl = async (url: string) => {
  try {
    await navigator.clipboard.writeText(url)
    message.success('已复制到剪贴板')
  } catch (error) {
    message.error('复制失败')
  }
}

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp)
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')} ${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
}
</script>

<style scoped>
.sub-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px;
}

.sub-card {
  border-radius: 8px;
  transition: all 0.3s ease;
}

.sub-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-title {
  display: flex;
  align-items: center;
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.card-icon {
  margin-right: 8px;
  color: var(--primary-color);
}

.sub-node-card {
  transition: all 0.3s ease;
  border-radius: 8px;
  background-color: var(--card-color);
}

.sub-node-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.sub-node-card-active {
  background-color: rgba(var(--primary-color-rgb), 0.1);
  border: 1px solid var(--primary-color);
}

:deep(.n-button.n-button--ghost) {
  border-color: var(--primary-color);
  color: var(--primary-color);
}

:deep(.n-button.n-button--ghost:hover) {
  background-color: var(--primary-color);
  color: white;
}

:deep(.n-button.n-button--ghost.n-button--error) {
  border-color: var(--error-color);
  color: var(--error-color);
}

:deep(.n-button.n-button--ghost.n-button--error:hover) {
  background-color: var(--error-color);
  color: white;
}

:deep(.n-modal) {
  --n-title-font-size: 16px;
  --n-padding: 20px;
}

:deep(.n-form) {
  margin-top: 8px;
}

:deep(.n-form-item-label) {
  font-weight: 500;
}

:deep(.n-input) {
  --n-border-radius: 4px;
}

:deep(.n-tag) {
  --n-border-radius: 4px;
}

:deep(.n-empty) {
  padding: 32px 0;
}
</style>
