import './assets/main.css'

import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
import { usePinia } from '@/stores'
import i18n from './locales'

// 导入性能优化工具
import { memoryMonitor, componentPreloader, eventListenerManager } from '@/utils/performance'
import { bundleAnalyzer } from '@/utils/bundleAnalyzer'
import { codeSplittingManager } from '@/utils/codeSplitting'

const app = createApp(App)

// 设置Pinia状态管理
usePinia(app)

// 设置路由
app.use(router)

// 设置国际化
app.use(i18n)

// 性能优化初始化
if (import.meta.env.DEV) {
  console.log('🚀 开发环境性能优化工具已启用')

  // 启动内存监控
  memoryMonitor.startMonitoring(15000) // 每15秒监控一次

  // 预加载关键组件
  componentPreloader.preloadComponent('HomeView').catch(console.error)

  // 输出初始化信息
  console.log('📊 性能监控工具状态:')
  console.log('- 内存监控: 已启动')
  console.log('- 组件预加载器: 已启动')
  console.log('- Bundle分析器: 已启动')
  console.log('- 代码分割管理器: 已启动')
  console.log('- 事件监听器管理: 已启动')
}

// 应用挂载
app.mount('#app')

// 应用性能测量
const navigationEntry = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming

if (navigationEntry) {
  const domContentLoaded =
    navigationEntry.domContentLoadedEventEnd - navigationEntry.domContentLoadedEventStart
  const loadComplete = navigationEntry.loadEventEnd - navigationEntry.loadEventStart

  console.log('⚡ 应用性能指标:')
  console.log(`- DOMContentLoaded: ${domContentLoaded.toFixed(2)}ms`)
  console.log(`- Load Complete: ${loadComplete.toFixed(2)}ms`)
  console.log(
    `- DNS Lookup: ${(navigationEntry.domainLookupEnd - navigationEntry.domainLookupStart).toFixed(2)}ms`,
  )
  console.log(
    `- TCP Connect: ${(navigationEntry.connectEnd - navigationEntry.connectStart).toFixed(2)}ms`,
  )
}

// 在应用卸载时清理资源
window.addEventListener('beforeunload', () => {
  if (import.meta.env.DEV) {
    console.log('🧹 清理性能优化工具资源...')

    // 停止内存监控
    memoryMonitor.stopMonitoring()

    // 清理组件预加载器
    componentPreloader.destroy()

    // 清理事件监听器
    eventListenerManager.cleanup()

    // 清理代码分割管理器
    codeSplittingManager.cleanup()

    // 最终输出Bundle分析报告
    bundleAnalyzer.printReport()
  }
})

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

// 全局属性（仅开发环境）
if (import.meta.env.DEV) {
  app.config.globalProperties.$performance = {
    memoryMonitor,
    bundleAnalyzer,
    codeSplittingManager,
    componentPreloader,
    eventListenerManager,
  }

  // 暴露到window对象方便调试
  const performanceTools = {
    memoryMonitor,
    bundleAnalyzer,
    codeSplittingManager,
    componentPreloader,
    eventListenerManager,
  }

  Object.defineProperty(window, '__PERF_TOOLS__', {
    value: performanceTools,
    writable: false,
    configurable: false,
  })

  console.log('🔧 性能工具已挂载到 window.__PERF_TOOLS__')
}
