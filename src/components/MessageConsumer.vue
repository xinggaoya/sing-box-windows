<script setup lang="ts">
/**
 * 仅用于捕获 naive-ui 的 useMessage() 实例并写入 AppStore。
 *
 * useMessage() 必须在 <n-message-provider> 的直接子级上下文中调用，
 * 因此用一个不渲染任何内容的子组件来获取实例，再通过事件总线交给
 * 那些无法处于 provider 上下文内的调用方（如 useAppBootstrap）。
 */
import { onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import { useAppStore } from '@/stores'

const message = useMessage()

const appStore = useAppStore()

onMounted(() => {
  // 直接写入 store：此前经 mitt 转发给 useAppBootstrap 再写 store，
  // 多绕两跳且让「谁持有 message 实例」这件事分散在 3 个文件里。
  appStore.setMessageInstance(message)
})
</script>

<template>
  <!-- 故意为空：该组件仅作为 provider 上下文探针，不渲染任何可见内容 -->
  <span aria-hidden="true" style="display: none" />
</template>
