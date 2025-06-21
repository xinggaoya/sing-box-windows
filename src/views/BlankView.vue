<template>
  <div class="blank-view">
    <!-- 最小化的空白页面，用于托盘状态下减少内存占用 -->
    <div class="tray-indicator">
      <div class="tray-icon"></div>
      <span class="tray-text">{{ t('common.minimizedToTray') }}</span>
    </div>

    <!-- 调试信息（仅开发环境） -->
    <div v-if="isDev" class="debug-info">
      <div class="debug-text">内存优化模式</div>
      <div class="debug-text">DOM节点数: {{ domNodeCount }}</div>
      <div class="debug-text">已清理资源数: {{ cleanedResourcesCount }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'

const { t } = useI18n()

// 开发环境标志
const isDev = computed(() => import.meta.env.DEV)

// 调试信息
const domNodeCount = ref(0)
const cleanedResourcesCount = ref(0)

// 清理所有可能的内存占用
const performMemoryCleanup = () => {
  let cleaned = 0

  // 触发垃圾回收提示
  if (window.gc) {
    window.gc()
  }

  // 清理未使用的图片和媒体资源
  const images = document.querySelectorAll('img')
  images.forEach((img) => {
    if (!img.closest('.blank-view')) {
      img.src = ''
      cleaned++
    }
  })

  // 清理canvas元素
  const canvases = document.querySelectorAll('canvas')
  canvases.forEach((canvas) => {
    if (!canvas.closest('.blank-view')) {
      const ctx = canvas.getContext('2d')
      if (ctx) {
        ctx.clearRect(0, 0, canvas.width, canvas.height)
        cleaned++
      }
    }
  })

  // 清理视频元素
  const videos = document.querySelectorAll('video')
  videos.forEach((video) => {
    if (!video.closest('.blank-view')) {
      video.pause()
      video.src = ''
      cleaned++
    }
  })

  cleanedResourcesCount.value = cleaned

  // 更新DOM节点计数
  if (isDev.value) {
    domNodeCount.value = document.querySelectorAll('*').length
  }
}

onMounted(() => {
  console.log('🔋 空白页面已挂载，激活内存优化模式')

  // 初始DOM节点计数
  if (isDev.value) {
    domNodeCount.value = document.querySelectorAll('*').length
  }

  // 延迟执行内存清理，确保页面切换完成
  setTimeout(() => {
    performMemoryCleanup()
    console.log('✨ 内存优化完成，已清理资源:', cleanedResourcesCount.value)
  }, 500)

  // 监听内存清理请求
  mitt.on('memory-cleanup-requested', performMemoryCleanup)
})

onBeforeUnmount(() => {
  console.log('🔋 空白页面卸载，清理事件监听器')
  // 清理事件监听
  mitt.off('memory-cleanup-requested', performMemoryCleanup)
})
</script>

<style scoped>
.blank-view {
  width: 100%;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--n-card-color);
  /* 最小化DOM渲染开销 */
  contain: layout style paint;
}

.tray-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  opacity: 0.3;
  transition: opacity 0.3s ease;
}

.tray-indicator:hover {
  opacity: 0.6;
}

.tray-icon {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--n-text-color-disabled);
  position: relative;
}

.tray-icon::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--n-success-color);
  animation: pulse 2s infinite;
}

.tray-text {
  font-size: 12px;
  color: var(--n-text-color-disabled);
  font-weight: 500;
}

@keyframes pulse {
  0%,
  100% {
    opacity: 0.8;
    transform: translate(-50%, -50%) scale(1);
  }
  50% {
    opacity: 0.4;
    transform: translate(-50%, -50%) scale(1.2);
  }
}

/* 减少重绘和回流 */
* {
  will-change: auto;
}

/* 调试信息样式 */
.debug-info {
  position: absolute;
  bottom: 20px;
  right: 20px;
  background: rgba(0, 0, 0, 0.1);
  backdrop-filter: blur(10px);
  border-radius: 8px;
  padding: 12px 16px;
  font-family: 'Courier New', monospace;
  font-size: 11px;
  color: var(--n-text-color-disabled);
  border: 1px solid var(--n-border-color);
}

.debug-text {
  margin: 2px 0;
  white-space: nowrap;
}

/* 针对高性能模式的优化 */
@media (prefers-reduced-motion: reduce) {
  .tray-icon::after {
    animation: none;
  }

  .tray-indicator {
    transition: none;
  }
}
</style>
