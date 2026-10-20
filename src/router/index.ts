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
          // 出站总览：内核里所有出站（含分组）的平铺快照。
          // 独立成页面而不是代理页里的卡片/抽屉——代理页是定高布局，
          // 同层挂内容高度不定的面板会把代理组与节点列表挤没。
          path: '/outbounds',
          name: 'Outbounds',
          component: () => import('@/views/OutboundsView.vue'),
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
