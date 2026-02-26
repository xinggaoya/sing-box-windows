import './assets/main.css'

import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
import { usePinia } from '@/stores'
import i18n from './locales'
import { initializationService } from './services/initialization-service'

const app = createApp(App)

// 设置Pinia状态管理
usePinia(app)

// 设置路由
app.use(router)

// 设置国际化
app.use(i18n)

// 异步初始化应用
const initializeApp = async () => {
  try {
    // 使用新的初始化服务
    await initializationService.initializeApp()

    // 应用挂载（在初始化完成后）
    app.mount('#app')
  } catch {
    // 即使初始化失败，也尝试挂载应用以显示错误页面
    app.mount('#app')
  }
}

// 开始初始化
initializeApp()

// 错误边界
app.config.errorHandler = (err, _instance, info) => {
  console.error('Vue应用错误:', err, info)
}
