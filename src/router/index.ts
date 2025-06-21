import { createRouter, createWebHashHistory } from 'vue-router'
import Layout from '@/components/layout/MainLayout.vue'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    // 空白页面 - 独立路由，用于托盘模式下减少内存占用
    {
      path: '/blank',
      name: 'Blank',
      component: () => import('@/views/BlankView.vue'),
      meta: {
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
        },
        {
          path: '/sub',
          name: 'Sub',
          component: () => import('@/views/SubView.vue'),
        },
        {
          path: '/proxy',
          name: 'Proxy',
          component: () => import('@/views/ProxyView.vue'),
        },
        {
          path: '/log',
          name: 'Log',
          component: () => import('@/views/LogView.vue'),
        },
        {
          path: '/setting',
          name: 'Setting',
          component: () => import('@/views/SettingView.vue'),
        },
        {
          path: '/rules',
          name: 'Rules',
          component: () => import('@/views/RulesView.vue'),
        },
        {
          path: '/connections',
          name: 'Connections',
          component: () => import('@/views/ConnectionsView.vue'),
        },
      ],
    },
  ],
})

// 路由守卫
router.beforeEach((to, from, next) => {
  next()
})

export default router
