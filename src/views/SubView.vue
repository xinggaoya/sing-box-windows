<template>
  <n-space vertical size="large">
    <n-card>
      <template #header>
        <n-space align="center" justify="space-between">
          <n-space align="center">
            <n-h3 style="margin: 0">订阅管理</n-h3>
            <n-tag :bordered="false" type="info">{{ subStore.list.length }} 个订阅</n-tag>
          </n-space>
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
        </n-space>
      </template>

      <n-grid :x-gap="12" :y-gap="12" :cols="3">
        <n-grid-item v-for="(item, index) in subStore.list" :key="index">
          <n-card :class="{ 'sub-card': true, 'sub-card-loading': item.isLoading }" hoverable>
            <n-space vertical :size="12">
              <n-flex justify="space-between" align="center">
                <n-space align="center">
                  <n-icon size="18" color="#2080f0">
                    <link-outline />
                  </n-icon>
                  <n-text strong>{{ item.name }}</n-text>
                </n-space>
                <n-space>
                  <n-button
                    quaternary
                    circle
                    size="small"
                    @click="copyUrl(item.url)"
                  >
                    <template #icon>
                      <n-icon>
                        <copy-outline />
                      </n-icon>
                    </template>
                  </n-button>
                  <n-button
                    quaternary
                    circle
                    size="small"
                    @click="handleEdit(index, item)"
                  >
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
                  {{ item.lastUpdate ? formatTime(item.lastUpdate) : '从未更新' }}
                </n-text>
                <n-button
                  secondary
                  size="small"
                  :loading="item.isLoading"
                  @click="downloadSubscription(item.url, index)"
                >
                  <template #icon>
                    <n-icon>
                      <refresh-outline />
                    </n-icon>
                  </template>
                  更新
                </n-button>
              </n-flex>
            </n-space>
          </n-card>
        </n-grid-item>
      </n-grid>

      <n-empty
        v-if="!subStore.list.length"
        description="暂无订阅"
      >
        <template #extra>
          <n-button
            type="primary"
            @click="showAddModal = true"
          >
            添加订阅
          </n-button>
        </template>
      </n-empty>
    </n-card>
  </n-space>

  <!-- 添加/编辑订阅对话框 -->
  <n-modal
    v-model:show="showAddModal"
    :mask-closable="false"
    preset="dialog"
    :title="editIndex === null ? '添加订阅' : '编辑订阅'"
    positive-text="确认"
    negative-text="取消"
    @positive-click="handleConfirm"
    @negative-click="handleCancel"
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
  RefreshOutline
} from '@vicons/ionicons5'
import type { FormInst, FormRules } from 'naive-ui'

const message = useMessage()
const subStore = useSubStore()
const showAddModal = ref(false)
const editIndex = ref<number | null>(null)
const formRef = ref<FormInst | null>(null)
const isLoading = ref(false)

const formValue = ref({
  name: '',
  url: ''
})

const rules: FormRules = {
  name: [
    { required: true, message: '请输入订阅名称', trigger: 'blur' }
  ],
  url: [
    { required: true, message: '请输入订阅链接', trigger: 'blur' },
    { type: 'url', message: '请输入有效的URL', trigger: 'blur' }
  ]
}

const resetForm = () => {
  formValue.value = {
    name: '',
    url: ''
  }
  editIndex.value = null
}

const handleEdit = (index: number, item: any) => {
  editIndex.value = index
  formValue.value = {
    name: item.name,
    url: item.url
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
          url: formValue.value.url
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

const downloadSubscription = async (url: string, index: number) => {
  if (!subStore.list[index]) return
  subStore.list[index].isLoading = true
  
  try {
    await invoke('download_subscription', { url })
    subStore.list[index].lastUpdate = Date.now()
    message.success('更新成功')
  } catch (error) {
    message.error(error as string)
  } finally {
    if (subStore.list[index]) {
      subStore.list[index].isLoading = false
    }
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
.sub-card {
  transition: all 0.3s ease;
}

.sub-card:hover {
  transform: translateY(-2px);
}

.sub-card-loading {
  opacity: 0.7;
}
</style>
