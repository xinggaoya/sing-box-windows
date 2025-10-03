<template>
  <div class="blank-view">
    <!-- 最小化的空白页面，用于托盘状态下减少内存占用 -->
    <div class="tray-indicator" v-show="showIndicator">
      <div class="tray-icon"></div>
      <span class="tray-text">{{ t('common.minimizedToTray') }}</span>
    </div>

    <!-- 调试信息（仅开发环境） -->
    <div v-if="isDev" class="debug-info">
      <div class="debug-text">内存优化模式</div>
      <div class="debug-text">DOM节点数: {{ domNodeCount }}</div>
      <div class="debug-text">已清理资源数: {{ cleanedResourcesCount }}</div>
      <div class="debug-text">GPU优化: {{ gpuOptimized ? '已启用' : '未启用' }}</div>
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
const showIndicator = ref(false)
const gpuOptimized = ref(false)

// 增强的内存清理函数
const performMemoryCleanup = () => {
  let cleaned = 0

  // 触发垃圾回收提示
  if (window.gc) {
    window.gc()
  }

  // 1. 强化GPU内存优化
  optimizeGPUMemory()
  cleaned++

  // 2. 清理所有动画和过渡效果
  cleanupAnimations()
  cleaned++

  // 3. 清理媒体资源
  cleanupMediaResources()
  cleaned++

  // 4. 清理定时器和事件监听器
  cleanupTimersAndListeners()
  cleaned++

  // 5. 优化Vue组件内存
  optimizeVueComponents()
  cleaned++

  // 6. 清理Canvas和WebGL上下文
  cleanupGraphicsContexts()
  cleaned++

  cleanedResourcesCount.value = cleaned
  gpuOptimized.value = true

  // 更新DOM节点计数
  if (isDev.value) {
    domNodeCount.value = document.querySelectorAll('*').length
    console.log(`🧹 内存清理完成，清理了 ${cleaned} 类资源，剩余 DOM 节点: ${domNodeCount.value}`)
  }
}

// GPU内存优化
const optimizeGPUMemory = () => {
  console.log('🎯 开始GPU内存优化')

  // 强制停止所有CSS动画和GPU加速
  const allElements = document.querySelectorAll('*')
  allElements.forEach((element) => {
    if (!element.closest('.blank-view')) {
      const htmlElement = element as HTMLElement
      htmlElement.style.animation = 'none'
      htmlElement.style.transition = 'none'
      htmlElement.style.transform = 'none'
      htmlElement.style.willChange = 'auto'
      htmlElement.style.backfaceVisibility = 'hidden'
      htmlElement.style.perspective = 'none'

      // 移除GPU加速相关的CSS属性
      htmlElement.style.filter = 'none'
      htmlElement.style.backdropFilter = 'none'
      htmlElement.style.boxShadow = 'none'
      htmlElement.style.textShadow = 'none'
    }
  })
}

// 清理动画
const cleanupAnimations = () => {
  console.log('🎬 清理CSS动画和过渡')

  // 查找所有有动画的元素
  const animatedElements = document.querySelectorAll('[style*="animation"], [style*="transition"]')
  animatedElements.forEach((element) => {
    const htmlElement = element as HTMLElement
    htmlElement.style.animation = 'none'
    htmlElement.style.transition = 'none'
  })
}

// 清理媒体资源
const cleanupMediaResources = () => {
  console.log('🖼️ 清理媒体资源')

  // 清理图片
  const images = document.querySelectorAll('img')
  images.forEach((img) => {
    if (!img.closest('.blank-view')) {
      const htmlImg = img as HTMLImageElement
      htmlImg.src = ''
      htmlImg.removeAttribute('srcset')
      htmlImg.removeAttribute('loading')
    }
  })

  // 清理视频
  const videos = document.querySelectorAll('video')
  videos.forEach((video) => {
    if (!video.closest('.blank-view')) {
      const htmlVideo = video as HTMLVideoElement
      htmlVideo.pause()
      htmlVideo.src = ''
      htmlVideo.removeAttribute('src')
    }
  })
}

// 清理定时器和事件监听器
const cleanupTimersAndListeners = () => {
  console.log('⏰ 清理定时器和事件监听器')

  // 触发全局清理事件
  mitt.emit('global-cleanup-requested')

  // 清理可能的定时器
  for (let i = 1; i < 99999; i++) {
    clearTimeout(i)
    clearInterval(i)
  }
}

// 优化Vue组件内存
const optimizeVueComponents = () => {
  console.log('🔧 优化Vue组件内存')

  // 触发Vue组件的内存清理
  mitt.emit('vue-component-cleanup')

  // 清理可能的Vue实例引用
  if ((window as any).__VUE_DEVTOOLS_GLOBAL_HOOK__) {
    // 开发环境下的Vue开发者工具清理
    console.log('开发环境：清理Vue开发者工具缓存')
  }
}

// 清理图形上下文
const cleanupGraphicsContexts = () => {
  console.log('🎨 清理Canvas和WebGL上下文')

  // 清理Canvas 2D上下文
  const canvases = document.querySelectorAll('canvas')
  canvases.forEach((canvas) => {
    if (!canvas.closest('.blank-view')) {
      const htmlCanvas = canvas as HTMLCanvasElement

      // 清理2D上下文
      const ctx = htmlCanvas.getContext('2d')
      if (ctx) {
        ctx.clearRect(0, 0, htmlCanvas.width, htmlCanvas.height)
        ctx.resetTransform()
        ctx.globalAlpha = 1
        ctx.globalCompositeOperation = 'source-over'
      }

      // 清理WebGL上下文
      const gl = htmlCanvas.getContext('webgl') || htmlCanvas.getContext('experimental-webgl')
      if (gl) {
        const loseContext = (gl as any).getExtension('WEBGL_lose_context')
        if (loseContext) {
          loseContext.loseContext()
        }
      }

      // 重置canvas尺寸以释放GPU内存
      htmlCanvas.width = 1
      htmlCanvas.height = 1
      htmlCanvas.style.width = '1px'
      htmlCanvas.style.height = '1px'
    }
  })
}

onMounted(() => {
  console.log('🔋 空白页面已挂载，激活内存优化模式')

  // 初始DOM节点计数
  if (isDev.value) {
    domNodeCount.value = document.querySelectorAll('*').length
  }

  // 延迟显示托盘指示器，避免立即渲染
  setTimeout(() => {
    showIndicator.value = true
  }, 2000)

  // 延迟执行内存清理，确保页面切换完成
  setTimeout(() => {
    performMemoryCleanup()
    console.log('✨ 内存优化完成，已清理资源:', cleanedResourcesCount.value)
  }, 500)

  // 设置定时器，定期进行深度内存清理
  const deepCleanupTimer = setInterval(() => {
    performMemoryCleanup()
    console.log('🧹 定期深度内存清理完成')
  }, 30000) // 每30秒进行一次深度清理

  // 监听内存清理请求
  mitt.on('memory-cleanup-requested', performMemoryCleanup)

  // 保存定时器以便清理
  ;(window as any)._blankViewCleanupTimer = deepCleanupTimer
})

onBeforeUnmount(() => {
  console.log('🔋 空白页面卸载，清理事件监听器')

  // 清理定时器
  const cleanupTimer = (window as any)._blankViewCleanupTimer
  if (cleanupTimer) {
    clearInterval(cleanupTimer)
    delete (window as any)._blankViewCleanupTimer
  }

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
  /* 最小化DOM渲染开销和GPU占用 */
  contain: layout style paint;
  /* 强制GPU优化，减少合成层 */
  transform: translateZ(0);
  will-change: auto;
  /* 减少重绘和回流 */
  backface-visibility: hidden;
  perspective: 1000px;
}

.tray-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  opacity: 0.3;
  /* 移除过渡效果以减少GPU占用 */
  /* transition: opacity 0.3s ease; */
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
  /* 禁用GPU加速动画 */
  will-change: auto;
  transform: none;
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
  /* 禁用动画以节省GPU资源 */
  /* animation: pulse 2s infinite; */
}

.tray-text {
  font-size: 12px;
  color: var(--n-text-color-disabled);
  font-weight: 500;
  /* 优化字体渲染 */
  text-rendering: optimizeSpeed;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* 禁用所有动画以节省GPU资源 */
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
  transform: none;
  animation: none !important;
  transition: none !important;
}

/* 针对托盘模式的特殊优化 */
.blank-view * {
  /* 强制使用CPU渲染，减少GPU占用 */
  transform: translateZ(0) !important;
  will-change: auto !important;
  backface-visibility: hidden !important;
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
