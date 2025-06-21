<template>
  <div
    ref="containerRef"
    class="virtual-list-container"
    :style="{ height: containerHeight + 'px' }"
    @scroll="handleScroll"
  >
    <div class="virtual-list-spacer" :style="{ height: totalHeight + 'px' }">
      <div class="virtual-list-content" :style="{ transform: `translateY(${offsetY}px)` }">
        <div
          v-for="item in visibleItems"
          :key="getItemKey(item.data, item.index)"
          :style="{ height: itemHeight + 'px' }"
          class="virtual-list-item"
        >
          <slot :item="item.data" :index="item.index" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts" generic="T">
import { ref, computed, onMounted, onUnmounted, watchEffect } from 'vue'
// 简单的throttle函数
const throttle = (func: (...args: unknown[]) => void, delay: number) => {
  let timeoutId: number | null = null
  let lastExecTime = 0
  return function (...args: unknown[]) {
    const currentTime = Date.now()

    if (currentTime - lastExecTime > delay) {
      func(...args)
      lastExecTime = currentTime
    } else {
      if (timeoutId !== null) {
        clearTimeout(timeoutId)
      }
      timeoutId = window.setTimeout(
        () => {
          func(...args)
          lastExecTime = Date.now()
          timeoutId = null
        },
        delay - (currentTime - lastExecTime),
      )
    }
  }
}

interface Props {
  items: T[]
  itemHeight: number
  containerHeight: number
  buffer?: number
  keyField?: string | ((item: T, index: number) => string | number)
}

interface VisibleItem<T> {
  data: T
  index: number
}

const props = withDefaults(defineProps<Props>(), {
  buffer: 5,
  keyField: 'id',
})

const containerRef = ref<HTMLElement>()
const scrollTop = ref(0)

// 计算总高度
const totalHeight = computed(() => props.items.length * props.itemHeight)

// 计算可见区域内的起始和结束索引
const visibleRange = computed(() => {
  const start = Math.floor(scrollTop.value / props.itemHeight)
  const end = Math.min(
    start + Math.ceil(props.containerHeight / props.itemHeight) + props.buffer * 2,
    props.items.length,
  )
  const safeStart = Math.max(0, start - props.buffer)

  return {
    start: safeStart,
    end,
  }
})

// 计算可见项目
const visibleItems = computed(() => {
  const { start, end } = visibleRange.value
  const items: VisibleItem<T>[] = []

  for (let i = start; i < end; i++) {
    if (props.items[i] !== undefined) {
      items.push({
        data: props.items[i],
        index: i,
      })
    }
  }

  return items
})

// 计算偏移量
const offsetY = computed(() => visibleRange.value.start * props.itemHeight)

// 获取项目的唯一键
function getItemKey(item: T, index: number): string | number {
  if (typeof props.keyField === 'function') {
    return props.keyField(item, index)
  }

  if (typeof props.keyField === 'string') {
    const value = (item as Record<string, unknown>)[props.keyField]
    return value !== undefined ? String(value) : index
  }

  return index
}

// 滚动处理（简化版避免类型问题）
let scrollTimer: number | null = null
function handleScroll(event: Event) {
  if (scrollTimer) {
    clearTimeout(scrollTimer)
  }

  scrollTimer = window.setTimeout(() => {
    const target = event.target as HTMLElement
    scrollTop.value = target.scrollTop
  }, 16) // 约60fps
}

// 滚动到指定索引
function scrollToIndex(index: number, behavior: ScrollBehavior = 'smooth') {
  if (!containerRef.value) return

  const targetScrollTop = index * props.itemHeight
  containerRef.value.scrollTo({
    top: targetScrollTop,
    behavior,
  })
}

// 滚动到顶部
function scrollToTop(behavior: ScrollBehavior = 'smooth') {
  scrollToIndex(0, behavior)
}

// 滚动到底部
function scrollToBottom(behavior: ScrollBehavior = 'smooth') {
  scrollToIndex(props.items.length - 1, behavior)
}

// 暴露方法供父组件使用
defineExpose({
  scrollToIndex,
  scrollToTop,
  scrollToBottom,
  getVisibleRange: () => visibleRange.value,
})

// 清理函数
onUnmounted(() => {
  if (scrollTimer) {
    clearTimeout(scrollTimer)
    scrollTimer = null
  }
})
</script>

<style scoped>
.virtual-list-container {
  position: relative;
  overflow-y: auto;
  overflow-x: hidden;
}

.virtual-list-spacer {
  position: relative;
  width: 100%;
}

.virtual-list-content {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
}

.virtual-list-item {
  width: 100%;
  box-sizing: border-box;
}
</style>
