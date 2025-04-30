import { createRouter, createWebHashHistory } from 'vue-router'
import Layout from '@/components/layout/MainLayout.vue'
import { useServiceStore } from '@/stores/system/ServiceStore'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/service-install',
      name: 'ServiceInstall',
      component: () => import('@/views/ServiceInstallView.vue'),
    },
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
        {
          path: '/blank',
          name: 'Blank',
          component: () => import('@/views/BlankView.vue'),
        }
      ],
    },
  ],
})

// 服务安装检查路由守卫
router.beforeEach((to, from, next) => {
  // 如果目标路径是服务安装页面，直接放行
  if (to.path === '/service-install') {
    return next()
  }
  
  // 从本地存储获取服务安装状态，不再每次都检查服务
  const serviceStore = useServiceStore()
  
  // 如果服务未安装，重定向到服务安装页面
  if (!serviceStore.isServiceInstalled) {
    return next('/service-install')
  }
  
  // 其他情况正常放行
  next()
})

export default router
