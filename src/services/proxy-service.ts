import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/AppStore'
import { useInfoStore } from '@/stores/infoStore'
import { tauriApi } from './tauri-api'

export class ProxyService {
  private static instance: ProxyService
  private appStore = useAppStore()
  private infoStore = useInfoStore()

  private constructor() {}

  public static getInstance(): ProxyService {
    if (!ProxyService.instance) {
      ProxyService.instance = new ProxyService()
    }
    return ProxyService.instance
  }

  /**
   * 切换代理模式
   * @param mode 代理模式
   * @param showMessage 消息提示函数
   * @returns 是否需要关闭窗口
   */
  public async switchMode(
    mode: 'system' | 'tun',
    showMessage: (type: 'success' | 'info' | 'error', content: string) => void,
  ): Promise<boolean> {
    try {
      if (mode === 'system') {
        await tauriApi.proxy.setSystemProxy()
        this.appStore.mode = 'system'
        showMessage('success', '已切换到系统代理模式')
      } else {
        // TUN模式需要管理员权限
        const isAdmin = await tauriApi.proxy.checkAdmin()
        if (!isAdmin) {
          await tauriApi.proxy.restartAsAdmin()
          return true // 需要关闭窗口
        }
        await tauriApi.proxy.setTunProxy()
        this.appStore.mode = 'tun'
        showMessage('success', '已切换到TUN模式')
      }

      // 如果内核正在运行，需要重启
      if (this.appStore.isRunning) {
        try {
          showMessage('info', '正在重启内核...')
          await this.infoStore.restartKernel()
          showMessage('success', '内核已重启')
        } catch (error) {
          showMessage('error', `重启内核失败: ${error}`)
        }
      }

      return false // 不需要关闭窗口
    } catch (error) {
      showMessage('error', `切换代理模式失败: ${error}`)
      return false
    }
  }
}
