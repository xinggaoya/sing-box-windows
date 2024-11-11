<template>
  <n-card style="height: 100%" content-style="height: 100%;padding: 10px">
    <n-scrollbar>
      <n-collapse>
        <n-collapse-item v-for="(item,index) in list" :title="item.name" :name="index">
          <n-flex justify="space-between">
            <n-card v-for="(t,i) in item.all" :key="i" style="width: 200px">
              <template #header>
                <n-space>
                  <n-text>{{ t }}</n-text>
                  <n-text type="info">{{ item.history[i]?.delay }}</n-text>
                </n-space>
              </template>
            </n-card>
          </n-flex>
        </n-collapse-item>
      </n-collapse>
    </n-scrollbar>
  </n-card>
</template>

<script lang="ts" setup>
import { onMounted, ref } from 'vue'

const list = ref<any>([])

onMounted(() => {
  init()
})

const init = () => {
  fetch('http://127.0.0.1:9090/proxies', {
    headers: {
      'Content-Type': 'application/json'
    }
  }).then(res => res.json()).then(res => {
    const data = res.proxies
    // 循环对象
    Object.keys(data).forEach(key => {
      const item = data[key]
      if (item.type === 'Selector' || item.type === 'URLTest') {
        list.value.push(item)
      }
    })
  })
}
</script>

<style scoped>
.about {
  padding: 10px;
}
</style>
