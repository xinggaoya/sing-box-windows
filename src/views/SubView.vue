<template>
  <n-card title="订阅列表" content-style="padding: 10px">
    <n-space>
      <n-card :title="item.name" v-for="(item, index) in subStore.list" :key="index">
        <n-ellipsis>{{ item.url }}</n-ellipsis>
        <div>
          <n-button type="primary" @click="downloadSubscription(url)">下载</n-button>
          <n-button type="error" @click="deleteSubscription(index)">删除</n-button>
        </div>
      </n-card>
    </n-space>
    <n-space vertical>
      <n-input type="text" v-model:value="name" placeholder="请输入订阅名称" />
      <n-input type="textarea" v-model:value="url" placeholder="请输入订阅链接" />
      <n-button type="primary" @click="addSubscription" :loading>添加订阅</n-button>
    </n-space>
  </n-card>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { ref } from 'vue'
import { useSubStore } from '@/stores/SubStore'

const name = ref('')
const url = ref('')
const loading = ref(false)
const message = useMessage()
const subStore = useSubStore()

// 添加订阅
const addSubscription = async () => {
  if (name.value === '' || url.value === '') {
    message.error('请输入订阅名称和链接')
    return
  }
  subStore.add(name.value, url.value)
}

// 删除
const deleteSubscription = (index: number) => {
  subStore.list.splice(index, 1)
}

const downloadSubscription = async (url: string) => {
  loading.value = true
  const res = await invoke('download_subscription', { url: url })
  loading.value = false
  message.success('下载完成')
}
</script>
<style scoped></style>
