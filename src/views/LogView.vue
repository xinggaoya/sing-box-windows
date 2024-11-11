<template>
  <n-card style="height: 100%" content-style="height: 100%;padding: 10px">
    <n-scrollbar>
      <n-flex vertical>
        <div v-for="(log,index) in logs" :key="index">
          <n-tag :type="log.type"> {{ log.type }}</n-tag>
          {{ log.payload }}
        </div>
      </n-flex>
    </n-scrollbar>
  </n-card>
</template>
<script setup lang="ts">
import { createWebSocket } from '@/utils'
import { onMounted, ref } from 'vue'

const logs = ref<any>([])

onMounted(() => {
  getLogs()
})

const getLogs = async () => {
  createWebSocket(`ws://127.0.0.1:9090/logs?token=`, (data) => {
    logs.value.push(data)
  })
}
</script>

<style scoped>

</style>