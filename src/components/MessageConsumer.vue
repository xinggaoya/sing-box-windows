<script setup lang="ts">
/**
 * 仅用于捕获 naive-ui 的 useMessage() 实例并经 mitt 转发。
 *
 * useMessage() 必须在 <n-message-provider> 的直接子级上下文中调用，
 * 因此用一个不渲染任何内容的子组件来获取实例，再通过事件总线交给
 * 那些无法处于 provider 上下文内的调用方（如 useAppBootstrap）。
 */
import { onMounted } from 'vue'
import { useMessage } from 'naive-ui'
import mitt from '@/utils/mitt'

const message = useMessage()

onMounted(() => {
  mitt.emit('message-instance-ready', message)
})
</script>

<template>
  <!-- 故意为空：该组件仅作为 provider 上下文探针，不渲染任何可见内容 -->
  <span aria-hidden="true" style="display: none" />
</template>
