import { createRouter, createWebHashHistory } from 'vue-router'
import Layout from '@/components/layout/MainLayout.vue'
import { storeManager, routeStoreMap } from '@/stores/StoreManager'
import { componentPreloader } from '@/utils/performance'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
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
        {
          path: '/blank',
          name: 'Blank',
          component: () => import('@/views/BlankView.vue'),
          meta: {
            preloadStores: [],
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

  // 预加载指定的组件
  const preloadComponents = (to.meta?.preloadComponents as string[]) || []
  if (preloadComponents.length > 0) {
    Promise.allSettled(
      preloadComponents.map((componentName) => componentPreloader.preloadComponent(componentName)),
    ).catch((error) => {
      console.error('预加载组件失败:', error)
    })
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
