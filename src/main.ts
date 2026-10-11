import './assets/main.css'

import { createApp } from 'vue'

import App from './App.vue'
import router from './router'
import { usePinia } from '@/stores'
import i18n from './locales'

const app = createApp(App)

// 设置 Pinia 状态管理
usePinia(app)

// 设置路由
app.use(router)

// 设置国际化
app.use(i18n)

// 错误边界：必须在 mount 之前设置。
// 原实现把它写在 mount 之后，导致挂载阶段抛出的错误根本捕获不到，
// 白屏时控制台也没有任何归因线索。
app.config.errorHandler = (err, _instance, info) => {
  console.error('Vue应用错误:', err, info)
}

// 直接挂载。
// 此前这里 await 了一个 `initializationService.initializeApp()`，
// 而该方法的 try 块是空的（只有一行注释），纯粹是为了给一个 no-op 加一层 await，
// 反而让首屏多一个 microtask 且掩盖了「初始化其实什么都没做」这一事实，已删除。
app.mount('#app')
