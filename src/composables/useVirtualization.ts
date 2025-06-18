import { ref, computed, onMounted, onUnmounted, Ref } from 'vue'

export interface VirtualizationOptions {
  itemHeight: number
  buffer?: number
  container?: Ref<HTMLElement | null>
}

export function useVirtualization<T>(items: Ref<T[]>, options: VirtualizationOptions) {
  const { itemHeight, buffer = 5, container } = options

  const scrollTop = ref(0)
  const containerHeight = ref(0)

  // 计算可见范围
  const visibleRange = computed(() => {
    const start = Math.floor(scrollTop.value / itemHeight) - buffer
    const end = Math.ceil((scrollTop.value + containerHeight.value) / itemHeight) + buffer

    return {
      start: Math.max(0, start),
      end: Math.min(items.value.length, end),
    }
  })

  // 可见项目
  const visibleItems = computed(() => {
    const { start, end } = visibleRange.value
    return items.value.slice(start, end).map((item, index) => ({
      item,
      index: start + index,
      style: {
        position: 'absolute' as const,
        top: `${(start + index) * itemHeight}px`,
        height: `${itemHeight}px`,
        width: '100%',
      },
    }))
  })

  // 容器高度
  const totalHeight = computed(() => items.value.length * itemHeight)

  // 滚动处理
  const handleScroll = (e: Event) => {
    const target = e.target as HTMLElement
    scrollTop.value = target.scrollTop
  }

  // 更新容器高度
  const updateContainerHeight = () => {
    if (container?.value) {
      containerHeight.value = container.value.clientHeight
    }
  }

  // ResizeObserver
  let resizeObserver: ResizeObserver | null = null

  onMounted(() => {
    updateContainerHeight()

    if (container?.value) {
      container.value.addEventListener('scroll', handleScroll)

      resizeObserver = new ResizeObserver(() => {
        updateContainerHeight()
      })
      resizeObserver.observe(container.value)
    }
  })

  onUnmounted(() => {
    if (container?.value) {
      container.value.removeEventListener('scroll', handleScroll)
    }

    if (resizeObserver) {
      resizeObserver.disconnect()
    }
  })

  return {
    visibleItems,
    totalHeight,
    visibleRange,
    containerStyle: computed(() => ({
      position: 'relative' as const,
      height: `${totalHeight.value}px`,
    })),
  }
}
