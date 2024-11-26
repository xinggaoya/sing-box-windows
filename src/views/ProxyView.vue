<template>
  <n-card style="height: 100%" content-style="height: 100%;padding: 10px">
    <n-scrollbar>
      <n-collapse>
        <n-collapse-item
          v-for="(item, index) in list"
          :key="index"
          :title="item.name"
          :name="index"
        >
          <n-flex>
            <n-card
              v-for="(t, i) in item.all"
              :key="i"
              header-style="padding: 5px;font-size: 14px"
              hoverable
              :embedded="item.now === t.name"
              style="width: 160px; height: 70px; cursor: pointer"
            >
              <template #header>
                <n-space vertical>
                  <n-ellipsis>{{ t.name }}</n-ellipsis>
                  <n-flex justify="space-between">
                    <n-tag size="small" type="info">{{ t.delay }}</n-tag>
                    <n-space :size="2">
                      <n-button
                        @click="changeProxy(item.name, t.name, index)"
                        type="success"
                        size="tiny"
                      >
                        使用
                      </n-button>
                      <n-button @click="getDelay(t.name, index, i)" type="primary" size="tiny">
                        测速
                      </n-button>
                    </n-space>
                  </n-flex>
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
import { useMessage } from 'naive-ui'

const list = ref<any>([])
const message = useMessage()

onMounted(() => {
  init()
})

const init = () => {
  fetch('http://127.0.0.1:9090/proxies', {
    headers: {
      'Content-Type': 'application/json',
    },
    method: 'GET',
  })
    .then((res) => res.json())
    .then((res) => {
      const data = res.proxies
      // 循环对象
      Object.keys(data).forEach((key) => {
        const item = data[key]
        if (item.type === 'Selector' || item.type === 'URLTest') {
          const data: any = {
            name: key,
            now: item.now,
            delay: '0ms',
            all: [],
          }
          item.all.forEach((item: string) => {
            data.all.push({
              name: item,
              delay: '0ms',
            })
          })
          list.value.push(data)
        }
      })
    })
}

// 获取延迟
const getDelay = (
  name: string,
  index: number,
  i: number,
  server: string = 'https://www.gstatic.com/generate_204',
) => {
  fetch(`http://127.0.0.1:9090/proxies/${name}/delay?url=${server}&timeout=5000`, {
    headers: {
      'Content-Type': 'application/json',
    },
    method: 'GET',
  }).then((res) =>
    res.json().then((res) => {
      list.value[index].all[i].delay = res.delay + 'ms'
    }),
  )
}

// 切换代理
const changeProxy = (type: string, name: string, index: number) => {
  fetch(`http://127.0.0.1:9090/proxies/${type}`, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      name: name,
    }),
  }).then((res) => {
    if (res.status === 400) {
      message.error('切换失败,可能是不可切换的代理组')
      return
    }
    if (res.status === 204) {
      message.success('切换成功')
      list.value[index].now = name
      return
    }
  })
}
</script>

<style scoped>
.about {
  padding: 10px;
}
</style>
