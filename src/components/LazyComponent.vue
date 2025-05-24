<template>
  <div class="lazy-component-wrapper">
    <!-- 加载中状态 -->
    <div v-if="isLoading" class="lazy-loading">
      <slot name="loading">
        <div class="loading-spinner">
          <n-spin size="medium" />
        </div>
      </slot>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="hasError" class="lazy-error">
      <slot name="error" :error="error" :retry="retryLoad">
        <div class="error-content">
          <n-result status="error" title="组件加载失败" :description="errorMessage">
            <template #footer>
              <n-button @click="retryLoad">重试</n-button>
            </template>
          </n-result>
        </div>
      </slot>
    </div>

    <!-- 成功加载的组件 -->
    <component
      v-else-if="loadedComponent"
      :is="loadedComponent"
      v-bind="componentProps"
      @error="handleComponentError"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed, type Component, type PropType } from 'vue'
import { NSpin, NResult, NButton } from 'naive-ui'

interface Props {
  // 组件加载器函数
  loader: () => Promise<Component>
  // 传递给组件的属性
  componentProps?: Record<string, unknown>
  // 是否立即加载
  immediate?: boolean
  // 延迟加载时间（毫秒）
  delay?: number
  // 最大重试次数
  maxRetries?: number
  // 加载超时时间（毫秒）
  timeout?: number
}

const props = withDefaults(defineProps<Props>(), {
  immediate: true,
  delay: 0,
  maxRetries: 3,
  timeout: 10000,
})

const isLoading = ref(false)
const hasError = ref(false)
const error = ref<Error | null>(null)
const loadedComponent = ref<Component | null>(null)
const retryCount = ref(0)

const errorMessage = computed(() => {
  if (!error.value) return ''
  return error.value.message || '未知错误'
})

// 加载组件
async function loadComponent() {
  if (loadedComponent.value) return

  isLoading.value = true
  hasError.value = false
  error.value = null

  try {
    // 创建超时Promise
    const timeoutPromise = new Promise<never>((_, reject) => {
      setTimeout(() => {
        reject(new Error(`组件加载超时 (${props.timeout}ms)`))
      }, props.timeout)
    })

    // 竞争加载Promise和超时Promise
    const component = await Promise.race([props.loader(), timeoutPromise])

    loadedComponent.value = component
    console.log('懒加载组件加载成功')
  } catch (err) {
    console.error('懒加载组件加载失败:', err)
    error.value = err instanceof Error ? err : new Error('组件加载失败')
    hasError.value = true
  } finally {
    isLoading.value = false
  }
}

// 重试加载
async function retryLoad() {
  if (retryCount.value >= props.maxRetries) {
    error.value = new Error(`已达到最大重试次数 (${props.maxRetries})`)
    return
  }

  retryCount.value++
  console.log(`重试加载组件，第 ${retryCount.value} 次`)
  await loadComponent()
}

// 处理组件运行时错误
function handleComponentError(err: Error) {
  console.error('组件运行时错误:', err)
  error.value = err
  hasError.value = true
  loadedComponent.value = null
}

// 延迟加载
async function delayedLoad() {
  if (props.delay > 0) {
    await new Promise((resolve) => setTimeout(resolve, props.delay))
  }
  await loadComponent()
}

// 根据immediate属性决定是否立即加载
onMounted(() => {
  if (props.immediate) {
    delayedLoad()
  }
})

// 暴露加载方法供父组件调用
defineExpose({
  load: loadComponent,
  retry: retryLoad,
  reset: () => {
    loadedComponent.value = null
    hasError.value = false
    error.value = null
    retryCount.value = 0
  },
})
</script>

<style scoped>
.lazy-component-wrapper {
  width: 100%;
  height: 100%;
}

.lazy-loading {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}

.lazy-error {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
}

.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
}

.error-content {
  width: 100%;
  max-width: 400px;
}
</style>
