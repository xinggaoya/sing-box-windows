<template>
  <n-card content-style="padding: 10px">
    <n-space>
      <n-input type="textarea" v-model:value="url" placeholder="请输入订阅链接" />
      <n-button type="primary" @click="downloadSubscription" :loading>
        下载订阅
      </n-button>
    </n-space>
  </n-card>
</template>
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { useMessage } from 'naive-ui'
import { ref } from 'vue'

const url = ref('')
const loading = ref(false)
const message = useMessage()

const downloadSubscription = async () => {
  loading.value = true
  const res = await invoke('download_subscription', { url: url.value })
  loading.value = false
  message.success('下载完成')
}

</script>
<style scoped>

</style>