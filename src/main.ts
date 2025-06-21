import './assets/main.css'

import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
import { usePinia } from '@/stores'
import i18n from './locales'
import { storeManager } from './stores/StoreManager'
import { webSocketCleaner } from '@/utils/memory-leak-fix'

const app = createApp(App)

// 设置Pinia状态管理
usePinia(app)

// 设置路由
app.use(router)

// 设置国际化
app.use(i18n)

// 初始化Store管理器
storeManager.initialize()

// 设置应用关闭时的清理逻辑
window.addEventListener('beforeunload', async () => {
  console.log('应用关闭，执行清理...')

  // 首先强制保存所有待保存的Store数据
  try {
    const { flushAllPendingSaves } = await import('@/stores')
    await flushAllPendingSaves()
    console.log('所有待保存数据已强制保存')
  } catch (error) {
    console.error('强制保存数据失败:', error)
  }

  // 销毁WebSocketService实例
  try {
    const { WebSocketService } = await import('@/services/websocket-service')
    WebSocketService.destroyInstance()
  } catch (error) {
    console.error('WebSocketService清理失败:', error)
  }

  // 清理所有WebSocket连接和定时器
  webSocketCleaner.cleanupAll()
})

// 应用挂载
app.mount('#app')

// 应用性能测量（开发环境）
if (import.meta.env.DEV) {
  const navigationEntry = performance.getEntriesByType(
    'navigation',
  )[0] as PerformanceNavigationTiming

  if (navigationEntry) {
    const domContentLoaded =
      navigationEntry.domContentLoadedEventEnd - navigationEntry.domContentLoadedEventStart
    const loadComplete = navigationEntry.loadEventEnd - navigationEntry.loadEventStart

    console.log('应用性能指标:')
    console.log(`- DOMContentLoaded: ${domContentLoaded.toFixed(2)}ms`)
    console.log(`- Load Complete: ${loadComplete.toFixed(2)}ms`)
  }
}

// 错误边界
app.config.errorHandler = (err, instance, info) => {
  console.error('Vue应用错误:', err)
  console.error('错误信息:', info)
  console.error('组件实例:', instance)

  // 可以在这里发送错误报告到监控服务
  if (import.meta.env.PROD) {
    // 生产环境错误报告
    // reportError(err, instance, info)
  }
}
