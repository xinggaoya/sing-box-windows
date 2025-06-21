import { createRouter, createWebHashHistory } from 'vue-router'
import Layout from '@/components/layout/MainLayout.vue'
import { storeManager, routeStoreMap } from '@/stores/StoreManager'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    // 空白页面 - 独立路由，用于托盘模式下减少内存占用
    {
      path: '/blank',
      name: 'Blank',
      component: () => import('@/views/BlankView.vue'),
      meta: {
        preloadStores: [], // 不预加载任何Store
        isBlankPage: true, // 标记为空白页面
      },
    },
    // 主应用布局 - 包含所有功能页面
    {
      path: '/',
      name: 'index',
      component: Layout,
      children: [
        {
          path: '/',
          name: 'Home',
          component: () => import('@/views/HomeView.vue'),
          meta: {
            preloadStores: ['app', 'kernel', 'traffic'],
            preloadComponents: ['ProxyView', 'SubView'],
          },
        },
        {
          path: '/sub',
          name: 'Sub',
          component: () => import('@/views/SubView.vue'),
          meta: {
            preloadStores: ['subscription'],
            preloadComponents: ['SettingView'],
          },
        },
        {
          path: '/proxy',
          name: 'Proxy',
          component: () => import('@/views/ProxyView.vue'),
          meta: {
            preloadStores: ['proxy', 'kernel'],
            preloadComponents: ['RulesView', 'ConnectionsView'],
          },
        },
        {
          path: '/log',
          name: 'Log',
          component: () => import('@/views/LogView.vue'),
          meta: {
            preloadStores: ['log', 'kernel'],
          },
        },
        {
          path: '/setting',
          name: 'Setting',
          component: () => import('@/views/SettingView.vue'),
          meta: {
            preloadStores: ['app', 'theme', 'locale', 'update'],
          },
        },
        {
          path: '/rules',
          name: 'Rules',
          component: () => import('@/views/RulesView.vue'),
          meta: {
            preloadStores: ['proxy', 'kernel'],
          },
        },
        {
          path: '/connections',
          name: 'Connections',
          component: () => import('@/views/ConnectionsView.vue'),
          meta: {
            preloadStores: ['connection', 'traffic'],
          },
        },
      ],
    },
  ],
})

// 路由守卫 - 预加载Store和组件
router.beforeEach(async (to, from, next) => {
  // 预加载当前路由需要的Store
  const requiredStores = routeStoreMap[to.path] || []
  if (requiredStores.length > 0) {
    try {
      await storeManager.preloadStores(requiredStores)
    } catch (error) {
      console.error('预加载Store失败:', error)
    }
  }

  next()
})

// 路由后置守卫 - 清理和优化
router.afterEach((to, from) => {
  // 在开发环境记录路由性能信息
  if (import.meta.env.DEV) {
    console.log('Route Stats:', storeManager.getStats())
  }
})

export default router
