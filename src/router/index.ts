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
router.beforeEach(async (to, from, next) => {
  // 如果目标路径是服务安装页面，直接放行
  if (to.path === '/service-install') {
    return next()
  }
  
  // 获取服务状态
  const serviceStore = useServiceStore()
  
  // 首次访问非安装页面时，先检查服务状态
  if (from.path === '' && !serviceStore.isServiceInstalled) {
    try {
      // 尝试实际检查服务状态
      const status = await serviceStore.checkServiceStatus()
      
      // 如果服务已安装，则不需要重定向
      if (status.installed) {
        return next()
      }
      
      // 服务确实未安装，重定向到安装页面
      return next('/service-install')
    } catch (error) {
      console.error('检查服务状态失败:', error)
      // 如果无法确定状态，先放行
      return next()
    }
  }
  
  // 其他情况正常放行
  next()
})

export default router
