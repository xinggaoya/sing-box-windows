import { createRouter, createWebHashHistory } from 'vue-router'
import Layout from '@/components/layout/Layout.vue'

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
          component: () => import('@/views/HomeView.vue')
        },
        {
          path: '/sub',
          name: 'Sub',
          component: () => import('@/views/SubView.vue')
        },
        {
          path: '/proxy',
          name: 'Proxy',
          component: () => import('@/views/ProxyView.vue')
        },
        {
          path: '/log',
          name: 'Log',
          component: () => import('@/views/LogView.vue')
        },
        {
          path: '/setting',
          name: 'Setting',
          component: () => import('@/views/SettingView.vue')
        }
      ]
    },

  ]
})


export default router
