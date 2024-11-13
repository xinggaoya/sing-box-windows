<template>
  <n-card style="height: 100%" content-style="height: 100%;padding: 10px">
    <n-scrollbar>
      <n-flex vertical>
        <div v-for="(log,index) in infoStore.logs" :key="index">
          <n-tag :type="log.type"> {{ log.type }}</n-tag>
          {{ log.payload }}
        </div>
      </n-flex>
    </n-scrollbar>
  </n-card>
</template>
<script setup lang="ts">
import { useInfoStore } from '@/stores/infoStore'

const infoStore = useInfoStore()
</script>

<style scoped>

</style>