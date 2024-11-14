<template>
  <n-card style="height: 100%" content-style="height: 100%;padding: 10px">
    <n-infinite-scroll @load="handleLoad">
      <n-flex vertical>
        <div v-for="(log, index) in logs" :key="index">
          <n-tag :type="log.type"> {{ log.type }}</n-tag>
          {{ log.payload }}
        </div>
      </n-flex>
    </n-infinite-scroll>
  </n-card>
</template>
<script setup lang="ts">
import { useInfoStore } from '@/stores/infoStore'
import { onMounted, ref } from 'vue'

const infoStore = useInfoStore()

const logs = ref<any>([])

onMounted(() => {
  logs.value = infoStore.logs.slice(0, 20)
})

const handleLoad = () => {
  // 每次放入二十条
  logs.value.push(...infoStore.logs.slice(logs.value.length, logs.value.length + 20))
  console.log(logs.value)
}
</script>

<style scoped></style>
