import { defineStore } from 'pinia'
import { ref } from 'vue'
import { tauriApi } from '@/services/tauri'
import { useAppStore } from '@/stores'

// 代理模式声明
import { ProxyMode } from '@/stores'

// 导入ProxyData类型
interface ProxyData {
  type: string
  name: string
  now: string
  all: string[]
  history: Array<{ time: string; delay: number }>
  udp: boolean
}

export const useProxyStore = defineStore('proxy', () => {
  const appStore = useAppStore()

  // 代理设置相关
  const selectedNodeIndex = ref<number | null>(null)
  const nodeList = ref<string[]>([])
  const nodeDelays = ref<Record<string, number>>({})

  // 更新代理数据
  const updateProxies = (data: { proxies?: Record<string, ProxyData> }) => {
    // 实现更新代理数据的逻辑
    console.log('更新代理数据', data)
  }

  // 切换代理模式
  const switchProxyMode = async (targetMode: ProxyMode) => {
    // 如果当前模式与目标模式相同，则不需要切换
    if (appStore.proxyMode === targetMode) return false

    // 根据模式调用对应服务
    try {
      if (targetMode === 'system') {
        await setSystemProxy()
      } else {
        // TUN模式可能需要管理员权限，检查并处理
        const isAdmin = await checkAdmin()
        if (!isAdmin) {
          // 需要管理员权限，实现重启
          await restartAsAdmin()
          return true
        }
        await setTUNProxy()
      }

      // 切换成功后更新状态
      appStore.switchProxyMode(targetMode)

      // 代理模式变更事件现在可以通过Pinia的响应式系统处理
      console.log('代理模式已切换到:', targetMode)

      return false // 不需要关闭窗口
    } catch (error) {
      console.error('切换代理模式失败:', error)
      throw error
    }
  }

  // 设置系统代理
  const setSystemProxy = async () => {
    try {
      await tauriApi.proxy.setSystemProxy()
      return true
    } catch (error) {
      console.error('设置系统代理失败:', error)
      throw error
    }
  }

  // 设置TUN代理
  const setTUNProxy = async () => {
    try {
      await tauriApi.proxy.setTunProxy()
      return true
    } catch (error) {
      console.error('设置TUN代理失败:', error)
      throw error
    }
  }

  // 检查管理员权限
  const checkAdmin = async () => {
    try {
      return await tauriApi.system.checkAdmin()
    } catch (error) {
      console.error('检查管理员权限失败:', error)
      return false
    }
  }

  // 以管理员身份重启
  const restartAsAdmin = async () => {
    try {
      await tauriApi.system.restartAsAdmin()
      return true
    } catch (error) {
      console.error('以管理员身份重启失败:', error)
      throw error
    }
  }

  // 获取代理节点列表
  const getProxyNodes = async () => {
    try {
      const result = await tauriApi.proxy.getProxies()
      // 假设返回的是一个对象，需要提取节点列表
      if (result && Array.isArray(result.proxies)) {
        nodeList.value = result.proxies
        return result.proxies
      }
      return []
    } catch (error) {
      console.error('获取代理节点失败:', error)
      return []
    }
  }

  // 切换代理节点
  const changeProxyNode = async (index: number) => {
    try {
      if (index >= 0 && index < nodeList.value.length) {
        const nodeName = nodeList.value[index]
        await tauriApi.proxy.changeProxy('GLOBAL', nodeName)
        selectedNodeIndex.value = index
        return true
      }
      return false
    } catch (error) {
      console.error(`切换到节点 ${index} 失败:`, error)
      return false
    }
  }

  // 测试节点延迟
  const testNodeDelay = async (nodeName: string): Promise<number> => {
    try {
      const delay = await tauriApi.proxy.testNodeDelay(nodeName)
      // 确保delay是数字类型
      const delayNum = typeof delay === 'number' ? delay : -1
      nodeDelays.value[nodeName] = delayNum
      return delayNum
    } catch (error) {
      console.error(`测试节点 ${nodeName} 延迟失败:`, error)
      nodeDelays.value[nodeName] = -1
      return -1
    }
  }

  // 测试所有节点延迟
  const testAllNodesDelay = async (_port: number) => {
    const results: Record<string, number> = {}

    for (const node of nodeList.value) {
      try {
        const delay = await testNodeDelay(node)
        // delay已经在testNodeDelay中处理成数字
        results[node] = delay
      } catch {
        results[node] = -1
      }
    }

    return results
  }

  return {
    selectedNodeIndex,
    nodeList,
    nodeDelays,
    updateProxies,
    switchProxyMode,
    setSystemProxy,
    setTUNProxy,
    getProxyNodes,
    changeProxyNode,
    testNodeDelay,
    testAllNodesDelay,
  }
})
